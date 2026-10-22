//! VoiceStudio local TTS engine (external process, NEVER bundled).
//!
//! VoiceStudio (debpalash/VoiceStudio, AGPL-3.0) is a separate macOS app
//! installed at /Applications/VoiceStudio.app. Its backend only runs while
//! the app is open, exposing a local API at http://127.0.0.1:3900.
//!
//! This module talks to that local API. yovoice (providers.rs) remains the
//! automatic fallback whenever VoiceStudio is unreachable or fails.

use serde::Serialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:3900";
/// Health check probe budget for `voicestudio_status`.
const STATUS_TIMEOUT: Duration = Duration::from_secs(5);
/// Budget for the voices listing.
const VOICES_TIMEOUT: Duration = Duration::from_secs(10);
/// Local models can be slow on first load; be generous.
const TTS_TIMEOUT: Duration = Duration::from_secs(180);
/// VoiceStudio caps `input` at 4096 characters.
const MAX_INPUT_CHARS: usize = 4096;

/// Base URL, overridable for tests/dev via env.
fn base_url() -> String {
    std::env::var("MINIME_VOICESTUDIO_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

fn client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Gagal membuat HTTP client: {e}"))
}

/// Is the yovoice fallback usable (binary present)?
fn yovoice_available() -> bool {
    std::path::Path::new(&crate::providers::yovoice_bin()).exists()
}

/* ---------------- status ---------------- */

#[derive(Serialize)]
struct StatusReport {
    reachable: bool,
    version: Option<String>,
    engine: &'static str,
}

/// Best-effort `version` field from /v1/models; never fails.
fn extract_version(json: &serde_json::Value) -> Option<String> {
    json.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

async fn probe_status(base: &str) -> StatusReport {
    let client = match client(STATUS_TIMEOUT) {
        Ok(c) => c,
        Err(_) => {
            return StatusReport {
                reachable: false,
                version: None,
                engine: if yovoice_available() { "yovoice" } else { "none" },
            }
        }
    };
    let resp = client.get(format!("{base}/v1/models")).send().await;
    match resp {
        Ok(r) if r.status().is_success() => {
            let version = r
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|j| extract_version(&j));
            StatusReport {
                reachable: true,
                version,
                engine: "voicestudio",
            }
        }
        _ => StatusReport {
            reachable: false,
            version: None,
            engine: if yovoice_available() { "yovoice" } else { "none" },
        },
    }
}

/// JSON: {"reachable": bool, "version": string|null,
///        "engine": "voicestudio"|"yovoice"|"none"}.
/// Never fails — a dead backend is reported as reachable=false.
#[tauri::command]
pub async fn voicestudio_status() -> String {
    let report = probe_status(&base_url()).await;
    serde_json::to_string(&report).unwrap_or_else(|_| {
        r#"{"reachable":false,"version":null,"engine":"none"}"#.to_string()
    })
}

/* ---------------- voices ---------------- */

/// Normalize one voice entry; unknown shapes degrade to empty strings.
fn parse_voices(json: &serde_json::Value) -> Vec<serde_json::Value> {
    json.get("voices")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| {
            serde_json::json!({
                "voice_id": v.get("voice_id").and_then(|x| x.as_str()).unwrap_or(""),
                "name": v.get("name").and_then(|x| x.as_str()).unwrap_or(""),
                "type": v.get("type").and_then(|x| x.as_str()).unwrap_or(""),
                "language": v.get("language").and_then(|x| x.as_str()).unwrap_or(""),
            })
        })
        .collect()
}

