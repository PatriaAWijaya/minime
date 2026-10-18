//! Built-in browser (Fase 4): Chromium headless dikontrol daemon.
//!
//! Design (arsitektur §12):
//! - One isolated profile (`user-data-dir`) per session → cookies/sessions
//!   never leak between agents.
//! - Daemon-side only: keeps working when the UI is closed (rule #17).
//! - Every action is logged to `messages` as proof of work (kind='browser-action').
//! - NO automatic login to Patria's personal accounts (rule): this module
//!   exposes no credential filling; navigation/screenshot/DOM only.
//! - UI mirrors sessions via IPC `browser.list` (screenshot paths included).

use super::{BrowserSession, BrowserStore, ToolCtx, ToolStatus};
use crate::db::Db;
use anyhow::{Context, Result};
use rusqlite::params;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

const ENV_BIN: &str = "MINIME_CHROMIUM_BIN";

/// Resolve the Chromium binary:
/// env override → hyperframes' Chrome Headless Shell cache → PATH candidates.
pub fn chromium_bin() -> Option<PathBuf> {
    if let Ok(p) = std::env::var(ENV_BIN) {
        let p = PathBuf::from(p.trim());
        if p.is_file() {
            return Some(p);
        }
    }
    // HyperFrames downloads Chrome Headless Shell into its cache — reuse it.
    if let Ok(home) = std::env::var("HOME") {
        let base = PathBuf::from(home).join(".cache/hyperframes/chrome/chrome-headless-shell");
        if let Ok(entries) = std::fs::read_dir(&base) {
            let mut versions: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            versions.sort();
            for v in versions.iter().rev() {
                let p = v.join("chrome-headless-shell-linux64/chrome-headless-shell");
                if p.is_file() {
                    return Some(p);
                }
                // macOS layout (for the MacBook target)
                let p2 = v.join("chrome-headless-shell-mac-arm64/chrome-headless-shell");
                if p2.is_file() {
                    return Some(p2);
                }
            }
        }
    }
    super::find_bin(
        "MINIME_CHROMIUM_BIN_UNUSED",
        &[
            "chromium",
            "chromium-browser",
            "google-chrome",
            "google-chrome-stable",
            "chrome-headless-shell",
        ],
        &[
            "/opt/hatch/bin",
            "/usr/bin",
            "/Applications/Chromium.app/Contents/MacOS",
        ],
    )
}

pub async fn readiness(_data_dir: &Path, _repo_dir: &Path) -> ToolStatus {
    match chromium_bin() {
        Some(p) => ToolStatus::ready(
            super::TOOL_BROWSER,
            "Browser bawaan",
            "browser",
            &format!("Chromium ditemukan: {}", p.display()),
            "Riset via Gemini long-context / Agent-Reach.",
        ),
        None => ToolStatus::not(
            super::TOOL_BROWSER,
            "Browser bawaan",
            "browser",
            super::STATE_NOT_INSTALLED,
            "Chromium tidak ditemukan (coba `npx hyperframes browser ensure` atau set MINIME_CHROMIUM_BIN).",
            "Riset via Gemini long-context / Agent-Reach.",
        ),
    }
}

fn profile_dir(data_dir: &Path, session_id: &str) -> PathBuf {
    data_dir.join("browser").join(session_id)
}

fn shots_dir(data_dir: &Path, session_id: &str) -> PathBuf {
    data_dir.join("browser").join(session_id).join("shots")
}

/// Run headless Chromium once with the session's isolated profile.
async fn chromium_once(
    bin: &Path,
    profile: &Path,
    extra_args: &[&str],
    url: &str,
    timeout: Duration,
) -> Result<String> {
    let mut args: Vec<&str> = vec![
        "--headless",
        "--no-sandbox",
        "--disable-gpu",
        "--hide-scrollbars",
    ];
    let profile_arg = format!("--user-data-dir={}", profile.display());
    args.push(&profile_arg);
    args.extend_from_slice(extra_args);
    args.push(url);
    super::run_cmd(bin, &args, None, timeout).await
}

