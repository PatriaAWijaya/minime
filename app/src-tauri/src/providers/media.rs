use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

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
            "-i", list_path.to_str().ok_or("Path tidak valid (non-UTF8)")?,
            "-vf", "scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,format=yuv420p",
            "-c:v", "libx264", "-preset", "fast", "-crf", "23",
            "-movflags", "+faststart",
            out_path.to_str().ok_or("Path tidak valid (non-UTF8)")?,
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
        .args([video_path.as_str(), "-o", out_dir.to_str().ok_or("Path tidak valid (non-UTF8)")?, "--model", model.as_str()])
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
    let out_s = out_path.to_str().ok_or("Path tidak valid (non-UTF8)")?.replace('\'', "");
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
            "-o", out_path.to_str().ok_or("Path tidak valid (non-UTF8)")?,
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

/// Rakit video AI ala MoneyPrinterTurbo: gabung footage + narasi + subtitle.
/// - videos: list path file video footage (akan di-concat berurutan)
/// - audio_path: path file audio narasi (opsional)
/// - subtitle_path: path file SRT subtitle (opsional, akan di-burn-in)
/// - output_name: nama file output
/// - aspect: "9:16" (reels/shorts) atau "16:9" (landscape)
/// Mengembalikan path file MP4 hasil.
/// Cara kerja: concat footage → scale/crop ke aspek → gabung audio (loop/trim menyesuaikan durasi) → burn subtitle.
#[tauri::command]
pub async fn assemble_video_ai(
    videos: Vec<String>,
    audio_path: Option<String>,
    subtitle_path: Option<String>,
    output_name: String,
    aspect: String,
) -> Result<String, String> {
    if videos.is_empty() {
        return Err("Tidak ada footage video.".to_string());
    }
    let ffmpeg = which_ffmpeg().ok_or_else(|| "FFmpeg tidak ditemukan di sistem.".to_string())?;

    let tmp = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let safe_name: String = output_name.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    let out_path = tmp.join(format!("{}.mp4", if safe_name.is_empty() { "video-ai".to_string() } else { safe_name }));

    let (w, h) = match aspect.as_str() {
        "16:9" => (1920, 1080),
        "4:5" => (1080, 1350),
        "1:1" => (1080, 1080),
        _ => (1080, 1920), // 9:16 default
    };

    // 1. Concat footage
    let list_path = tmp.join("ai_concat_list.txt");
    let mut list_content = String::new();
    for v in &videos {
        let escaped = v.replace('\'', "'\\''");
        list_content.push_str(&format!("file '{}'\n", escaped));
    }
    std::fs::write(&list_path, list_content).map_err(|e| e.to_string())?;
    let concat_path = tmp.join("ai_concat.mp4");
    let status = tokio::process::Command::new(&ffmpeg)
        .args([
            "-y", "-f", "concat", "-safe", "0",
            "-i", list_path.to_str().ok_or("Path tidak valid (non-UTF8)")?,
            "-c", "copy",
            concat_path.to_str().ok_or("Path tidak valid (non-UTF8)")?,
        ])
        .status()
        .await
        .map_err(|e| format!("Gagal concat footage: {}", e))?;
    if !status.success() {
        return Err("FFmpeg gagal menggabung footage.".to_string());
    }

    // 2. Bangun filter video: scale + crop + subtitle
    let mut vf = format!(
        "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{},format=yuv420p",
        w, h, w, h
    );
    if let Some(sub) = &subtitle_path {
        if std::path::Path::new(sub).exists() {
            let escaped = sub.replace('\\', "/").replace('\'', "");
            vf.push_str(&format!(",subtitles='{}'", escaped));
        }
    }

    // 3. Gabung dengan audio (jika ada): -shortest agar durasi ikut audio/video terpendek
    let mut cmd = tokio::process::Command::new(&ffmpeg);
    cmd.args(["-y", "-i", concat_path.to_str().ok_or("Path tidak valid (non-UTF8)")?]);
    if let Some(aud) = &audio_path {
        if std::path::Path::new(aud).exists() {
            cmd.args(["-i", aud]);
        }
    }
    cmd.args(["-vf", &vf]);
    if audio_path.as_ref().map(|a| std::path::Path::new(a).exists()).unwrap_or(false) {
        // Map video dari input 0, audio dari input 1, durasi ikut yang terpendek
        cmd.args(["-map", "0:v", "-map", "1:a", "-shortest"]);
    }
    cmd.args([
        "-c:v", "libx264", "-preset", "fast", "-crf", "23",
        "-c:a", "aac", "-b:a", "128k",
        "-movflags", "+faststart",
        out_path.to_str().ok_or("Path tidak valid (non-UTF8)")?,
    ]);
    let status = cmd.status().await.map_err(|e| format!("Gagal merakit video: {}", e))?;
    if !status.success() {
        return Err("FFmpeg gagal merakit video final.".to_string());
    }

    // Bersihkan file sementara
    let _ = std::fs::remove_file(&list_path);
    let _ = std::fs::remove_file(&concat_path);

    out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path output tidak valid.".to_string())
}


