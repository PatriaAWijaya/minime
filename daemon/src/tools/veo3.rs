//! Veo3 wiring via fal.ai queue API (pipeline konten video otomatis).
//!
//! Adaptasi STEP 2 diagram n8n Patria: call Veo3 API (queue) → tunggu → ambil
//! URL video final → log. Model LLM di STEP 1 diganti routing provider Minime.
//! - readiness: cek API key fal (Keychain akun `fal` / env MINIME_FAL_KEY).
//! - render: prompt → POST queue.fal.run/{model} → poll status → unduh MP4.
//!   Tanpa key → FALLBACK jujur: render lokal via HyperFrames (dicatat di log).
//! - MINIME_MOCK_LLM=1 → hasil mock eksplisit (uji wiring pipeline).
//! Kredensial tidak pernah di-log (rule #2).

use super::{ToolCtx, ToolStatus};
use crate::db::Db;
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

const FAL_QUEUE_BASE: &str = "https://queue.fal.run";
const DEFAULT_MODEL: &str = "fal-ai/veo3";
/// Poll tiap 15 detik, maksimal ~10 menit (diagram: tunggu ±5 menit).
const POLL_EVERY: Duration = Duration::from_secs(15);
const POLL_MAX: usize = 40;

fn api_key() -> Option<String> {
    secrets::api_key("fal")
}

fn mock_mode() -> bool {
    std::env::var("MINIME_MOCK_LLM").is_ok()
}

pub async fn readiness() -> ToolStatus {
    match api_key() {
        Some(_) => ToolStatus::ready(
            super::TOOL_VEO3,
            "Veo3 (fal.ai)",
            "api",
            "API key fal.ai tersedia — siap render video via Veo3.",
            "Render lokal via HyperFrames/manimgx bila API bermasalah.",
        ),
        None => ToolStatus::not(
            super::TOOL_VEO3,
            "Veo3 (fal.ai)",
            "api",
            super::STATE_NOT_CONFIGURED,
            "API key fal.ai belum diset (Keychain akun `fal` / env MINIME_FAL_KEY).",
            "Fallback otomatis: render lokal via HyperFrames (dicatat jujur di log).",
        ),
    }
}

pub async fn run(db: &Db, op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    match op {
        "render" => render(db, params, ctx).await,
        _ => anyhow::bail!("op veo3 tidak dikenal: '{op}' (render)"),
    }
}

/// Ambil output stage "skrip" (atau stage terakhir yang punya output) dari
/// runs.context — supaya prompt yang dibuat LLM stage 1 bisa dipakai stage 2
/// tanpa harus di-hardcode di YAML.
async fn prev_stage_output(db: &Db, run_id: &str) -> String {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    drop(guard);
    let stages = serde_json::from_str::<Value>(&ctx_raw)
        .ok()
        .and_then(|v| v.get("stages").cloned())
        .and_then(|s| s.as_object().cloned());
    let Some(stages) = stages else {
        return String::new();
    };
    // Prioritas: stage "skrip" (konvensi pipeline konten-video-otomatis).
    for key in ["skrip", "script", "konsep"] {
        if let Some(out) = stages
            .get(key)
            .and_then(|s| s.get("output"))
            .and_then(|o| o.as_str())
        {
            if !out.trim().is_empty() {
                return out.to_string();
            }
        }
    }
    // Fallback: output stage terakhir yang tidak kosong.
    stages
        .values()
        .rfind(|s| {
            s.get("output")
                .and_then(|o| o.as_str())
                .map(|o| !o.trim().is_empty())
                .unwrap_or(false)
        })
        .and_then(|s| s.get("output"))
        .and_then(|o| o.as_str())
        .unwrap_or("")
        .to_string()
}

/// Ekstrak baris "PROMPT_VEO3: ..." dari output stage skrip (konvensi pipeline).
fn extract_prompt_veo3(text: &str) -> Option<String> {
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("PROMPT_VEO3:") {
            let p = rest.trim().to_string();
            if !p.is_empty() {
                return Some(p);
            }
        }
    }
    None
}

async fn render(db: &Db, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    // Mode mock: hasil eksplisit untuk uji wiring — tanpa prompt, tanpa API.
    if mock_mode() {
        let output = params
            .get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("video-harian.mp4");
        let dir = ctx.artifact_dir()?;
        let video_path = dir.join(output);
        return Ok(json!({
            "mock": true,
            "video_path": video_path.to_string_lossy(),
            "note": "MOCK (MINIME_MOCK_LLM=1): render Veo3 dilewati — file MP4 tidak dibuat. Uji wiring saja.",
        }));
    }

    let duration_s = params
        .get("duration_s")
        .and_then(|v| v.as_u64())
        .unwrap_or(8)
        .clamp(1, 60);
    let aspect_ratio = params
        .get("aspect_ratio")
        .and_then(|v| v.as_str())
        .unwrap_or("16:9");
    let model = params
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_MODEL);
    let output = params
        .get("output")
        .and_then(|v| v.as_str())
        .unwrap_or("video-harian.mp4");

    // 1. prompt eksplisit → 2. output stage skrip → 3. gagal eksplisit.
    let mut prompt = params
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let mut prompt_source = "params.prompt";
    if prompt.is_empty() {
        if let Some(run_id) = &ctx.run_id {
            let prev = prev_stage_output(db, run_id).await;
            if let Some(p) = extract_prompt_veo3(&prev) {
                prompt = p;
                prompt_source = "stage skrip (PROMPT_VEO3:)";
            } else if !prev.trim().is_empty() {
                prompt = prev.chars().take(1000).collect();
                prompt_source = "stage skrip (output penuh)";
            }
        }
    }
    if prompt.is_empty() {
        anyhow::bail!(
            "prompt kosong: isi params.prompt atau pastikan stage skrip menulis 'PROMPT_VEO3:'"
        );
    }

    // Mode mock: hasil eksplisit untuk uji wiring (bukan render beneran).
    if mock_mode() {
        let dir = ctx.artifact_dir()?;
        let video_path = dir.join(output);
        return Ok(json!({
            "mock": true,
            "video_path": video_path.to_string_lossy(),
            "prompt_source": prompt_source,
            "note": "MOCK (MINIME_MOCK_LLM=1): render Veo3 dilewati — file MP4 tidak dibuat. Uji wiring saja.",
        }));
    }

    match api_key() {
        Some(key) => render_fal(&key, model, &prompt, duration_s, aspect_ratio, output, ctx)
            .await
            .map(|mut v| {
                v["prompt_source"] = json!(prompt_source);
                v
            }),
        None => {
            // Fallback jujur: HyperFrames lokal (tanpa video model).
            let v = fallback_hyperframes(&prompt, output, ctx).await?;
            Ok(json!({
                "fallback": "hyperframes",
                "video_path": v["video_path"],
                "prompt_source": prompt_source,
                "note": "JUJUR: API key fal.ai belum diset — video di-render lokal via HyperFrames (HTML→MP4), BUKAN Veo3. Minta key ke Patria untuk render AI asli.",
            }))
        }
    }
}

