use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub struct ProviderCatalogEntry {
    pub id: String,
    pub name: String,
    pub local: bool,
    pub requires_key: bool,
    pub requires_base_url: bool,
    pub key_env: Option<String>,
    pub base_url_env: Option<String>,
    pub default_base_url: Option<String>,
    pub suggested: Vec<String>,
    pub keys_url: Option<String>,
    pub key_set: bool,
    pub base_url_set: bool,
}

/// Static provider catalog with suggested models, mirroring Xiryn's approach.
#[tauri::command]
pub fn provider_catalog() -> Vec<ProviderCatalogEntry> {
    let key_set = |account: &str| {
        crate::secrets::api_key(account)
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    };
    vec![
        ProviderCatalogEntry {
            id: "ollama".into(), name: "Ollama".into(), local: true,
            requires_key: false, requires_base_url: false,
            key_env: None, base_url_env: Some("OLLAMA_HOST".into()),
            default_base_url: Some("http://localhost:11434".into()),
            suggested: vec![],
            keys_url: None,
            key_set: false,
            base_url_set: std::env::var("OLLAMA_HOST").is_ok(),
        },
        ProviderCatalogEntry {
            id: "muse".into(), name: "Muse".into(), local: false,
            requires_key: true, requires_base_url: false,
            key_env: Some("MUSE_API_KEY".into()), base_url_env: None,
            default_base_url: Some("https://api.meta.ai/v1".into()),
            suggested: vec![
                "muse-spark-1.3".into(),
                "muse-wonder-1.0".into(),
                "muse-image-1.0".into(),
                "muse-voice-transcribe-1.0".into(),
                "sam-3.1".into(),
            ],
            keys_url: None,
            key_set: key_set("muse"),
            base_url_set: false,
        },
        ProviderCatalogEntry {
            id: "gemini".into(), name: "Gemini".into(), local: false,
            requires_key: true, requires_base_url: false,
            key_env: Some("GEMINI_API_KEY".into()), base_url_env: None,
            default_base_url: Some("https://generativelanguage.googleapis.com/v1beta/openai".into()),
            suggested: vec!["gemini-3.8-flash".into()],
            keys_url: Some("https://aistudio.google.com/apikey".into()),
            key_set: key_set("gemini"),
            base_url_set: false,
        },
        ProviderCatalogEntry {
            id: "deepseek".into(), name: "DeepSeek".into(), local: false,
            requires_key: true, requires_base_url: false,
            key_env: Some("DEEPSEEK_API_KEY".into()), base_url_env: None,
            default_base_url: Some("https://api.deepseek.com/v1".into()),
            suggested: vec!["deepseek-chat".into()],
            keys_url: Some("https://platform.deepseek.com/api_keys".into()),
            key_set: key_set("deepseek"),
            base_url_set: false,
        },
        ProviderCatalogEntry {
            id: "routerku".into(), name: "RouterKu".into(), local: false,
            requires_key: true, requires_base_url: true,
            key_env: Some("ROUTERKU_API_KEY".into()), base_url_env: Some("ROUTERKU_BASE_URL".into()),
            default_base_url: None,
            suggested: vec![],
            keys_url: Some("https://my.routerku.com".into()),
            key_set: key_set("routerku"),
            base_url_set: crate::secrets::api_key("minime.routerku_url")
                .map(|u| !u.trim().is_empty()).unwrap_or(false),
        },
        ProviderCatalogEntry {
            id: "opencode".into(), name: "OpenCode".into(), local: true,
            requires_key: false, requires_base_url: false,
            key_env: None, base_url_env: None,
            default_base_url: None,
            suggested: vec![],
            keys_url: None,
            key_set: false,
            base_url_set: false,
        },
    ]
}

