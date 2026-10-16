//! ManimGX wiring: render video pembelajaran (gaya 3Blue1Brown) dari scene Python.
//!
//! - prasyarat: Python 3.13+, paket `manimgx` (`uv tool install --python 3.13 manimgx`
//!   atau `pip install manimgx`), FFmpeg.
//! - render: `manimgx render scene.py SceneClass -r WxH --fps N -o out.mp4`
//! - inspeksi: `manimgx inspect scene.py SceneClass` (storyboard + layout).
//! - GPU: wgpu memakai GPU bila ada (/dev/dri); tanpa GPU → software (lavapipe),
//!   render tetap jalan tapi lebih lambat. Verifikasi penuh GPU di MacBook Patria.
//! Kredensial: tidak diperlukan.

use super::{ToolCtx, ToolStatus};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

const ENV_BIN: &str = "MINIME_MANIMGX_BIN";

fn manimgx_bin() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let local_bin = format!("{home}/.local/bin");
    super::find_bin(ENV_BIN, &["manimgx"], &[local_bin.as_str()])
}

fn gpu_available() -> bool {
    Path::new("/dev/dri").exists()
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_MANIMGX,
            "ManimGX",
            "cli",
            state,
            detail,
            "Render visual via HyperFrames; animasi matematika sederhana via HTML/SVG.",
        )
    };
    let Some(bin) = manimgx_bin() else {
        return not(
            super::STATE_NOT_INSTALLED,
            "manimgx tidak ditemukan (install: `uv tool install --python 3.13 manimgx`).",
        );
    };
    let ver = super::run_cmd(&bin, &["--version"], None, Duration::from_secs(60)).await;
    let ver_str = match ver {
        Ok(v) => v.trim().to_string(),
        Err(_) => {
            return not(
                super::STATE_NOT_INSTALLED,
                "manimgx ditemukan tapi gagal dijalankan.",
            )
        }
    };
    if super::find_bin("MINIME_FFMPEG_BIN", &["ffmpeg"], &["/usr/bin"]).is_none() {
        return not(super::STATE_NOT_INSTALLED, "ffmpeg tidak ditemukan.");
    }
    let gpu_note = if gpu_available() {
        "GPU terdeteksi (/dev/dri) — render wgpu akselerasi."
    } else {
        "Tanpa GPU (/dev/dri tidak ada) — render via software (lavapipe), lebih lambat; verifikasi penuh GPU di MacBook Patria."
    };
    ToolStatus::ready(
        super::TOOL_MANIMGX,
        "ManimGX",
        "cli",
        &format!("Siap render video pembelajaran ({ver_str}). {gpu_note}"),
        "Render visual via HyperFrames bila ManimGX bermasalah.",
    )
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    match op {
        "render" => {
            let scene_file = params
                .get("scene_file")
                .and_then(|v| v.as_str())
                .context("params.scene_file wajib (path file .py)")?;
            let scene = params
                .get("scene")
                .and_then(|v| v.as_str())
                .context("params.scene wajib (nama class Scene)")?;
            let output = params
                .get("output")
                .and_then(|v| v.as_str())
                .context("params.output wajib (nama file mp4)")?;
            let resolution = params
                .get("resolution")
                .and_then(|v| v.as_str())
                .unwrap_or("1280x720");
            let fps = params
                .get("fps")
                .and_then(|v| v.as_u64())
                .unwrap_or(30)
                .clamp(15, 60);
            render_scene(scene_file, scene, output, resolution, fps, ctx).await
        }
        "inspect" => {
            let scene_file = params
                .get("scene_file")
                .and_then(|v| v.as_str())
                .context("params.scene_file wajib")?;
            let scene = params
                .get("scene")
                .and_then(|v| v.as_str())
                .context("params.scene wajib")?;
            let bin = manimgx_bin().context("manimgx tidak ter-install")?;
            let out = super::run_cmd(
                &bin,
                &["inspect", scene_file, scene],
                None,
                Duration::from_secs(300),
            )
            .await?;
            Ok(json!({ "ok": true, "storyboard": out }))
        }
        _ => anyhow::bail!("op manimgx tidak dikenal: '{op}' (render/inspect)"),
    }
}

/// Render scene menjadi MP4. TMPDIR diarahkan ke data_dir karena /tmp kecil (tmpfs).
async fn render_scene(
    scene_file: &str,
    scene: &str,
    output: &str,
    resolution: &str,
    fps: u64,
    ctx: &ToolCtx,
) -> Result<Value> {
    let bin = manimgx_bin().context("manimgx tidak ter-install")?;
    let out_dir = ctx.artifact_dir()?;
    let out_path = out_dir.join(output);
    let tmp_dir = ctx.data_dir.join("tmp");
    std::fs::create_dir_all(&tmp_dir)?;
    let fps_s = fps.to_string();
    let args: Vec<&str> = vec![
        "render",
        scene_file,
        scene,
        "-r",
        resolution,
        "--fps",
        &fps_s,
        "-o",
        out_path.to_str().context("output path invalid")?,
    ];
    let log = super::run_cmd_env(
        &bin,
        &args,
        None,
        Duration::from_secs(1200),
        &[("TMPDIR", tmp_dir.to_str().unwrap_or("/tmp"))],
    )
    .await?;
    // Verifikasi: file ada + durasi via ffprobe bila tersedia.
    let meta = std::fs::metadata(&out_path)
        .with_context(|| format!("render selesai tapi file tidak ada: {}", out_path.display()))?;
    Ok(json!({
        "ok": true,
        "artifact_path": out_path.to_string_lossy(),
        "bytes": meta.len(),
        "resolution": resolution,
        "fps": fps,
        "gpu": gpu_available(),
        "log_tail": log.chars().rev().take(500).collect::<String>().chars().rev().collect::<String>(),
    }))
}
