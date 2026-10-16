//! Affinity wiring (Fase 4): via MCP bridge di `127.0.0.1:6767` (HTTP).
//!
//! The bridge runs on Patria's MacBook (token dipegang Patria). This module:
//! - readiness: TCP/HTTP probe ke bridge + cek token di Keychain,
//! - `call`: generic JSON-RPC 2.0 passthrough ke method MCP bridge.
//! Method names follow the bridge's own MCP surface (see skills/affinity/SKILL.md).

use super::{ToolCtx, ToolStatus};
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

const ENV_URL: &str = "MINIME_AFFINITY_URL";
const DEFAULT_URL: &str = "http://127.0.0.1:6767";
const SECRET_ACCOUNT: &str = "affinity";

fn bridge_url() -> String {
    std::env::var(ENV_URL)
        .ok()
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEFAULT_URL.to_string())
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_AFFINITY,
            "Affinity",
            "mcp-http",
            state,
            detail,
            "Olah desain via Canva / HyperFrames; minta Patria nyalakan bridge bila perlu.",
        )
    };
    let url = bridge_url();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build();
    let Ok(client) = client else {
        return not(super::STATE_UNREACHABLE, "gagal membuat HTTP client");
    };
    // Probe: root atau /health — yang penting TCP+HTTP menjawab.
    let probe = async {
        let r = client.get(format!("{url}/")).send().await;
        match r {
            Ok(resp) => Ok(resp.status().as_u16()),
            Err(e) => {
                // Coba JSON-RPC ping sebagai fallback probe.
                let r2 = client
                    .post(&url)
                    .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": {}}))
                    .send()
                    .await;
                match r2 {
                    Ok(resp) => Ok(resp.status().as_u16()),
                    Err(_) => Err(e),
                }
            }
        }
    };
    match probe.await {
        Ok(_) => {
            if secrets::api_key(SECRET_ACCOUNT).is_none() {
                not(
                    super::STATE_NO_KEY,
                    "Bridge terjangkau, tetapi token Affinity belum disimpan (secrets.set account='affinity').",
                )
            } else {
                ToolStatus::ready(
                    super::TOOL_AFFINITY,
                    "Affinity",
                    "mcp-http",
                    &format!("Bridge terjangkau di {url}; token tersedia."),
                    "Olah desain via Canva / HyperFrames bila bridge mati.",
                )
            }
        }
        Err(e) => not(
            super::STATE_UNREACHABLE,
            &format!(
                "Bridge tidak terjangkau di {url} ({}) — nyalakan di MacBook Patria.",
                e.to_string().chars().take(120).collect::<String>()
            ),
        ),
    }
}

pub async fn run(op: &str, params: &Value, _ctx: &ToolCtx) -> Result<Value> {
    match op {
        "call" => {
            let method = params
                .get("method")
                .and_then(|v| v.as_str())
                .context("params.method wajib (nama method MCP bridge)")?;
            let call_params = params.get("params").cloned().unwrap_or(json!({}));
            call_bridge(method, &call_params).await
        }
        _ => anyhow::bail!("op affinity tidak dikenal: '{op}' (call)"),
    }
}

/// Generic JSON-RPC 2.0 call ke MCP bridge.
async fn call_bridge(method: &str, params: &Value) -> Result<Value> {
    let url = bridge_url();
    let token = secrets::api_key(SECRET_ACCOUNT);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .context("buat HTTP client")?;
    let mut req = client.post(&url).json(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    }));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.with_context(|| format!("POST {url}"))?;
    let status = resp.status();
    let body: Value = resp.json().await.context("parse respons bridge")?;
    if !status.is_success() {
        anyhow::bail!(
            "bridge HTTP {status}: {}",
            body.to_string().chars().take(300).collect::<String>()
        );
    }
    if let Some(err) = body.get("error") {
        anyhow::bail!(
            "bridge error: {}",
            err.to_string().chars().take(300).collect::<String>()
        );
    }
    Ok(body.get("result").cloned().unwrap_or(body))
}
