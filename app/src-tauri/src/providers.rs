//! Model provider abstraction + router (Tahap 2).
//!
//! Design (keputusan Patria 18 Okt 2026):
//! - GRATIS: default Ollama (lokal), fallback OpenCode (free tier).
//! - BERBAYAR: default Muse, fallback Gemini, lalu DeepSeek.
//! - RouterKu: gerbang OpenAI-compatible untuk tier berbayar (base URL
//!   konfigurabel per user; ter-block sampai gateway + saldo tersedia).
//!
//! Semua provider HTTP OpenAI-compatible (Muse, Gemini, DeepSeek, RouterKu)
//! memakai satu klien generik. Kredensial dibaca dari [`crate::secrets`]
//! (OS keychain) — tidak pernah di-hardcode.

use serde::{Deserialize, Serialize};

use crate::secrets;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Free,
    #[serde(rename = "paid")]
    PaidApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub id: String,
    pub label: String,
    pub kind: ProviderKind,
    /// "connected" | "no_key" | "unreachable" | "unknown"
    pub state: String,
    pub detail: String,
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone)]
pub enum ProviderError {
    /// Provider butuh API key yang belum dikonfigurasi → lewati ke fallback.
    NoKey(String),
    /// Provider lokal (Ollama / CLI) tidak terjangkau → lewati ke fallback.
    NotReachable(String),
    /// Error lain (HTTP, timeout, CLI) → boleh coba fallback.
    Failed(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NoKey(p) => write!(f, "provider '{p}' belum punya API key"),
            ProviderError::NotReachable(p) => write!(f, "provider '{p}' tidak terjangkau"),
            ProviderError::Failed(m) => write!(f, "{m}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Hasil chat via router: teks balasan + id provider yang melayani
/// (berguna untuk menampilkan "dijawab oleh ..." di UI).
/// Token opsional — diisi bila provider melaporkannya (untuk metering).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResult {
    pub text: String,
    pub served_by: String,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    /// Diisi bila router gagal; frontend menampilkan ini sebagai pesan agen.
    /// (Dikirim sebagai field agar tidak bergantung pada invoke rejection.)
    #[serde(default)]
    pub error: Option<String>,
}

impl ChatResult {
    fn text_only(text: String, served_by: String) -> Self {
        Self {
            text,
            served_by,
            input_tokens: None,
            output_tokens: None,
            error: None,
        }
    }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

// ---------------------------------------------------------------------------
// Ollama (gratis, lokal)
// ---------------------------------------------------------------------------

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        let base_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        Self {
            base_url,
            client: http_client(),
        }
    }

    pub fn id() -> &'static str {
        "ollama"
    }

