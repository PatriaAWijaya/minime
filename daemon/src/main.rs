//! minime-daemon — background service: owns SQLite, executes tasks FIFO,
//! runs pipelines, serves IPC. Survives UI quits (launchd KeepAlive on macOS).

mod agent_loader;
mod db;
mod drive_oauth;
mod events;
mod executor;
mod ipc;
mod models;
mod notify;
mod pipeline;
mod provider;
mod providers_deepseek;
mod providers_gemini;
mod providers_mock;
mod providers_muse;
mod providers_opencode;
mod runner;
mod secrets;
mod tools;

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

use executor::Providers;
use provider::ModelProvider;

/// Build the provider registry: OpenCode free (no key) + Gemini + Muse + DeepSeek.
/// Providers without keys register anyway and fail fast with NoKey at call
/// time, so the fallback chain can skip them transparently.
fn build_providers(repo_dir: &std::path::Path) -> Providers {
    let mut map: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    let oc: Arc<dyn ModelProvider> = Arc::new(providers_opencode::OpenCodeProvider::new(
        repo_dir.to_path_buf(),
    ));
    map.insert(oc.id().to_string(), oc);
    let gm: Arc<dyn ModelProvider> = Arc::new(providers_gemini::GeminiProvider::new());
    map.insert(gm.id().to_string(), gm);
    let ms: Arc<dyn ModelProvider> = Arc::new(providers_muse::MuseProvider::new());
    map.insert(ms.id().to_string(), ms);
    let ds: Arc<dyn ModelProvider> = Arc::new(providers_deepseek::DeepSeekProvider::new());
    map.insert(ds.id().to_string(), ds);
    // Mock provider — hanya bila MINIME_MOCK_LLM=1 (integration test, bukan produksi).
    if std::env::var("MINIME_MOCK_LLM").is_ok() {
        let mk: Arc<dyn ModelProvider> = Arc::new(providers_mock::MockProvider::new());
        map.insert(mk.id().to_string(), mk);
        eprintln!("[minime-daemon] mock provider AKTIF (MINIME_MOCK_LLM=1)");
    }
    eprintln!(
        "[minime-daemon] providers: {}",
        map.keys()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Arc::new(map)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Live-test harness (Gate 4): `--live-test` memanggil Gemini / Muse /
    // DeepSeek langsung via kode provider daemon, tanpa DB/IPC. Kunci dibaca
    // dari env MINIME_*_KEY atau Keychain (service com.minime.app) — sama
    // seperti jalur produksi di secrets.rs.
    if std::env::args().any(|a| a == "--live-test") {
        return live_test().await;
    }
    // Store a provider key into the OS keychain via the daemon's own write
    // path (same as IPC `secrets.set`, which the Tauri Providers UI uses).
    // Keys are read from stdin (paste it; nothing lands in shell history).
    // Needed because items created manually in Keychain Access.app are not
    // visible to the keyring query (Gate 4 finding, 18 Okt 2026).
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--set-key") {
        let account = args.get(i + 1).cloned().unwrap_or_default();
        if account.is_empty() {
            anyhow::bail!("usage: minime-daemon --set-key <account>");
        }
        return set_key_interactive(&account);
    }

    let data_dir = db::data_dir()?;
    let repo_dir = db::repo_dir();
    eprintln!(
        "[minime-daemon] data_dir={} repo_dir={}",
        data_dir.display(),
        repo_dir.display()
    );

    // 1. Database + migrations.
    let db = db::open(&data_dir.join("minime.db")).await?;
    let applied = db::run_migrations(&db, &repo_dir).await?;
    if !applied.is_empty() {
        eprintln!("[minime-daemon] migrations applied: {applied:?}");
    }

    // 2. Honest crash recovery.
    let (tasks, runs) = db::crash_recovery(&db).await?;
    if tasks + runs > 0 {
        eprintln!("[minime-daemon] crash recovery: {tasks} tasks, {runs} runs marked failed");
    }

    // 3. Load agent + skill definitions (source of truth: agents/<id>/, skills/*/).
    let agents = agent_loader::load_agents(&db, &repo_dir, &data_dir).await?;
    eprintln!("[minime-daemon] agents loaded: {}", agents.join(", "));
    let skills = agent_loader::load_skills(&db, &repo_dir).await?;
    eprintln!("[minime-daemon] skills loaded: {}", skills.len());

    // 4. Validate bundled pipelines at startup — fail fast on bad YAML.
    let agent_ids = agent_loader::agent_ids(&db).await?;
    let skill_names = agent_loader::skill_names(&db).await?;
    for (name, _) in pipeline::list_pipelines(&repo_dir) {
        let p = pipeline::load_pipeline(&repo_dir, &name)?;
        pipeline::validate_pipeline(&p, &repo_dir, &agent_ids, &skill_names)
            .with_context(|| format!("invalid bundled pipeline '{name}'"))?;
    }
    eprintln!("[minime-daemon] pipelines validated");

    // 5. Executor + IPC.
    let (tx, _rx) = broadcast::channel::<events::Event>(128);
    let providers = build_providers(&repo_dir);
    let browser_store = tools::new_browser_store();
    let exec_handle_providers = providers.clone();
    let _exec_handle = executor::Executor::new(
        db.clone(),
        tx.clone(),
        exec_handle_providers,
        browser_store.clone(),
    )
    .start();

    // 5b. Tool readiness check (Fase 4) — eksplisit, tidak ada yang gagal diam-diam.
    for t in tools::readiness(&data_dir, &repo_dir).await {
        eprintln!(
            "[minime-daemon] tool {:<14} ready={:<5} state={:<15} :: {}",
            t.id, t.ready, t.state, t.detail
        );
    }

    let sock = data_dir.join("minime.sock");
    let app = ipc::App {
        db,
        tx,
        repo: repo_dir,
        data_dir: data_dir.clone(),
        start: std::time::Instant::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        providers,
        browser_store,
    };
    eprintln!("[minime-daemon] listening on {}", sock.display());

    tokio::select! {
        r = ipc::serve(app, &sock) => {
            if let Err(e) = r {
                eprintln!("[minime-daemon] ipc error: {e:#}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("[minime-daemon] shutting down (socket removed)");
            std::fs::remove_file(&sock).ok();
        }
    }
    Ok(())
}

/// Live-test harness for Gate 4: calls Gemini / Muse / DeepSeek through the
/// real provider implementations with a tiny prompt. Keys resolve via
/// secrets::api_key (env MINIME_*_KEY, then OS keychain service
/// `com.minime.app`) — the same path production uses. Prints one line per
/// provider: OK + reply snippet, or FAIL + typed error.
async fn live_test() -> Result<()> {
    use provider::{ChatRequest, ModelProvider};

    struct Case {
        name: &'static str,
        provider: std::sync::Arc<dyn ModelProvider>,
        model: &'static str,
    }
    let cases = vec![
        Case {
            name: "gemini",
            provider: std::sync::Arc::new(providers_gemini::GeminiProvider::new()),
            model: "gemini-2.0-flash",
        },
        Case {
            name: "muse",
            provider: std::sync::Arc::new(providers_muse::MuseProvider::new()),
            model: providers_muse::DEFAULT_MODEL,
        },
        Case {
            name: "deepseek",
            provider: std::sync::Arc::new(providers_deepseek::DeepSeekProvider::new()),
            model: "deepseek-chat",
        },
    ];

    // Diagnose keychain access before the provider calls: the production
    // path (secrets::api_key) swallows keyring errors, so surface them here.
    for acct in ["gemini", "muse", "deepseek"] {
        match keyring::Entry::new("com.minime.app", acct) {
            Ok(e) => match e.get_password() {
                Ok(_) => println!("[live-test] keychain {acct}: readable"),
                Err(err) => println!("[live-test] keychain {acct}: READ FAILED: {err:?}"),
            },
            Err(err) => println!("[live-test] keychain {acct}: ENTRY FAILED: {err:?}"),
        }
    }

    let mut failures = 0;
    for c in cases {
        let req = ChatRequest {
            model: c.model.to_string(),
            system_prompt: String::new(),
            messages: serde_json::json!([
                {"role": "user", "content": "Balas persis dengan satu kata: OK"}
            ]),
            effort: "low".to_string(),
            max_tokens: Some(64),
            workdir: None,
        };
        let mut rx = c.provider.complete_stream(req);
        let mut text = String::new();
        let mut err: Option<String> = None;
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(60), async {
            while let Some(chunk) = rx.recv().await {
                match chunk {
                    Ok(ch) => {
                        text.push_str(&ch.delta_text);
                        if ch.done {
                            break;
                        }
                    }
                    Err(e) => {
                        err = Some(format!("{e:?}"));
                        break;
                    }
                }
            }
        })
        .await;
        match (outcome, err) {
            (Err(_), _) => {
                failures += 1;
                println!("[live-test] {:<8} FAIL timeout after 60s", c.name);
            }
            (Ok(_), Some(e)) => {
                failures += 1;
                println!("[live-test] {:<8} FAIL {e}", c.name);
            }
            (Ok(_), None) => {
                let snippet: String = text.trim().chars().take(80).collect();
                println!(
                    "[live-test] {:<8} OK model={} reply={:?}",
                    c.name, c.model, snippet
                );
            }
        }
    }
    if failures > 0 {
        anyhow::bail!("{failures} provider(s) failed live test");
    }
    println!("[live-test] all 3 providers OK");
    Ok(())
}

/// Read a key from stdin (no shell history) and store it via secrets::set_api_key.
fn set_key_interactive(account: &str) -> Result<()> {
    use std::io::{self, BufRead, Write};
    print!("API key untuk '{account}' (paste, lalu Enter): ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    let key = line.trim();
    if key.is_empty() {
        anyhow::bail!("key kosong, dibatalkan");
    }
    secrets::set_api_key(account, key)
        .map_err(|e| anyhow::anyhow!("gagal simpan ke keychain: {e}"))?;
    // Verify the round-trip through the same read path production uses.
    match secrets::api_key(account) {
        Some(_) => println!("[set-key] '{account}' tersimpan & terbaca OK"),
        None => anyhow::bail!("tersimpan tapi tidak terbaca kembali (keychain bermasalah)"),
    }
    Ok(())
}
