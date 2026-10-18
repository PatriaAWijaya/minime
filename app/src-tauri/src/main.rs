// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

/// Basic app metadata exposed to the frontend.
/// Phase 2 will extend this with agent runtime state.
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
        phase: "tahap-1-ui".to_string(),
    }
}

/// Placeholder command. Real agent execution (model router + skills)
/// lands in Phase 2; the frontend currently runs a local mock.
#[tauri::command]
fn agent_send_stub(agent_id: String, _message: String) -> String {
    format!("ack:{agent_id}")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_info, agent_send_stub])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
