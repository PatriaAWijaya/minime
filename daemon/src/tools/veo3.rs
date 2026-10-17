//! Render video Veo 3.1 — rantai backend: Gemini API (primer) → fal.ai → HyperFrames lokal.
//!
//! Perintah Patria 17 Okt 2026: primer pindah dari fal.ai ke Gemini API (Veo 3.1),
//! fal.ai jadi fallback. Endpoint terverifikasi dari docs resmi Google AI for
//! Developers (https://ai.google.dev/gemini-api/docs/veo):
//!   POST {BASE}/models/{model}:predictLongRunning
//!     header `x-goog-api-key`, body {"instances":[{"prompt"}],"parameters":{...}}
//!   → {"name": "operations/..."} → GET {BASE}/{name} → cek `.done`
//!   → `.response.generateVideoResponse.generatedSamples[0].video.uri`
//!   → unduh MP4 (follow redirect).
//! Model IDs (Gemini API): `veo-3.1-generate-preview` (Standard),
//! `veo-3.1-fast-generate-preview` (Fast). ID `-001` adalah Vertex AI, bukan Gemini API.
//! Parameters: aspectRatio "16:9"/"9:16"; durationSeconds "4"/"6"/"8" (wajib "8"
//! untuk 1080p/4k); resolution "720p"/"1080p"; personGeneration "allow_all"
//! untuk text-to-video. 1 video per request, 24fps, MP4, audio native.
//!
//! - readiness: cek key Gemini (Keychain akun `gemini` / env MINIME_GEMINI_KEY),
//!   lalu key fal (Keychain akun `fal` / env MINIME_FAL_KEY).
//! - render: params.backend = auto|gemini|fal|local (default auto).
//!   auto: gemini (bila key ada) → fal.ai (bila key ada) → HyperFrames lokal.
//!   Setiap percobaan dicatat di "attempts" — tidak ada kegagalan diam-diam.
//! - MINIME_MOCK_LLM=1 → hasil mock eksplisit (uji wiring pipeline).
//! Kredensial tidak pernah di-log (rule #2).

use super::{ToolCtx, ToolStatus};
use crate::db::Db;
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

const GEMINI_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const FAL_QUEUE_BASE: &str = "https://queue.fal.run";
/// Default: Veo 3.1 Fast via Gemini API (1080p/8 dtk ≈ tercepat & termurah untuk harian).
const DEFAULT_MODEL: &str = "veo-3.1-fast-generate-preview";
/// Poll tiap 15 detik, maksimal ~10 menit (docs: latensi maks 6 menit).
const POLL_EVERY: Duration = Duration::from_secs(15);
const POLL_MAX: usize = 40;

fn gemini_key() -> Option<String> {
    secrets::api_key("gemini")
}

fn fal_key() -> Option<String> {
    secrets::api_key("fal")
}

fn mock_mode() -> bool {
    std::env::var("MINIME_MOCK_LLM").is_ok()
}

