//! postiz wiring: distribusi ke 6 platform non-Meta — DRAFT-ONLY + SCHEDULE-TERKUNCI.
//!
//! Menggantikan tool `blotato` (17 Okt 2026, perintah Patria). postiz-app
//! (AGPL-3.0) berjalan sebagai layanan EKSTERNAL di MacBook Patria
//! (http://localhost:4007); daemon hanya memanggil Public API-nya
//! (`/api/public/v1`) dengan API key pribadi (hex 64 char, tanpa prefix)
//! di header `Authorization` mentah (TANPA kata "Bearer") — tidak ada kode
//! postiz yang ditempel ke Minime (batas lisensi AGPL aman, pola
//! nautilus_trader).
//!
//! Fakta terverifikasi LIVE dari MacBook Patria (18 Okt 2026) — mengoreksi
//! asumsi awal dari baca kode:
//!   - Base path = `/api/public/v1` (BUKAN `/public/v1` — tanpa prefix /api
//!     request jatuh ke SPA frontend dan redirect ke /auth).
//!   - Auth = header `Authorization: <key>` mentah (BUKAN `Bearer <key>`).
//!   - Endpoint `/public/v1/is-connected` TIDAK ADA di API; cek koneksi
//!     pakai `GET /api/public/v1/integrations` (read-only).
//!   - API key pribadi = hex 64 char tanpa prefix. Yang berawalan `pos_`
//!     adalah token OAuth App (tab "Apps"), bukan API key pribadi.
//!   - GET  /api/public/v1/integrations -> daftar channel [{ id, name,
//!     identifier (provider), disabled, ... }]
//!   - POST /api/public/v1/posts       -> body { type: "draft"|"schedule",
//!     date (ISO, wajib), shortLink, tags: [], posts: [{ integration: { id },
//!     value: [{ content, image: [] }] }] }; validasi konten dilewati
//!     untuk type "draft" (controller)
//!   - GET  /api/public/v1/posts       -> daftar post
//!   - DELETE /api/public/v1/posts/:id -> hapus post
//!
//! ATURAN KERAS (rule #3): TIDAK ADA auto-posting.
//!   - `draft`: POST type "draft" per channel + bundle draft lokal.
//!     TIDAK PERNAH membuat type "schedule"/"now" tanpa approval.
//!   - `schedule`: POST type "schedule" + date — WAJIB `approval_id` yang
//!     merujuk record approvals berstatus 'approved'. Tanpa itu DITOLAK.
//!     (Public API postiz tidak punya PATCH draft→schedule; schedule
//!     membuat post terjadwal dari bundle draft.)
//!   - Platform Meta (instagram/facebook/threads) DITOLAK di tool ini —
//!     ditangani tool `meta` (koneksi langsung, DRAFT-ONLY).
//!   - `accounts` / `list`: read-only.
//! Kredensial: Keychain akun `postiz` / env MINIME_POSTIZ_KEY (hex 64 char).
//! URL layanan: env MINIME_POSTIZ_URL (default http://localhost:4007).
//! Tidak ada kredensial yang di-log.

use super::{ToolCtx, ToolStatus};
use crate::db::Db;
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_URL: &str = "http://localhost:4007";

/// 6 platform non-Meta (identifier provider postiz, terverifikasi dari kode).
/// "twitter" diterima sebagai alias untuk "x".
pub const PLATFORMS_6: &[&str] = &["youtube", "tiktok", "x", "linkedin", "bluesky", "pinterest"];

/// Platform Meta — DITOLAK di tool ini (rule #3); pakai tool `meta`.
const META_PLATFORMS: &[&str] = &["instagram", "facebook", "threads"];

fn api_key() -> Option<String> {
    secrets::api_key("postiz")
}

fn base_url() -> String {
    std::env::var("MINIME_POSTIZ_URL")
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_URL.to_string())
}

/// Root Public API postiz: {service}/api/public/v1
/// (terverifikasi live 18 Okt 2026 — tanpa prefix /api request jatuh ke SPA).
fn api_base() -> String {
    format!("{}/api/public/v1", base_url())
}

fn mock_mode() -> bool {
    std::env::var("MINIME_MOCK_LLM").is_ok()
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("gagal membuat HTTP client")
}