/// JSON array [{"voice_id","name","type","language"}] from the backend.
#[tauri::command]
pub async fn voicestudio_voices() -> Result<String, String> {
    let base = base_url();
    let client = client(VOICES_TIMEOUT)?;
    let resp = client
        .get(format!("{base}/v1/audio/voices"))
        .send()
        .await
        .map_err(|e| {
            format!(
                "VoiceStudio tidak berjalan di {base} ({e}). Buka dulu aplikasi VoiceStudio, lalu coba lagi."
            )
        })?;
    if !resp.status().is_success() {
        return Err(format!(
            "VoiceStudio menjawab HTTP {} untuk daftar suara.",
            resp.status()
        ));
    }
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Respons daftar suara VoiceStudio tidak valid: {e}"))?;
    serde_json::to_string(&parse_voices(&json)).map_err(|e| e.to_string())
}

/* ---------------- shared helpers ---------------- */

#[derive(Serialize)]
struct TtsResult {
    path: String,
    engine: &'static str,
}

fn json_result(path: &std::path::Path, engine: &'static str) -> Result<String, String> {
    json_result_path(&path.to_string_lossy(), engine)
}

fn json_result_path(path: &str, engine: &'static str) -> Result<String, String> {
    serde_json::to_string(&TtsResult {
        path: path.to_string(),
        engine,
    })
    .map_err(|e| e.to_string())
}

fn validate_input(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Teks kosong — tidak ada yang bisa dibacakan.".to_string());
    }
    if text.chars().count() > MAX_INPUT_CHARS {
        return Err(format!(
            "Teks terlalu panjang (maks {MAX_INPUT_CHARS} karakter)."
        ));
    }
    Ok(())
}

/// Output file under temp_dir()/minime-studio, sanitized against traversal.
fn studio_out_path(output_name: &str) -> Result<std::path::PathBuf, String> {
    let name = output_name.trim();
    if name.is_empty() {
        return Err("Nama file output kosong.".to_string());
    }
    let safe: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if safe.is_empty() {
        return Err("Nama file output tidak valid.".to_string());
    }
    let dir = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat folder output: {e}"))?;
    Ok(dir.join(format!("{safe}.wav")))
}

/// Read the response as audio bytes, detecting JSON error bodies and
/// empty payloads honestly.
async fn read_audio_bytes(resp: reqwest::Response, what: &str) -> Result<Vec<u8>, String> {
    let status = resp.status();
    if !status.is_success() {
        let detail: String = resp.text().await.unwrap_or_default().chars().take(300).collect();
        return Err(format!("VoiceStudio gagal {what} (HTTP {status}): {detail}"));
    }
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Gagal membaca audio dari VoiceStudio: {e}"))?;
    if content_type.contains("json") {
        let msg: String = String::from_utf8_lossy(&bytes).chars().take(300).collect();
        return Err(format!("VoiceStudio gagal {what}: {msg}"));
    }
    if bytes.is_empty() {
        return Err("VoiceStudio mengembalikan audio kosong.".to_string());
    }
    Ok(bytes.to_vec())
}

fn save_audio(bytes: &[u8], out: &std::path::Path) -> Result<(), String> {
    std::fs::write(out, bytes).map_err(|e| format!("Gagal menyimpan file audio: {e}"))
}

/* ---------------- TTS ---------------- */

/// Raw synthesis via POST /v1/audio/speech; returns the audio bytes.
async fn voicestudio_speech(text: &str, voice: &str, speed: f32) -> Result<Vec<u8>, String> {
    let client = client(TTS_TIMEOUT)?;
    let voice = if voice.trim().is_empty() {
        "default"
    } else {
        voice.trim()
    };
    let speed = if speed.is_finite() { speed } else { 1.0 };
    let body = serde_json::json!({
        "model": "omnivoice",
        "input": text,
        "voice": voice,
        "response_format": "wav",
        "speed": speed,
        "language": "id",
    });
    let resp = client
        .post(format!("{}/v1/audio/speech", base_url()))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("VoiceStudio tidak terjangkau: {e}"))?;
    read_audio_bytes(resp, "mensintesis suara").await
}