pub async fn readiness() -> ToolStatus {
    match (gemini_key().is_some(), fal_key().is_some()) {
        (true, _) => ToolStatus::ready(
            super::TOOL_VEO3,
            "Veo 3.1 (Gemini API)",
            "api",
            "API key Gemini tersedia — render primer via Veo 3.1 Fast (1080p/8 dtk/16:9).",
            "Fallback berantai: fal.ai → HyperFrames lokal (dicatat jujur di log).",
        ),
        (false, true) => ToolStatus::ready(
            super::TOOL_VEO3,
            "Veo 3.1 (fallback fal.ai)",
            "api",
            "JUJUR: key Gemini belum diset — primer nonaktif; render via fallback fal.ai.",
            "Fallback berantai: HyperFrames lokal bila fal.ai gagal (dicatat jujur).",
        ),
        (false, false) => ToolStatus::not(
            super::TOOL_VEO3,
            "Veo 3.1",
            "api",
            super::STATE_NOT_CONFIGURED,
            "Key Gemini (akun `gemini`/MINIME_GEMINI_KEY) dan key fal.ai belum diset.",
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

/// Petakan durasi detik ke durationSeconds valid ("4"/"6"/"8").
/// Docs: wajib "8" untuk resolusi 1080p/4k.
fn map_duration(secs: u64, resolution: &str) -> &'static str {
    if resolution == "1080p" || resolution == "4k" {
        return "8";
    }
    if secs <= 4 {
        "4"
    } else if secs <= 6 {
        "6"
    } else {
        "8"
    }
}

/// Perkiraan biaya kasar per klip (harga publik Google, BUKAN janji).
/// Fast 1080p ≈ $0,12/dtk; Lite 1080p ≈ $0,08/dtk; Standard ≈ $0,40/dtk.
fn est_cost_usd(model: &str, secs: u64) -> f64 {
    let per_sec = if model.contains("lite") {
        0.08
    } else if model.contains("fast") {
        0.12
    } else {
        0.40
    };
    // Pembulatan ke 2 desimal.
    (per_sec * secs as f64 * 100.0).round() / 100.0
}

async fn render(db: &Db, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let backend = params
        .get("backend")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");
    let model = params
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_MODEL);
    let duration_s = params
        .get("duration_s")
        .and_then(|v| v.as_u64())
        .unwrap_or(8)
        .clamp(1, 60);
    let aspect_ratio = params
        .get("aspect_ratio")
        .and_then(|v| v.as_str())
        .unwrap_or("16:9");
    let aspect_ratio = match aspect_ratio {
        "9:16" => "9:16",
        _ => "16:9",
    };
    let resolution = params
        .get("resolution")
        .and_then(|v| v.as_str())
        .unwrap_or("1080p");
    let resolution = match resolution {
        "720p" | "4k" => resolution,
        _ => "1080p",
    };
    let duration = map_duration(duration_s, resolution);
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
            "backend_chain": ["gemini", "fal", "hyperframes"],
            "prompt_source": prompt_source,
            "note": "MOCK (MINIME_MOCK_LLM=1): render dilewati — file MP4 tidak dibuat. Rantai nyata: Gemini API (Veo 3.1) → fal.ai → HyperFrames lokal. Uji wiring saja.",
        }));
    }

    // Rantai backend: primer → fallback. "local" = pamungkas tanpa AI.
    let order: &[&str] = match backend {
        "gemini" => &["gemini", "fal", "hyperframes"],
        "fal" => &["fal", "hyperframes"],
        "local" => &["hyperframes"],
        _ => &["gemini", "fal", "hyperframes"], // "auto" (default)
    };
    let mut attempts: Vec<Value> = Vec::new();
    for b in order {
        match *b {
            "gemini" => match gemini_key() {
                None => attempts.push(json!({
                    "backend": "gemini",
                    "status": "skipped",
                    "reason": "API key Gemini belum diset (Keychain akun `gemini` / env MINIME_GEMINI_KEY) — lanjut ke fallback.",
                })),
                Some(key) => {
                    match render_gemini(
                        &key,
                        model,
                        &prompt,
                        duration,
                        aspect_ratio,
                        resolution,
                        output,
                        ctx,
                    )
                    .await
                    {
                        Ok(mut v) => {
                            v["backend"] = json!("gemini");
                            v["attempts"] = json!(attempts);
                            v["prompt_source"] = json!(prompt_source);
                            return Ok(v);
                        }
                        Err(e) => attempts.push(json!({
                            "backend": "gemini",
                            "status": "failed",
                            "reason": format!("{e:#}"),
                        })),
                    }
                }
            },
            "fal" => match fal_key() {
                None => attempts.push(json!({
                    "backend": "fal",
                    "status": "skipped",
                    "reason": "API key fal.ai belum diset (Keychain akun `fal` / env MINIME_FAL_KEY) — lanjut ke fallback.",
                })),
                Some(key) => {
                    match render_fal(&key, model, &prompt, duration_s, aspect_ratio, output, ctx)
                        .await
                    {
                        Ok(mut v) => {
                            v["backend"] = json!("fal");
                            v["attempts"] = json!(attempts);
                            v["prompt_source"] = json!(prompt_source);
                            return Ok(v);
                        }
                        Err(e) => attempts.push(json!({
                            "backend": "fal",
                            "status": "failed",
                            "reason": format!("{e:#}"),
                        })),
                    }
                }
            },
            _ => {
                // HyperFrames lokal — pamungkas rantai, selalu dicoba terakhir.
                match fallback_hyperframes(&prompt, output, ctx).await {
                    Ok(v) => {
                        return Ok(json!({
                            "backend": "hyperframes",
                            "fallback": true,
                            "attempts": attempts,
                            "video_path": v["video_path"],
                            "prompt_source": prompt_source,
                            "note": "JUJUR: backend AI tidak tersedia/gagal — video di-render lokal via HyperFrames (HTML→MP4), BUKAN Veo 3.1. Lihat 'attempts' untuk detail.",
                        }));
                    }
                    Err(e) => {
                        attempts.push(json!({
                            "backend": "hyperframes",
                            "status": "failed",
                            "reason": format!("{e:#}"),
                        }));
                        anyhow::bail!(
                            "semua backend render gagal: {}",
                            serde_json::to_string(&attempts).unwrap_or_default()
                        );
                    }
                }
            }
        }
    }
    unreachable!("rantai backend selalu diakhiri hyperframes")
}

