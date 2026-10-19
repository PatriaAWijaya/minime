// Tools gateway — Tahap 3 (keputusan Patria 19 Okt).
// Registry peralatan eksternal: status koneksi, allowlist per agen.
// Kredensial disimpan di OS keychain (via secrets.rs), tidak di kode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub connected: bool,
    pub requires_auth: bool,
    pub draft_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolAllowlist {
    pub agent_id: String,
    pub tool_ids: Vec<String>,
}

pub struct ToolRegistry {
    // tool_id -> connected
    status: Mutex<HashMap<String, bool>>,
    // agent_id -> allowed tool_ids
    allowlists: Mutex<HashMap<String, Vec<String>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(HashMap::new()),
            allowlists: Mutex::new(default_allowlists()),
        }
    }
}

/// Allowlist default per agen (keputusan Patria 19 Okt):
/// - kreatif → pireel, canva, meta, postiz
/// - riset → web (browser), ruangapi
/// - finance → ruangapi
/// - strategis → meta, postiz, ruangapi
/// - code → github, ruangapi
/// - mimi (CIO) → semua (orkestrator)
fn default_allowlists() -> HashMap<String, Vec<String>> {
    let mut m = HashMap::new();
    m.insert("mimi".into(), vec!["*".into()]);
    m.insert(
        "kreatif".into(),
        vec!["pireel".into(), "canva".into(), "meta".into(), "postiz".into()],
    );
    m.insert("riset".into(), vec!["browser".into(), "ruangapi".into()]);
    m.insert("finance".into(), vec!["ruangapi".into()]);
    m.insert(
        "strategis".into(),
        vec!["meta".into(), "postiz".into(), "ruangapi".into()],
    );
    m.insert("code".into(), vec!["github".into(), "ruangapi".into()]);
    m
}

/// Daftar tool yang dikenal aplikasi.
fn known_tools() -> Vec<(String, String, String, bool, bool)> {
    vec![
        ("meta".into(), "Meta".into(), "social".into(), true, true),
        ("postiz".into(), "Postiz".into(), "social".into(), true, true),
        ("pireel".into(), "Pireel".into(), "video".into(), true, false),
        ("drive".into(), "Google Drive".into(), "storage".into(), true, false),
        ("slack".into(), "Slack".into(), "chat".into(), true, false),
        ("discord".into(), "Discord".into(), "chat".into(), true, false),
        ("telegram".into(), "Telegram".into(), "chat".into(), true, false),
        ("github".into(), "GitHub".into(), "code".into(), true, false),
        ("notion".into(), "Notion".into(), "docs".into(), true, false),
        ("canva".into(), "Canva".into(), "design".into(), true, false),
        ("ruangapi".into(), "RuangAPI".into(), "api".into(), true, false),
        ("browser".into(), "Browser".into(), "web".into(), false, false),
    ]
}

#[tauri::command]
pub fn list_tools(registry: State<ToolRegistry>) -> Vec<ToolInfo> {
    let status = registry.status.lock().unwrap();
    known_tools()
        .into_iter()
        .map(|(id, name, category, requires_auth, draft_only)| ToolInfo {
            connected: status.get(&id).copied().unwrap_or(false),
            id,
            name,
            category,
            requires_auth,
            draft_only,
        })
        .collect()
}

#[tauri::command]
pub fn get_tool_status(tool_id: String, registry: State<ToolRegistry>) -> bool {
    registry
        .status
        .lock()
        .unwrap()
        .get(&tool_id)
        .copied()
        .unwrap_or(false)
}

#[tauri::command]
pub fn set_tool_connected(
    tool_id: String,
    connected: bool,
    registry: State<ToolRegistry>,
) -> Result<(), String> {
    let known: Vec<String> = known_tools().into_iter().map(|(id, _, _, _, _)| id).collect();
    if !known.contains(&tool_id) {
        return Err(format!("Tool tidak dikenal: {}", tool_id));
    }
    registry
        .status
        .lock()
        .unwrap()
        .insert(tool_id, connected);
    Ok(())
}

#[tauri::command]
pub fn get_agent_tools(
    agent_id: String,
    registry: State<ToolRegistry>,
) -> Vec<String> {
    registry
        .allowlists
        .lock()
        .unwrap()
        .get(&agent_id)
        .cloned()
        .unwrap_or_default()
}

#[tauri::command]
pub fn set_agent_tools(
    agent_id: String,
    tool_ids: Vec<String>,
    registry: State<ToolRegistry>,
) -> Result<(), String> {
    // Validasi: hanya tool yang dikenal.
    let known: Vec<String> = known_tools().into_iter().map(|(id, _, _, _, _)| id).collect();
    for tid in &tool_ids {
        if tid != "*" && !known.contains(tid) {
            return Err(format!("Tool tidak dikenal: {}", tid));
        }
    }
    registry
        .allowlists
        .lock()
        .unwrap()
        .insert(agent_id, tool_ids);
    Ok(())
}

/// Cek apakah agen boleh memakai tool (untuk gate sebelum eksekusi).
#[tauri::command]
pub fn agent_can_use_tool(
    agent_id: String,
    tool_id: String,
    registry: State<ToolRegistry>,
) -> bool {
    let allowlists = registry.allowlists.lock().unwrap();
    match allowlists.get(&agent_id) {
        Some(list) => list.contains(&"*".to_string()) || list.contains(&tool_id),
        None => false,
    }
}
