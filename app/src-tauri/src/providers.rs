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
            error: None,
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
            error: None,
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
        // 3. Lokasi instalasi standar opencode (lihat README anomalyco/opencode
        //    "Installation Directory"): $HOME/bin lalu $HOME/.opencode/bin.
        if let Ok(home) = std::env::var("HOME") {
            for sub in ["bin/opencode", ".opencode/bin/opencode"] {
                let p = std::path::PathBuf::from(&home).join(sub);
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

/// Generate image via OpenAI DALL-E (AI Media Studio).
/// API key dari OS keychain (account: minime.dalle).
#[tauri::command]
pub async fn generate_image(prompt: String, size: String) -> Result<String, String> {
    let api_key = secrets::api_key("minime.dalle")
        .ok_or_else(|| "DALL-E API key belum diset. Masukkan di Pengaturan > Media Studio.".to_string())?;
    if api_key.trim().is_empty() {
        return Err("DALL-E API key kosong.".to_string());
    }
    let sz = match size.as_str() {
        "square" => "1024x1024",
        "portrait" => "1024x1792",
        "landscape" => "1792x1024",
        _ => "1024x1024",
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post("https://api.openai.com/v1/images/generations")
        .bearer_auth(api_key.trim())
        .json(&serde_json::json!({
            "model": "dall-e-3",
            "prompt": prompt,
            "size": sz,
            "n": 1,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!("DALL-E error: {}", txt.chars().take(200).collect::<String>()));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    body.get("data")
        .and_then(|d| d.get(0))
        .and_then(|m| m.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "URL gambar tidak ditemukan di respons.".to_string())
}

/// Simpan API key DALL-E ke keychain.
#[tauri::command]
pub fn set_dalle_key(api_key: String) -> Result<(), String> {
    secrets::set_api_key("minime.dalle", api_key.trim()).map_err(|e| e.to_string())
}

/// Render video slideshow dari daftar image URL/path via FFmpeg.
/// images: list path file lokal. duration_per: detik per gambar.
/// Return: path file MP4 hasil.
#[tauri::command]
pub async fn render_slideshow(images: Vec<String>, duration_per: f64, output_name: String) -> Result<String, String> {
    if images.is_empty() {
        return Err("Tidak ada gambar untuk dirender.".to_string());
    }
    let ffmpeg = which_ffmpeg().ok_or_else(|| "FFmpeg tidak ditemukan di sistem.".to_string())?;

    let out_dir = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let safe_name: String = output_name.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    let out_path = out_dir.join(format!("{}.mp4", if safe_name.is_empty() { "video".to_string() } else { safe_name }));

    // Buat file list untuk concat
    let list_path = out_dir.join("ffmpeg_list.txt");
    let mut list_content = String::new();
    for img in &images {
        let escaped = img.replace('\'', "'\\''");
        list_content.push_str(&format!("file '{}'\nduration {}\n", escaped, duration_per));
    }
    // FFmpeg concat butuh file terakhir diulang (tanpa duration)
    if let Some(last) = images.last() {
        let escaped = last.replace('\'', "'\\''");
        list_content.push_str(&format!("file '{}'\n", escaped));
    }
    std::fs::write(&list_path, list_content).map_err(|e| e.to_string())?;

    let status = tokio::process::Command::new(&ffmpeg)
        .args([
            "-y", "-f", "concat", "-safe", "0",
            "-i", list_path.to_str().unwrap(),
            "-vf", "scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,format=yuv420p",
            "-c:v", "libx264", "-preset", "fast", "-crf", "23",
            "-movflags", "+faststart",
            out_path.to_str().unwrap(),
        ])
        .status()
        .await
        .map_err(|e| format!("Gagal menjalankan FFmpeg: {}", e))?;

    if !status.success() {
        return Err("FFmpeg gagal merender video.".to_string());
    }
    out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path output tidak valid.".to_string())
}

fn which_ffmpeg() -> Option<String> {
    for p in ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg", "/usr/bin/ffmpeg"] {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    None
}

/// Generate subtitle otomatis dari video via auto-subtitle (Whisper + FFmpeg).
/// video_path: path file video lokal. model: tiny/base/small/medium (default small).
/// Return: path file video dengan subtitle burned-in.
#[tauri::command]
pub async fn generate_subtitle(video_path: String, model: String) -> Result<String, String> {
    let auto_sub = which_auto_subtitle().ok_or_else(|| "auto-subtitle tidak ditemukan.".to_string())?;
    let ffmpeg = which_ffmpeg().ok_or_else(|| "FFmpeg tidak ditemukan.".to_string())?;
    let _ = ffmpeg; // auto-subtitle butuh ffmpeg di PATH

    if !std::path::Path::new(&video_path).exists() {
        return Err("File video tidak ditemukan.".to_string());
    }

    let model = match model.as_str() {
        "tiny" | "base" | "small" | "medium" => model,
        _ => "small".to_string(),
    };

    let out_dir = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    // auto_subtitle <video> -o <out_dir> --model <model>
    let status = tokio::process::Command::new(&auto_sub)
        .args([video_path.as_str(), "-o", out_dir.to_str().unwrap(), "--model", model.as_str()])
        .env("PATH", format!("/opt/homebrew/bin:/usr/local/bin:{}", std::env::var("PATH").unwrap_or_default()))
        .status()
        .await
        .map_err(|e| format!("Gagal menjalankan auto-subtitle: {}", e))?;

    if !status.success() {
        return Err("auto-subtitle gagal memproses video.".to_string());
    }

    // Cari file output (nama sama + _subtitled atau sejenisnya)
    let video_name = std::path::Path::new(&video_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");
    for entry in std::fs::read_dir(&out_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(video_name) && name.ends_with(".mp4") && !name.contains("_tmp") {
            return Ok(entry.path().to_string_lossy().to_string());
        }
    }
    Err("File output tidak ditemukan.".to_string())
}

fn which_auto_subtitle() -> Option<String> {
    for p in [
        "/Library/Frameworks/Python.framework/Versions/3.13/bin/auto_subtitle",
        "/opt/homebrew/bin/auto_subtitle",
        "/usr/local/bin/auto_subtitle",
    ] {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    None
}

/// Generate thumbnail YouTube via youtube-thumbnail-generator.
/// title: teks utama, output_name: nama file.
/// Return: path file PNG hasil.
#[tauri::command]
pub async fn generate_thumbnail(title: String, output_name: String) -> Result<String, String> {
    let out_dir = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let safe_name: String = output_name.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    let out_path = out_dir.join(format!("{}.png", if safe_name.is_empty() { "thumb".to_string() } else { safe_name }));

    // Gunakan Python API langsung
    let py_code = format!(
        "from youtube_thumbnail_generator import generate_thumbnail as gen; gen('{}', '{}')",
        title.replace('\'', "\\'"),
        out_path.to_str().unwrap().replace('\'', "\\'")
    );
    let status = tokio::process::Command::new("python3")
        .args(["-c", &py_code])
        .env("PATH", format!("/Library/Frameworks/Python.framework/Versions/3.13/bin:/opt/homebrew/bin:/usr/local/bin:{}", std::env::var("PATH").unwrap_or_default()))
        .status()
        .await
        .map_err(|e| format!("Gagal menjalankan thumbnail generator: {}", e))?;

    if !status.success() || !out_path.exists() {
        return Err("Thumbnail generator gagal.".to_string());
    }
    out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path tidak valid.".to_string())
}

/// Generate voiceover Bahasa Indonesia via arboTTS.
/// text: teks narasi, persona: arbo/bimo/dimas/danu/arba/sita/maya, output_name: nama file.
/// Return: path file WAV hasil.
#[tauri::command]
pub async fn generate_voiceover(text: String, persona: String, output_name: String) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Teks kosong.".to_string());
    }
    let arbotts = which_arbotts().ok_or_else(|| "arboTTS tidak ditemukan.".to_string())?;

    let valid_persona = match persona.as_str() {
        "arbo" | "bimo" | "dimas" | "danu" | "arba" | "sita" | "maya" => persona,
        _ => "arbo".to_string(),
    };

    let out_dir = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let safe_name: String = output_name.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    let out_path = out_dir.join(format!("{}.wav", if safe_name.is_empty() { "voiceover".to_string() } else { safe_name }));

    let status = tokio::process::Command::new(&arbotts)
        .args([
            text.trim(),
            "-o", out_path.to_str().unwrap(),
            "-l", "id",
            "-sp", valid_persona.as_str(),
        ])
        .env("PATH", format!("/opt/homebrew/bin:/usr/local/bin:/Library/Frameworks/Python.framework/Versions/3.13/bin:{}", std::env::var("PATH").unwrap_or_default()))
        .status()
        .await
        .map_err(|e| format!("Gagal menjalankan arboTTS: {}", e))?;

    if !status.success() || !out_path.exists() {
        return Err("arboTTS gagal generate voiceover.".to_string());
    }
    out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path tidak valid.".to_string())
}

fn which_arbotts() -> Option<String> {
    for p in [
        "/Library/Frameworks/Python.framework/Versions/3.13/bin/arbotts",
        "/opt/homebrew/bin/arbotts",
        "/usr/local/bin/arbotts",
    ] {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    None
}
