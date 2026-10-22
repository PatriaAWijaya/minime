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

/// Simpan Pixabay API key.
#[tauri::command]
pub fn set_pixabay_key(api_key: String) -> Result<(), String> {
    crate::secrets::set_api_key("minime.pixabay_key", api_key.trim()).map_err(|e| e.to_string())
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
