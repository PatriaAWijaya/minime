//! Canva wiring (Fase 4): via Canva MCP CLI (`canva`).
//!
//! The daemon shells out to the CLI — the same protocol as
//! `/opt/hatch/connectors/canva/CONNECTOR.md`:
//!   `canva status` / `canva list-tools --names-only` /
//!   `canva call-tool --name <tool> --arguments-json '<json>'`
//! Async jobs (create-design, generate-image) are polled honestly:
//! every returned wait interval is honored, never restarted.
//!
//! Auth lives in the CLI's own OAuth store (connected by the user on their
//! machine). The daemon never handles Canva tokens directly.

use super::{ToolCtx, ToolStatus};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

const ENV_BIN: &str = "MINIME_CANVA_BIN";

fn canva_bin() -> Option<PathBuf> {
    super::find_bin(ENV_BIN, &["canva"], &["/opt/hatch/bin", "/usr/local/bin"])
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_CANVA,
            "Canva",
            "cli",
            state,
            detail,
            "Render lokal via HyperFrames; desain manual di Canva editor.",
        )
    };
    let Some(bin) = canva_bin() else {
        return not(
            super::STATE_NOT_INSTALLED,
            "CLI `canva` tidak ditemukan (set MINIME_CANVA_BIN bila perlu).",
        );
    };
    let out = super::run_cmd(&bin, &["status"], None, Duration::from_secs(20)).await;
    let Ok(stdout) = out else {
        return not(
            super::STATE_UNREACHABLE,
            "CLI `canva status` gagal dijalankan.",
        );
    };
    let v = super::parse_json_stdout(&stdout).unwrap_or(Value::Null);
    let connected = v
        .get("auth_status")
        .and_then(|s| s.as_str())
        .map(|s| s == "connected")
        .unwrap_or(false);
    if connected {
        let n = v
            .get("available_tools")
            .and_then(|t| t.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        ToolStatus::ready(
            super::TOOL_CANVA,
            "Canva",
            "cli",
            &format!("Terhubung ({} tools MCP tersedia).", n),
            "Render lokal via HyperFrames bila Canva bermasalah.",
        )
    } else {
        not(
            super::STATE_NOT_CONFIGURED,
            "Canva belum terhubung (jalankan `canva authorize-url` di mesin ini).",
        )
    }
}

async fn call_tool(bin: &Path, name: &str, args: &Value) -> Result<Value> {
    let args_json = serde_json::to_string(args)?;
    let stdout = super::run_cmd(
        bin,
        &["call-tool", "--name", name, "--arguments-json", &args_json],
        None,
        Duration::from_secs(120),
    )
    .await
    .with_context(|| format!("canva call-tool {name}"))?;
    let v = super::parse_json_stdout(&stdout)?;
    // MCP CLI: provider error arrives as ok:false JSON on stdout.
    if v.get("ok").and_then(|b| b.as_bool()) == Some(false) {
        anyhow::bail!(
            "canva {name}: {}",
            v.to_string().chars().take(300).collect::<String>()
        );
    }
    Ok(v)
}

/// Extract the JSON text payload from an MCP tool result.
fn result_text(v: &Value) -> Result<Value> {
    // Prefer structuredContent, else parse the text block.
    if let Some(sc) = v.get("result").and_then(|r| r.get("structuredContent")) {
        return Ok(sc.clone());
    }
    let text = v
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .context("respons canva tanpa content.text")?;
    serde_json::from_str(text).with_context(|| "parse JSON dari result.content.text")
}

