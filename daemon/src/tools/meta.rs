//! Koneksi Meta langsung: distribusi ke Instagram/Facebook/Threads — DRAFT-ONLY.
//!
//! Menggantikan Blotato untuk 3 platform Meta (keputusan Patria, 17 Okt 2026:
//! arsitektur hybrid — Meta langsung untuk IG/FB/Threads, Blotato tetap untuk
//! 6 platform non-Meta).
//!
//! ATURAN KERAS (rule #3): TIDAK ADA auto-posting ke Meta dalam kondisi apa
//! pun — konsisten dengan `instagram-cli` (Fase 4). Modul ini hanya boleh:
//!   - `draft`: menyusun bundle draft LOKAL (caption + media + hashtag +
//!     panduan posting manual). TIDAK PERNAH memanggil endpoint publish.
//!   - `accounts`: operasi baca read-only (verifikasi aset terhubung).
//! Setiap op yang menyerupai publikasi ("publish", "post", "upload", "share",
//! "send", "media_publish", "threads_publish", "feed", "reel") DITOLAK di
//! level kode.
//!
//! Endpoint resmi Meta (terverifikasi dari dokumentasi Meta, Okt 2026) —
//! didokumentasikan di sini agar tidak ditebak ulang:
//!
//! Instagram Graph API (hanya akun Bisnis/Kreator):
//!   1. `POST https://graph.facebook.com/v21.0/{ig-user-id}/media`
//!      (container: media_type=REELS, video_url, caption, share_to_feed)
//!   2. Poll `GET /{container-id}?fields=status_code` hingga FINISHED
//!   3. `POST /{ig-user-id}/media_publish` (creation_id)
//!   Permission: `instagram_business_basic` + `instagram_business_content_publish`
//!   Limit: 100 postingan API per 24 jam (rolling).
//!
//! Facebook Pages API:
//!   - `GET /{user-id}/accounts` → daftar Page + page access token
//!     (permission: `pages_show_list`)
//!   - `POST /{page-id}/feed` (message), `/photos`, `/videos`,
//!     `/video_reels` bertahap untuk reels (permission: `pages_manage_posts`)
//!
//! Threads API:
//!   1. `POST https://graph.threads.com/v1.0/{threads-user-id}/threads`
//!      (media_type, video_url/image_url, text)
//!   2. Tunggu ±30 detik → `POST /{threads-user-id}/threads_publish`
//!      (creation_id)
//!   Permission: `threads_basic` + `threads_content_publish`.
//!   Media harus di-host di server publik saat dipublish.
//!
//! Kredensial: Keychain akun `meta` / env MINIME_META_TOKEN (user access token
//! berumur panjang). Tidak di-log.

use super::{ToolCtx, ToolStatus};
use crate::db::Db;
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const GRAPH: &str = "https://graph.facebook.com/v21.0";
const GRAPH_THREADS: &str = "https://graph.threads.com/v1.0";

/// Platform yang dilayani tool ini (hanya Meta).
pub const META_DIRECT_PLATFORMS: &[&str] = &["instagram", "facebook", "threads"];

/// Ops yang menyerupai publikasi — selalu ditolak di kode (rule #3).
const BLOCKED_OPS: &[&str] = &[
    "publish",
    "post",
    "upload",
    "share",
    "send",
    "media_publish",
    "threads_publish",
    "feed",
    "reel",
    "go-live",
    "broadcast",
];

