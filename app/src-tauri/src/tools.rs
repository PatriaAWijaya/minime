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

/// Structured allowlist payload (contract type for the frontend / future
/// structured allowlist API; the current commands exchange plain lists).
#[allow(dead_code)]
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

    /// Allowlist gate for the runtime dispatcher.
    pub(crate) fn can_use(&self, agent_id: &str, tool_id: &str) -> bool {
        let allowlists = self.allowlists.lock().unwrap();
        can_use(&allowlists, agent_id, tool_id)
    }

    /// Allowed tool ids for an agent (for the UI tool picker).
    pub(crate) fn allowed_tools(&self, agent_id: &str) -> Vec<String> {
        self.allowlists
            .lock()
            .unwrap()
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Whether a tool id is in the known registry.
    pub(crate) fn is_known(&self, tool_id: &str) -> bool {
        known_tools().iter().any(|(id, ..)| id == tool_id)
    }

    /// Whether the tool requires an authenticated connection.
    pub(crate) fn requires_auth(&self, tool_id: &str) -> bool {
        known_tools()
            .iter()
            .find(|(id, ..)| id == tool_id)
            .map(|(_, _, _, requires_auth, _)| *requires_auth)
            .unwrap_or(false)
    }
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            status: Mutex::new(self.status.lock().unwrap().clone()),
            allowlists: Mutex::new(self.allowlists.lock().unwrap().clone()),
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
        vec!["pireel".into(), "meta".into(), "postiz".into()],
    );
    m.insert("riset".into(), vec!["browser".into()]);
    m.insert("finance".into(), Vec::new());
    m.insert(
        "strategis".into(),
        vec!["meta".into(), "postiz".into()],
    );
    m.insert("code".into(), Vec::new());
    m
}

/// Daftar tool yang dikenal aplikasi.
fn known_tools() -> Vec<(String, String, String, bool, bool)> {
    // Patria 21 Okt: hanya tool yang benar-benar ada dan terbukti bekerja.
    // meta/postiz = draft-only by design; pireel/drive = OAuth nyata; browser = fetch nyata.
    // 22 Okt: tools Media Studio dipindah ke sini (satu registry, tanpa duplikat).
    // postiz & meta TIDAK diduplikat — sudah ada di atas.
    vec![
        ("meta".into(), "Meta".into(), "social".into(), true, true),
        ("postiz".into(), "Postiz".into(), "social".into(), true, true),
        ("pireel".into(), "Pireel".into(), "video".into(), true, false),
        ("drive".into(), "Google Drive".into(), "storage".into(), true, false),
        ("browser".into(), "Browser".into(), "web".into(), false, false),
        ("pixabay".into(), "Pixabay".into(), "media".into(), true, false),
        ("ffmpeg".into(), "FFmpeg".into(), "media".into(), false, false),
        ("whisper".into(), "auto-subtitle".into(), "media".into(), false, false),
        ("arbotts".into(), "arboTTS".into(), "media".into(), false, false),
        ("sdimage".into(), "Image Gen (SD)".into(), "media".into(), false, false),
    ]
}

/// Status koneksi NYATA untuk tools media + postiz/meta (cek kondisi aktual, bukan toggle manual).
/// Tools lain tetap memakai toggle registry.
fn real_connected(tool_id: &str) -> Option<bool> {
    let has = |key: &str| {
        crate::secrets::api_key(key)
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    };
    match tool_id {
        "pixabay" => Some(has("minime.pixabay_key")),
        "postiz" => Some(has("minime.postiz_url") && has("minime.postiz_key")),
        "meta" => Some(has("minime.meta_page_id") && has("minime.meta_token")),
        "ffmpeg" => Some(crate::providers::which_ffmpeg().is_some()),
        "whisper" => Some(crate::providers::which_auto_subtitle().is_some()),
        "arbotts" => Some(crate::providers::which_arbotts().is_some()),
        "sdimage" => {
            let home = std::env::var("HOME").unwrap_or_default();
            let bin = std::path::Path::new("/Users/mac/.minime/sd/sd-server").exists()
                || std::path::Path::new(&format!("{}/.minime/sd/sd-server", home)).exists();
            let model = std::path::Path::new(&format!(
                "{}/.minime/sd/models/sd-turbo.safetensors",
                home
            ))
            .exists();
            Some(bin && model)
        }
        _ => None,
    }
}