    async fn tags(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|_| ProviderError::NotReachable("ollama".to_string()))?;
        if !resp.status().is_success() {
            return Err(ProviderError::NotReachable("ollama".to_string()));
        }
        #[derive(Deserialize)]
        struct Tags {
            models: Vec<TagModel>,
        }
        #[derive(Deserialize)]
        struct TagModel {
            name: String,
        }
        let tags: Tags = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("ollama tags: {e}")))?;
        Ok(tags
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: format!("ollama:{}", m.name),
                label: m.name,
            })
            .collect())
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        let url = format!("{}/api/chat", self.base_url);
        let model = model.strip_prefix("ollama:").unwrap_or(model);
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|_| ProviderError::NotReachable("ollama".to_string()))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "ollama chat HTTP {}",
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ChatResp {
            message: RespMsg,
            prompt_eval_count: Option<u64>,
            eval_count: Option<u64>,
        }
        #[derive(Deserialize)]
        struct RespMsg {
            content: String,
        }
        let out: ChatResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("ollama chat: {e}")))?;
        Ok(ChatResult {
            text: out.message.content,
            served_by: Self::id().to_string(),
            input_tokens: out.prompt_eval_count,
            output_tokens: out.eval_count,
        })
    }

    pub async fn status(&self) -> ProviderStatus {
        match self.tags().await {
            Ok(models) => ProviderStatus {
                id: Self::id().to_string(),
                label: "Ollama".to_string(),
                kind: ProviderKind::Free,
                state: "connected".to_string(),
                detail: format!("{} model lokal", models.len()),
                models,
            },
            Err(_) => ProviderStatus {
                id: Self::id().to_string(),
                label: "Ollama".to_string(),
                kind: ProviderKind::Free,
                state: "unreachable".to_string(),
                detail: "Ollama tidak berjalan di localhost:11434".to_string(),
                models: vec![],
            },
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Klien OpenAI-compatible generik (Muse, Gemini, DeepSeek, RouterKu)
// ---------------------------------------------------------------------------

/// Satu klien untuk semua provider HTTP yang bicara skema
/// `POST {base_url}/chat/completions` ala OpenAI.
pub struct OpenAiCompatProvider {
    id: &'static str,
    label: &'static str,
    base_url: String,
    key_account: &'static str,
    models: Vec<ModelInfo>,
    client: reqwest::Client,
}

impl OpenAiCompatProvider {
    pub fn muse() -> Self {
        Self {
            id: "muse",
            label: "Muse",
            base_url: "https://api.meta.ai/v1".to_string(),
            key_account: "muse",
            models: vec![ModelInfo {
                id: "muse:muse-spark-1.3".to_string(),
                label: "Muse Spark 1.3".to_string(),
            }],
            client: http_client(),
        }
    }

    pub fn gemini() -> Self {
        Self {
            id: "gemini",
            label: "Gemini",
            // Endpoint OpenAI-compatible resmi Google.
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            key_account: "gemini",
            models: vec![ModelInfo {
                id: "gemini:gemini-3.8-flash".to_string(),
                label: "Gemini 3.8 Flash".to_string(),
            }],
            client: http_client(),
        }
    }

    pub fn deepseek() -> Self {
        Self {
            id: "deepseek",
            label: "DeepSeek",
            base_url: "https://api.deepseek.com/v1".to_string(),
            key_account: "deepseek",
            models: vec![ModelInfo {
                id: "deepseek:deepseek-chat".to_string(),
                label: "DeepSeek Chat".to_string(),
            }],
            client: http_client(),
        }
    }

    pub fn routerku(base_url: Option<String>) -> Self {
        Self {
            id: "routerku",
            label: "RouterKu",
            base_url: base_url
                .filter(|u| !u.trim().is_empty())
                .unwrap_or_else(|| "https://organization.api-github.com/v1".to_string()),
            key_account: "routerku",
            // Katalog diisi dari GET {base_url}/models saat terhubung.
            models: vec![],
            client: http_client(),
        }
    }

    fn key(&self) -> Result<String, ProviderError> {
        secrets::api_key(self.key_account)
            .ok_or_else(|| ProviderError::NoKey(self.id.to_string()))
    }

    pub async fn status(&self) -> ProviderStatus {
        let has_key = self.key().is_ok();
        // Coba daftar model bila ada key (GET /models, skema OpenAI).
        let mut models = self.models.clone();
        let mut detail = String::new();
        let state = if !has_key {
            "no_key".to_string()
        } else {
            match self.fetch_models().await {
                Ok(fetched) => {
                    if !fetched.is_empty() {
                        models = fetched;
                    }
                    detail = format!("{} model", models.len());
                    "connected".to_string()
                }
                Err(e) => {
                    detail = e.to_string();
                    "unreachable".to_string()
                }
            }
        };
        if detail.is_empty() && !has_key {
            detail = "API key belum dikonfigurasi".to_string();
        }
        ProviderStatus {
            id: self.id.to_string(),
            label: self.label.to_string(),
            kind: ProviderKind::PaidApiKey,
            state,
            detail,
            models,
        }
    }

    async fn fetch_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let key = self.key()?;
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&key)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} models HTTP {}",
                self.id,
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ModelsResp {
            data: Vec<ModelEntry>,
        }
        #[derive(Deserialize)]
        struct ModelEntry {
            id: String,
        }
        let out: ModelsResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        Ok(out
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: format!("{}:{}", self.id, m.id),
                label: m.id,
            })
            .collect())
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        let key = self.key()?;
        let model = model
            .strip_prefix(&format!("{}:", self.id))
            .unwrap_or(model);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
        });
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} chat HTTP {}",
                self.id,
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ChatResp {
            choices: Vec<Choice>,
            usage: Option<Usage>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: RespMsg,
        }
        #[derive(Deserialize)]
        struct RespMsg {
            content: Option<String>,
        }
        #[derive(Deserialize)]
        struct Usage {
            prompt_tokens: Option<u64>,
            completion_tokens: Option<u64>,
        }
        let out: ChatResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        let text = out
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        Ok(ChatResult {
            text,
            served_by: self.id.to_string(),
            input_tokens: out.usage.as_ref().and_then(|u| u.prompt_tokens),
            output_tokens: out.usage.as_ref().and_then(|u| u.completion_tokens),
        })
    }
}

