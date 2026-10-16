//! Blotato wiring: distribusi video ke 9 platform — DRAFT-ONLY.
//!
//! Adaptasi STEP 3 diagram n8n Patria: ambil video → upload ke Blotato →
//! assign social media IDs → publish ke 9 platform → log.
//!
//! ATURAN KERAS (rule #3): TIDAK ADA auto-posting ke Meta (Instagram,
//! Facebook, Threads) — konsisten dengan `instagram-cli` Fase 4 yang
//! publikasinya DITOLAK di kode. Di tool ini:
//!   - `draft`: upload media ke Blotato (bila key ada) + simpan bundle draft
//!     LOKAL per platform. TIDAK PERNAH memanggil POST /v2/posts.
//!   - `publish`: DITOLAK untuk instagram/facebook/threads dalam kondisi apa
//!     pun (kode). 6 platform lain: WAJIB `approval_id` yang merujuk record
//!     approvals berstatus 'approved' — tanpa itu DITOLAK.
//!   - `accounts`: read-only (daftar akun terhubung).
//! Kredensial: Keychain akun `blotato` / env MINIME_BLOTATO_KEY. Tidak di-log.

use super::{ToolCtx, ToolStatus};
use crate::db::Db;
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const BASE: &str = "https://backend.blotato.com/v2";

/// 9 platform sesuai diagram Patria.
pub const PLATFORMS_9: &[&str] = &[
    "instagram",
    "youtube",
    "tiktok",
    "facebook",
    "threads",
    "twitter",
    "linkedin",
    "bluesky",
    "pinterest",
];

/// Platform Meta — publish DITOLAK di kode (rule #3).
const META_PLATFORMS: &[&str] = &["instagram", "facebook", "threads"];

fn api_key() -> Option<String> {
    secrets::api_key("blotato")
}

fn mock_mode() -> bool {
    std::env::var("MINIME_MOCK_LLM").is_ok()
}

pub async fn readiness() -> ToolStatus {
    match api_key() {
        Some(_) => ToolStatus::ready(
            super::TOOL_BLOTATO,
            "Blotato",
            "api",
            "API key Blotato tersedia — siap upload media & susun draft 9 platform.",
            "Draft tetap disusun sebagai file lokal tanpa API.",
        ),
        None => ToolStatus::not(
            super::TOOL_BLOTATO,
            "Blotato",
            "api",
            super::STATE_NOT_CONFIGURED,
            "API key Blotato belum diset (Keychain akun `blotato` / env MINIME_BLOTATO_KEY).",
            "Draft disusun sebagai file lokal; upload & publish menunggu key.",
        ),
    }
}

pub async fn run(db: &Db, op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    match op {
        "draft" => draft(db, params, ctx).await,
        "publish" => publish(db, params, ctx).await,
        "accounts" => accounts().await,
        _ => anyhow::bail!("op blotato tidak dikenal: '{op}' (draft/publish/accounts)"),
    }
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .context("gagal membuat HTTP client")
}

/// Ambil artefak MP4 terbaru dari runs.context (stage render) bila
/// params.video_path kosong.
async fn latest_video_artifact(db: &Db, run_id: &str) -> Option<String> {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    drop(guard);
    let stages = serde_json::from_str::<Value>(&ctx_raw)
        .ok()?
        .get("stages")?
        .as_object()?
        .clone();
    let mut found: Option<String> = None;
    for (_id, sv) in &stages {
        if let Some(arr) = sv.get("artifacts").and_then(|a| a.as_array()) {
            for a in arr {
                if let Some(p) = a.as_str() {
                    if p.ends_with(".mp4") {
                        found = Some(p.to_string());
                    }
                }
            }
        }
    }
    found
}

/// Ekstrak baris "CAPTION: ..." dari output stage skrip (konvensi pipeline).
async fn caption_from_skrip(db: &Db, run_id: &str) -> String {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    drop(guard);
    let out = serde_json::from_str::<Value>(&ctx_raw)
        .ok()
        .and_then(|v| {
            v.get("stages")?
                .get("skrip")?
                .get("output")?
                .as_str()
                .map(|s| s.to_string())
        })
        .unwrap_or_default();
    for line in out.lines() {
        if let Some(rest) = line.trim().strip_prefix("CAPTION:") {
            let c = rest.trim();
            if !c.is_empty() {
                return c.to_string();
            }
        }
    }
    String::new()
}

