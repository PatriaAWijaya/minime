//! Valyu wiring: data finansial institutional-grade via Valyu Search API.
//!
//! Berdasarkan `yorkeccak/finance` (MIT): chat interface di atas SATU search API
//! (Valyu) — data pasar live, SEC filings, berita, riset akademik.
//! - endpoint: POST https://api.valyu.ai/v1/search (header `x-api-key`)
//! - kredensial: Keychain `com.minime.app` akun `valyu` (override env MINIME_VALYU_KEY
//!   untuk dev); gratis $10 di platform.valyu.ai.
//! - tanpa key → state not_configured, bukan error.

use super::{ToolCtx, ToolStatus};
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

const BASE: &str = "https://api.valyu.ai/v1";

fn api_key() -> Option<String> {
    // Standar proyek: MINIME_VALYU_KEY (env, dev) → macOS Keychain.
    // Fallback: VALYU_API_KEY (env, kompatibilitas).
    secrets::api_key("valyu").or_else(|| {
        std::env::var("VALYU_API_KEY")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
    })
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_VALYU,
            "Valyu",
            "api",
            state,
            detail,
            "Riset finansial manual via browser bawaan; data pasar via Polymarket connector.",
        )
    };
    let Some(_) = api_key() else {
        return not(
            super::STATE_NOT_CONFIGURED,
            "API key Valyu belum diset (minta ke Patria: gratis $10 di platform.valyu.ai, simpan di Keychain akun `valyu`).",
        );
    };
    // Verifikasi ringan: GET /v1/datasources butuh key juga; cukup cek konektivitas
    // via search murah bila key ada. Untuk hemat kuota, readiness hanya cek key.
    ToolStatus::ready(
        super::TOOL_VALYU,
        "Valyu",
        "api",
        "API key tersedia — siap query data finansial institutional-grade.",
        "Riset finansial manual via browser bawaan bila API bermasalah.",
    )
}

pub async fn run(op: &str, params: &Value, _ctx: &ToolCtx) -> Result<Value> {
    match op {
        "search" => {
            let query = params
                .get("query")
                .and_then(|v| v.as_str())
                .context("params.query wajib")?;
            let search_type = params
                .get("search_type")
                .and_then(|v| v.as_str())
                .unwrap_or("all");
            let max_results = params
                .get("max_num_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(10)
                .clamp(1, 20);
            let key = api_key().context("API key Valyu belum diset")?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()?;
            let resp = client
                .post(format!("{BASE}/search"))
                .header("x-api-key", key)
                .json(&json!({
                    "query": query,
                    "search_type": search_type,
                    "max_num_results": max_results,
                    "response_length": "medium",
                }))
                .send()
                .await
                .context("gagal menghubungi Valyu API")?;
            let status = resp.status();
            let body: Value = resp.json().await.context("respons Valyu bukan JSON")?;
            if !status.is_success() {
                anyhow::bail!(
                    "Valyu API error {status}: {}",
                    body.get("error")
                        .or_else(|| body.get("message"))
                        .unwrap_or(&json!("unknown"))
                );
            }
            Ok(json!({
                "ok": true,
                "query": query,
                "results": body.get("results").unwrap_or(&json!([])),
                "cost": body.get("total_cost"),
            }))
        }
        _ => anyhow::bail!("op valyu tidak dikenal: '{op}' (search)"),
    }
}
