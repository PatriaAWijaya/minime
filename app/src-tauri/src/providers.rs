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

    pub async fn tags(&self) -> Result<Vec<ModelInfo>, ProviderError> {
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

    pub async fn fetch_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
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

/// Generate image via local stable-diffusion.cpp (AI Media Studio).
/// sd-server runs on localhost:8080 (auto-started if needed).
#[tauri::command]
pub async fn generate_image(prompt: String, size: String) -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME tidak ditemukan.".to_string())?;
    let sd_dir = format!("{}/.minime/sd", home);
    let server_bin = format!("{}/sd-server", sd_dir);
    let model_path = format!("{}/models/sd-turbo.safetensors", sd_dir);
    if !std::path::Path::new(&server_bin).exists() {
        return Err("sd-server belum terinstall di ~/.minime/sd.".to_string());
    }
    if !std::path::Path::new(&model_path).exists() {
        return Err("Model SD masih diunduh, coba lagi nanti.".to_string());
    }

    // Pastikan server jalan
    ensure_sd_server(&server_bin, &model_path).await?;

    let (w, h) = match size.as_str() {
        "square" => (1024, 1024),
        "portrait45" => (1080, 1350),
        "portrait" => (1024, 1792),
        "landscape" => (1792, 1024),
        _ => (1024, 1024),
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post("http://127.0.0.1:8080/v1/images/generations")
        .json(&serde_json::json!({
            "prompt": prompt,
            "n": 1,
            "size": format!("{}x{}", w, h),
        }))
        .send()
        .await
        .map_err(|e| format!("SD server error: {}", e))?;
    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!("SD error: {}", txt.chars().take(200).collect::<String>()));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    // sd-server bisa return b64_json atau url
    if let Some(b64) = body.get("data").and_then(|d| d.get(0)).and_then(|m| m.get("b64_json")).and_then(|v| v.as_str()) {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|e| e.to_string())?;
        let out_dir = std::env::temp_dir().join("minime-studio");
        std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let out_path = out_dir.join(format!("sd-{}.png", chrono::Utc::now().timestamp()));
        std::fs::write(&out_path, bytes).map_err(|e| e.to_string())?;
        return out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path tidak valid.".to_string());
    }
    if let Some(url) = body.get("data").and_then(|d| d.get(0)).and_then(|m| m.get("url")).and_then(|v| v.as_str()) {
        return Ok(url.to_string());
    }
    Err("Respons SD tidak dikenali.".to_string())
}

