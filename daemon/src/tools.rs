//! Tool registry (Fase 4): explicit readiness check per tool + dispatch.
//!
//! Every production tool has:
//!   - a readiness check (available? configured? credential present?),
//!   - a documented fallback when it is not ready,
//!   - proof-of-work logging into `messages` (no tool may fail silently).
//!
//! English code; user-facing strings in Bahasa Indonesia.

use crate::db::Db;
use anyhow::{Context, Result};
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

pub mod affinity;
pub mod browser;
pub mod canva;
pub mod hyperframes;
pub mod instagram;
pub mod manimgx;
pub mod meta;
pub mod opencreator;
pub mod postiz;
pub mod quant;
pub mod valyu;
pub mod veo3;
pub mod voicestudio;

/// Tool ids (must match agents' `tools_allowlist` + skills/<name>/).
pub const TOOL_CANVA: &str = "canva";
pub const TOOL_AFFINITY: &str = "affinity";
pub const TOOL_OPEN_CREATOR: &str = "open-creator";
pub const TOOL_VOICE_STUDIO: &str = "voice-studio";
pub const TOOL_HYPERFRAMES: &str = "hyperframes";
pub const TOOL_INSTAGRAM: &str = "instagram-cli";
pub const TOOL_BROWSER: &str = "browser";
pub const TOOL_MANIMGX: &str = "manimgx";
pub const TOOL_QUANT: &str = "quant";
pub const TOOL_VALYU: &str = "valyu";
pub const TOOL_VEO3: &str = "veo3";
pub const TOOL_POSTIZ: &str = "postiz";
pub const TOOL_META: &str = "meta";

pub const KNOWN_TOOLS: &[&str] = &[
    TOOL_CANVA,
    TOOL_AFFINITY,
    TOOL_OPEN_CREATOR,
    TOOL_VOICE_STUDIO,
    TOOL_HYPERFRAMES,
    TOOL_INSTAGRAM,
    TOOL_BROWSER,
    TOOL_MANIMGX,
    TOOL_QUANT,
    TOOL_VALYU,
    TOOL_VEO3,
    TOOL_POSTIZ,
    TOOL_META,
];

/// Readiness states (machine-readable).
pub const STATE_READY: &str = "ready";
pub const STATE_NOT_INSTALLED: &str = "not_installed";
pub const STATE_NOT_CONFIGURED: &str = "not_configured";
pub const STATE_NO_KEY: &str = "no_key";
pub const STATE_UNREACHABLE: &str = "unreachable";

/// Context handed to every tool implementation.
#[derive(Clone)]
pub struct ToolCtx {
    pub data_dir: PathBuf,
    pub repo_dir: PathBuf,
    /// Run that triggered this tool call (None for ad-hoc `tools.run`).
    pub run_id: Option<String>,
    /// Agent that owns this tool call (for allowlist + proof of work).
    pub agent_id: String,
}

impl ToolCtx {
    /// Directory for artifacts produced by this call.
    pub fn artifact_dir(&self) -> Result<PathBuf> {
        let dir = self
            .data_dir
            .join("artifacts")
            .join(self.run_id.as_deref().unwrap_or("adhoc"));
        std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        Ok(dir)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatus {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub ready: bool,
    pub state: String,
    pub detail: String,
    pub fallback: String,
}

impl ToolStatus {
    fn ready(id: &str, label: &str, kind: &str, detail: &str, fallback: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: kind.into(),
            ready: true,
            state: STATE_READY.into(),
            detail: detail.into(),
            fallback: fallback.into(),
        }
    }
    fn not(id: &str, label: &str, kind: &str, state: &str, detail: &str, fallback: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: kind.into(),
            ready: false,
            state: state.into(),
            detail: detail.into(),
            fallback: fallback.into(),
        }
    }
}

/// Run readiness checks for all known tools. Never fails as a whole:
/// a broken check is reported, not raised.
pub async fn readiness(data_dir: &Path, repo_dir: &Path) -> Vec<ToolStatus> {
    let mut out = Vec::new();
    out.push(canva::readiness().await);
    out.push(affinity::readiness().await);
    out.push(opencreator::readiness().await);
    out.push(voicestudio::readiness().await);
    out.push(hyperframes::readiness().await);
    out.push(instagram::readiness().await);
    out.push(browser::readiness(data_dir, repo_dir).await);
    out.push(manimgx::readiness().await);
    out.push(quant::readiness().await);
    out.push(valyu::readiness().await);
    out.push(veo3::readiness().await);
    out.push(postiz::readiness().await);
    out.push(meta::readiness().await);
    out
}

/// Dispatch a tool operation. Returns a JSON result; when the result contains
/// `artifact_path`, the caller should register it as a deliverable artifact.
pub async fn run_tool(
    db: &Db,
    browser_store: &BrowserStore,
    tool: &str,
    op: &str,
    params: &serde_json::Value,
    ctx: &ToolCtx,
) -> Result<serde_json::Value> {
    let started = std::time::Instant::now();
    let result = dispatch(db, browser_store, tool, op, params, ctx).await;
    let elapsed_ms = started.elapsed().as_millis() as i64;
    match result {
        Ok(v) => {
            log_tool_run(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                tool,
                op,
                true,
                &summarize_value(&v),
                elapsed_ms,
            )
            .await;
            Ok(v.clone())
        }
        Err(e) => {
            // Explicit failure — never silent (rule: proof of work).
            log_tool_run(
                db,
                ctx.run_id.as_deref(),
                &ctx.agent_id,
                tool,
                op,
                false,
                &format!("{e:#}"),
                elapsed_ms,
            )
            .await;
            Err(e)
        }
    }
}

