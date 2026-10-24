// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod drive;
mod island;
mod logger;
mod e2e_test;
mod model_selector;
mod orchestrate;
mod oauth_local;
mod pireel;
mod providers;
mod runtime;
mod secrets;
mod skills;
mod tools;
mod social;
mod voicestudio;

use providers::{ChatMessage, ModelTier, ProviderStatus, Router};
use serde::Serialize;
use tauri::State;

/// Basic app metadata exposed to the frontend.
#[derive(Serialize)]
struct AppInfo {
    name: String,
    version: String,
    phase: String,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Minime".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        phase: "tahap-2-router".to_string(),
    }
}

/// RouterKu gateway base URL for this session (per user; the frontend
/// persists it in its own settings store).
fn routerku_base_url() -> Option<String> {
    std::env::var("MINIME_ROUTERKU_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
}

/// Probe every provider: id, kind, state, detail, models.
/// The Penyedia Model screen renders this instead of mock data.
#[tauri::command]
async fn provider_probe() -> Result<Vec<ProviderStatus>, String> {
    Ok(Router::new(routerku_base_url()).statuses().await)
}

/// Send a chat message through the model router with automatic fallback.
/// `model_id` format: `provider:model` (e.g. `ollama:llama3.1`,
/// `muse:muse-spark-1.3`). `tier`: "gratis" | "berbayar".
/// `system_prompt`: persona + instruksi agen dari UI; playbook skill
/// disuntikkan otomatis di backend. Riwayat + usage dicatat ke SQLite.
#[tauri::command]
async fn agent_chat(
    db: State<'_, db::Db>,
    agent_id: String,
    model_id: String,
    tier: String,
    system_prompt: String,
    messages: Vec<ChatMessage>,
) -> Result<providers::ChatResult, String> {
    let tier = match tier.as_str() {
        "berbayar" => ModelTier::Berbayar,
        _ => ModelTier::Gratis,
    };

    // System prompt final: persona dari UI + playbook skill agen.
    let skill_ctx = skills::skill_context(&db, &agent_id);
    let full_system = if skill_ctx.is_empty() {
        system_prompt.clone()
    } else {
        format!("{system_prompt}\n\n{skill_ctx}")
    };
    let mut wire: Vec<ChatMessage> = Vec::with_capacity(messages.len() + 1);
    if !full_system.trim().is_empty() {
        wire.push(ChatMessage {
            role: "system".to_string(),
            content: full_system,
        });
    }
    wire.extend(messages.iter().cloned());

    eprintln!("[agent_chat] masuk: agent={agent_id} model={model_id} tier={tier:?} n_msg={}", messages.len());
    // Simpan pesan user dulu (riwayat tetap ada walau provider gagal).
    if let Some(last) = messages.last() {
        let _ = db.save_message(&agent_id, &last.role, &last.content, None);
    }
    eprintln!("[agent_chat] pesan tersimpan, panggil router...");

    let router = Router::new(routerku_base_url());
    let mut res = match router.chat(&model_id, tier, wire).await {
        Ok(r) => {
            eprintln!("[agent_chat] router OK via {}", r.served_by);
            r
        }
        Err(e) => {
            eprintln!("[agent_chat] router GAGAL: {e}");
            // Kembalikan sebagai field error (Ok) — invoke rejection terbukti
            // tidak selalu sampai ke webview.
            return Ok(providers::ChatResult {
                text: String::new(),
                served_by: String::new(),
                input_tokens: None,
                output_tokens: None,
                error: Some(e),
            });
        }
    };
    res.error = None;

    // Simpan balasan + catat usage (metering untuk tab Cost).
    let _ = db.save_message(&agent_id, "assistant", &res.text, Some(&res.served_by));
    let _ = db.record_usage(
        &agent_id,
        &res.served_by,
        &model_id,
        res.input_tokens.unwrap_or(0) as i64,
        res.output_tokens.unwrap_or(0) as i64,
    );
    Ok(res)
}

/// Orkestrasi tugas multi-agen via Mimi (CIO).
/// Alur: parse intent → dispatch paralel → agregat → QC → return.
/// `mimi_model_id`: model untuk Mimi (default: muse:muse-spark-1.3).
#[tauri::command]
async fn orchestrate_task(
    db: State<'_, db::Db>,
    user_request: String,
    mimi_model_id: Option<String>,
) -> Result<orchestrate::OrchestrateResult, String> {
    let model_id = mimi_model_id.unwrap_or_else(|| "muse:muse-spark-1.3".to_string());
    let router = Router::new(routerku_base_url());
    Ok(orchestrate::orchestrate(&db, &router, &user_request, &model_id).await)
}

/// Transkripsi audio dari base64 (untuk STT tombol mic).
/// Menyimpan ke temp file lalu memanggil meta_transcribe.
#[tauri::command]
async fn transcribe_audio_b64(
    router: tauri::State<'_, Router>,
    audio_b64: String,
    ext: String,
) -> Result<String, String> {
    use std::io::Write;
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &audio_b64,
    )
    .map_err(|e| format!("base64 invalid: {e}"))?;
    let ext = if ext.chars().all(|c| c.is_alphanumeric()) && ext.len() <= 5 {
        ext
    } else {
        "webm".to_string()
    };
    let path = std::env::temp_dir().join(format!("minime_stt_{}.{}", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        ext));
    std::fs::File::create(&path)
        .and_then(|mut f| f.write_all(&bytes))
        .map_err(|e| format!("gagal simpan audio: {e}"))?;
    let path_str = path.to_string_lossy().to_string();
    let result = router
        .muse()
        .transcribe(&path_str)
        .await
        .map_err(|e| format!("transkripsi gagal: {e}"));
    let _ = std::fs::remove_file(&path);
    result
}