async fn ensure_sd_server(server_bin: &str, model_path: &str) -> Result<(), String> {
    // Cek apakah server sudah jalan
    if reqwest::get("http://127.0.0.1:8080/").await.map(|r| r.status().is_success()).unwrap_or(false) {
        return Ok(());
    }
    // Start server di background
    tokio::process::Command::new(server_bin)
        .args(["-m", model_path, "--listen-port", "8080", "-l", "127.0.0.1"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Gagal start sd-server: {}", e))?;
    // Tunggu sampai siap (max 60 detik)
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if reqwest::get("http://127.0.0.1:8080/").await.map(|r| r.status().is_success()).unwrap_or(false) {
            return Ok(());
        }
    }
    Err("SD server tidak merespons.".to_string())
}

/// Simpan API key DALL-E ke keychain.
#[tauri::command]
pub fn set_dalle_key(api_key: String) -> Result<(), String> {
    secrets::set_api_key("minime.dalle", api_key.trim()).map_err(|e| e.to_string())
}

/// Render a slideshow video from image paths via FFmpeg.
/// images: list of local paths. duration_per: seconds per image.
/// Returns: MP4 file path.
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

pub(crate) fn which_ffmpeg() -> Option<String> {
    for p in ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg", "/usr/bin/ffmpeg"] {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    None
}

/// Auto-generate subtitles from video via auto-subtitle (Whisper + FFmpeg).
/// video_path: local video path. model: tiny/base/small/medium (default small).
/// Returns: video path with burned-in subtitles.
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

pub(crate) fn which_auto_subtitle() -> Option<String> {
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

/// Generate a YouTube thumbnail (1600x900) via Pillow.
/// title: main text, output_name: file name.
/// Returns: PNG file path.
#[tauri::command]
pub async fn generate_thumbnail(title: String, output_name: String) -> Result<String, String> {
    let out_dir = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let safe_name: String = output_name.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    let out_path = out_dir.join(format!("{}.png", if safe_name.is_empty() { "thumb".to_string() } else { safe_name }));

    // Gunakan Python API langsung
    let esc_title: String = title.chars().filter(|c| *c != '"' && *c != '\'').collect();
    let out_s = out_path.to_str().unwrap().replace('\'', "");
    let py_code = format!(
        "from PIL import Image, ImageDraw; img=Image.new('RGB',(1600,900),(18,18,28)); d=ImageDraw.Draw(img); d.text((80,380), \"{}\", fill=(255,255,255)); d.text((80,480), 'Minime', fill=(120,120,140)); img.save('{}')",
        esc_title, out_s
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

/// Generate Indonesian voiceover via arboTTS.
/// text: narration text, persona: arbo/bimo/dimas/danu/arba/sita/maya, output_name: file name.
/// Returns: WAV file path.
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

pub(crate) fn which_arbotts() -> Option<String> {
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

// ---------------------------------------------------------------------------
// Provider catalog (Xiryn-style): metadata for settings UI
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, Clone)]
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
            suggested: vec!["muse-spark-1.3".into()],
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

/// Check yovoice status. Returns JSON string from `yovoice status --json`.
#[tauri::command]
pub fn yovoice_status() -> Result<String, String> {
    let bin = yovoice_bin();
    if !std::path::Path::new(&bin).exists() {
        return Err("yovoice belum terinstall di ~/.minime/yovoice.".to_string());
    }
    let out = std::process::Command::new(&bin)
        .args(["status", "--json"])
        .output()
        .map_err(|e| format!("Gagal menjalankan yovoice: {}", e))?;
    if !out.status.success() {
        return Err("yovoice status gagal.".to_string());
    }
    String::from_utf8(out.stdout).map_err(|e| e.to_string())
}

/// List installed yovoice models. Returns JSON array [{id, name, size}].
#[tauri::command]
pub fn yovoice_models() -> Result<String, String> {
    let status = yovoice_status()?;
    let v: serde_json::Value = serde_json::from_str(&status).map_err(|e| e.to_string())?;
    let installed = v
        .pointer("/state/models")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();
    let catalog = v
        .pointer("/catalog")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    let list: Vec<serde_json::Value> = installed
        .iter()
        .map(|m| {
            let id = m.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            let cat = catalog.iter().find(|c| c.get("id").and_then(|i| i.as_str()) == Some(id));
            serde_json::json!({
                "id": id,
                "name": cat.and_then(|c| c.get("name")).and_then(|n| n.as_str()).unwrap_or(id),
                "size": cat.and_then(|c| c.get("size")).and_then(|s| s.as_u64()).map(|b| format!("{:.0} MB", b as f64 / 1e6)).unwrap_or_default(),
            })
        })
        .collect();
    serde_json::to_string(&list).map_err(|e| e.to_string())
}

/// List available yovoice voices: saved/imported voices plus Kokoro built-in speakers.
/// Returns JSON array [{id, name, kind}] where kind = "saved" | "kokoro".
#[tauri::command]
pub fn yovoice_voices() -> Result<String, String> {
    let bin = yovoice_bin();
    if !std::path::Path::new(&bin).exists() {
        return Err("yovoice belum terinstall.".to_string());
    }
    let mut list: Vec<serde_json::Value> = Vec::new();
    // Saved / imported voices
    if let Ok(out) = std::process::Command::new(&bin).args(["voices", "list"]).output() {
        if out.status.success() {
            if let Ok(arr) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(items) = arr.as_array() {
                    for v in items {
                        let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
                        if id.is_empty() {
                            continue;
                        }
                        list.push(serde_json::json!({
                            "id": id,
                            "name": v.get("name").and_then(|n| n.as_str()).unwrap_or(id),
                            "kind": "saved",
                        }));
                    }
                }
            }
        }
    }
    // Kokoro built-in speakers (from model catalog)
    if let Ok(status) = yovoice_status() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&status) {
            if let Some(catalog) = v.pointer("/catalog").and_then(|c| c.as_array()) {
                for c in catalog {
                    let family = c.get("family").and_then(|f| f.as_str()).unwrap_or("");
                    if family != "kokoro_tts" {
                        continue;
                    }
                    if let Some(speakers) = c.get("voices").and_then(|s| s.as_array()) {
                        for s in speakers {
                            if let Some(id) = s.as_str() {
                                if !list.iter().any(|e| e.get("id").and_then(|i| i.as_str()) == Some(id)) {
                                    list.push(serde_json::json!({
                                        "id": id,
                                        "name": id,
                                        "kind": "kokoro",
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    serde_json::to_string(&list).map_err(|e| e.to_string())
}

/// Generate TTS via yovoice. Returns WAV file path.
/// `model`: yovoice model id (e.g. "kokoro-82m-q8"); defaults to first installed.
/// `voice`: voice id — saved voice id, or Kokoro built-in speaker id (routed automatically).
#[tauri::command]
pub async fn yovoice_tts(
    text: String,
    voice: String,
    reference: String,
    output_name: String,
    model: String,
) -> Result<String, String> {
    let bin = yovoice_bin();
    if !std::path::Path::new(&bin).exists() {
        return Err("yovoice belum terinstall.".to_string());
    }
    if text.trim().is_empty() {
        return Err("Teks kosong.".to_string());
    }
    // Resolve model: explicit > first installed > error
    let model_id = if !model.trim().is_empty() {
        model.trim().to_string()
    } else {
        let models_json = yovoice_models()?;
        let arr: serde_json::Value =
            serde_json::from_str(&models_json).map_err(|e| e.to_string())?;
        arr.as_array()
            .and_then(|a| a.first())
            .and_then(|m| m.get("id"))
            .and_then(|i| i.as_str())
            .ok_or_else(|| "Tidak ada model yovoice terinstall.".to_string())?
            .to_string()
    };
    let tmp = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let text_file = tmp.join(format!("{}.txt", output_name));
    std::fs::write(&text_file, &text).map_err(|e| e.to_string())?;
    let out_path = tmp.join(format!("{}.wav", output_name));
    let mut cmd = tokio::process::Command::new(&bin);
    cmd.args([
        "generate",
        "--text-file",
        &text_file.to_string_lossy(),
        "--model",
        &model_id,
        "--output",
        &out_path.to_string_lossy(),
    ]);
    // Route voice id: saved voice -> --voice, Kokoro built-in speaker -> --speaker
    let voice_id = voice.trim();
    if !voice_id.is_empty() {
        let is_saved = yovoice_voices().ok().and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
            .and_then(|v| v.as_array().cloned()).unwrap_or_default()
            .iter()
            .any(|e| {
                e.get("id").and_then(|i| i.as_str()) == Some(voice_id)
                    && e.get("kind").and_then(|k| k.as_str()) == Some("saved")
            });
        if is_saved {
            cmd.args(["--voice", voice_id]);
        } else {
            cmd.args(["--speaker", voice_id]);
        }
    }
    // Voice cloning: reference audio (5-15 detik, suara yang diizinkan saja).
    let ref_path = reference.trim();
    if !ref_path.is_empty() {
        if !std::path::Path::new(ref_path).exists() {
            return Err(format!("File referensi tidak ditemukan: {}", ref_path));
        }
        cmd.args(["--reference", ref_path]);
        // VoxCPM2 needs explicit clone mode when a reference is given
        if model_id.starts_with("voxcpm") {
            cmd.args(["--vox-mode", "clone"]);
        }
    }
    let out = cmd.output().await.map_err(|e| format!("Gagal menjalankan yovoice: {}", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let msg = format!("{}{}", stdout, err);
        // CLI prints JSON error like {"error":"..."} — surface the message
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&msg) {
            if let Some(e) = j.get("error").and_then(|e| e.as_str()) {
                return Err(format!("yovoice: {}", e));
            }
        }
        return Err(format!("yovoice gagal: {}", msg.chars().take(300).collect::<String>()));
    }
    Ok(out_path.to_string_lossy().to_string())
}

/// Audio mastering: loudness normalization (EBU R128) + denoise via FFmpeg.
/// Returns mastered audio file path.
#[tauri::command]
pub async fn audio_master(input_path: String, output_name: String) -> Result<String, String> {
    let tmp = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let out_path = tmp.join(format!("{}_mastered.wav", output_name));
    // loudnorm (EBU R128, target YouTube/TikTok) + highpass + afftdn denoise
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-y", "-loglevel", "error",
            "-i", &input_path,
            "-af", "highpass=f=80,afftdn=nf=-25,loudnorm=I=-14:TP=-1.5:LRA=11",
            "-ar", "44100", "-ac", "2",
            &out_path.to_string_lossy(),
        ])
        .status()
        .await
        .map_err(|e| format!("FFmpeg tidak ditemukan: {}", e))?;
    if !status.success() {
        return Err("Audio mastering gagal.".to_string());
    }
    Ok(out_path.to_string_lossy().to_string())
}

/// Export multi-track timeline ke MP4 via FFmpeg.
/// timeline_json: { tracks: [{id, kind, name}], clips: [{id, trackId, src, name, start, duration, in, volume, text}] }
#[tauri::command]
pub async fn export_timeline(timeline_json: String, output_name: String) -> Result<String, String> {
    let tl: serde_json::Value =
        serde_json::from_str(&timeline_json).map_err(|e| format!("JSON timeline tidak valid: {}", e))?;
    let tracks = tl.get("tracks").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    let clips = tl.get("clips").and_then(|c| c.as_array()).cloned().unwrap_or_default();

    let kind_of = |track_id: &str| -> &str {
        tracks
            .iter()
            .find(|t| t.get("id").and_then(|i| i.as_str()) == Some(track_id))
            .and_then(|t| t.get("kind"))
            .and_then(|k| k.as_str())
            .unwrap_or("video")
    };

    // Kumpulkan klip per jenis, urut by start
    let mut vids: Vec<&serde_json::Value> = clips.iter().filter(|c| {
        let tid = c.get("trackId").and_then(|i| i.as_str()).unwrap_or("");
        kind_of(tid) == "video"
    }).collect();
    let mut auds: Vec<&serde_json::Value> = clips.iter().filter(|c| {
        let tid = c.get("trackId").and_then(|i| i.as_str()).unwrap_or("");
        kind_of(tid) == "audio"
    }).collect();
    let texts: Vec<&serde_json::Value> = clips.iter().filter(|c| {
        let tid = c.get("trackId").and_then(|i| i.as_str()).unwrap_or("");
        kind_of(tid) == "text"
    }).collect();
    vids.sort_by(|a, b| {
        let sa = a.get("start").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let sb = b.get("start").and_then(|s| s.as_f64()).unwrap_or(0.0);
        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
    });
    auds.sort_by(|a, b| {
        let sa = a.get("start").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let sb = b.get("start").and_then(|s| s.as_f64()).unwrap_or(0.0);
        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
    });

    if vids.is_empty() && auds.is_empty() && texts.is_empty() {
        return Err("Timeline kosong.".to_string());
    }

    let tmp = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let out_path = tmp.join(format!("{}.mp4", output_name));

    // Input unik: tiap file src sekali
    let mut inputs: Vec<String> = Vec::new();
    let mut input_idx: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let take_input = |src: &str, inputs: &mut Vec<String>, map: &mut std::collections::HashMap<String, usize>| -> usize {
        if let Some(&i) = map.get(src) { return i; }
        let i = inputs.len();
        inputs.push(src.to_string());
        map.insert(src.to_string(), i);
        i
    };
    for c in vids.iter().chain(auds.iter()) {
        let src = c.get("src").and_then(|s| s.as_str()).unwrap_or("");
        if src.is_empty() {
            return Err("Ada klip tanpa file sumber.".to_string());
        }
        if !std::path::Path::new(src).exists() {
            return Err(format!("File tidak ditemukan: {}", src));
        }
        take_input(src, &mut inputs, &mut input_idx);
    }

    // Bangun filter_complex
    let mut fc = String::new();
    let mut vlabels: Vec<String> = Vec::new();
    let mut alabels: Vec<String> = Vec::new();

    for (n, c) in vids.iter().enumerate() {
        let src = c.get("src").and_then(|s| s.as_str()).unwrap_or("");
        let idx = input_idx[src];
        let vin = c.get("in").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let dur = c.get("duration").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let start = c.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        // potong + set durasi + tunda ke posisi start
        fc.push_str(&format!(
            "[{idx}:v]trim=start={vin}:duration={dur},setpts=PTS-STARTPTS+{start}/TB,format=yuv420p[v{n}];"
        ));
        vlabels.push(format!("[v{n}]"));
    }
    for (n, c) in auds.iter().enumerate() {
        let src = c.get("src").and_then(|s| s.as_str()).unwrap_or("");
        let idx = input_idx[src];
        let vin = c.get("in").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let dur = c.get("duration").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let start = c.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let vol = c.get("volume").and_then(|v| v.as_f64()).unwrap_or(100.0) / 100.0;
        fc.push_str(&format!(
            "[{idx}:a]atrim=start={vin}:duration={dur},asetpts=PTS-STARTPTS,volume={vol},adelay={ms}|{ms}[a{n}];",
            ms = (start * 1000.0) as u64
        ));
        alabels.push(format!("[a{n}]"));
    }

    // Durasi total
    let total = clips.iter().map(|c| {
        let s = c.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let d = c.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
        s + d
    }).fold(0.0_f64, f64::max);

    let mut vout = "vfinal".to_string();
    let mut aout = "afinal".to_string();

    if !vlabels.is_empty() {
        // overlay berlapis sesuai urutan start
        let mut base = vlabels[0].clone();
        // buat background hitam seukuran 1280x720 durasi total
        fc.push_str(&format!("color=c=black:s=1280x720:d={total}[bg];"));
        fc.push_str(&format!("[bg]{base}overlay=0:0:shortest=0[ov0];"));
        base = "[ov0]".to_string();
        for (n, lb) in vlabels.iter().enumerate().skip(1) {
            fc.push_str(&format!("{base}{lb}overlay=0:0[ov{n}];"));
            base = format!("[ov{n}]");
        }
        vout = base.trim_matches(|c| c == '[' || c == ']').to_string();
    }
    if !alabels.is_empty() {
        fc.push_str(&format!("{}amix=inputs={}:normalize=0[amx];", alabels.concat(), alabels.len()));
        aout = "amx".to_string();
    }

    // Teks overlay via drawtext (opsional — lewati bila filter tak tersedia)
    let has_drawtext = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-filters"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("drawtext"))
        .unwrap_or(false);
    let mut drawtexts: Vec<String> = Vec::new();
    for c in texts.iter() {
        if !has_drawtext { continue; }
        let text = c.get("text").and_then(|t| t.as_str()).unwrap_or("").replace(':', "\\:").replace('\'', "");
        if text.is_empty() { continue; }
        let start = c.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let end = start + c.get("duration").and_then(|v| v.as_f64()).unwrap_or(3.0);
        drawtexts.push(format!(
            "drawtext=text='{text}':fontsize=48:fontcolor=white:borderw=2:bordercolor=black:x=(w-text_w)/2:y=h-120:enable='between(t,{start},{end})'"
        ));
    }

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.arg("-y").arg("-loglevel").arg("error");
    for inp in &inputs {
        cmd.arg("-i").arg(inp);
    }
    // pastikan filter tidak kosong
    if fc.is_empty() {
        return Err("Tidak ada klip video/audio untuk diekspor.".to_string());
    }
    // hapus trailing ';'
    let fc = fc.trim_end_matches(';');
    let mut vf_chain = vout.clone();
    if !drawtexts.is_empty() {
        vf_chain = format!("{},{}", vout, drawtexts.join(","));
    }
    cmd.arg("-filter_complex").arg(fc);
    if !vlabels.is_empty() {
        cmd.arg("-map").arg(format!("[{}]", vf_chain));
    }
    if !alabels.is_empty() {
        cmd.arg("-map").arg(format!("[{}]", aout));
    }
    // bila hanya teks tanpa video: butuh background
    if vlabels.is_empty() {
        // buat video hitam dari awal via lavfi sudah dihandle? tidak — tolak
        return Err("Export butuh minimal satu klip video atau audio.".to_string());
    }
    cmd.arg("-t").arg(format!("{}", total));
    cmd.arg("-c:v").arg("libx264").arg("-preset").arg("fast").arg("-crf").arg("20");
    if !alabels.is_empty() {
        cmd.arg("-c:a").arg("aac").arg("-b:a").arg("128k");
    }
    cmd.arg(&out_path);

    let out = cmd.output().await.map_err(|e| format!("FFmpeg tidak ditemukan: {}", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("Export gagal: {}", err.chars().take(400).collect::<String>()));
    }
    Ok(out_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_img(path: &std::path::Path) {
        // Buat gambar uji 320x240 via FFmpeg (test pattern)
        let out = std::process::Command::new("ffmpeg")
            .args([
                "-y", "-f", "lavfi", "-i", "testsrc=duration=1:size=320x240:rate=1",
                "-frames:v", "1",
            ])
            .arg(path)
            .output()
            .expect("ffmpeg harus tersedia untuk test");
        assert!(out.status.success(), "ffmpeg testsrc gagal");
    }

    #[tokio::test]
    async fn export_timeline_single_image_clip() {
        let tmp = std::env::temp_dir().join("minime-audit");
        std::fs::create_dir_all(&tmp).unwrap();
        let img = tmp.join("audit_frame.png");
        test_img(&img);
        let img_s = img.to_string_lossy().to_string();
        let tl = serde_json::json!({
            "tracks": [{ "id": "v1", "kind": "video" }],
            "clips": [{
                "id": "c1", "trackId": "v1",
                "src": img_s,
                "start": 0.0, "duration": 2.0,
            }],
        });
        let out = export_timeline(tl.to_string(), "audit_timeline".to_string()).await;
        assert!(out.is_ok(), "export_timeline gagal: {:?}", out.err());
        let out_s = out.unwrap();
        let p = std::path::Path::new(out_s.as_str());
        assert!(p.exists(), "file output tidak ada");
        assert!(p.metadata().unwrap().len() > 1000, "output terlalu kecil");
    }

    #[tokio::test]
    async fn render_slideshow_two_images() {
        let tmp = std::env::temp_dir().join("minime-audit");
        std::fs::create_dir_all(&tmp).unwrap();
        let img1 = tmp.join("audit_slide1.png");
        let img2 = tmp.join("audit_slide2.png");
        test_img(&img1);
        test_img(&img2);
        let s1 = img1.to_string_lossy().to_string();
        let s2 = img2.to_string_lossy().to_string();
        let out = render_slideshow(vec![s1, s2], 1.0, "audit_slideshow".to_string()).await;
        assert!(out.is_ok(), "render_slideshow gagal: {:?}", out.err());
        let out_s = out.unwrap();
        let p = std::path::Path::new(out_s.as_str());
        assert!(p.exists() && p.metadata().unwrap().len() > 1000);
    }

    #[tokio::test]
    async fn generate_thumbnail_produces_png() {
        let out = generate_thumbnail("Audit Judul".to_string(), "audit_thumb".to_string()).await;
        assert!(out.is_ok(), "generate_thumbnail gagal: {:?}", out.err());
        let out_s = out.unwrap();
        let p = std::path::Path::new(out_s.as_str());
        assert!(p.exists() && p.metadata().unwrap().len() > 1000);
    }
}