/// Text-to-video via AI model (Sora / Wan 2.3 / LTX-Video).
/// Membutuhkan API key di keychain: minime.sora_key, minime.wan_key, minime.ltx_key.
/// Mengembalikan path file video hasil generate.
#[tauri::command]
pub async fn text_to_video(prompt: String, model: String, aspect: String) -> Result<String, String> {
    if prompt.trim().is_empty() {
        return Err("Prompt kosong.".to_string());
    }
    let tmp = std::env::temp_dir().join("minime-studio");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    match model.as_str() {
        "wan" => text_to_video_wan(&prompt, &aspect, &tmp).await,
        "ltx" => text_to_video_ltx(&prompt, &aspect, &tmp).await,
        "sora" => Err("Sora sudah tidak tersedia — OpenAI menutup Videos API pada Sep 2026.".to_string()),
        _ => Err(format!("Model tidak dikenal: {}", model)),
    }
}

async fn text_to_video_wan(prompt: &str, aspect: &str, tmp: &std::path::Path) -> Result<String, String> {
    // Wan 2.3 via API (Alibaba Model Studio atau fal.ai)
    let api_key = crate::secrets::api_key("minime.wan_key")
        .ok_or_else(|| "API key Wan belum diset (Tools → Apps).".to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key Wan belum diset (Tools → Apps).".to_string());
    }
    // Via fal.ai (mendukung Wan): POST https://queue.fal.run/fal-ai/wan-v2-3
    let size = if aspect == "16:9" { "1280x720" } else { "720x1280" };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post("https://queue.fal.run/fal-ai/wan/v2.6/text-to-video")
        .header("Authorization", format!("Key {}", api_key.trim()))
        .json(&serde_json::json!({
            "prompt": prompt,
            "image_size": size,
            "duration": "5",
        }))
        .send()
        .await
        .map_err(|e| format!("Gagal memanggil Wan API: {}", e))?;
    if !resp.status().is_success() {
        let t = resp.text().await.unwrap_or_default();
        return Err(format!("Wan API error: {}", t.chars().take(300).collect::<String>()));
    }
    let j: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let req_id = j.get("request_id").and_then(|v| v.as_str()).ok_or_else(|| "Respons Wan tidak valid.".to_string())?;
    // Polling
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let st = client
            .get(format!("https://queue.fal.run/fal-ai/wan/v2.6/text-to-video/requests/{}/status", req_id))
            .header("Authorization", format!("Key {}", api_key.trim()))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let sj: serde_json::Value = st.json().await.map_err(|e| e.to_string())?;
        let status = sj.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status == "COMPLETED" {
            let res = client
                .get(format!("https://queue.fal.run/fal-ai/wan/v2.6/text-to-video/requests/{}/response", req_id))
                .header("Authorization", format!("Key {}", api_key.trim()))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let rj: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            let video_url = rj.get("video").and_then(|v| v.get("url")).and_then(|u| u.as_str())
                .ok_or_else(|| "URL video tidak ditemukan.".to_string())?;
            // Unduh
            let dl = client.get(video_url).send().await.map_err(|e| e.to_string())?;
            let bytes = dl.bytes().await.map_err(|e| e.to_string())?;
            let out_path = tmp.join(format!("wan-{}.mp4", req_id));
            std::fs::write(&out_path, &bytes).map_err(|e| e.to_string())?;
            return out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path tidak valid.".to_string());
        }
    }
    Err("Wan timeout (10 menit). Coba lagi.".to_string())
}

async fn text_to_video_ltx(prompt: &str, aspect: &str, tmp: &std::path::Path) -> Result<String, String> {
    // LTX-Video via fal.ai: POST https://queue.fal.run/fal-ai/ltx-video
    let api_key = crate::secrets::api_key("minime.ltx_key")
        .ok_or_else(|| "API key LTX belum diset (Tools → Apps).".to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key LTX belum diset (Tools → Apps).".to_string());
    }
    let size = if aspect == "16:9" { "1280x720" } else { "720x1280" };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post("https://queue.fal.run/fal-ai/ltx-video")
        .header("Authorization", format!("Key {}", api_key.trim()))
        .json(&serde_json::json!({
            "prompt": prompt,
            "image_size": size,
        }))
        .send()
        .await
        .map_err(|e| format!("Gagal memanggil LTX API: {}", e))?;
    if !resp.status().is_success() {
        let t = resp.text().await.unwrap_or_default();
        return Err(format!("LTX API error: {}", t.chars().take(300).collect::<String>()));
    }
    let j: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let req_id = j.get("request_id").and_then(|v| v.as_str()).ok_or_else(|| "Respons LTX tidak valid.".to_string())?;
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let st = client
            .get(format!("https://queue.fal.run/fal-ai/ltx-video/requests/{}/status", req_id))
            .header("Authorization", format!("Key {}", api_key.trim()))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let sj: serde_json::Value = st.json().await.map_err(|e| e.to_string())?;
        let status = sj.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status == "COMPLETED" {
            let res = client
                .get(format!("https://queue.fal.run/fal-ai/ltx-video/requests/{}/response", req_id))
                .header("Authorization", format!("Key {}", api_key.trim()))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let rj: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            let video_url = rj.get("video").and_then(|v| v.get("url")).and_then(|u| u.as_str())
                .ok_or_else(|| "URL video tidak ditemukan.".to_string())?;
            let dl = client.get(video_url).send().await.map_err(|e| e.to_string())?;
            let bytes = dl.bytes().await.map_err(|e| e.to_string())?;
            let out_path = tmp.join(format!("ltx-{}.mp4", req_id));
            std::fs::write(&out_path, &bytes).map_err(|e| e.to_string())?;
            return out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path tidak valid.".to_string());
        }
    }
    Err("LTX timeout (10 menit). Coba lagi.".to_string())
}