#[tauri::command]
pub fn list_tools(registry: State<ToolRegistry>) -> Vec<ToolInfo> {
    let status = registry.status.lock().unwrap();
    known_tools()
        .into_iter()
        .map(|(id, name, category, requires_auth, draft_only)| ToolInfo {
            // Tools media: status nyata dari kondisi aktual; lainnya dari toggle.
            connected: real_connected(&id).unwrap_or_else(|| status.get(&id).copied().unwrap_or(false)),
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
    if let Some(real) = real_connected(&tool_id) {
        return real;
    }
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
    // Tools media memakai status nyata — toggle manual ditolak agar tidak tumpang tindih.
    if real_connected(&tool_id).is_some() {
        return Err("Status tool ini otomatis dari kondisi aktual.".to_string());
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
    can_use(&allowlists, &agent_id, &tool_id)
}

/// Pure gate logic (extracted so it can be unit-tested without Tauri state).
fn can_use(
    allowlists: &HashMap<String, Vec<String>>,
    agent_id: &str,
    tool_id: &str,
) -> bool {
    match allowlists.get(agent_id) {
        Some(list) => list.iter().any(|t| t == "*" || t == tool_id),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_5_tools_with_draft_only_social() {
        let tools = known_tools();
        assert_eq!(tools.len(), 10);
        let meta = tools.iter().find(|(id, ..)| id == "meta").unwrap();
        assert!(meta.4, "meta must be draft_only");
        let postiz = tools.iter().find(|(id, ..)| id == "postiz").unwrap();
        assert!(postiz.4, "postiz must be draft_only");
        // Media Studio tools: no duplicates of meta/postiz
        for id in ["pixabay", "ffmpeg", "whisper", "arbotts", "sdimage"] {
            assert!(tools.iter().any(|(tid, ..)| tid == id), "{id} missing");
        }
        assert_eq!(
            tools.iter().filter(|(id, ..)| id == "meta" || id == "postiz").count(),
            2,
            "meta/postiz must appear exactly once"
        );
    }

    #[test]
    fn default_allowlists_match_decision() {
        let m = default_allowlists();
        assert_eq!(m.get("mimi").unwrap(), &vec!["*".to_string()]);
        assert!(m.get("kreatif").unwrap().contains(&"pireel".to_string()));
        assert!(!m.get("kreatif").unwrap().contains(&"canva".to_string()));
        assert!(m.get("finance").unwrap().is_empty());
        assert!(m.get("code").unwrap().is_empty());
        assert!(m.get("riset").unwrap().contains(&"browser".to_string()));
    }

    #[test]
    fn gate_allows_and_denies_correctly() {
        let m = default_allowlists();
        // Mimi (orchestrator) may use everything.
        assert!(can_use(&m, "mimi", "pireel"));
        assert!(can_use(&m, "mimi", "drive"));
        // Finance: no tools (ruangapi belum ada).
        assert!(!can_use(&m, "finance", "ruangapi"));
        assert!(!can_use(&m, "finance", "pireel"));
        assert!(!can_use(&m, "finance", "meta"));
        // Code: no tools (github belum ada), pireel no.
        assert!(!can_use(&m, "code", "github"));
        assert!(!can_use(&m, "code", "pireel"));
        // Unknown agent: denied everything.
        assert!(!can_use(&m, "hantu", "browser"));
    }

    #[test]
    fn registry_status_roundtrip() {
        let reg = ToolRegistry::new();
        {
            let mut s = reg.status.lock().unwrap();
            s.insert("meta".to_string(), true);
        }
        let s = reg.status.lock().unwrap();
        assert_eq!(s.get("meta"), Some(&true));
        assert_eq!(s.get("drive"), None);
    }
}
