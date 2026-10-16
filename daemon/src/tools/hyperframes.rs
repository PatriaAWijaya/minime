//! HyperFrames wiring (Fase 4): render MP4 lokal dari komposisi HTML.
//!
//! Pola mengikuti skill `local_hyperframes`:
//!   - prasyarat: Node 22+, FFmpeg, Chrome Headless Shell (`npx hyperframes browser ensure`),
//!   - render: `npx hyperframes render --quality <q> --output <out.mp4>`,
//!   - verifikasi: ffprobe (durasi + resolusi sesuai komposisi).
//! Aturan: semua aset HARUS lokal (CDN diblokir correctness gate) — komposisi
//! yang di-generate modul ini tidak memakai CDN/JS eksternal sama sekali.

use super::{escape_html, ToolCtx, ToolStatus};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

const ENV_NPX: &str = "MINIME_NPX_BIN";

fn npx_bin() -> Option<PathBuf> {
    super::find_bin(
        ENV_NPX,
        &["npx"],
        &["/opt/hatch-image/bin", "/usr/local/bin"],
    )
}

fn chrome_headless_shell() -> Option<PathBuf> {
    // Sama seperti browser bawaan — reuse bila ada.
    super::browser::chromium_bin()
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_HYPERFRAMES,
            "HyperFrames",
            "cli",
            state,
            detail,
            "Render visual via Canva generate-image; video via OpenCreator.",
        )
    };
    let Some(npx) = npx_bin() else {
        return not(super::STATE_NOT_INSTALLED, "npx tidak ditemukan.");
    };
    let ver = super::run_cmd(
        &npx,
        &["hyperframes", "--version"],
        None,
        Duration::from_secs(60),
    )
    .await;
    if ver.is_err() {
        return not(
            super::STATE_NOT_INSTALLED,
            "paket `hyperframes` belum tersedia via npx.",
        );
    }
    if chrome_headless_shell().is_none() {
        return not(
            super::STATE_NOT_CONFIGURED,
            "Chrome Headless Shell belum diunduh (jalankan `npx hyperframes browser ensure`).",
        );
    }
    if super::find_bin("MINIME_FFMPEG_BIN", &["ffmpeg"], &["/usr/bin"]).is_none() {
        return not(super::STATE_NOT_INSTALLED, "ffmpeg tidak ditemukan.");
    }
    ToolStatus::ready(
        super::TOOL_HYPERFRAMES,
        "HyperFrames",
        "cli",
        "Siap render MP4 lokal (npx + Chrome Headless Shell + ffmpeg).",
        "Render visual via Canva bila HyperFrames bermasalah.",
    )
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    match op {
        "render" => {
            let project = params
                .get("project_dir")
                .and_then(|v| v.as_str())
                .context("params.project_dir wajib")?;
            let quality = params
                .get("quality")
                .and_then(|v| v.as_str())
                .unwrap_or("draft");
            let output = params
                .get("output")
                .and_then(|v| v.as_str())
                .context("params.output wajib (nama file mp4)")?;
            render_project(Path::new(project), quality, output, ctx).await
        }
        "render-reels" => {
            // Generate komposisi reels 9:16 dari slides, lalu render + verifikasi.
            let title = params
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Minime");
            let slides = params
                .get("slides")
                .and_then(|v| v.as_array())
                .context("params.slides wajib (array {heading, body, duration_s})")?;
            if slides.is_empty() {
                anyhow::bail!("params.slides kosong");
            }
            let output = params
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("reels.mp4");
            let quality = params
                .get("quality")
                .and_then(|v| v.as_str())
                .unwrap_or("draft");
            let project_dir = compose_reels_project(ctx, title, slides)?;
            render_project(&project_dir, quality, output, ctx).await
        }
        _ => anyhow::bail!("op hyperframes tidak dikenal: '{op}' (render/render-reels)"),
    }
}