/// Fetch live models from a provider (Xiryn-style `fetchProviderModels`).
/// Returns model ids; used by Test connection and the model picker.
#[tauri::command]
pub async fn fetch_provider_models(provider_id: String) -> Result<Vec<String>, String> {
    match provider_id.as_str() {
        "ollama" => {
            let p = OllamaProvider::new();
            p.tags().await
                .map(|m| m.into_iter().map(|x| x.label).collect())
                .map_err(|e| e.to_string())
        }
        "muse" | "gemini" | "deepseek" => {
            let p = match provider_id.as_str() {
                "muse" => OpenAiCompatProvider::muse(),
                "gemini" => OpenAiCompatProvider::gemini(),
                _ => OpenAiCompatProvider::deepseek(),
            };
            p.fetch_models().await
                .map(|m| m.into_iter().map(|x| x.label).collect())
                .map_err(|e| e.to_string())
        }
        "routerku" => {
            let base = crate::secrets::api_key("minime.routerku_url").unwrap_or_default();
            let p = OpenAiCompatProvider::routerku(
                if base.trim().is_empty() { None } else { Some(base) }
            );
            p.fetch_models().await
                .map(|m| m.into_iter().map(|x| x.label).collect())
                .map_err(|e| e.to_string())
        }
        "opencode" => {
            Ok(OpenCodeProvider::free_models()
                .into_iter()
                .map(|m| m.label)
                .collect())
        }
        _ => Err(format!("Provider tidak dikenal: {}", provider_id)),
    }
}

/// Download (pull) an Ollama model. Returns when pull completes.
#[tauri::command]
pub async fn ollama_pull_model(model_name: String) -> Result<String, String> {
    let base = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1800))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(format!("{}/api/pull", base))
        .json(&serde_json::json!({ "name": model_name, "stream": false }))
        .send()
        .await
        .map_err(|e| format!("Ollama tidak terjangkau: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Pull gagal: HTTP {}", resp.status()));
    }
    Ok(format!("Model {} berhasil diunduh", model_name))
}

/// Real installed Ollama models (from /api/tags) — NOT a dummy list.
/// Returns [{id, size_gb, modified}]. Empty if Ollama is not running.
#[tauri::command]
pub async fn ollama_installed_models() -> Vec<serde_json::Value> {
    let base = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let resp = client.get(format!("{}/api/tags", base)).send().await;
    let resp = match resp {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let data: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    let models = data.get("models").and_then(|m| m.as_array()).cloned().unwrap_or_default();
    models
        .iter()
        .map(|m| {
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let gb = m.get("size").and_then(|s| s.as_u64()).unwrap_or(0) as f64 / 1e9;
            serde_json::json!({
                "id": name,
                "size": format!("{:.1} GB", gb),
                "modified": m.get("modified_at").and_then(|t| t.as_str()).unwrap_or("").chars().take(10).collect::<String>(),
            })
        })
        .collect()
}

/// Real free models from OpenCode Zen (https://opencode.ai/zen/v1/models).
/// Models with "-free" in ID work without API key inside OpenCode. NOT a dummy list.
#[tauri::command]
pub async fn opencode_free_models() -> Vec<serde_json::Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let resp = client.get("https://opencode.ai/zen/v1/models").send().await;
    let resp = match resp {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let data: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    let models: Vec<serde_json::Value> = if let Some(arr) = data.as_array() {
        arr.clone()
    } else {
        data.get("data")
            .or(data.get("models"))
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default()
    };
    models
        .iter()
        .filter(|m| m.get("id").and_then(|i| i.as_str()).map(|id| id.contains("-free")).unwrap_or(false))
        .map(|m| {
            let id = m.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            serde_json::json!({
                "id": id,
                "name": m.get("name").and_then(|n| n.as_str()).unwrap_or(id),
                "context": m.get("context_length").or(m.get("contextLength")).map(|c| c.to_string()).unwrap_or_default(),
            })
        })
        .collect()
}

/// Path to yovoice CLI binary.
pub(crate) fn yovoice_bin() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/.minime/yovoice/yovoice", home)
}