/// Render via Gemini API (Veo 3.1) — pola REST terverifikasi dari docs resmi:
/// POST :predictLongRunning → poll operation → unduh video dari URI.
async fn render_gemini(
    key: &str,
    model: &str,
    prompt: &str,
    duration: &str,
    aspect_ratio: &str,
    resolution: &str,
    output: &str,
    ctx: &ToolCtx,
) -> Result<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    // 1. Submit long-running operation.
    let submit: Value = client
        .post(format!("{GEMINI_BASE}/models/{model}:predictLongRunning"))
        .header("x-goog-api-key", key)
        .json(&json!({
            "instances": [{ "prompt": prompt }],
            "parameters": {
                "aspectRatio": aspect_ratio,
                "durationSeconds": duration,
                "resolution": resolution,
                "personGeneration": "allow_all",
            },
        }))
        .send()
        .await
        .context("gagal menghubungi Gemini API (veo)")?
        .json()
        .await
        .context("respons submit Gemini bukan JSON")?;
    let op_name = submit
        .get("name")
        .and_then(|v| v.as_str())
        .context("Gemini tidak mengembalikan operation name")?
        .to_string();

    // 2. Poll status operasi.
    let op_url = format!("{GEMINI_BASE}/{op_name}");
    let mut video_uri: Option<String> = None;
    for _ in 0..POLL_MAX {
        tokio::time::sleep(POLL_EVERY).await;
        let st: Value = client
            .get(&op_url)
            .header("x-goog-api-key", key)
            .send()
            .await
            .context("gagal poll operasi Gemini")?
            .json()
            .await
            .context("respons poll Gemini bukan JSON")?;
        if st.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
            if let Some(err) = st.get("error") {
                let msg = err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                anyhow::bail!("render Gemini gagal: {msg}");
            }
            video_uri = st
                .pointer("/response/generateVideoResponse/generatedSamples/0/video/uri")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());
            break;
        }
    }
    let video_uri = video_uri.context("timeout menunggu render Gemini (±10 menit)")?;

    // 3. Unduh MP4 ke artifact dir (deliverable lokal, bukan cuma URI).
    // reqwest mengikuti redirect otomatis (maks 10) — setara `curl -L`.
    let bytes = client
        .get(&video_uri)
        .header("x-goog-api-key", key)
        .send()
        .await
        .context("gagal mengunduh video Gemini")?
        .error_for_status()
        .context("unduh video Gemini: status HTTP error")?
        .bytes()
        .await
        .context("gagal membaca body video")?;
    let dir = ctx.artifact_dir()?;
    let video_path = dir.join(output);
    std::fs::write(&video_path, &bytes)
        .with_context(|| format!("gagal menulis {}", video_path.display()))?;
    let secs: u64 = duration.parse().unwrap_or(8);
    Ok(json!({
        "video_path": video_path.to_string_lossy(),
        "model": model,
        "operation": op_name,
        "bytes": bytes.len(),
        "est_cost_usd": est_cost_usd(model, secs),
        "cost_note": "PERKIRAAN kasar dari harga publik Google (bukan janji/tagihan aktual).",
    }))
}

/// Alur queue fal.ai: POST → request_id → poll status → unduh video.
/// Kini berperan sebagai FALLBACK bila primer Gemini tidak tersedia/gagal.
async fn render_fal(
    key: &str,
    model: &str,
    prompt: &str,
    duration_s: u64,
    aspect_ratio: &str,
    output: &str,
    ctx: &ToolCtx,
) -> Result<Value> {
    // Model default tool ini kini ID Gemini; untuk fal.ai pakai ID fal bila
    // params.model masih default Gemini.
    let fal_model = if model.starts_with("veo-3.1") {
        "fal-ai/veo3.1"
    } else {
        model
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    // 1. Submit queue.
    let submit: Value = client
        .post(format!("{FAL_QUEUE_BASE}/{fal_model}"))
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
    let status_url = format!("{FAL_QUEUE_BASE}/{fal_model}/requests/{request_id}/status");
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
                let resp_url = format!("{FAL_QUEUE_BASE}/{fal_model}/requests/{request_id}");
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
        "model": fal_model,
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
            let start = (i * chunk).min(words.len());
            let end = ((i + 1) * chunk).min(words.len()).max(start);
            let body = words[start..end].join(" ");
            json!({
                "kicker": format!("Bagian {}", i + 1),
                "heading": body.chars().take(60).collect::<String>(),
                "body": "Render lokal (fallback Veo 3.1).",
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
