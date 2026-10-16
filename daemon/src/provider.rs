//! Model provider abstraction — interface only in Fase 1.
//! Implementations (OpenCode free, Ollama, Muse, Gemini, DeepSeek, ...) land in Fase 2.
//! Mirrors docs/arsitektur.md §2.2. Clean-room design; no OpenBot code.

use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub delta_text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub done: bool,
}

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
    /// a final chunk has `done == true`. Fase 1 has no implementations.
    fn complete_stream(
        &self,
        req: ChatRequest,
    ) -> tokio::sync::mpsc::Receiver<Result<ChatChunk, String>>;
}
