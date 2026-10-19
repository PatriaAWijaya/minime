// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod providers;
mod secrets;

use providers::{ChatMessage, ModelTier, ProviderStatus, Router};
use serde::Serialize;

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
#[tauri::command]
async fn agent_chat(
    model_id: String,
    tier: String,
    messages: Vec<ChatMessage>,
) -> Result<providers::ChatResult, String> {
    let tier = match tier.as_str() {
        "berbayar" => ModelTier::Berbayar,
        _ => ModelTier::Gratis,
    };
    let router = Router::new(routerku_base_url());
    let (text, served_by) = router.chat(&model_id, tier, messages).await?;
    Ok(providers::ChatResult { text, served_by })
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
        other => Err(format!("provider '{other}' tidak butuh API key")),
    }
}

#[tauri::command]
fn agent_send_stub(agent_id: String, _message: String) -> String {
    format!("ack:{agent_id}")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            provider_probe,
            agent_chat,
            provider_connect,
            provider_disconnect,
            routerku_set_base_url,
            agent_send_stub,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