fn api_token() -> Option<String> {
    if let Ok(t) = std::env::var("MINIME_META_TOKEN") {
        let t = t.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    secrets::api_key("meta")
}

fn mock_mode() -> bool {
    std::env::var("MINIME_MOCK_LLM").is_ok()
}

pub async fn readiness() -> ToolStatus {
    match api_token() {
        Some(_) => ToolStatus::ready(
            super::TOOL_META,
            "Meta langsung",
            "api",
            "Token Meta tersedia — siap verifikasi akun (read-only) & susun draft IG/FB/Threads.",
            "Draft tetap disusun sebagai file lokal; publikasi ditolak di kode.",
        ),
        None => ToolStatus::not(
            super::TOOL_META,
            "Meta langsung",
            "api",
            super::STATE_NOT_CONFIGURED,
            "Token Meta belum diset (Keychain akun `meta` / env MINIME_META_TOKEN). Lihat docs/koneksi-meta.md.",
            "Draft disusun sebagai file lokal; verifikasi akun menunggu token.",
        ),
    }
}

pub async fn run(db: &Db, op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let op_l = op.to_lowercase();
    // Guard keras: tolak apa pun yang menyerupai publikasi (rule #3).
    if BLOCKED_OPS.iter().any(|b| op_l.contains(b)) {
        anyhow::bail!(
            "DITOLAK: op '{op}' menyerupai publikasi — auto-posting ke Meta dilarang (rule #3). Gunakan 'draft'."
        );
    }
    match op {
        "draft" => draft(db, params, ctx).await,
        "accounts" => accounts().await,
        _ => anyhow::bail!("op meta tidak dikenal: '{op}' (draft/accounts)"),
    }
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("gagal membuat HTTP client")
}

/// Susun bundle draft lokal untuk 1–3 platform Meta.
/// TIDAK PERNAH memanggil endpoint publish — murni file lokal + panduan manual.
///
/// Konvensi pipeline: `media`/`video_path` kosong → ambil artefak MP4 terbaru
/// dari runs.context (stage render); `caption` kosong → baris CAPTION: stage skrip.
async fn draft(db: &Db, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let platforms: Vec<String> = params
        .get("platforms")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string().to_lowercase()))
                .collect()
        })
        .unwrap_or_else(|| {
            META_DIRECT_PLATFORMS
                .iter()
                .map(|s| s.to_string())
                .collect()
        });
    if platforms.is_empty() {
        anyhow::bail!("params.platforms kosong");
    }
    for p in &platforms {
        if !META_DIRECT_PLATFORMS.contains(&p.as_str()) {
            anyhow::bail!(
                "platform '{p}' bukan Meta — tool ini hanya untuk instagram/facebook/threads (6 platform lain via postiz)"
            );
        }
    }
    let caption = params
        .get("caption")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let caption = if caption.is_empty() {
        if let Some(run_id) = &ctx.run_id {
            caption_from_skrip(db, run_id).await
        } else {
            String::new()
        }
    } else {
        caption
    };
    let mut media: Vec<String> = params
        .get("media")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    // Terima juga params.video_path tunggal (konvensi stage pipeline).
    if media.is_empty() {
        if let Some(vp) = params.get("video_path").and_then(|v| v.as_str()) {
            if !vp.is_empty() {
                media.push(vp.to_string());
            }
        }
    }
    // Fallback: artefak MP4 terbaru dari runs.context (stage render).
    if media.is_empty() {
        if let Some(run_id) = &ctx.run_id {
            if let Some(p) = latest_video_artifact(db, run_id).await {
                media.push(p);
            }
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
    // Verifikasi file media ada (draft valid merujuk file nyata).
    for m in &media {
        if !PathBuf::from(m).is_file() {
            if mock_mode() {
                continue;
            }
            anyhow::bail!("file media tidak ditemukan: {m}");
        }
    }

    let dir = ctx.data_dir.join("drafts");
    std::fs::create_dir_all(&dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut drafts = Vec::new();
    for platform in &platforms {
        let bundle = json!({
            "platform": platform,
            "caption": caption,
            "media": media,
            "hashtags": hashtags,
            "status": "draft",
            "publish_note": "DITOLAK di kode: publikasi ke Meta hanya manual oleh Patria (rule #3).",
            "manual_steps": manual_steps(platform, &media),
            "created_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        let file = dir.join(format!("meta-draft-{platform}-{ts}.json"));
        std::fs::write(&file, serde_json::to_string_pretty(&bundle)?)?;
        drafts.push(json!({
            "platform": platform,
            "draft_path": file.to_string_lossy(),
        }));
    }
    Ok(json!({
        "drafts": drafts,
        "platforms": platforms,
        "mock": mock_mode(),
        "note": "DRAFT-ONLY: tidak ada yang dipublish ke Meta. Serahkan bundle ke Patria untuk posting manual.",
    }))
}

/// Ambil artefak MP4 terbaru dari runs.context (stage render).
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

/// Panduan posting manual per platform (untuk Patria).
fn manual_steps(platform: &str, media: &[String]) -> Value {
    let kinds: Vec<&str> = media
        .iter()
        .map(|m| {
            if m.ends_with(".mp4") {
                "video"
            } else {
                "gambar"
            }
        })
        .collect();
    let media_desc = if kinds.is_empty() {
        "tanpa media".to_string()
    } else {
        kinds.join("+")
    };
    let steps: Vec<String> = match platform {
        "instagram" => vec![
            format!(
                "Buka aplikasi Instagram / Meta Business Suite → buat Reels baru ({media_desc})."
            ),
            "Tempel caption + hashtag dari bundle ini.".to_string(),
            "Posting manual — JANGAN via tool (rule #3).".to_string(),
        ],
        "facebook" => vec![
            "Buka Meta Business Suite → konten → buat postingan/Reels di FB Page.".to_string(),
            "Lampirkan media + tempel caption dari bundle ini.".to_string(),
            "Posting/jadwalkan manual — JANGAN via tool (rule #3).".to_string(),
        ],
        "threads" => vec![
            "Buka aplikasi Threads → buat post baru.".to_string(),
            "Lampirkan media + tempel teks dari bundle ini.".to_string(),
            "Posting manual — JANGAN via tool (rule #3).".to_string(),
        ],
        _ => vec!["Posting manual oleh Patria.".to_string()],
    };
    json!(steps)
}

/// Read-only: verifikasi aset Meta yang terhubung (FB Pages + IG Bisnis + Threads).
/// Tidak membuat/mengubah apa pun di sisi Meta.
async fn accounts() -> Result<Value> {
    if mock_mode() {
        return Ok(json!({
            "mock": true,
            "pages": [{"id": "mock-page-id", "name": "Mock FB Page"}],
            "instagram": [{"id": "mock-ig-id", "username": "mock.ig"}],
            "threads": [{"id": "mock-threads-id", "username": "mock.threads"}],
            "note": "MOCK: akun dilewati.",
        }));
    }
    let token = api_token().context(
        "Token Meta belum diset (Keychain akun `meta` / env MINIME_META_TOKEN). Lihat docs/koneksi-meta.md.",
    )?;

    // 1. FB Pages milik user.
    let pages_resp: Value = client()?
        .get(format!("{GRAPH}/me/accounts"))
        .query(&[
            ("fields", "id,name,tasks"),
            ("access_token", token.as_str()),
        ])
        .send()
        .await
        .context("gagal menghubungi Graph API /me/accounts")?
        .json()
        .await
        .context("respons /me/accounts bukan JSON")?;
    if let Some(err) = pages_resp.get("error") {
        anyhow::bail!(
            "Graph API error: {}",
            err.get("message").unwrap_or(&json!("unknown"))
        );
    }
    let pages: Vec<Value> = pages_resp
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    // 2. IG Business yang terhubung ke tiap Page.
    let mut instagram = Vec::new();
    for page in &pages {
        let page_id = page.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if page_id.is_empty() {
            continue;
        }
        let ig_resp: Value = client()?
            .get(format!("{GRAPH}/{page_id}"))
            .query(&[
                ("fields", "instagram_business_account{id,username,name}"),
                ("access_token", token.as_str()),
            ])
            .send()
            .await
            .context("gagal query instagram_business_account")?
            .json()
            .await
            .unwrap_or(json!({}));
        if let Some(ig) = ig_resp.get("instagram_business_account") {
            instagram.push(json!({
                "page_id": page_id,
                "page_name": page.get("name"),
                "account": ig,
            }));
        }
    }

    // 3. Threads profile (host terpisah).
    let threads_resp: Value = client()?
        .get(format!("{GRAPH_THREADS}/me"))
        .query(&[("fields", "id,username"), ("access_token", token.as_str())])
        .send()
        .await
        .context("gagal menghubungi Threads API /me")?
        .json()
        .await
        .unwrap_or(json!({}));
    let threads = if threads_resp.get("id").is_some() {
        vec![threads_resp]
    } else {
        vec![]
    };

    Ok(json!({
        "pages": pages.iter().map(|p| json!({"id": p.get("id"), "name": p.get("name")})).collect::<Vec<_>>(),
        "instagram": instagram,
        "threads": threads,
        "note": "Read-only: tidak ada yang dibuat/diubah di sisi Meta.",
    }))
}
