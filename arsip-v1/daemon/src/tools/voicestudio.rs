//! VoiceStudio wiring (Fase 4): REST/MCP untuk voiceover TTS.
//!
//! Instalasi di MacBook Patria masih pending → modul ini:
//! - readiness jujur (URL + kunci + probe),
//! - `tts` via REST bila siap,
//! - fallback ke `edge-tts` CLI bila tersedia (tanpa kunci).
//! Panduan instalasi: docs/voice-studio.md.

use super::{ToolCtx, ToolStatus};
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const ENV_URL: &str = "MINIME_VOICESTUDIO_URL";
const DEFAULT_URL: &str = "http://127.0.0.1:8765";
const SECRET_ACCOUNT: &str = "voice-studio";

fn base_url() -> String {
    std::env::var(ENV_URL)
        .ok()
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEFAULT_URL.to_string())
}

fn edge_tts_bin() -> Option<PathBuf> {
    super::find_bin(
        "MINIME_EDGE_TTS_BIN",
        &["edge-tts"],
        &["/opt/hatch/bin", "/usr/local/bin"],
    )
}

pub async fn readiness() -> ToolStatus {
    let url = base_url();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build();
    let reachable = match client {
        Ok(c) => c
            .get(format!("{url}/health"))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false),
        Err(_) => false,
    };
    let key = secrets::api_key(SECRET_ACCOUNT).is_some();
    let edge = edge_tts_bin().is_some();
    let fallback = if edge {
        "Fallback siap: edge-tts CLI (tanpa kunci)."
    } else {
        "Fallback: instal edge-tts (`pip install edge-tts`) atau hubungi Patria untuk instalasi VoiceStudio."
    };
    if reachable && key {
        ToolStatus::ready(
            super::TOOL_VOICE_STUDIO,
            "VoiceStudio",
            "rest",
            &format!("Service terjangkau di {url}; kunci tersedia."),
            fallback,
        )
    } else if reachable {
        ToolStatus::not(
            super::TOOL_VOICE_STUDIO,
            "VoiceStudio",
            "rest",
            super::STATE_NO_KEY,
            &format!("Service terjangkau di {url}, tetapi kunci belum disimpan (secrets.set account='voice-studio')."),
            fallback,
        )
    } else {
        ToolStatus::not(
            super::TOOL_VOICE_STUDIO,
            "VoiceStudio",
            "rest",
            super::STATE_UNREACHABLE,
            &format!("Service tidak terjangkau di {url} — instalasi MacBook masih pending (lihat docs/voice-studio.md)."),
            fallback,
        )
    }
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    match op {
        "tts" => {
            let text = params
                .get("text")
                .and_then(|v| v.as_str())
                .context("params.text wajib")?;
            if text.trim().is_empty() {
                anyhow::bail!("params.text kosong");
            }
            // 1. Coba VoiceStudio REST bila siap.
            if let Ok(v) = tts_voicestudio(params, text, ctx).await {
                return Ok(v);
            }
            // 2. Fallback: edge-tts CLI (jujur dicatat di hasil).
            tts_edge_fallback(params, text, ctx).await
        }
        _ => anyhow::bail!("op voice-studio tidak dikenal: '{op}' (tts)"),
    }
}

async fn tts_voicestudio(params: &Value, text: &str, ctx: &ToolCtx) -> Result<Value> {
    let url = base_url();
    let key = secrets::api_key(SECRET_ACCOUNT)
        .with_context(|| "kunci voice-studio belum dikonfigurasi")?;
    let lang = params.get("lang").and_then(|v| v.as_str()).unwrap_or("id");
    let voice = params.get("voice").and_then(|v| v.as_str()).unwrap_or("");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .context("buat HTTP client")?;
    let mut body = json!({"text": text, "lang": lang});
    if !voice.is_empty() {
        body["voice"] = json!(voice);
    }
    let resp = client
        .post(format!("{url}/v1/tts"))
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("POST {url}/v1/tts"))?;
    if !resp.status().is_success() {
        anyhow::bail!("voice-studio HTTP {}", resp.status());
    }
    let bytes = resp.bytes().await.context("baca audio")?;
    if bytes.len() < 1024 {
        anyhow::bail!(
            "respons audio terlalu kecil ({} bytes) — kemungkinan error",
            bytes.len()
        );
    }
    let dir = ctx.artifact_dir()?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let file = dir.join(format!("voicestudio-{ts}.mp3"));
    std::fs::write(&file, &bytes)?;
    Ok(json!({
        "engine": "voice-studio",
        "artifact_path": file.to_string_lossy(),
        "bytes": bytes.len(),
    }))
}

async fn tts_edge_fallback(params: &Value, text: &str, ctx: &ToolCtx) -> Result<Value> {
    let bin = edge_tts_bin()
        .with_context(|| "VoiceStudio tidak siap dan edge-tts tidak ditemukan — instal salah satunya (docs/voice-studio.md)")?;
    let voice = params
        .get("voice")
        .and_then(|v| v.as_str())
        .unwrap_or("id-ID-ArdiNeural");
    let dir = ctx.artifact_dir()?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let file = dir.join(format!("edge-tts-{ts}.mp3"));
    super::run_cmd(
        &bin,
        &[
            "--text",
            text,
            "--voice",
            voice,
            "--write-media",
            &file.to_string_lossy(),
        ],
        None,
        Duration::from_secs(180),
    )
    .await
    .with_context(|| "edge-tts gagal")?;
    Ok(json!({
        "engine": "edge-tts (fallback)",
        "artifact_path": file.to_string_lossy(),
        "note": "VoiceStudio belum siap — dipakai fallback edge-tts.",
    }))
}