/// Pilih model optimal (Spark vs Wonder) berdasarkan kompleksitas tugas.
/// Heuristik: panjang prompt + kata kunci + jenis agen.
#[tauri::command]
fn select_model_smart(
    prompt: String,
    agent_id: String,
) -> Result<model_selector::ModelDecision, String> {
    Ok(model_selector::select_model(&prompt, &agent_id))
}

/// E2E test sementara — workflow Ramadan (HAPUS setelah test).
#[tauri::command]
async fn e2e_ramadan(
    db: State<'_, db::Db>,
) -> Result<e2e_test::E2EResult, String> {
    let router = Router::new(routerku_base_url());
    Ok(e2e_test::run_ramadan_e2e(&db, &router).await)
}

/// Cek file trigger untuk auto-run E2E (sementara).
#[tauri::command]
fn check_e2e_trigger() -> bool {
    std::path::Path::new("/tmp/minime_e2e_trigger").exists()
}

/// Ambil riwayat chat agen (terbaru `limit`, urutan kronologis).
#[tauri::command]
fn chat_history(
    db: State<'_, db::Db>,
    agent_id: String,
    limit: Option<i64>,
) -> Result<Vec<db::StoredMessage>, String> {
    db.history(&agent_id, limit.unwrap_or(100).clamp(1, 500))
}

/// Ringkasan usage per agen+provider+model (untuk tab Cost).
#[tauri::command]
fn usage_summary(db: State<'_, db::Db>) -> Result<Vec<db::UsageRow>, String> {
    db.usage_summary()
}

/// Connect a paid provider: store its API key in the OS keychain.
#[tauri::command]
fn provider_connect(provider_id: String, api_key: String) -> Result<(), String> {
    let account = provider_account(&provider_id)?;
    secrets::set_api_key(account, &api_key)
}

/// Disconnect a paid provider: remove its API key from the OS keychain.
#[tauri::command]
fn provider_disconnect(provider_id: String) -> Result<(), String> {
    let account = provider_account(&provider_id)?;
    secrets::delete_api_key(account)
}

/// Set the RouterKu gateway base URL (per user).
#[tauri::command]
fn routerku_set_base_url(base_url: String) -> Result<(), String> {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return Err("base URL kosong".to_string());
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("base URL harus http(s)".to_string());
    }
    std::env::set_var("MINIME_ROUTERKU_BASE_URL", trimmed);
    Ok(())
}

fn provider_account(provider_id: &str) -> Result<&'static str, String> {
    match provider_id {
        "muse" => Ok("muse"),
        "gemini" => Ok("gemini"),
        "deepseek" => Ok("deepseek"),
        "routerku" => Ok("routerku"),
        // ElevenLabs Scribe (Settings > Voice): the key only lives in the OS
        // keychain; it is not a chat-router provider.
        "elevenlabs" => Ok("elevenlabs"),
        other => Err(format!("provider '{other}' tidak butuh API key")),
    }
}

/// Send a native OS notification (Phase 3.2: approval gate).
/// Used when a task needs approval or finishes.
#[tauri::command]
fn send_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| format!("Gagal kirim notifikasi: {}", e))
}

