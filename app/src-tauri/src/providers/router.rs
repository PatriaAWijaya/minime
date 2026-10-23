use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub struct Router {
    ollama: OllamaProvider,
    opencode: OpenCodeProvider,
    muse: OpenAiCompatProvider,
    gemini: OpenAiCompatProvider,
    deepseek: OpenAiCompatProvider,
    routerku: OpenAiCompatProvider,
}

impl Router {
    pub fn new(routerku_base_url: Option<String>) -> Self {
        Self {
            ollama: OllamaProvider::new(),
            opencode: OpenCodeProvider::new(),
            muse: OpenAiCompatProvider::muse(),
            gemini: OpenAiCompatProvider::gemini(),
            deepseek: OpenAiCompatProvider::deepseek(),
            routerku: OpenAiCompatProvider::routerku(routerku_base_url),
        }
    }

    /// Status semua provider (untuk layar Penyedia Model).
    pub fn muse(&self) -> &OpenAiCompatProvider {
        &self.muse
    }

    pub async fn statuses(&self) -> Vec<ProviderStatus> {
        // Sekuensial agar sederhana; tiap probe cepat (timeout reqwest).
        let mut out = Vec::with_capacity(6);
        out.push(self.ollama.status().await);
        out.push(self.opencode.status().await);
        out.push(self.muse.status().await);
        out.push(self.gemini.status().await);
        out.push(self.deepseek.status().await);
        out.push(self.routerku.status().await);
        out
    }

    /// Rantai provider untuk sebuah model id (`provider:model`).
    /// Model tanpa prefix provider → default tier-nya.
    fn chain_for(&self, model_id: &str, tier: ModelTier) -> Vec<ChainLink<'_>> {
        let (want_provider, _want_model) = match model_id.split_once(':') {
            Some((p, _)) => (Some(p), model_id),
            None => (None, model_id),
        };
        // Bila user memilih provider spesifik, coba itu dulu lalu fallback
        // se-tier; bila tidak, pakai urutan default tier.
        let gratis: Vec<ChainLink<'_>> = vec![
            ChainLink::Ollama(&self.ollama),
            ChainLink::OpenCode(&self.opencode),
        ];
        let berbayar: Vec<ChainLink<'_>> = vec![
            ChainLink::Compat(&self.muse),
            ChainLink::Compat(&self.gemini),
            ChainLink::Compat(&self.deepseek),
            ChainLink::Compat(&self.routerku),
        ];
        let base: Vec<ChainLink<'_>> = match tier {
            ModelTier::Gratis => gratis,
            ModelTier::Berbayar => berbayar,
        };
        match want_provider {
            Some(p) => {
                let mut chain: Vec<ChainLink<'_>> = base
                    .iter()
                    .filter(|l| l.id() == p)
                    .cloned()
                    .collect();
                for l in base {
                    if l.id() != p {
                        chain.push(l);
                    }
                }
                chain
            }
            None => base,
        }
    }

    /// Kirim chat lewat rantai provider; fallback otomatis bila gagal.
    /// Mengembalikan ChatResult dari provider yang berhasil melayani.
    pub async fn chat(
        &self,
        model_id: &str,
        tier: ModelTier,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResult, String> {
        let mut last_err = String::new();
        for link in self.chain_for(model_id, tier) {
            eprintln!("[router] coba provider: {}", link.id());
            match link.chat(model_id, &messages).await {
                Ok(res) => return Ok(res),
                Err(ProviderError::NoKey(_)) | Err(ProviderError::NotReachable(_)) => {
                    last_err = format!("{} dilewati (belum terkonfigurasi)", link.id());
                    eprintln!("[router] {last_err}");
                    continue;
                }
                Err(e) => {
                    last_err = e.to_string();
                    eprintln!("[router] {} gagal: {last_err}", link.id());
                    continue;
                }
            }
        }
        Err(if last_err.is_empty() {
            "tidak ada provider tersedia".to_string()
        } else {
            format!("semua provider gagal; terakhir: {last_err}")
        })
    }
}

#[derive(Clone)]
enum ChainLink<'a> {
    Ollama(&'a OllamaProvider),
    OpenCode(&'a OpenCodeProvider),
    Compat(&'a OpenAiCompatProvider),
}

impl<'a> ChainLink<'a> {
    fn id(&self) -> &'static str {
        match self {
            ChainLink::Ollama(_) => OllamaProvider::id(),
            ChainLink::OpenCode(_) => OpenCodeProvider::id(),
            ChainLink::Compat(p) => p.id,
        }
    }

    async fn chat(
        &self,
        model_id: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        match self {
            ChainLink::Ollama(p) => p.chat(model_id, messages).await,
            ChainLink::OpenCode(p) => p.chat(model_id, messages).await,
            ChainLink::Compat(p) => p.chat(model_id, messages).await,
        }
    }
}

/// Discover models from an OpenAI-compatible endpoint (OpenBot "Find again").
/// GET {base_url}/models, returns list of model IDs.
#[tauri::command]
pub async fn discover_models(base_url: String, api_key: Option<String>) -> Result<Vec<String>, String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = client.get(&url);
    if let Some(key) = api_key {
        let k = key.trim();
        if !k.is_empty() {
            req = req.bearer_auth(k);
        }
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let models = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(models)
}