/// STEP 3 (draft): upload video ke Blotato + bundle draft lokal per platform.
/// TIDAK PERNAH memanggil POST /v2/posts — murni draft.
async fn draft(db: &Db, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    // 1. Video: params → artefak MP4 stage render → gagal eksplisit.
    let mut video_path = params
        .get("video_path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if video_path.is_empty() {
        if let Some(run_id) = &ctx.run_id {
            if let Some(p) = latest_video_artifact(db, run_id).await {
                video_path = p;
            }
        }
    }
    // Mode mock: video placeholder agar uji wiring pipeline deterministik.
    let mut mock_video = false;
    if video_path.is_empty() || !PathBuf::from(&video_path).is_file() {
        if mock_mode() {
            video_path = String::from("mock-video.mp4");
            mock_video = true;
        } else {
            anyhow::bail!(
                "video tidak ditemukan: isi params.video_path atau pastikan stage render menghasilkan artefak MP4"
            );
        }
    }
    // 2. Caption: params → stage skrip (CAPTION:) → kosong (tetap draft valid).
    let mut caption = params
        .get("caption")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if caption.is_empty() {
        if let Some(run_id) = &ctx.run_id {
            caption = caption_from_skrip(db, run_id).await;
        }
    }
    let hashtags: Vec<String> = params
        .get("hashtags")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let platforms: Vec<String> = params
        .get("platforms")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| PLATFORMS_9.iter().map(|s| s.to_string()).collect());
    if platforms.is_empty() {
        anyhow::bail!("params.platforms kosong");
    }
    for p in &platforms {
        if !PLATFORMS_9.contains(&p.as_str()) {
            anyhow::bail!("platform tidak dikenal: '{p}'");
        }
    }

    // 3. Upload media ke Blotato bila key ada (untuk draft siap-jadwal).
    //    Gagal upload ≠ gagal draft — draft lokal tetap dibuat (jujur di log).
    let mut media_url: Option<String> = None;
    let mut upload_note = "tanpa upload (API key Blotato belum diset)".to_string();
    if mock_mode() {
        upload_note = "MOCK: upload dilewati".to_string();
    } else if let Some(key) = api_key() {
        match upload_media(&key, &video_path).await {
            Ok(url) => {
                media_url = Some(url);
                upload_note = "media ter-upload ke Blotato".to_string();
            }
            Err(e) => {
                upload_note = format!("upload gagal ({e:#}) — draft lokal tetap dibuat");
            }
        }
    }

    // 4. Bundle draft lokal per platform.
    let dir = ctx.data_dir.join("drafts");
    std::fs::create_dir_all(&dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut drafts = Vec::new();
    for platform in &platforms {
        let meta_only = META_PLATFORMS.contains(&platform.as_str());
        let bundle = json!({
            "platform": platform,
            "video_path": video_path,
            "media_url": media_url,
            "caption": caption,
            "hashtags": hashtags,
            "status": "draft",
            "publish_note": if meta_only {
                "DITOLAK di kode: publikasi ke Meta hanya manual oleh Patria (rule #3)."
            } else {
                "Publish hanya via op 'publish' dengan approval_id yang sudah di-approve Patria."
            },
            "created_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        let file = dir.join(format!("blotato-draft-{platform}-{ts}.json"));
        std::fs::write(&file, serde_json::to_string_pretty(&bundle)?)?;
        drafts.push(json!({
            "platform": platform,
            "draft_path": file.to_string_lossy(),
        }));
    }
    Ok(json!({
        "drafts": drafts,
        "platforms": platforms,
        "upload": upload_note,
        "mock": mock_mode(),
        "mock_video": mock_video,
        "note": "DRAFT-ONLY: tidak ada yang dipublish. Menunggu approval Patria di inbox.",
    }))
}

/// Upload file lokal ke Blotato → mediaUrl (multipart POST /v2/media).
async fn upload_media(key: &str, video_path: &str) -> Result<String> {
    let bytes = std::fs::read(video_path).with_context(|| format!("gagal membaca {video_path}"))?;
    let filename = PathBuf::from(video_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4")
        .to_string();
    let part = reqwest::multipart::Part::bytes(bytes).file_name(filename);
    let form = reqwest::multipart::Form::new().part("file", part);
    let resp = client()?
        .post(format!("{BASE}/media"))
        .header("blotato-api-key", key)
        .multipart(form)
        .send()
        .await
        .context("gagal menghubungi Blotato /v2/media")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("respons Blotato bukan JSON")?;
    if !status.is_success() {
        anyhow::bail!(
            "Blotato upload error {status}: {}",
            body.get("message").unwrap_or(&json!("unknown"))
        );
    }
    body.get("mediaUrl")
        .or_else(|| body.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .context("Blotato tidak mengembalikan mediaUrl")
}

/// Publish TERKUNCI: Meta selalu ditolak; platform lain wajib approval approved.
async fn publish(db: &Db, params: &Value, _ctx: &ToolCtx) -> Result<Value> {
    let platform = params
        .get("platform")
        .and_then(|v| v.as_str())
        .context("params.platform wajib")?
        .to_lowercase();
    if !PLATFORMS_9.contains(&platform.as_str()) {
        anyhow::bail!("platform tidak dikenal: '{platform}'");
    }
    // Guard keras #1: Meta tidak bisa dipublish dari kode — titik.
    if META_PLATFORMS.contains(&platform.as_str()) {
        anyhow::bail!(
            "DITOLAK: publikasi ke '{platform}' diblokir di kode (rule #3 — tidak ada auto-posting ke Meta). Patria memposting manual."
        );
    }
    // Guard keras #2: wajib approval yang sudah di-approve Patria.
    let approval_id = params
        .get("approval_id")
        .and_then(|v| v.as_str())
        .context("params.approval_id wajib (publish hanya setelah Patria approve di inbox)")?;
    let status: String = {
        let guard = db.lock().await;
        guard
            .query_row(
                "SELECT status FROM approvals WHERE id = ?1",
                [approval_id],
                |r| r.get(0),
            )
            .with_context(|| format!("approval '{approval_id}' tidak ditemukan"))?
    };
    if status != "approved" {
        anyhow::bail!(
            "DITOLAK: approval '{approval_id}' berstatus '{status}' (bukan 'approved'). Publish dibatalkan."
        );
    }
    if mock_mode() {
        return Ok(json!({
            "mock": true,
            "platform": platform,
            "note": "MOCK: publish dilewati (approval terverifikasi di DB).",
        }));
    }
    let key = api_key().context("API key Blotato belum diset")?;
    let account_id = params
        .get("account_id")
        .and_then(|v| v.as_str())
        .context("params.account_id wajib (ID akun Blotato untuk platform ini)")?;
    let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let media_urls: Vec<String> = params
        .get("media_urls")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let resp = client()?
        .post(format!("{BASE}/posts"))
        .header("blotato-api-key", key)
        .json(&json!({
            "post": {
                "accountId": account_id,
                "content": {
                    "platform": platform,
                    "text": text,
                    "mediaUrls": media_urls,
                },
                "target": { "targetType": platform },
            }
        }))
        .send()
        .await
        .context("gagal menghubungi Blotato /v2/posts")?;
    let status_code = resp.status();
    let body: Value = resp.json().await.context("respons Blotato bukan JSON")?;
    if !status_code.is_success() {
        anyhow::bail!(
            "Blotato publish error {status_code}: {}",
            body.get("message").unwrap_or(&json!("unknown"))
        );
    }
    Ok(json!({
        "platform": platform,
        "approval_id": approval_id,
        "post_submission_id": body.get("postSubmissionId"),
    }))
}

/// Read-only: daftar akun Blotato terhubung.
async fn accounts() -> Result<Value> {
    let key = api_key().context("API key Blotato belum diset")?;
    let resp = client()?
        .get(format!("{BASE}/accounts"))
        .header("blotato-api-key", key)
        .send()
        .await
        .context("gagal menghubungi Blotato /v2/accounts")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("respons Blotato bukan JSON")?;
    if !status.is_success() {
        anyhow::bail!("Blotato accounts error {status}");
    }
    Ok(json!({ "accounts": body.get("accounts").unwrap_or(&json!([])) }))
}