// ---------------------------------------------------------------------------
// OpenCode (gratis, via CLI)
// ---------------------------------------------------------------------------

/// Free tier via CLI `opencode` yang terpasang (pola yang sama dipakai
/// OpenBot: tanpa akun untuk tier gratis). Perintah:
/// `opencode run -m <model> <prompt>`.
pub struct OpenCodeProvider;

impl OpenCodeProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn id() -> &'static str {
        "opencode"
    }

    fn find_cli(&self) -> Option<std::path::PathBuf> {
        // 1. Env override explisit.
        if let Ok(p) = std::env::var("OPENCODE_BIN") {
            let p = std::path::PathBuf::from(p);
            if p.is_file() {
                return Some(p);
            }
        }
        // 2. PATH.
        if let Ok(paths) = std::env::var("PATH") {
            for dir in std::env::split_paths(&paths) {
                let p = dir.join("opencode");
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn free_models() -> Vec<ModelInfo> {
        // Katalog free tier (label = daftar yang disetujui Patria).
        const MODELS: &[(&str, &str)] = &[
            ("opencode:glm-5.3-flash-free", "GLM-5.3-Flash Free"),
            ("opencode:deepseek-v4.1-flash-free", "DeepSeek V4.1 Flash Free"),
            ("opencode:nemotron-3.5-lightning-free", "Nemotron 3.5 Lightning Free"),
            ("opencode:nemotron-3.5-super-vl-free", "Nemotron 3.5 Super VL Free"),
            ("opencode:nemotron-3-ultra-free", "Nemotron 3 Ultra Free"),
            ("opencode:mimo-v2.6-flash-free", "MiMo-V2.6-Flash Free"),
            ("opencode:longcat-2.5-preview-free", "LongCat 2.5 Preview Free"),
            ("opencode:beam-free", "Beam Free"),
            ("opencode:big-pickle", "Big Pickle"),
            ("opencode:space-bunny-free", "Space Bunny Free"),
        ];
        MODELS
            .iter()
            .map(|(id, label)| ModelInfo {
                id: id.to_string(),
                label: label.to_string(),
            })
            .collect()
    }

    pub async fn status(&self) -> ProviderStatus {
        match self.find_cli() {
            Some(_) => ProviderStatus {
                id: Self::id().to_string(),
                label: "OpenCode".to_string(),
                kind: ProviderKind::Free,
                state: "connected".to_string(),
                detail: "CLI terpasang".to_string(),
                models: Self::free_models(),
            },
            None => ProviderStatus {
                id: Self::id().to_string(),
                label: "OpenCode".to_string(),
                kind: ProviderKind::Free,
                state: "unreachable".to_string(),
                detail: "CLI opencode tidak ditemukan di PATH".to_string(),
                models: Self::free_models(),
            },
        }
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        let bin = self
            .find_cli()
            .ok_or_else(|| ProviderError::NotReachable("opencode".to_string()))?;
        let model = model.strip_prefix("opencode:").unwrap_or(model);
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        let out = tokio::process::Command::new(&bin)
            .args(["run", "-m", model])
            .arg(prompt)
            .output()
            .await
            .map_err(|e| ProviderError::Failed(format!("opencode: {e}")))?;
        if !out.status.success() {
            return Err(ProviderError::Failed(format!(
                "opencode exit {}: {}",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(ChatResult::text_only(text, Self::id().to_string()))
    }
}

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Tier model yang dipilih user di UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    Gratis,
    Berbayar,
}

/// Router: pilih provider sesuai tier + model, dengan fallback berantai.
/// Urutan default (keputusan Patria):
/// - Gratis: Ollama → OpenCode
/// - Berbayar: Muse → Gemini → DeepSeek (→ RouterKu bila terkonfigurasi)
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
