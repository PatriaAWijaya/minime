//! Upload deliverable ke Google Drive folder MIMI (Fase 5).
//! Dipanggil best-effort setelah approval: bila Drive belum terhubung,
//! approval tetap sukses (draft lokal tetap tersimpan) dan kegagalan dicatat.

use anyhow::{Context, Result};

use crate::drive_oauth;
use crate::provider::http_client;

/// Cari folder anak bernama `name` di bawah `parent_id`; buat bila belum ada.
/// Mengembalikan Drive file ID folder.
async fn find_or_create_folder(
    client: &reqwest::Client,
    token: &str,
    name: &str,
    parent_id: &str,
) -> Result<String> {
    // Cari dulu.
    let q = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and '{}' in parents and trashed = false",
        name.replace('\'', "\\'"),
        parent_id
    );
    let found: serde_json::Value = client
        .get("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(token)
        .query(&[
            ("q", q.as_str()),
            ("fields", "files(id,name)"),
            ("pageSize", "1"),
        ])
        .send()
        .await
        .context("drive: cari folder gagal")?
        .json()
        .await
        .context("drive: respon cari folder bukan JSON")?;
    if let Some(id) = found
        .get("files")
        .and_then(|f| f.as_array())
        .and_then(|a| a.first())
        .and_then(|f| f.get("id"))
        .and_then(|i| i.as_str())
    {
        return Ok(id.to_string());
    }
    // Buat baru.
    let created: serde_json::Value = client
        .post("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(token)
        .json(&serde_json::json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder",
            "parents": [parent_id],
        }))
        .send()
        .await
        .context("drive: buat folder gagal")?
        .json()
        .await
        .context("drive: respon buat folder bukan JSON")?;
    created
        .get("id")
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .context("drive: folder baru tanpa id")
}

/// Upload satu file teks ke folder Drive. Mengembalikan file ID.
pub async fn upload_text(folder_name: &str, filename: &str, content: &str) -> Result<String> {
    let token = drive_oauth::access_token().await?;
    let client = http_client();

    // Folder MIMI di root Drive; subfolder per nama yang diminta.
    let mimi_id = find_or_create_folder(&client, &token, "MIMI", "root").await?;
    let target_id = if folder_name == "MIMI" {
        mimi_id
    } else {
        find_or_create_folder(&client, &token, folder_name, &mimi_id).await?
    };

    // Multipart upload: metadata + konten.
    let metadata = serde_json::json!({
        "name": filename,
        "parents": [target_id],
        "mimeType": "text/markdown",
    });
    let form = reqwest::multipart::Form::new()
        .text("metadata", metadata.to_string())
        .part(
            "file",
            reqwest::multipart::Part::text(content.to_string()).mime_str("text/markdown")?,
        );
    let uploaded: serde_json::Value = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .context("drive: upload gagal")?
        .json()
        .await
        .context("drive: respon upload bukan JSON")?;
    uploaded
        .get("id")
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .context("drive: upload tanpa file id")
}