/// macOS privacy permissions relevant to Computer Use (Settings page).
/// Checked with the official preflight APIs — they never prompt; the
/// user grants access in macOS Settings (see `open_privacy_settings`).
#[derive(Serialize)]
struct SystemPermissions {
    screen_recording: bool,
    accessibility: bool,
    /// False on platforms without this permission model (Windows/Linux).
    supported: bool,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[tauri::command]
fn system_permissions() -> SystemPermissions {
    #[cfg(target_os = "macos")]
    unsafe {
        SystemPermissions {
            screen_recording: CGPreflightScreenCaptureAccess(),
            accessibility: AXIsProcessTrusted(),
            supported: true,
        }
    }
    #[cfg(not(target_os = "macos"))]
    SystemPermissions {
        screen_recording: false,
        accessibility: false,
        supported: false,
    }
}

/// This device's display name + OS label (Settings > Profile sessions).
#[derive(Serialize)]
struct DeviceInfo {
    name: String,
    os: String,
}

#[tauri::command]
fn device_info() -> DeviceInfo {
    let os_name = match std::env::consts::OS {
        "macos" => "macOS",
        "windows" => "Windows",
        "linux" => "Linux",
        other => other,
    };
    let name = device_name();
    DeviceInfo {
        name,
        os: format!("{os_name} · {}", std::env::consts::ARCH),
    }
}

fn device_name() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("scutil")
            .args(["--get", "ComputerName"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(v) = std::env::var("COMPUTERNAME") {
            if !v.trim().is_empty() {
                return v;
            }
        }
    }
    if let Ok(out) = std::process::Command::new("hostname").output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return s;
        }
    }
    "Desktop".to_string()
}

/// Open the macOS Privacy & Security pane for one permission kind
/// ("screen" | "accessibility").
#[tauri::command]
fn open_privacy_settings(kind: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let pane = match kind.as_str() {
            "screen" => "Privacy_ScreenCapture",
            _ => "Privacy_Accessibility",
        };
        let url = format!(
            "x-apple.systempreferences:com.apple.preference.security?{pane}"
        );
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Gagal membuka Pengaturan macOS: {e}"))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        Err("Hanya tersedia di macOS".to_string())
    }
}

/// Update check against the public GitHub releases of Minime
/// (Settings > Updates). Stable track = latest non-prerelease release;
/// beta track = most recent release including prereleases.
#[derive(Serialize)]
struct UpdateInfo {
    current: String,
    latest: Option<String>,
    update_available: bool,
    notes: String,
    url: String,
}

fn semver_gt(candidate: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0)
            })
            .collect()
    };
    let a = parse(candidate);
    let b = parse(current);
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::semver_gt;

    #[test]
    fn semver_compares_numerically() {
        assert!(semver_gt("0.2.0", "0.1.0"));
        assert!(semver_gt("v1.0.0", "0.9.9"));
        assert!(semver_gt("0.1.1", "0.1.0"));
        assert!(semver_gt("0.10.0", "0.9.0"));
        assert!(!semver_gt("0.1.0", "0.1.0"));
        assert!(!semver_gt("0.1.0", "0.2.0"));
        assert!(!semver_gt("0.9.9", "1.0.0"));
        assert!(!semver_gt("1.0.0", "1.0.0"));
    }
}

#[tauri::command]
async fn check_update(track: String) -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let releases_page = "https://github.com/PatriaAWijaya/minime/releases".to_string();
    let client = reqwest::Client::builder()
        .user_agent("Minime-Updater")
        .build()
        .map_err(|e| e.to_string())?;
    let api = if track == "beta" {
        "https://api.github.com/repos/PatriaAWijaya/minime/releases?per_page=1"
    } else {
        "https://api.github.com/repos/PatriaAWijaya/minime/releases/latest"
    };
    let resp = client.get(api).send().await.map_err(|e| e.to_string())?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        // No releases published yet.
        return Ok(UpdateInfo {
            current,
            latest: None,
            update_available: false,
            notes: String::new(),
            url: releases_page,
        });
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let release = if track == "beta" {
        json.as_array().and_then(|a| a.first().cloned())
    } else {
        Some(json.clone())
    };
    let Some(release) = release else {
        return Ok(UpdateInfo {
            current,
            latest: None,
            update_available: false,
            notes: String::new(),
            url: releases_page,
        });
    };
    let tag = release["tag_name"].as_str().unwrap_or("").to_string();
    let latest = tag.trim_start_matches('v').to_string();
    Ok(UpdateInfo {
        update_available: semver_gt(&latest, &current),
        latest: Some(latest),
        notes: release["body"].as_str().unwrap_or("").to_string(),
        url: release["html_url"]
            .as_str()
            .unwrap_or(&releases_page)
            .to_string(),
        current,
    })
}

/// Restart the app (the apply step once an update is downloaded).
#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

/* ---------- Marketplace > Skills (catalog + assignment) ---------- */

/// Skill catalog with the agents that have each skill (Marketplace).
#[tauri::command]
fn list_skill_catalog(
    db: State<'_, db::Db>,
    agent_ids: Vec<String>,
) -> Vec<skills::CatalogEntry> {
    skills::catalog_with_agents(&db, &agent_ids)
}

/// Replace an agent's skill set (persisted override; drives the skills
/// injected into that agent's system prompt from the next message on).
#[tauri::command]
fn set_agent_skills(
    db: State<'_, db::Db>,
    agent_id: String,
    skills: Vec<String>,
) -> Result<(), String> {
    db.set_skill_override(&agent_id, &skills)
}