/// Poll an async job until terminal state. Honors returned wait intervals.
async fn poll_job(
    bin: &Path,
    poll_tool: &str,
    mut job_id: String,
    mut token: String,
    what: &str,
) -> Result<Value> {
    let deadline = std::time::Instant::now() + Duration::from_secs(600);
    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("{what}: timeout menunggu job {job_id}");
        }
        let v = call_tool(
            bin,
            poll_tool,
            &json!({"job_id": job_id, "continuation_token": token}),
        )
        .await?;
        let body = result_text(&v).unwrap_or(v.clone());
        let status = body
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_lowercase();
        if status.contains("succeed")
            || status.contains("complete")
            || body.get("design").is_some()
            || body.get("media_id").is_some()
        {
            return Ok(body);
        }
        if status.contains("fail") || status.contains("error") {
            anyhow::bail!(
                "{what}: job gagal ({})",
                body.to_string().chars().take(300).collect::<String>()
            );
        }
        // Honor the returned wait interval / updated token; never restart.
        if let Some(t) = body.get("continuation_token").and_then(|t| t.as_str()) {
            if !t.is_empty() {
                token = t.to_string();
            }
        }
        if let Some(j) = body.get("job_id").and_then(|j| j.as_str()) {
            if !j.is_empty() {
                job_id = j.to_string();
            }
        }
        let wait_s: u64 = body
            .get("wait_seconds")
            .and_then(|w| w.as_u64())
            .or_else(|| body.get("retry_after").and_then(|w| w.as_u64()))
            .unwrap_or(5)
            .clamp(2, 30);
        tokio::time::sleep(Duration::from_secs(wait_s)).await;
    }
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let bin = canva_bin().with_context(|| "CLI `canva` tidak ditemukan")?;
    match op {
        "status" => {
            let stdout = super::run_cmd(&bin, &["status"], None, Duration::from_secs(20)).await?;
            super::parse_json_stdout(&stdout)
        }
        "design.create" => {
            let brief = params
                .get("brief")
                .and_then(|v| v.as_str())
                .context("params.brief wajib")?;
            let mut args = json!({"brief": brief, "user_intent": "Buat desain via Minime daemon"});
            for key in ["format", "width", "height", "orientation"] {
                if let Some(v) = params.get(key) {
                    args[key] = v.clone();
                }
            }
            let v = call_tool(&bin, "create-design", &args).await?;
            let body = result_text(&v)?;
            let job_id = body
                .get("job_id")
                .and_then(|j| j.as_str())
                .context("create-design tanpa job_id")?
                .to_string();
            let token = body
                .get("continuation_token")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let done = poll_job(
                &bin,
                "get-create-design-async-job",
                job_id,
                token,
                "create-design",
            )
            .await?;
            Ok(json!({"design": done}))
        }
        "image.generate" => {
            let prompt = params
                .get("prompt")
                .and_then(|v| v.as_str())
                .context("params.prompt wajib")?;
            let v = call_tool(
                &bin,
                "generate-image",
                &json!({"prompt": prompt, "user_intent": "Generate gambar via Minime daemon"}),
            )
            .await?;
            let body = result_text(&v)?;
            // generate-image may be sync (media_id) or async (job_id).
            if body.get("media_id").is_some() {
                return Ok(json!({"image": body}));
            }
            let job_id = body
                .get("job_id")
                .and_then(|j| j.as_str())
                .context("generate-image tanpa job_id/media_id")?
                .to_string();
            let token = body
                .get("continuation_token")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let done = poll_job(
                &bin,
                "get-generate-image-job",
                job_id,
                token,
                "generate-image",
            )
            .await?;
            Ok(json!({"image": done}))
        }
        "design.export" => {
            let design_id = params
                .get("design_id")
                .and_then(|v| v.as_str())
                .context("params.design_id wajib")?;
            let format = params
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("png");
            let v = call_tool(
                &bin,
                "export-design",
                &json!({"design_id": design_id, "format": format}),
            )
            .await?;
            let body = result_text(&v)?;
            // Find an export URL and download it locally (proof of work = file).
            let url = body
                .get("export_url")
                .or_else(|| body.get("url"))
                .and_then(|u| u.as_str())
                .context("export-design tanpa URL unduhan")?
                .to_string();
            let dir = ctx.artifact_dir()?;
            let file = dir.join(format!("canva-{design_id}.{format}"));
            download_to(&url, &file).await?;
            Ok(json!({
                "design_id": design_id,
                "format": format,
                "artifact_path": file.to_string_lossy(),
            }))
        }
        _ => anyhow::bail!(
            "op canva tidak dikenal: '{op}' (status/design.create/image.generate/design.export)"
        ),
    }
}

async fn download_to(url: &str, dest: &Path) -> Result<()> {
    let resp = reqwest::get(url)
        .await
        .with_context(|| "unduh export canva")?;
    if !resp.status().is_success() {
        anyhow::bail!("unduh export canva: HTTP {}", resp.status());
    }
    let bytes = resp.bytes().await?;
    std::fs::write(dest, &bytes).with_context(|| format!("tulis {}", dest.display()))?;
    Ok(())
}
