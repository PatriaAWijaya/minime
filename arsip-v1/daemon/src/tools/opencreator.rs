//! OpenCreator wiring (Fase 4): sidecar binary di samping binary daemon.
//!
//! Contract (didokumentasikan di docs/open-creator.md):
//!   `<sidecar> --op <op> --params-json '<json>'` → JSON di stdout.
//! Ops: `render` (naskah → video), `voices` (daftar suara), `script` (draf naskah).
//!
//! LLM backend: Gemini (keputusan Patria 17 Okt 2026 — bukan kunci OpenAI-compatible
//! terpisah). Daemon memvalidasi keberadaan kunci Gemini
//! (Keychain akun `gemini` / env MINIME_GEMINI_KEY); penyetelan provider Gemini
//! di dalam aplikasi OpenCreator (Settings → AI Services) dilakukan manual
//! sekali di MacBook. Live test menunggu kunci Gemini dari Patria.

use super::{ToolCtx, ToolStatus};
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const ENV_BIN: &str = "MINIME_OPENCREATOR_BIN";
/// Akun Keychain untuk kunci Gemini (keputusan Patria 17 Okt 2026:
/// OpenCreator memakai Gemini, bukan kunci OpenAI-compatible terpisah).
const SECRET_ACCOUNT: &str = "gemini";

fn sidecar_bin() -> Option<PathBuf> {
    // 1. Env override eksplisit.
    if let Ok(p) = std::env::var(ENV_BIN) {
        let p = PathBuf::from(p.trim());
        if p.is_file() {
            return Some(p);
        }
    }
    // 2. Di samping binary daemon (pola sidecar).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["open-creator", "open-creator.exe"] {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn has_key() -> bool {
    secrets::api_key(SECRET_ACCOUNT).is_some()
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_OPEN_CREATOR,
            "OpenCreator",
            "sidecar",
            state,
            detail,
            "Render video via HyperFrames (lokal, tanpa kunci).",
        )
    };
    let Some(bin) = sidecar_bin() else {
        return not(
            super::STATE_NOT_INSTALLED,
            "Sidecar `open-creator` tidak ditemukan di samping binary daemon (atau set MINIME_OPENCREATOR_BIN).",
        );
    };
    if !has_key() {
        return not(
            super::STATE_NO_KEY,
            "Kunci Gemini belum disimpan (Keychain akun `gemini` / env MINIME_GEMINI_KEY). OpenCreator memakai Gemini (keputusan Patria 17 Okt 2026).",
        );
    }
    ToolStatus::ready(
        super::TOOL_OPEN_CREATOR,
        "OpenCreator",
        "sidecar",
        &format!("Sidecar siap: {} (kunci tersedia).", bin.display()),
        "Render video via HyperFrames bila sidecar bermasalah.",
    )
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let bin = sidecar_bin().with_context(|| "sidecar `open-creator` tidak ditemukan")?;
    if !has_key() {
        anyhow::bail!(
            "Kunci Gemini belum dikonfigurasi (Keychain akun `gemini` / env MINIME_GEMINI_KEY) — OpenCreator memakai Gemini (keputusan Patria 17 Okt 2026)"
        );
    }
    match op {
        "render" | "voices" | "script" => {
            let params_json = serde_json::to_string(params)?;
            let stdout = super::run_cmd(
                &bin,
                &["--op", op, "--params-json", &params_json],
                None,
                Duration::from_secs(900),
            )
            .await
            .with_context(|| format!("open-creator op '{op}'"))?;
            let v = super::parse_json_stdout(&stdout)?;
            // Bila sidecar mengembalikan artifact file, catat path-nya.
            if let Some(p) = v.get("output_path").and_then(|s| s.as_str()) {
                let _ = ctx; // artifact dicatat pemanggil via artifact_path
                return Ok(json!({"result": v, "artifact_path": p}));
            }
            Ok(v)
        }
        _ => anyhow::bail!("op open-creator tidak dikenal: '{op}' (render/voices/script)"),
    }
}
