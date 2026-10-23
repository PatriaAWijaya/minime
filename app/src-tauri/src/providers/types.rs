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