/// Rakit video via HyperFrames: buat komposisi HTML dari footage + audio, render MP4.
/// Untuk kontrol kreatif penuh (teks animasi, layout kustom).
#[tauri::command]
pub async fn hyperframes_assemble(
    videos: Vec<String>,
    audio_path: Option<String>,
    aspect: String,
    output_name: String,
) -> Result<String, String> {
    if videos.is_empty() {
        return Err("Tidak ada footage.".to_string());
    }
    // Cek HyperFrames CLI
    let hf = which_hyperframes().ok_or_else(|| "HyperFrames belum terinstall. Jalankan: npx hyperframes --help".to_string())?;

    let tmp = std::env::temp_dir().join("minime-hf");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let proj_dir = tmp.join(&output_name);
    if proj_dir.exists() {
        std::fs::remove_dir_all(&proj_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&proj_dir).map_err(|e| e.to_string())?;

    let (w, h) = if aspect == "16:9" { (1920, 1080) } else { (1080, 1920) };

    // Salin footage ke folder proyek (HyperFrames butuh aset lokal)
    let mut local_vids: Vec<String> = Vec::new();
    for (i, v) in videos.iter().enumerate() {
        let dest = proj_dir.join(format!("clip{}.mp4", i));
        // Jika URL http, unduh dulu; jika path lokal, copy
        if v.starts_with("http") {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| e.to_string())?;
            let bytes = client.get(v).send().await.map_err(|e| e.to_string())?
                .bytes().await.map_err(|e| e.to_string())?;
            std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        } else {
            std::fs::copy(v, &dest).map_err(|e| format!("Gagal menyalin footage: {}", e))?;
        }
        local_vids.push(format!("clip{}.mp4", i));
    }

    // Salin audio jika ada
    let audio_file = if let Some(aud) = &audio_path {
        if std::path::Path::new(aud).exists() {
            let dest = proj_dir.join("narasi.mp3");
            std::fs::copy(aud, &dest).map_err(|e| e.to_string())?;
            Some("narasi.mp3".to_string())
        } else { None }
    } else { None };

    // Buat index.html komposisi HyperFrames
    let clips_html: String = local_vids.iter().enumerate().map(|(i, f)| {
        format!(r#"<video class="clip" data-duration="5" src="{}" {}></video>"#,
            f, if i == 0 { r#"data-audio="narasi.mp3""# } else { "" })
    }).collect::<Vec<_>>().join("
    ");

    let html = format!(r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8">
<style>
  body {{ margin: 0; background: #000; }}
  .clip {{ width: 100%; height: 100%; object-fit: cover; }}
</style>
</head>
<body data-width="{w}" data-height="{h}" data-duration="{dur}">
    {clips}
    {audio_tag}
</body>
</html>"#,
        w = w, h = h,
        dur = local_vids.len() * 5,
        clips = clips_html,
        audio_tag = audio_file.map(|f| format!(r#"<audio src="{}" data-duration="{}"></audio>"#, f, local_vids.len() * 5)).unwrap_or_default(),
    );
    std::fs::write(proj_dir.join("index.html"), html).map_err(|e| e.to_string())?;
    std::fs::write(proj_dir.join("hyperframes.json"), r#"{"quality": "draft"}"#).map_err(|e| e.to_string())?;

    // Render via HyperFrames CLI
    let out_path = tmp.join(format!("{}.mp4", output_name));
    let status = tokio::process::Command::new(&hf)
        .args(["render", "--quality", "draft", "--output", out_path.to_str().ok_or("Path tidak valid (non-UTF8)")?])
        .current_dir(&proj_dir)
        .status()
        .await
        .map_err(|e| format!("Gagal menjalankan HyperFrames: {}", e))?;
    if !status.success() {
        return Err("HyperFrames gagal merender.".to_string());
    }
    out_path.to_str().map(|s| s.to_string()).ok_or_else(|| "Path tidak valid.".to_string())
}

fn which_hyperframes() -> Option<String> {
    for p in ["/opt/homebrew/bin/hyperframes", "/usr/local/bin/hyperframes"] {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    // Cek via npx
    if std::process::Command::new("npx").arg("hyperframes").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return Some("npx".to_string());
    }
    None
}