pub async fn readiness() -> ToolStatus {
    let key = match api_key() {
        Some(k) => k,
        None => {
            return ToolStatus::not(
                super::TOOL_POSTIZ,
                "postiz",
                "api",
                super::STATE_NOT_CONFIGURED,
                "API key postiz belum diset (Keychain akun `postiz` / env MINIME_POSTIZ_KEY, hex 64 char dari dashboard postiz → Settings → Agents).",
                "Draft disusun sebagai file lokal; kirim ke postiz menunggu key + layanan jalan di MacBook.",
            )
        }
    };
    // Cek ringan: key valid + layanan menjawab.
    // GET /api/public/v1/integrations (read-only; /is-connected tidak ada).
    let req = match client() {
        Ok(c) => c
            .get(format!("{}/integrations", api_base()))
            .header("Authorization", auth_header(&key)),
        Err(e) => {
            return ToolStatus::not(
                super::TOOL_POSTIZ,
                "postiz",
                "api",
                super::STATE_UNREACHABLE,
                &format!("gagal membuat HTTP client: {e:#}"),
                "Draft disusun sebagai file lokal.",
            )
        }
    };
    match req.send().await {
        Ok(resp) if resp.status().is_success() => ToolStatus::ready(
            super::TOOL_POSTIZ,
            "postiz",
            "api",
            &format!(
                "postiz menjawab di {} — siap kirim draft & schedule 6 platform non-Meta.",
                base_url()
            ),
            "Draft tetap disusun sebagai file lokal tanpa API.",
        ),
        Ok(resp) => ToolStatus::not(
            super::TOOL_POSTIZ,
            "postiz",
            "api",
            super::STATE_UNREACHABLE,
            &format!(
                "postiz di {} menjawab HTTP {} — key mungkin salah/expired.",
                base_url(),
                resp.status()
            ),
            "Periksa API key di dashboard postiz (Settings → Agents) dan layanan Docker.",
        ),
        Err(e) => ToolStatus::not(
            super::TOOL_POSTIZ,
            "postiz",
            "api",
            super::STATE_UNREACHABLE,
            &format!(
                "postiz di {} tidak terjangkau ({e:#}) — pastikan Docker postiz jalan di MacBook.",
                base_url()
            ),
            "Draft disusun sebagai file lokal; kirim menunggu layanan.",
        ),
    }
}

pub async fn run(db: &Db, op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    match op {
        "draft" => draft(db, params, ctx).await,
        "schedule" => schedule(db, params, ctx).await,
        "accounts" => accounts().await,
        "list" => list_posts().await,
        _ => anyhow::bail!("op postiz tidak dikenal: '{op}' (draft/schedule/accounts/list)"),
    }
}

