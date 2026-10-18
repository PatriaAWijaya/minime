//! Model provider abstraction — interface only in Fase 1.
//! Implementations (OpenCode free, Ollama, Muse, Gemini, DeepSeek, ...) land in Fase 2.
//! Mirrors docs/arsitektur.md §2.2. Clean-room design; no OpenBot code.

use serde::{Deserialize, Serialize};

/// Build the shared HTTP client for LLM providers.
///
/// TLS uses the OS-native trust store via rustls-platform-verifier (macOS
/// Keychain, Windows cert store, Linux system roots), so the daemon trusts
/// whatever the user's system trusts — including corporate MITM proxies.
/// (reqwest is built with `rustls-tls-manual-roots`, i.e. no bundled roots.)
pub fn http_client() -> reqwest::Client {
    let verifier = std::sync::Arc::new(rustls_platform_verifier::Verifier::new());
    let tls = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .use_preconfigured_tls(Some(tls))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Free,
    PaidApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
    pub context_window: u64,
    pub usd_per_1m_in: f64,
    pub usd_per_1m_out: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub system_prompt: String,
    /// OpenAI-style message list, serialized as JSON.
    pub messages: serde_json::Value,
    pub effort: String,
    pub max_tokens: Option<u64>,
    /// Working directory for providers that spawn local CLIs (e.g. opencode).
    #[serde(default)]
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub delta_text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub done: bool,
}

/// Typed provider errors so the executor can route intelligently:
/// - `NoKey`: provider butuh API key yang belum ada → lewati ke fallback.
/// - `RegionBlocked`: region diblokir (Meta Model API) → lewati ke fallback.
/// - `Failed`: error lain (timeout, HTTP, CLI) → boleh coba fallback.
#[derive(Debug, Clone)]
pub enum ProviderError {
    NoKey(String),
    RegionBlocked(String),
    Failed(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NoKey(p) => write!(f, "provider '{p}' tidak punya API key"),
            ProviderError::RegionBlocked(p) => {
                write!(f, "provider '{p}' diblokir untuk region ini")
            }
            ProviderError::Failed(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Every model provider (free or paid) implements this trait.
/// Adding a provider = implementing this trait; no executor refactor needed.
/// Streaming is represented as an async channel receiver to avoid
/// boxing futures in the trait.
pub trait ModelProvider: Send + Sync {
    fn id(&self) -> &str;
    fn kind(&self) -> ProviderKind;
    fn list_models(&self) -> Vec<ModelInfo>;
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough heuristic until a tokenizer lands (Fase 2).
        text.len().div_ceil(4)
    }
    /// Start a streaming completion. Chunks arrive on the returned receiver;
    /// a final chunk has `done == true`. Errors are typed so the executor
    /// can decide whether to try the next provider in the fallback chain.
    fn complete_stream(
        &self,
        req: ChatRequest,
    ) -> tokio::sync::mpsc::Receiver<Result<ChatChunk, ProviderError>>;
}
