// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod island;
mod providers;
mod secrets;
mod skills;
mod tools;

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
    let skill_ctx = skills::skill_context(&agent_id);
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

#[tauri::command]
fn agent_send_stub(agent_id: String, _message: String) -> String {
    format!("ack:{agent_id}")
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

fn main() {
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
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            provider_probe,
            agent_chat,
            chat_history,
            usage_summary,
            provider_connect,
            provider_disconnect,
            routerku_set_base_url,
            agent_send_stub,
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
            system_permissions,
            open_privacy_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
