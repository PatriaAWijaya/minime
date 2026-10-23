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

pub mod types;
pub mod ollama;
pub mod openai_compat;
pub mod opencode;
pub mod router;
pub mod meta;
pub mod media;
pub mod catalog;
pub mod yovoice;

// Re-export commonly used types at the providers:: level
// so existing `providers::X` paths keep working.
pub use types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
pub use ollama::OllamaProvider;
pub use openai_compat::OpenAiCompatProvider;
pub use opencode::{ModelTier, OpenCodeProvider};
pub use router::Router;

// Re-export Tauri command functions so `providers::fn_name` paths keep working.
pub use meta::{meta_generate_image, meta_segment, meta_transcribe};
pub use media::{
    assemble_video_ai, audio_master, export_timeline, generate_image, generate_subtitle,
    generate_thumbnail, generate_voiceover, hyperframes_assemble, render_slideshow, text_to_video,
};
pub use catalog::{
    discover_models, fetch_provider_models, ollama_installed_models, ollama_pull_model,
    opencode_free_models, provider_catalog, set_dalle_key,
};
pub use yovoice::{yovoice_models, yovoice_status, yovoice_tts, yovoice_voices};