async fn dispatch(
    db: &Db,
    browser_store: &BrowserStore,
    tool: &str,
    op: &str,
    params: &serde_json::Value,
    ctx: &ToolCtx,
) -> Result<serde_json::Value> {
    match tool {
        TOOL_CANVA => canva::run(op, params, ctx).await,
        TOOL_AFFINITY => affinity::run(op, params, ctx).await,
        TOOL_OPEN_CREATOR => opencreator::run(op, params, ctx).await,
        TOOL_VOICE_STUDIO => voicestudio::run(op, params, ctx).await,
        TOOL_HYPERFRAMES => hyperframes::run(op, params, ctx).await,
        TOOL_INSTAGRAM => instagram::run(op, params, ctx).await,
        TOOL_BROWSER => browser::run(db, browser_store, op, params, ctx).await,
        TOOL_MANIMGX => manimgx::run(op, params, ctx).await,
        TOOL_QUANT => quant::run(op, params, ctx).await,
        TOOL_VALYU => valyu::run(op, params, ctx).await,
        TOOL_VEO3 => veo3::run(db, op, params, ctx).await,
        TOOL_POSTIZ => postiz::run(db, op, params, ctx).await,
        TOOL_META => meta::run(db, op, params, ctx).await,
        _ => anyhow::bail!("tool tidak dikenal: '{tool}'"),
    }
}

fn summarize_value(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() <= 400 {
        s
    } else {
        format!("{}…", &s[..400])
    }
}

/// Proof of work: every tool invocation is recorded in `messages`.
async fn log_tool_run(
    db: &Db,
    run_id: Option<&str>,
    agent_id: &str,
    tool: &str,
    op: &str,
    ok: bool,
    summary: &str,
    elapsed_ms: i64,
) {
    let id = Uuid::new_v4().to_string();
    let content = serde_json::json!({
        "tool": tool,
        "op": op,
        "ok": ok,
        "elapsed_ms": elapsed_ms,
        "summary": summary,
    })
    .to_string();
    let guard = db.lock().await;
    guard
        .execute(
            "INSERT INTO messages (id, run_id, from_agent, to_agent, kind, content)
             VALUES (?1,?2,?3,'tools','tool-run',?4)",
            params![id, run_id, agent_id, content],
        )
        .ok();
    eprintln!("[tools] {tool}.{op} agent={agent_id} ok={ok} {elapsed_ms}ms");
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Find an executable: explicit env var first, then extra dirs, then PATH.
pub fn find_bin(env_var: &str, candidates: &[&str], extra_dirs: &[&str]) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(env_var) {
        let p = PathBuf::from(p.trim());
        if p.is_file() {
            return Some(p);
        }
    }
    let mut dirs: Vec<PathBuf> = extra_dirs.iter().map(PathBuf::from).collect();
    if let Ok(path) = std::env::var("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    for dir in dirs {
        for name in candidates {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Run a subprocess with a timeout. Returns stdout on success; stderr on failure.
pub async fn run_cmd(
    bin: &Path,
    args: &[&str],
    workdir: Option<&Path>,
    timeout: Duration,
) -> Result<String> {
    run_cmd_env(bin, args, workdir, timeout, &[]).await
}

/// run_cmd + extra environment variables (e.g. TMPDIR for render temp files).
pub async fn run_cmd_env(
    bin: &Path,
    args: &[&str],
    workdir: Option<&Path>,
    timeout: Duration,
    extra_env: &[(&str, &str)],
) -> Result<String> {
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args);
    if let Some(wd) = workdir {
        cmd.current_dir(wd);
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    // Never inherit secrets via env beyond what the process already has.
    let out = tokio::time::timeout(timeout, cmd.output())
        .await
        .with_context(|| {
            format!(
                "timeout {}s menjalankan {}",
                timeout.as_secs(),
                bin.display()
            )
        })?
        .with_context(|| format!("gagal menjalankan {}", bin.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!(
            "{} gagal (exit {:?}): {}",
            bin.display(),
            out.status.code(),
            stderr.chars().take(500).collect::<String>()
        )
    }
}

/// Parse stdout as JSON (tolerant: finds the first '{' if there is preamble).
pub fn parse_json_stdout(stdout: &str) -> Result<serde_json::Value> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(stdout) {
        return Ok(v);
    }
    // Some CLIs print human preamble before JSON — find the first '{'.
    if let Some(idx) = stdout.find('{') {
        let v: serde_json::Value =
            serde_json::from_str(&stdout[idx..]).with_context(|| "parse JSON dari output CLI")?;
        return Ok(v);
    }
    anyhow::bail!(
        "output CLI bukan JSON: {}",
        stdout.chars().take(200).collect::<String>()
    )
}

// ---------------------------------------------------------------------------
// Browser session store (in-memory; sessions die with the daemon — honest).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct BrowserSession {
    pub id: String,
    pub agent_id: String,
    pub created_at: String,
    pub last_url: String,
    pub shots: i64,
}

pub type BrowserStore = Arc<Mutex<HashMap<String, BrowserSession>>>;

pub fn new_browser_store() -> BrowserStore {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Minimal HTML escaping for generated compositions.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