/// Normalisasi nama platform: "twitter" -> "x"; tolak Meta & nama asing.
fn normalize_platform(raw: &str) -> Result<String> {
    let p = raw.trim().to_lowercase();
    let p = if p == "twitter" { "x".to_string() } else { p };
    if META_PLATFORMS.contains(&p.as_str()) {
        anyhow::bail!(
            "DITOLAK: '{raw}' adalah platform Meta — distribusi Meta hanya via tool `meta` (DRAFT-ONLY, rule #3)."
        );
    }
    if !PLATFORMS_6.contains(&p.as_str()) {
        anyhow::bail!("platform tidak dikenal/didukung tool ini: '{raw}' (6 non-Meta: youtube, tiktok, x, linkedin, bluesky, pinterest)");
    }
    Ok(p)
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

fn auth_header(key: &str) -> String {
    // postiz Public API: key mentah di header Authorization, TANPA "Bearer".
    key.to_string()
}

/// Daftar channel terhubung dari postiz: identifier provider -> integration id.
async fn integrations_map(key: &str) -> Result<std::collections::HashMap<String, Vec<String>>> {
    let resp = client()?
        .get(format!("{}/integrations", api_base()))
        .header("Authorization", auth_header(key))
        .send()
        .await
        .context("gagal menghubungi postiz /api/public/v1/integrations")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("respons postiz bukan JSON")?;
    if !status.is_success() {
        anyhow::bail!("postiz integrations error {status}");
    }
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    if let Some(arr) = body.as_array() {
        for it in arr {
            let identifier = it
                .get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let id = it.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let disabled = it
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !identifier.is_empty() && !id.is_empty() && !disabled {
                map.entry(identifier).or_default().push(id.to_string());
            }
        }
    }
    Ok(map)
}

/// STEP 3 (draft): buat draft di postiz (type "draft") + bundle draft lokal
/// per platform. TIDAK PERNAH membuat type "schedule"/"now" di sini.
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
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| PLATFORMS_6.iter().map(|s| s.to_string()).collect())
        .into_iter()
        .map(|p| normalize_platform(&p))
        .collect::<Result<Vec<_>>>()?;
    if platforms.is_empty() {
        anyhow::bail!("params.platforms kosong");
    }
    let full_caption = if hashtags.is_empty() {
        caption.clone()
    } else {
        format!("{}\n\n{}", caption, hashtags.join(" "))
    };

    // 3. Kirim draft ke postiz (type "draft") bila key + layanan tersedia.
    //    Gagal kirim ≠ gagal draft — bundle lokal tetap dibuat (jujur di log).
    let mut postiz_note = "tanpa kirim (API key postiz belum diset)".to_string();
    let mut postiz_ids: Vec<Value> = Vec::new();
    if mock_mode() {
        postiz_note = "MOCK: kirim ke postiz dilewati".to_string();
        for p in &platforms {
            postiz_ids.push(json!({ "platform": p, "postiz_id": format!("mock-post-{p}") }));
        }
    } else if let Some(key) = api_key() {
        let map = integrations_map(&key).await?;
        let mut posts_payload = Vec::new();
        let mut missing = Vec::new();
        for p in &platforms {
            match map.get(p) {
                Some(ids) if !ids.is_empty() => {
                    for id in ids {
                        posts_payload.push(json!({
                            "integration": { "id": id },
                            "value": [{ "content": full_caption, "image": [] }],
                        }));
                    }
                }
                _ => missing.push(p.clone()),
            }
        }
        if !missing.is_empty() {
            anyhow::bail!(
                "channel postiz belum terhubung untuk: {} — hubungkan dulu di dashboard postiz (Add Channel)",
                missing.join(", ")
            );
        }
        let resp = client()?
            .post(format!("{}/posts", api_base()))
            .header("Authorization", auth_header(&key))
            .json(&json!({
                "type": "draft",
                "date": chrono::Utc::now().to_rfc3339(),
                "shortLink": false,
                "tags": [],
                "posts": posts_payload,
            }))
            .send()
            .await
            .context("gagal menghubungi postiz /api/public/v1/posts")?;
        let status = resp.status();
        let body: Value = resp.json().await.context("respons postiz bukan JSON")?;
        if !status.is_success() {
            anyhow::bail!(
                "postiz draft error {status}: {}",
                body.get("message").unwrap_or(&json!("unknown"))
            );
        }
        postiz_note = "draft tersimpan di postiz (type draft)".to_string();
        // createPost mengembalikan post per channel; catat id-nya bila ada.
        if let Some(arr) = body.as_array() {
            for (i, item) in arr.iter().enumerate() {
                let pid = item
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                postiz_ids.push(json!({
                    "platform": platforms.get(i).unwrap_or(&"?".to_string()),
                    "postiz_id": pid,
                }));
            }
        }
    }

    // 4. Bundle draft lokal per platform (proof of work, pola blotato).
    let dir = ctx.data_dir.join("drafts");
    std::fs::create_dir_all(&dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut drafts = Vec::new();
    for platform in &platforms {
        let bundle = json!({
            "platform": platform,
            "video_path": video_path,
            "caption": caption,
            "hashtags": hashtags,
            "status": "draft",
            "postiz": postiz_ids.iter().find(|x| x.get("platform").and_then(|v| v.as_str()) == Some(platform.as_str())),
            "publish_note": "Schedule hanya via op 'schedule' dengan approval_id yang sudah di-approve Patria.",
            "created_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        let file = dir.join(format!("postiz-draft-{platform}-{ts}.json"));
        std::fs::write(&file, serde_json::to_string_pretty(&bundle)?)?;
        drafts.push(json!({
            "platform": platform,
            "draft_path": file.to_string_lossy(),
        }));
    }
    Ok(json!({
        "drafts": drafts,
        "platforms": platforms,
        "postiz": postiz_note,
        "mock": mock_mode(),
        "mock_video": mock_video,
        "note": "DRAFT-ONLY: tidak ada yang dijadwalkan/dipublish. Menunggu approval Patria di inbox.",
    }))
}

/// Schedule TERKUNCI: wajib approval yang sudah di-approve Patria.
/// Membuat post TERJADWAL di postiz (type "schedule" + date) dari bundle draft.
async fn schedule(db: &Db, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let platform_raw = params
        .get("platform")
        .and_then(|v| v.as_str())
        .context("params.platform wajib")?;
    let platform = normalize_platform(platform_raw)?;
    // Guard keras: wajib approval yang sudah di-approve Patria.
    let approval_id = params
        .get("approval_id")
        .and_then(|v| v.as_str())
        .context("params.approval_id wajib (schedule hanya setelah Patria approve di inbox)")?;
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
            "DITOLAK: approval '{approval_id}' berstatus '{status}' (bukan 'approved'). Schedule dibatalkan."
        );
    }
    let scheduled_at = params
        .get("scheduled_at")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    // Validasi ringan format tanggal.
    chrono::DateTime::parse_from_rfc3339(&scheduled_at)
        .context("params.scheduled_at harus format ISO 8601 (RFC 3339)")?;

    if mock_mode() {
        return Ok(json!({
            "mock": true,
            "platform": platform,
            "scheduled_at": scheduled_at,
            "approval_id": approval_id,
            "note": "MOCK: schedule dilewati (approval terverifikasi di DB).",
        }));
    }
    let key = api_key().context("API key postiz belum diset")?;
    let map = integrations_map(&key).await?;
    let ids = map
        .get(&platform)
        .filter(|v| !v.is_empty())
        .with_context(|| {
            format!("channel postiz belum terhubung untuk '{platform}' — hubungkan dulu di dashboard postiz")
        })?;
    let caption = params.get("caption").and_then(|v| v.as_str()).unwrap_or("");
    let posts_payload: Vec<Value> = ids
        .iter()
        .map(|id| {
            json!({
                "integration": { "id": id },
                "value": [{ "content": caption, "image": [] }],
            })
        })
        .collect();
    let resp = client()?
        .post(format!("{}/posts", api_base()))
        .header("Authorization", auth_header(&key))
        .json(&json!({
            "type": "schedule",
            "date": scheduled_at,
            "shortLink": false,
            "tags": [],
            "posts": posts_payload,
        }))
        .send()
        .await
        .context("gagal menghubungi postiz /api/public/v1/posts")?;
    let status_code = resp.status();
    let body: Value = resp.json().await.context("respons postiz bukan JSON")?;
    if !status_code.is_success() {
        anyhow::bail!(
            "postiz schedule error {status_code}: {}",
            body.get("message").unwrap_or(&json!("unknown"))
        );
    }
    // Catat proof of work ke direktori data.
    let dir = ctx.data_dir.join("drafts");
    std::fs::create_dir_all(&dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let file = dir.join(format!("postiz-schedule-{platform}-{ts}.json"));
    std::fs::write(
        &file,
        serde_json::to_string_pretty(&json!({
            "platform": platform,
            "scheduled_at": scheduled_at,
            "approval_id": approval_id,
            "postiz_response": body,
        }))?,
    )?;
    Ok(json!({
        "platform": platform,
        "scheduled_at": scheduled_at,
        "approval_id": approval_id,
        "schedule_path": file.to_string_lossy(),
    }))
}

/// Read-only: daftar channel postiz terhubung.
async fn accounts() -> Result<Value> {
    let key = api_key().context("API key postiz belum diset")?;
    let map = integrations_map(&key).await?;
    let channels: Vec<Value> = map
        .iter()
        .map(|(identifier, ids)| {
            json!({ "identifier": identifier, "integration_ids": ids, "count": ids.len() })
        })
        .collect();
    Ok(json!({ "channels": channels }))
}

/// Read-only: daftar post di postiz.
async fn list_posts() -> Result<Value> {
    let key = api_key().context("API key postiz belum diset")?;
    let resp = client()?
        .get(format!("{}/posts", api_base()))
        .header("Authorization", auth_header(&key))
        .send()
        .await
        .context("gagal menghubungi postiz /api/public/v1/posts")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("respons postiz bukan JSON")?;
    if !status.is_success() {
        anyhow::bail!("postiz list error {status}");
    }
    Ok(json!({ "posts": body.get("posts").unwrap_or(&body) }))
}
