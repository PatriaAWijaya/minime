//! Minime Tauri thin client — proxy IPC ke minime-daemon via Unix socket.
//! Tidak menyentuh DB, tidak mengeksekusi task (rule #18: single-writer = daemon).
//! Realtime: langganan event daemon diteruskan sebagai Tauri events (Fase 2 penuh;
//! Fase 1: polling dari frontend).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

fn sock_path() -> PathBuf {
    if let Ok(dir) = std::env::var("MINIME_DATA_DIR") {
        return PathBuf::from(dir).join("minime.sock");
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("Library/Application Support/com.minime.app/minime.sock")
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{home}/.local/share")
        });
        PathBuf::from(base).join("minime/minime.sock")
    }
}

async fn call_daemon(method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let stream = UnixStream::connect(sock_path())
        .await
        .map_err(|e| format!("daemon tidak terjangkau: {e}"))?;
    let (rd, mut wr) = stream.into_split();
    let req = serde_json::json!({"id": 1, "method": method, "params": params});
    let mut msg = serde_json::to_string(&req).map_err(|e| e.to_string())?;
    msg.push('\n');
    wr.write_all(msg.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(rd);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| e.to_string())?;
    let resp: serde_json::Value =
        serde_json::from_str(line.trim()).map_err(|e| e.to_string())?;
    if resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        Ok(resp
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    } else {
        Err(resp
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error")
            .to_string())
    }
}

#[tauri::command]
async fn daemon_status() -> Result<serde_json::Value, String> {
    call_daemon("daemon.status", serde_json::json!({})).await
}

#[tauri::command]
async fn agents_list() -> Result<serde_json::Value, String> {
    call_daemon("agents.list", serde_json::json!({})).await
}

#[tauri::command]
async fn tasks_list(filter: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    call_daemon("tasks.list", filter.unwrap_or(serde_json::json!({}))).await
}

#[tauri::command]
async fn tasks_enqueue(agent_id: String, title: String) -> Result<serde_json::Value, String> {
    call_daemon(
        "tasks.enqueue",
        serde_json::json!({"agent_id": agent_id, "title": title}),
    )
    .await
}

#[tauri::command]
async fn approvals_list(status: Option<String>) -> Result<serde_json::Value, String> {
    let mut p = serde_json::json!({});
    if let Some(s) = status {
        p["status"] = serde_json::Value::String(s);
    }
    call_daemon("approvals.list", p).await
}

#[tauri::command]
async fn approvals_decide(
    approval_id: String,
    decision: String,
    note: Option<String>,
) -> Result<serde_json::Value, String> {
    call_daemon(
        "approvals.decide",
        serde_json::json!({
            "approval_id": approval_id,
            "decision": decision,
            "note": note.unwrap_or_default(),
        }),
    )
    .await
}

#[tauri::command]
async fn tools_status() -> Result<serde_json::Value, String> {
    call_daemon("tools.status", serde_json::json!({})).await
}

#[tauri::command]
async fn browser_list() -> Result<serde_json::Value, String> {
    call_daemon("browser.list", serde_json::json!({})).await
}

#[tauri::command]
async fn secrets_set(account: String, value: String) -> Result<serde_json::Value, String> {
    call_daemon("secrets.set", serde_json::json!({"account": account, "value": value})).await
}

#[tauri::command]
async fn secrets_has(account: String) -> Result<serde_json::Value, String> {
    call_daemon("secrets.has", serde_json::json!({"account": account})).await
}

#[tauri::command]
async fn usage_summary() -> Result<serde_json::Value, String> {
    call_daemon("usage.summary", serde_json::json!({})).await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            daemon_status,
            agents_list,
            tasks_list,
            tasks_enqueue,
            approvals_list,
            approvals_decide,
            tools_status,
            browser_list,
            secrets_set,
            secrets_has,
            usage_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
