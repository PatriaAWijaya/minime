//! Social draft integration — Postiz & Meta Business Suite.
//!
//! ATURAN KERAS: hanya membuat DRAFT. Tidak ada auto-posting.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftResult {
    pub platform: String,
    pub draft_id: String,
    pub status: String,
}

/// Buat draft di Postiz. Membutuhkan POSTIZ_API_URL dan POSTIZ_API_KEY di keychain.
#[tauri::command]
pub async fn create_postiz_draft(
    content: String,
    platforms: Vec<String>,
    scheduled_at: Option<String>,
    media_urls: Vec<String>,
) -> Result<DraftResult, String> {
    let base_url = crate::secrets::api_key("minime.postiz_url")
        .ok_or_else(|| "Postiz URL belum diset.".to_string())?;
    let api_key = crate::secrets::api_key("minime.postiz_key")
        .ok_or_else(|| "Postiz API key belum diset.".to_string())?;

    if base_url.trim().is_empty() || api_key.trim().is_empty() {
        return Err("Postiz belum dikonfigurasi.".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // Postiz API: POST /api/posts dengan status draft
    let body = serde_json::json!({
        "content": content,
        "platforms": platforms,
        "scheduledAt": scheduled_at,
        "media": media_urls,
        "status": "draft",
    });

    let url = format!("{}/api/posts", base_url.trim().trim_end_matches('/'));
    let resp = client
        .post(&url)
        .bearer_auth(api_key.trim())
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Postiz API error: {}", e))?;

    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!("Postiz: {}", txt.chars().take(200).collect::<String>()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let draft_id = data
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(DraftResult {
        platform: "postiz".to_string(),
        draft_id,
        status: "draft".to_string(),
    })
}

/// Buat draft di Meta (Facebook/Instagram) via Graph API.
/// Membutuhkan META_PAGE_TOKEN di keychain. Hanya membuat unpublished post (draft).
#[tauri::command]
pub async fn create_meta_draft(
    page_id: String,
    message: String,
    image_url: Option<String>,
) -> Result<DraftResult, String> {
    let token = crate::secrets::api_key("minime.meta_token")
        .ok_or_else(|| "Meta token belum diset.".to_string())?;

    if token.trim().is_empty() || page_id.trim().is_empty() {
        return Err("Meta belum dikonfigurasi.".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // Buat unpublished post (draft) di Facebook Page
    let mut params = vec![
        ("message", message.as_str()),
        ("published", "false"),
        ("access_token", token.trim()),
    ];

    let url = if let Some(img) = &image_url {
        params.push(("url", img.as_str()));
        format!("https://graph.facebook.com/v21.0/{}/photos", page_id.trim())
    } else {
        format!("https://graph.facebook.com/v21.0/{}/feed", page_id.trim())
    };

    let resp = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Meta API error: {}", e))?;

    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!("Meta: {}", txt.chars().take(200).collect::<String>()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let draft_id = data
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(DraftResult {
        platform: "meta".to_string(),
        draft_id,
        status: "draft (unpublished)".to_string(),
    })
}

/// Simpan konfigurasi Postiz.
#[tauri::command]
pub fn set_postiz_config(api_url: String, api_key: String) -> Result<(), String> {
    crate::secrets::set_api_key("minime.postiz_url", api_url.trim()).map_err(|e| e.to_string())?;
    crate::secrets::set_api_key("minime.postiz_key", api_key.trim()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Simpan Meta Page token.
#[tauri::command]
pub fn set_meta_token(page_id: String, token: String) -> Result<(), String> {
    crate::secrets::set_api_key("minime.meta_page_id", page_id.trim()).map_err(|e| e.to_string())?;
    crate::secrets::set_api_key("minime.meta_token", token.trim()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn draft_result_serializes() {
        let r = super::DraftResult {
            platform: "postiz".into(),
            draft_id: "abc".into(),
            status: "draft".into(),
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("postiz"));
    }

    #[tokio::test]
    async fn fetch_image_as_data_url_real_download() {
        // Unduh gambar PNG nyata dan pastikan hasilnya data URL valid
        let url = "https://httpbin.org/image/png".to_string();
        let out = super::fetch_image_as_data_url(url).await;
        assert!(out.is_ok(), "download gagal: {:?}", out.err());
        let s = out.unwrap();
        assert!(s.starts_with("data:image/png;base64,"), "prefix salah");
        assert!(s.len() > 1000, "data terlalu kecil");
    }

    #[tokio::test]
    async fn fetch_image_as_data_url_bad_url_errors() {
        let out = super::fetch_image_as_data_url("https://127.0.0.1:9/tidak-ada.png".to_string()).await;
        assert!(out.is_err(), "seharusnya error untuk URL mati");
    }

    #[tokio::test]
    async fn read_image_as_data_url_detects_mime() {
        // Buat file PNG dan JPEG minimal di temp, pastikan deteksi MIME benar
        let dir = std::env::temp_dir();
        let png_path = dir.join("minime-test.png");
        let jpg_path = dir.join("minime-test.jpg");
        // PNG minimal: signature + IHDR + IEND
        let png: Vec<u8> = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
            0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54,
            0x08, 0xD7, 0x63, 0xF8, 0x0F, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x5C, 0xC2, 0x8A, 0x8B,
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
            0xAE, 0x42, 0x60, 0x82,
        ];
        std::fs::write(&png_path, &png).unwrap();
        let out = super::read_image_as_data_url(png_path.to_string_lossy().to_string()).await;
        assert!(out.is_ok(), "baca PNG gagal: {:?}", out.err());
        assert!(out.unwrap().starts_with("data:image/png;base64,"), "MIME PNG salah");

        let jpg: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00];
        std::fs::write(&jpg_path, &jpg).unwrap();
        let out = super::read_image_as_data_url(jpg_path.to_string_lossy().to_string()).await;
        assert!(out.is_ok(), "baca JPG gagal: {:?}", out.err());
        assert!(out.unwrap().starts_with("data:image/jpeg;base64,"), "MIME JPG salah");

        let out = super::read_image_as_data_url("/tidak/ada/file.png".to_string()).await;
        assert!(out.is_err(), "seharusnya error untuk file tidak ada");

        let _ = std::fs::remove_file(&png_path);
        let _ = std::fs::remove_file(&jpg_path);
    }

    #[tokio::test]
    async fn pixabay_search_without_key_gives_clear_error() {
        // Tanpa key di keychain (atau key kosong), harus error jelas bukan panic.
        // Jika key valid tersimpan, test ini skip (hasil Ok juga valid).
        let out = super::pixabay_search_images("masjid".to_string(), 3).await;
        match out {
            Ok(v) => {
                // Key valid tersimpan: pastikan struktur respons benar
                assert!(v.get("hits").is_some(), "respons Pixabay tak ada hits");
            }
            Err(e) => {
                assert!(
                    e.contains("Pixabay") || e.contains("key"),
                    "pesan error tak jelas: {}",
                    e
                );
            }
        }
    }
}

/// Cari foto stok di Pixabay. Membutuhkan PIXABAY_API_KEY di keychain.
#[tauri::command]
pub async fn pixabay_search_images(query: String, per_page: u8) -> Result<serde_json::Value, String> {
    let api_key = crate::secrets::api_key("minime.pixabay_key")
        .ok_or_else(|| "Pixabay API key belum diset.".to_string())?;
    if api_key.trim().is_empty() {
        return Err("Pixabay API key kosong.".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("https://pixabay.com/api/")
        .query(&[
            ("key", api_key.trim()),
            ("q", query.as_str()),
            ("image_type", "photo"),
            ("per_page", &per_page.min(20).to_string()),
            ("safesearch", "true"),
        ])
        .send()
        .await
        .map_err(|e| format!("Pixabay API error: {}", e))?;
    if !resp.status().is_success() {
        return Err("Pixabay API gagal.".to_string());
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// Cari video stok di Pixabay.
#[tauri::command]
pub async fn pixabay_search_videos(query: String, per_page: u8) -> Result<serde_json::Value, String> {
    let api_key = crate::secrets::api_key("minime.pixabay_key")
        .ok_or_else(|| "Pixabay API key belum diset.".to_string())?;
    if api_key.trim().is_empty() {
        return Err("Pixabay API key kosong.".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("https://pixabay.com/api/videos/")
        .query(&[
            ("key", api_key.trim()),
            ("q", query.as_str()),
            ("per_page", &per_page.min(20).to_string()),
            ("safesearch", "true"),
        ])
        .send()
        .await
        .map_err(|e| format!("Pixabay API error: {}", e))?;
    if !resp.status().is_success() {
        return Err("Pixabay API gagal.".to_string());
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// Cari video stok di Pexels (gratis, 200 req/jam).
/// Membutuhkan PEXELS_API_KEY di keychain (minime.pexels_key).
#[tauri::command]
pub async fn pexels_search_videos(query: String, per_page: u8) -> Result<String, String> {
    let api_key = crate::secrets::api_key("minime.pexels_key")
        .ok_or_else(|| "API key Pexels belum diset (Tools → Apps).".to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key Pexels belum diset (Tools → Apps).".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("https://api.pexels.com/videos/search")
        .header("Authorization", api_key.trim())
        .query(&[
            ("query", query.trim()),
            ("per_page", &per_page.min(20).to_string()),
            ("orientation", "portrait"),
        ])
        .send()
        .await
        .map_err(|e| format!("Gagal mencari di Pexels: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Pexels API error: HTTP {}", resp.status()));
    }
    let j: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    // Sederhanakan: kembalikan daftar {url, preview}
    let videos = j.get("videos").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let mut out = Vec::new();
    for v in videos {
        let files = v.get("video_files").and_then(|f| f.as_array()).cloned().unwrap_or_default();
        // Pilih file HD terkecil yang layak
        let best = files.iter()
            .filter(|f| f.get("width").and_then(|w| w.as_u64()).unwrap_or(0) >= 720)
            .min_by_key(|f| f.get("width").and_then(|w| w.as_u64()).unwrap_or(u64::MAX));
        if let Some(b) = best {
            let url = b.get("link").and_then(|l| l.as_str()).unwrap_or("").to_string();
            let preview = v.get("image").and_then(|i| i.as_str()).unwrap_or("").to_string();
            if !url.is_empty() {
                out.push(serde_json::json!({"url": url, "preview": preview}));
            }
        }
    }
    Ok(serde_json::to_string(&serde_json::json!({"videos": out})).unwrap())
}

/// Simpan API key Pexels.
#[tauri::command]
pub fn set_pexels_key(api_key: String) -> Result<(), String> {
    crate::secrets::set_api_key("minime.pexels_key", api_key.trim()).map_err(|e| e.to_string())
}


/// Simpan Pixabay API key.
#[tauri::command]
pub fn set_pixabay_key(api_key: String) -> Result<(), String> {
    crate::secrets::set_api_key("minime.pixabay_key", api_key.trim()).map_err(|e| e.to_string())
}

/// Simpan API key untuk text-to-video (Sora / Wan 2.3 / LTX-Video).
#[tauri::command]
pub fn set_video_ai_key(model: String, api_key: String) -> Result<(), String> {
    let key_name = match model.as_str() {
        "sora" => "minime.sora_key",
        "wan" => "minime.wan_key",
        "ltx" => "minime.ltx_key",
        _ => return Err(format!("Model tidak dikenal: {}", model)),
    };
    crate::secrets::set_api_key(key_name, api_key.trim()).map_err(|e| e.to_string())
}

/// Unduh gambar dari URL dan kembalikan sebagai data URL (base64).
/// Dipakai agar gambar dari CDN (mis. Pixabay) bisa dibuka di editor tanpa masalah CORS.
#[tauri::command]
pub async fn fetch_image_as_data_url(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url.trim())
        .send()
        .await
        .map_err(|e| format!("Gagal mengunduh gambar: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Err("Gambar kosong.".to_string());
    }
    use base64::Engine;
    Ok(format!(
        "data:{};base64,{}",
        ct,
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

/// Buka dialog pilih file native (macOS/Windows/Linux) untuk gambar.
/// Mengembalikan path file yang dipilih, atau None bila dibatalkan.
#[tauri::command]
pub async fn pick_image_file() -> Result<Option<String>, String> {
    let file = rfd::FileDialog::new()
        .set_title("Pilih gambar")
        .add_filter("Gambar", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
        .pick_file();
    Ok(file.map(|p| p.to_string_lossy().to_string()))
}

/// Baca file gambar lokal dan kembalikan sebagai data URL (base64).
#[tauri::command]
pub async fn read_image_as_data_url(path: String) -> Result<String, String> {
    let bytes = std::fs::read(path.trim()).map_err(|e| format!("Gagal membaca file: {}", e))?;
    if bytes.is_empty() {
        return Err("File kosong.".to_string());
    }
    // Deteksi tipe dari magic bytes
    let mime = if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.starts_with(b"RIFF") && bytes.len() > 11 && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else if bytes.starts_with(b"BM") {
        "image/bmp"
    } else {
        "image/jpeg"
    };
    Ok(format!(
        "data:{};base64,{}",
        mime,
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes)
    ))
}