/// Tulis proyek komposisi reels 9:16 (tanpa CDN/JS eksternal — static slides).
fn compose_reels_project(ctx: &ToolCtx, title: &str, slides: &[Value]) -> Result<PathBuf> {
    let dir = ctx.artifact_dir()?.join("reels-project");
    std::fs::create_dir_all(&dir)?;
    let mut total_s = 0.0f64;
    let mut clips = String::new();
    let mut t = 0.0f64;
    for (i, s) in slides.iter().enumerate() {
        let heading = s.get("heading").and_then(|v| v.as_str()).unwrap_or("");
        let body = s.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let dur = s
            .get("duration_s")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0)
            .clamp(0.5, 30.0);
        let kicker = s.get("kicker").and_then(|v| v.as_str()).unwrap_or("");
        // Variasi background per slide (tetap satu aksen, tanpa gradient norak).
        let bg = ["#0d1b2a", "#10243a", "#0b1f33"][i % 3];
        clips.push_str(&format!(
            r#"      <div class="clip" data-start="{t}" data-duration="{dur}" data-track-index="{i}" style="background:{bg}">
        <div class="kicker">{kicker}</div>
        <h1>{heading}</h1>
        <p>{body}</p>
        <div class="bar"></div>
      </div>
"#,
            t = t,
            dur = dur,
            i = i,
            bg = bg,
            kicker = escape_html(kicker),
            heading = escape_html(heading),
            body = escape_html(body),
        ));
        t += dur;
        total_s += dur;
    }
    let html = format!(
        r#"<!doctype html>
<html lang="id">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=1080, height=1920" />
    <style>
      * {{ margin: 0; padding: 0; box-sizing: border-box; }}
      html, body {{ width: 1080px; height: 1920px; overflow: hidden; background: #0d1b2a; }}
      #root {{ width: 1080px; height: 1920px; position: relative;
               font-family: system-ui, -apple-system, "Segoe UI", sans-serif; color: #fff; }}
      .clip {{ position: absolute; inset: 0; display: flex; flex-direction: column;
               align-items: center; justify-content: center; text-align: center; padding: 90px; }}
      .kicker {{ font-size: 40px; letter-spacing: 8px; text-transform: uppercase;
                 color: #7fb3ff; margin-bottom: 36px; font-weight: 600; }}
      h1 {{ font-size: 92px; line-height: 1.18; font-weight: 800; margin-bottom: 40px;
            max-width: 920px; }}
      p {{ font-size: 46px; line-height: 1.55; color: #d7e3f4; max-width: 880px; }}
      .bar {{ width: 220px; height: 10px; background: #2e7de9; border-radius: 5px; margin-top: 48px; }}
      .foot {{ position: absolute; bottom: 70px; left: 0; right: 0; text-align: center;
               font-size: 34px; color: #8fa8c8; letter-spacing: 2px; }}
    </style>
  </head>
  <body>
    <div id="root" data-composition-id="main" data-start="0" data-duration="{total_s}" data-width="1080" data-height="1920">
{clips}      <div class="foot">{title}</div>
    </div>
  </body>
</html>
"#,
        total_s = total_s,
        clips = clips,
        title = escape_html(title),
    );
    std::fs::write(dir.join("index.html"), html)?;
    std::fs::write(
        dir.join("hyperframes.json"),
        r#"{"paths": {"blocks": "compositions", "components": "compositions/components", "assets": "assets"}}"#,
    )?;
    Ok(dir)
}

/// Render proyek → verifikasi ffprobe → kembalikan info artefak.
async fn render_project(
    project_dir: &Path,
    quality: &str,
    output: &str,
    ctx: &ToolCtx,
) -> Result<Value> {
    let npx = npx_bin().with_context(|| "npx tidak ditemukan")?;
    if !project_dir.join("index.html").is_file() {
        anyhow::bail!(
            "project_dir tidak memuat index.html: {}",
            project_dir.display()
        );
    }
    let dir = ctx.artifact_dir()?;
    // Sanitize nama output (tanpa path traversal).
    let safe_name: String = output
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();
    if safe_name.is_empty() {
        anyhow::bail!("params.output tidak valid");
    }
    let out_path = dir.join(&safe_name);
    let quality = match quality {
        "draft" | "looks" | "delivery" => quality,
        _ => "draft",
    };
    // Temp render bisa GB-an — arahkan TMPDIR ke data_dir (bukan /tmp yang kecil).
    // (Pola dari skill local_hyperframes.)
    let render_tmp = dir.join("render-tmp");
    std::fs::create_dir_all(&render_tmp)?;
    let tmp_str = render_tmp.to_string_lossy().to_string();
    super::run_cmd_env(
        &npx,
        &[
            "hyperframes",
            "render",
            "--quality",
            quality,
            "--output",
            &out_path.to_string_lossy(),
        ],
        Some(project_dir),
        Duration::from_secs(900),
        &[("TMPDIR", tmp_str.as_str())],
    )
    .await
    .with_context(|| "hyperframes render gagal")?;

    // Verifikasi output (output contract skill): ffprobe durasi + resolusi.
    let (dur_s, w, h) = probe_video(&out_path).await?;
    // Bersihkan temp render — direktori artefak hanya berisi deliverable.
    let _ = std::fs::remove_dir_all(&render_tmp);
    Ok(json!({
        "engine": "hyperframes",
        "quality": quality,
        "artifact_path": out_path.to_string_lossy(),
        "duration_s": dur_s,
        "width": w,
        "height": h,
    }))
}

async fn probe_video(path: &Path) -> Result<(f64, i64, i64)> {
    let ffprobe = super::find_bin("MINIME_FFPROBE_BIN", &["ffprobe"], &["/usr/bin"])
        .with_context(|| "ffprobe tidak ditemukan — verifikasi video dilewati")?;
    let path_str = path.to_string_lossy().to_string();
    let out = super::run_cmd(
        &ffprobe,
        &[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,duration",
            "-show_entries",
            "format=duration",
            "-of",
            "json",
            &path_str,
        ],
        None,
        Duration::from_secs(30),
    )
    .await?;
    let v: Value = serde_json::from_str(&out).context("parse ffprobe")?;
    let stream = v
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .context("ffprobe: tidak ada video stream")?;
    let w = stream.get("width").and_then(|x| x.as_i64()).unwrap_or(0);
    let h = stream.get("height").and_then(|x| x.as_i64()).unwrap_or(0);
    let dur = v
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse::<f64>().ok())
        .or_else(|| {
            stream
                .get("duration")
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse::<f64>().ok())
        })
        .unwrap_or(0.0);
    if w <= 0 || h <= 0 {
        anyhow::bail!("ffprobe: resolusi tidak valid ({w}x{h})");
    }
    Ok((dur, w, h))
}