/// Alur queue fal.ai: POST → request_id → poll status → unduh video.
async fn render_fal(
    key: &str,
    model: &str,
    prompt: &str,
    duration_s: u64,
    aspect_ratio: &str,
    output: &str,
    ctx: &ToolCtx,
) -> Result<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    // 1. Submit queue.
    let submit: Value = client
        .post(format!("{FAL_QUEUE_BASE}/{model}"))
        .header("Authorization", format!("Key {key}"))
        .json(&json!({
            "prompt": prompt,
            "duration": format!("{duration_s}s"),
            "aspect_ratio": aspect_ratio,
        }))
        .send()
        .await
        .context("gagal menghubungi fal.ai queue")?
        .json()
        .await
        .context("respons submit fal.ai bukan JSON")?;
    let request_id = submit
        .get("request_id")
        .and_then(|v| v.as_str())
        .context("fal.ai tidak mengembalikan request_id")?
        .to_string();

    // 2. Poll status.
    let status_url = format!("{FAL_QUEUE_BASE}/{model}/requests/{request_id}/status");
    let mut video_url: Option<String> = None;
    for _ in 0..POLL_MAX {
        tokio::time::sleep(POLL_EVERY).await;
        let st: Value = client
            .get(&status_url)
            .header("Authorization", format!("Key {key}"))
            .send()
            .await
            .context("gagal poll status fal.ai")?
            .json()
            .await
            .context("respons status fal.ai bukan JSON")?;
        match st.get("status").and_then(|v| v.as_str()).unwrap_or("") {
            "COMPLETED" => {
                // Ambil hasil penuh via response_url.
                let resp_url = format!("{FAL_QUEUE_BASE}/{model}/requests/{request_id}");
                let full: Value = client
                    .get(&resp_url)
                    .header("Authorization", format!("Key {key}"))
                    .send()
                    .await
                    .context("gagal ambil hasil fal.ai")?
                    .json()
                    .await
                    .context("respons hasil fal.ai bukan JSON")?;
                video_url = full
                    .get("video")
                    .and_then(|v| v.get("url"))
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());
                break;
            }
            "FAILED" | "CANCELED" => {
                anyhow::bail!("render fal.ai gagal/dibatalkan: {request_id}");
            }
            _ => {} // IN_QUEUE / IN_PROGRESS → lanjut poll.
        }
    }
    let video_url = video_url.context("timeout menunggu render fal.ai (±10 menit)")?;

    // 3. Unduh MP4 ke artifact dir (deliverable lokal, bukan cuma URL).
    let bytes = client
        .get(&video_url)
        .send()
        .await
        .context("gagal mengunduh video fal.ai")?
        .bytes()
        .await
        .context("gagal membaca body video")?;
    let dir = ctx.artifact_dir()?;
    let video_path = dir.join(output);
    std::fs::write(&video_path, &bytes)
        .with_context(|| format!("gagal menulis {}", video_path.display()))?;
    Ok(json!({
        "video_path": video_path.to_string_lossy(),
        "video_url": video_url,
        "model": model,
        "request_id": request_id,
        "bytes": bytes.len(),
    }))
}

/// Fallback: potong prompt jadi 3 slide → render-reels HyperFrames lokal.
async fn fallback_hyperframes(prompt: &str, output: &str, ctx: &ToolCtx) -> Result<Value> {
    let words: Vec<&str> = prompt.split_whitespace().collect();
    let chunk = (words.len() / 3).max(8);
    let slides: Vec<Value> = (0..3)
        .map(|i| {
            let start = i * chunk;
            let end = ((i + 1) * chunk).min(words.len()).max(start);
            let body = words[start..end].join(" ");
            json!({
                "kicker": format!("Bagian {}", i + 1),
                "heading": body.chars().take(60).collect::<String>(),
                "body": "Render lokal (fallback Veo3).",
                "duration_s": 3,
            })
        })
        .collect();
    super::hyperframes::run(
        "render-reels",
        &json!({
            "title": "Video Harian",
            "slides": slides,
            "output": output,
            "quality": "draft",
        }),
        ctx,
    )
    .await
}
