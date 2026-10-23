use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

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