pub async fn run(
    db: &Db,
    store: &BrowserStore,
    op: &str,
    params: &Value,
    ctx: &ToolCtx,
) -> Result<Value> {
    let bin = chromium_bin().with_context(|| {
        "Chromium tidak tersedia — browser bawaan belum siap (lihat tools.status)"
    })?;
    match op {
        "open" => {
            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("about:blank");
            let id = format!("brw-{}", &Uuid::new_v4().to_string()[..8]);
            let profile = profile_dir(&ctx.data_dir, &id);
            std::fs::create_dir_all(&profile)?;
            let sess = BrowserSession {
                id: id.clone(),
                agent_id: ctx.agent_id.clone(),
                created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                last_url: url.to_string(),
                shots: 0,
            };
            store.lock().await.insert(id.clone(), sess.clone());
            log_action(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                &id,
                "open",
                url,
                None,
            )
            .await;
            Ok(json!({"session": sess}))
        }
        "navigate" => {
            let (id, url) = require_session_url(store, params).await?;
            // Stateless navigation proof: load + DOM dump (cookies persist in profile).
            let profile = profile_dir(&ctx.data_dir, &id);
            let dom = chromium_once(
                &bin,
                &profile,
                &["--dump-dom", "--virtual-time-budget=8000"],
                &url,
                Duration::from_secs(60),
            )
            .await?;
            update_last_url(store, &id, &url).await;
            let title = extract_title(&dom);
            log_action(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                &id,
                "navigate",
                &url,
                Some(&format!("title: {title}")),
            )
            .await;
            Ok(json!({"session_id": id, "url": url, "title": title}))
        }
        "screenshot" => {
            let (id, url) = require_session_url(store, params).await?;
            let profile = profile_dir(&ctx.data_dir, &id);
            let dir = shots_dir(&ctx.data_dir, &id);
            std::fs::create_dir_all(&dir)?;
            let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
            let out = dir.join(format!("shot-{ts}.png"));
            let shot_arg = format!("--screenshot={}", out.display());
            let size = params
                .get("size")
                .and_then(|v| v.as_str())
                .unwrap_or("1280,800");
            let size_arg = format!("--window-size={size}");
            chromium_once(
                &bin,
                &profile,
                &[&shot_arg, &size_arg, "--virtual-time-budget=8000"],
                &url,
                Duration::from_secs(90),
            )
            .await?;
            update_last_url(store, &id, &url).await;
            {
                let mut s = store.lock().await;
                if let Some(sess) = s.get_mut(&id) {
                    sess.shots += 1;
                }
            }
            log_action(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                &id,
                "screenshot",
                &url,
                Some(&out.display().to_string()),
            )
            .await;
            Ok(json!({"session_id": id, "path": out.to_string_lossy()}))
        }
        "dom" => {
            let (id, url) = require_session_url(store, params).await?;
            let profile = profile_dir(&ctx.data_dir, &id);
            let dom = chromium_once(
                &bin,
                &profile,
                &["--dump-dom", "--virtual-time-budget=8000"],
                &url,
                Duration::from_secs(60),
            )
            .await?;
            let text = html_to_text(&dom);
            let clipped: String = text.chars().take(6000).collect();
            log_action(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                &id,
                "dom",
                &url,
                Some(&format!("{} chars", text.len())),
            )
            .await;
            Ok(json!({"session_id": id, "url": url, "text": clipped}))
        }
        "list" => {
            let s = store.lock().await;
            let sessions: Vec<BrowserSession> = s.values().cloned().collect();
            Ok(json!({"sessions": sessions}))
        }
        "close" => {
            let id = params
                .get("session_id")
                .and_then(|v| v.as_str())
                .context("params.session_id wajib")?;
            let removed = store.lock().await.remove(id).is_some();
            log_action(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                id,
                "close",
                "",
                None,
            )
            .await;
            Ok(json!({"closed": id, "existed": removed}))
        }
        _ => anyhow::bail!(
            "op browser tidak dikenal: '{op}' (open/navigate/screenshot/dom/list/close)"
        ),
    }
}

async fn require_session_url(store: &BrowserStore, params: &Value) -> Result<(String, String)> {
    let id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .context("params.session_id wajib")?
        .to_string();
    let url = {
        let s = store.lock().await;
        let sess = s
            .get(&id)
            .with_context(|| format!("sesi browser '{id}' tidak ditemukan"))?;
        params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or(&sess.last_url)
            .to_string()
    };
    if url.trim().is_empty() || url == "about:blank" {
        anyhow::bail!("params.url wajib (sesi dibuka tanpa URL)");
    }
    Ok((id, url))
}

async fn update_last_url(store: &BrowserStore, id: &str, url: &str) {
    let mut s = store.lock().await;
    if let Some(sess) = s.get_mut(id) {
        sess.last_url = url.to_string();
    }
}

fn extract_title(dom: &str) -> String {
    let lower = dom.to_lowercase();
    let (Some(a), Some(b)) = (lower.find("<title>").map(|i| i + 7), lower.find("</title>")) else {
        return String::new();
    };
    if a >= b {
        return String::new();
    }
    // Use byte indices on the original (ASCII tags → same offsets).
    dom.get(a..b).unwrap_or("").trim().to_string()
}

/// Crude HTML → text (for agent consumption, not rendering).
fn html_to_text(dom: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let chars: Vec<char> = dom.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' {
            in_tag = true;
            let rest: String = chars[i..].iter().take(8).collect::<String>().to_lowercase();
            if rest.starts_with("<script") || rest.starts_with("<style") {
                in_script = true;
            }
            i += 1;
            continue;
        }
        if c == '>' {
            in_tag = false;
            // check if we just closed script/style
            let back: String = chars[i.saturating_sub(9)..i]
                .iter()
                .collect::<String>()
                .to_lowercase();
            if back.ends_with("/script") || back.ends_with("/style") {
                in_script = false;
            }
            if !in_script {
                out.push(' ');
            }
            i += 1;
            continue;
        }
        if !in_tag && !in_script {
            out.push(c);
        }
        i += 1;
    }
    // Decode a few entities + collapse whitespace (escape_html is the inverse;
    // here we do a light decode manually).
    let decoded = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Proof of work for browser actions (kind='browser-action').
async fn log_action(
    db: &Db,
    run_id: Option<&str>,
    agent_id: &str,
    session_id: &str,
    action: &str,
    url: &str,
    note: Option<&str>,
) {
    let id = Uuid::new_v4().to_string();
    let content = json!({
        "session_id": session_id,
        "action": action,
        "url": url,
        "note": note.unwrap_or(""),
        "at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
    .to_string();
    let guard = db.lock().await;
    guard
        .execute(
            "INSERT INTO messages (id, run_id, from_agent, to_agent, kind, content)
             VALUES (?1,?2,?3,'browser','browser-action',?4)",
            params![id, run_id, agent_id, content],
        )
        .ok();
    eprintln!("[browser] {action} session={session_id} url={url}");
}