/// Create a user skill (writes a real SKILL.md into the user skills dir).
#[tauri::command]
fn create_skill(name: String, description: String) -> Result<skills::CatalogEntry, String> {
    skills::create_skill(&name, &description)
}

fn main() {
    // User-created skills live in <app-data>/skills (writable), bundled
    // skills ship read-only inside the app (Marketplace > Skills).
    if let Some(dir) = db::app_data_dir() {
        skills::init_user_skills_dir(dir.join("skills"));
    }
    // open() sudah fallback ke in-memory bila file gagal; expect di sini
    // hanya untuk kegagalan total yang praktis tidak mungkin.
    let database = db::Db::open().expect("database (file/memory) harus bisa dibuka");
    let mut builder = tauri::Builder::default();
    // Native notifications (Phase 3.2: approval gate).
    builder = builder.plugin(tauri_plugin_notification::init());
    // WebDriver embedded untuk automated testing (hanya debug build).
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    }
    builder
        .manage(database)
        .manage(tools::ToolRegistry::new())
        .manage(island::IslandState(std::sync::Mutex::new(None)))
        .manage(pireel::PireelState::new())
        .manage(drive::DriveState::new())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            provider_probe,
            orchestrate_task,
            transcribe_audio_b64,
            select_model_smart,
            e2e_ramadan,
            check_e2e_trigger,
            providers::discover_models,
            providers::generate_image,
            providers::meta_generate_image,
            providers::meta_transcribe,
            providers::meta_segment,
            providers::set_dalle_key,
            providers::render_slideshow,
            providers::generate_subtitle,
            providers::generate_thumbnail,
            providers::generate_voiceover,
            providers::provider_catalog,
            providers::fetch_provider_models,
            providers::ollama_pull_model,
            providers::ollama_installed_models,
            providers::opencode_free_models,
            providers::yovoice_status,
            providers::yovoice_models,
            providers::yovoice_voices,
            providers::yovoice_tts,
            providers::audio_master,
            providers::export_timeline,
            providers::assemble_video_ai,
            providers::text_to_video,
            providers::hyperframes_assemble,
            voicestudio::voicestudio_status,
            voicestudio::voicestudio_voices,
            voicestudio::voicestudio_tts,
            voicestudio::voicestudio_clone_tts,
            voicestudio::voicestudio_open,
            social::create_postiz_draft,
            social::create_meta_draft,
            social::set_postiz_config,
            social::set_meta_token,
            social::pixabay_search_images,
            social::pixabay_search_videos,
            social::pexels_search_videos,
            social::set_pexels_key,
            social::set_pixabay_key,
            social::set_video_ai_key,
            social::fetch_image_as_data_url,
            social::pick_image_file,
            social::read_image_as_data_url,
            agent_chat,
            chat_history,
            usage_summary,
            provider_connect,
            provider_disconnect,
            routerku_set_base_url,
            send_notification,
            tools::list_tools,
            tools::get_tool_status,
            tools::set_tool_connected,
            tools::get_agent_tools,
            tools::set_agent_tools,
            tools::agent_can_use_tool,
            island::island_sync,
            island::island_get_state,
            island::island_haptic,
            logger::app_log,
            logger::read_logs,
            logger::prune_logs,
            system_permissions,
            open_privacy_settings,
            device_info,
            check_update,
            restart_app,
            list_skill_catalog,
            set_agent_skills,
            create_skill,
            pireel::pireel_connect_url,
            pireel::pireel_oauth_poll,
            pireel::pireel_status,
            pireel::pireel_disconnect,
            pireel::pireel_create_project,
            drive::drive_connect_url,
            drive::drive_oauth_poll,
            drive::drive_status,
            drive::drive_upload,
            drive::drive_disconnect,
            runtime::tool_run,
            runtime::runtime_allowed_tools,
        ])
        .setup(|app| {
            // TEMPORARY E2E TRIGGER via Rust (24 Okt 2026) — bypass WebView.
            // HAPUS setelah test selesai.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let trigger_path = std::path::Path::new("/tmp/minime_e2e_trigger");
                // Tunggu 10 detik agar app fully ready
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                // Poll setiap 5 detik, maksimal 10 menit
                for _ in 0..120 {
                    if trigger_path.exists() {
                        eprintln!("[e2e-trigger] File trigger ditemukan, jalankan E2E...");
                        // Hapus trigger agar tidak re-run
                        let _ = std::fs::remove_file(trigger_path);
                        let db = app_handle.state::<db::Db>();
                        let router = Router::new(routerku_base_url());
                        let result = e2e_test::run_ramadan_e2e(&db, &router).await;
                        eprintln!(
                            "[e2e-trigger] Selesai: success={}",
                            result.success
                        );
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                eprintln!("[e2e-trigger] Watcher selesai");
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