/// TTS: try VoiceStudio first, fall back to yovoice automatically.
/// Returns JSON {"path","engine"}.
#[tauri::command]
pub async fn voicestudio_tts(
    text: String,
    voice: String,
    speed: f32,
    output_name: String,
) -> Result<String, String> {
    validate_input(&text)?;
    let out = studio_out_path(&output_name)?;
    match voicestudio_speech(&text, &voice, speed).await {
        Ok(bytes) => {
            save_audio(&bytes, &out)?;
            json_result(&out, "voicestudio")
        }
        Err(vs_err) => {
            // Fallback otomatis ke yovoice (CLI lokal, tanpa backend).
            match crate::providers::yovoice_tts(text, voice, String::new(), output_name, String::new()).await {
                Ok(path) => json_result_path(&path, "yovoice"),
                Err(yo_err) => Err(format!(
                    "VoiceStudio: {vs_err} — yovoice (fallback) juga gagal: {yo_err}"
                )),
            }
        }
    }
}

/* ---------------- voice cloning ---------------- */

/// Voice cloning via POST /generate (multipart, ref_audio = audio file).
async fn voicestudio_clone(
    text: &str,
    ref_audio: &std::path::Path,
    language: &str,
) -> Result<Vec<u8>, String> {
    let client = client(TTS_TIMEOUT)?;
    let audio_bytes = tokio::fs::read(ref_audio)
        .await
        .map_err(|e| format!("Gagal membaca file referensi: {e}"))?;
    let file_name = ref_audio
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ref_audio.wav".to_string());
    let file_part = reqwest::multipart::Part::bytes(audio_bytes)
        .file_name(file_name)
        .mime_str("audio/wav")
        .map_err(|e| format!("Tipe MIME tidak valid: {e}"))?;
    let form = reqwest::multipart::Form::new()
        .text("text", text.to_string())
        .text("language", language.to_string())
        .text("ref_text", String::new())
        .text("speed", "1.0".to_string())
        .text("engine", "omnivoice".to_string())
        .part("ref_audio", file_part);
    let resp = client
        .post(format!("{}/generate", base_url()))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("VoiceStudio tidak terjangkau: {e}"))?;
    read_audio_bytes(resp, "mengkloning suara").await
}

/// Voice cloning: VoiceStudio first, yovoice (with --reference) as fallback.
/// Returns JSON {"path","engine"}.
#[tauri::command]
pub async fn voicestudio_clone_tts(
    text: String,
    ref_audio_path: String,
    language: String,
    output_name: String,
) -> Result<String, String> {
    validate_input(&text)?;
    let ref_path = std::path::Path::new(ref_audio_path.trim());
    if !ref_path.is_file() {
        return Err(format!(
            "File referensi tidak ditemukan: {}",
            ref_audio_path.trim()
        ));
    }
    let language = if language.trim().is_empty() {
        "id"
    } else {
        language.trim()
    };
    let out = studio_out_path(&output_name)?;
    match voicestudio_clone(&text, ref_path, language).await {
        Ok(bytes) => {
            save_audio(&bytes, &out)?;
            json_result(&out, "voicestudio")
        }
        Err(vs_err) => {
            // Fallback otomatis ke yovoice dengan file referensi yang sama.
            match crate::providers::yovoice_tts(text, String::new(), ref_audio_path, output_name, String::new())
                .await
            {
                Ok(path) => json_result_path(&path, "yovoice"),
                Err(yo_err) => Err(format!(
                    "VoiceStudio: {vs_err} — yovoice (fallback) juga gagal: {yo_err}"
                )),
            }
        }
    }
}

/* ---------------- open app ---------------- */

