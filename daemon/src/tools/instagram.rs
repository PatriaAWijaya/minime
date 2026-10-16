//! instagram-cli wiring (Fase 4): DRAFT-ONLY.
//!
//! ATURAN KERAS (rule #3): tidak ada auto-posting ke Meta dalam kondisi apa pun.
//! Modul ini hanya boleh:
//!   - `draft`: menyusun bundle draft (caption + media + hashtag) sebagai FILE LOKAL,
//!   - `account`: operasi baca read-only via CLI (info akun).
//! Setiap op yang menyerupai publikasi ("post", "publish", "upload", "share",
//! "send", "story") DITOLAK di level kode — bukan sekadar tidak diimplementasikan.

use super::{ToolCtx, ToolStatus};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

/// Ops yang menyerupai publikasi — selalu ditolak.
const BLOCKED_OPS: &[&str] = &[
    "post",
    "publish",
    "upload",
    "share",
    "send",
    "story",
    "reel",
    "go-live",
    "broadcast",
];

fn instagram_bin() -> Option<PathBuf> {
    super::find_bin(
        "MINIME_INSTAGRAM_CLI_BIN",
        &["instagram-cli"],
        &["/opt/hatch/bin", "/usr/local/bin"],
    )
}

pub async fn readiness() -> ToolStatus {
    match instagram_bin() {
        Some(p) => ToolStatus::ready(
            super::TOOL_INSTAGRAM,
            "Instagram CLI",
            "cli",
            &format!(
                "CLI ditemukan: {} — mode DRAFT-ONLY (publikasi ditolak di kode).",
                p.display()
            ),
            "Tidak ada fallback publikasi — draft diserahkan ke Patria untuk posting manual.",
        ),
        None => ToolStatus::not(
            super::TOOL_INSTAGRAM,
            "Instagram CLI",
            "cli",
            super::STATE_NOT_INSTALLED,
            "CLI `instagram-cli` tidak ditemukan (set MINIME_INSTAGRAM_CLI_BIN).",
            "Draft tetap bisa disusun sebagai file lokal tanpa CLI.",
        ),
    }
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let op_l = op.to_lowercase();
    // Guard keras: tolak apa pun yang menyerupai publikasi.
    if BLOCKED_OPS.iter().any(|b| op_l.contains(b)) {
        anyhow::bail!(
            "DITOLAK: op '{op}' menyerupai publikasi — auto-posting ke Meta dilarang (rule #3). Gunakan 'draft'."
        );
    }
    match op {
        "draft" => {
            let caption = params
                .get("caption")
                .and_then(|v| v.as_str())
                .context("params.caption wajib")?;
            let media: Vec<String> = params
                .get("media")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let hashtags: Vec<String> = params
                .get("hashtags")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let kind = params
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("post");
            // Verifikasi file media ada (draft yang valid merujuk file nyata).
            for m in &media {
                if !PathBuf::from(m).is_file() {
                    anyhow::bail!("file media tidak ditemukan: {m}");
                }
            }
            let dir = ctx.data_dir.join("drafts");
            std::fs::create_dir_all(&dir)?;
            let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
            let file = dir.join(format!("ig-draft-{ts}.json"));
            let bundle = json!({
                "kind": kind,
                "caption": caption,
                "media": media,
                "hashtags": hashtags,
                "status": "draft — MENUNGGU posting manual oleh Patria (auto-posting dilarang)",
                "created_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
            std::fs::write(&file, serde_json::to_string_pretty(&bundle)?)?;
            Ok(json!({
                "draft_path": file.to_string_lossy(),
                "note": "Draft tersimpan lokal. TIDAK diposting — serahkan ke Patria.",
            }))
        }
        "account" => {
            // Read-only passthrough (mis. info akun). Argumen eksplisit, tanpa default berbahaya.
            let bin = instagram_bin().with_context(|| "CLI `instagram-cli` tidak ditemukan")?;
            let args: Vec<String> = params
                .get("args")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            if args.is_empty() {
                anyhow::bail!("params.args wajib (contoh: [\"accounts\"])");
            }
            // Guard kedua: argumen pun tidak boleh memicu publikasi.
            let joined = args.join(" ").to_lowercase();
            if BLOCKED_OPS.iter().any(|b| joined.contains(b)) {
                anyhow::bail!("DITOLAK: argumen menyerupai publikasi (rule #3).");
            }
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let stdout = super::run_cmd(&bin, &arg_refs, None, Duration::from_secs(60)).await?;
            Ok(json!({"output": stdout.chars().take(3000).collect::<String>()}))
        }
        _ => anyhow::bail!("op instagram-cli tidak dikenal: '{op}' (draft/account)"),
    }
}
