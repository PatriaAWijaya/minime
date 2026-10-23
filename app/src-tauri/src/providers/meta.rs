use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub async fn meta_generate_image(
    router: tauri::State<'_, Router>,
    prompt: String,
    size: String,
) -> Result<String, String> {
    // Coba Meta dulu
    match router.muse().image(&prompt, &size).await {
        Ok(bytes) => {
            let path = std::env::temp_dir().join(format!("meta_img_{}.png", chrono::Utc::now().timestamp_millis()));
            std::fs::write(&path, &bytes).map_err(|e| format!("simpan gambar gagal: {e}"))?;
            // Catat biaya
            if let Ok(db) = crate::db::Db::open() {
                let _ = db.record_usage("studio", "muse", "muse:muse-image-1.0", 0, 0);
            }
            Ok(path.to_string_lossy().to_string())
        }
        Err(e) => {
            // Fallback ke SD.cpp lokal
            eprintln!("Meta image gagal ({}), fallback ke SD.cpp lokal", e);
            generate_image(prompt, size).await
        }
    }
}

/// Transkripsi audio via Meta muse-voice-transcribe-1.0.
#[tauri::command]
pub async fn meta_transcribe(
    router: tauri::State<'_, Router>,
    audio_path: String,
) -> Result<String, String> {
    router
        .muse()
        .transcribe(&audio_path)
        .await
        .map_err(|e| format!("transkripsi gagal: {e}"))
}

/// Segmentasi gambar/video-frame via Meta sam-3.1.
#[tauri::command]
pub async fn meta_segment(
    router: tauri::State<'_, Router>,
    image_b64: String,
    prompt: String,
) -> Result<serde_json::Value, String> {
    router
        .muse()
        .segment(&image_b64, &prompt)
        .await
        .map_err(|e| format!("segmentasi gagal: {e}"))
}

/// Generate image via local stable-diffusion.cpp (AI Media Studio).
/// sd-server runs on localhost:8080 (auto-started if needed).
#[tauri::command]