/// Launch the VoiceStudio macOS app (its backend only runs while open).
#[tauri::command]
pub fn voicestudio_open() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", "VoiceStudio"])
            .spawn()
            .map_err(|e| format!("Gagal membuka VoiceStudio: {e}"))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Membuka VoiceStudio hanya didukung di macOS.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Port tinggi yang hampir pasti tidak dipakai siapa pun di loopback.
    const DEAD_BASE: &str = "http://127.0.0.1:53999";

    #[tokio::test]
    async fn status_closed_port_reports_unreachable_without_hang() {
        // Connection refused di loopback langsung gagal — tidak menunggu
        // timeout penuh 5 detik.
        let report = probe_status(DEAD_BASE).await;
        assert!(!report.reachable);
        assert!(report.version.is_none());
        assert!(matches!(report.engine, "yovoice" | "none"));
    }

    #[test]
    fn version_extraction_is_optional() {
        let with: serde_json::Value =
            serde_json::from_str(r#"{"version":"0.5.6","models":[]}"#).unwrap();
        let without: serde_json::Value = serde_json::from_str(r#"{"models":[]}"#).unwrap();
        assert_eq!(extract_version(&with).as_deref(), Some("0.5.6"));
        assert_eq!(extract_version(&without), None);
    }

    #[test]
    fn parse_voices_maps_fields() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"voices":[{"voice_id":"v1","name":"Sari","type":"custom","language":"id"}],"engines":["omnivoice"]}"#,
        )
        .unwrap();
        let voices = parse_voices(&json);
        assert_eq!(voices.len(), 1);
        assert_eq!(voices[0]["voice_id"], "v1");
        assert_eq!(voices[0]["name"], "Sari");
        assert_eq!(voices[0]["type"], "custom");
        assert_eq!(voices[0]["language"], "id");
    }

    #[test]
    fn parse_voices_tolerates_unexpected_shape() {
        let json: serde_json::Value = serde_json::from_str(r#"{"engines":[]}"#).unwrap();
        assert!(parse_voices(&json).is_empty());
        let empty: serde_json::Value = serde_json::from_str(r#"{}"#).unwrap();
        assert!(parse_voices(&empty).is_empty());
    }

    #[test]
    fn output_name_is_sanitized_against_traversal() {
        let p = studio_out_path("../../etc/malicious").unwrap();
        let s = p.to_string_lossy();
        assert!(!s.contains(".."));
        assert!(s.ends_with("etcmalicious.wav"));
        assert!(studio_out_path("").is_err());
        assert!(studio_out_path("///").is_err());
    }

    #[tokio::test]
    async fn empty_text_rejected_before_any_network() {
        let err = voicestudio_tts(String::new(), "default".into(), 1.0, "t".into())
            .await
            .unwrap_err();
        assert!(err.contains("Teks kosong"));
    }

    #[tokio::test]
    async fn overlong_text_rejected_before_any_network() {
        let err = voicestudio_tts("a".repeat(MAX_INPUT_CHARS + 1), "default".into(), 1.0, "t".into())
            .await
            .unwrap_err();
        assert!(err.contains("terlalu panjang"));
    }

    #[tokio::test]
    async fn clone_rejects_missing_reference_file() {
        let err = voicestudio_clone_tts(
            "halo".into(),
            "/tmp/minime-ref-tidak-ada-xyz.wav".into(),
            "id".into(),
            "t".into(),
        )
        .await
        .unwrap_err();
        assert!(err.contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn voices_dead_backend_errors_honestly() {
        // Paksa base URL mati lewat env agar command memakai jalur aslinya.
        std::env::set_var("MINIME_VOICESTUDIO_BASE_URL", DEAD_BASE);
        let err = voicestudio_voices().await.unwrap_err();
        std::env::remove_var("MINIME_VOICESTUDIO_BASE_URL");
        assert!(err.contains("tidak berjalan"));
    }

    #[tokio::test]
    async fn status_command_returns_valid_json() {
        let s = voicestudio_status().await;
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.get("reachable").and_then(|r| r.as_bool()).is_some());
        assert!(v.get("engine").and_then(|e| e.as_str()).is_some());
        // "version" boleh null, tapi kuncinya harus ada.
        assert!(v.get("version").is_some());
    }
}
