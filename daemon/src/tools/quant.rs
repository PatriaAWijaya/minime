//! Quant/finance tool wiring: TradingAgents, qlib, NautilusTrader, OpenBB, Valyu.
//!
//! Semua op HANYA untuk riset, analisis, backtest, dan paper trading (venue SIM).
//! EKSEKUSI ORDER LIVE DITOLAK DI KODE — lihat guard di `run()`.
//!
//! - prasyarat: Python 3.11+ dengan paket ter-install (`pip install tradingagents
//!   pyqlib nautilus_trader openbb`). Env `MINIME_QUANT_PYTHON` untuk interpreter
//!   kustom (default: `python3` di PATH).
//! - Valyu butuh API key: `secrets::api_key("valyu")` (Keychain/env MINIME_VALYU_KEY).
//! - NautilusTrader (LGPL-3.0): dikoneksikan HANYA sebagai proses eksternal
//!   (subprocess), TIDAK di-link ke binary — mematuhi LGPL.

use super::{ToolCtx, ToolStatus};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const ENV_PYTHON: &str = "MINIME_QUANT_PYTHON";

/// Op yang menyerupai eksekusi live — DITOLAK tanpa kecuali.
fn is_live_op(op: &str) -> bool {
    let o = op.to_lowercase();
    [
        "live",
        "order",
        "execute",
        "trade",
        "buy",
        "sell",
        "paper_live",
    ]
    .iter()
    .any(|k| o.contains(k))
}

fn quant_python() -> Option<PathBuf> {
    super::find_bin(ENV_PYTHON, &["python3"], &[])
}

async fn py_import_ok(python: &PathBuf, module: &str) -> bool {
    super::run_cmd(
        python,
        &["-c", &format!("import {module}")],
        None,
        Duration::from_secs(60),
    )
    .await
    .is_ok()
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_QUANT,
            "Quant",
            "python",
            state,
            detail,
            "Analisis manual oleh agen Strategi Bisnis tanpa tool quant.",
        )
    };
    let Some(python) = quant_python() else {
        return not(
            super::STATE_NOT_INSTALLED,
            "python3 tidak ditemukan (set MINIME_QUANT_PYTHON bila perlu).",
        );
    };
    let mut parts = Vec::new();
    let mut missing = Vec::new();
    for (label, module) in [
        ("TradingAgents", "tradingagents"),
        ("qlib", "qlib"),
        ("NautilusTrader", "nautilus_trader"),
        ("OpenBB", "openbb"),
    ] {
        if py_import_ok(&python, module).await {
            parts.push(format!("{label} OK"));
        } else {
            missing.push(label);
        }
    }
    if !missing.is_empty() {
        return not(
            super::STATE_NOT_INSTALLED,
            &format!(
                "Paket belum ter-install: {}. Install: `pip install {}`. {}",
                missing.join(", "),
                missing
                    .iter()
                    .map(|m| match *m {
                        "TradingAgents" => "tradingagents",
                        "qlib" => "pyqlib",
                        "NautilusTrader" => "nautilus_trader",
                        "OpenBB" => "openbb",
                        _ => "",
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                parts.join("; ")
            ),
        );
    }
    let valyu = "Data Valyu via tool 'valyu' terpisah";
    ToolStatus::ready(
        super::TOOL_QUANT,
        "Quant",
        "python",
        &format!("{}. {}.", parts.join("; "), valyu),
        "Analisis manual oleh agen Strategi Bisnis tanpa tool quant.",
    )
}

pub async fn run(op: &str, params: &Value, ctx: &ToolCtx) -> Result<Value> {
    if is_live_op(op) {
        anyhow::bail!(
            "DITOLAK: op '{op}' menyerupai eksekusi live. Tool quant Minime HANYA \
             untuk riset/analisis/backtest/paper-trading. Order live butuh approval \
             eksplisit Patria per aksi dan TIDAK melalui tool ini."
        );
    }
    match op {
        "ta_analyze" => ta_analyze(params, ctx).await,
        "qlib_backtest" => run_python_script(params, ctx, "qlib").await,
        "nautilus_backtest" => run_python_script(params, ctx, "nautilus").await,
        "openbb_data" => openbb_data(params).await,
        _ => anyhow::bail!(
            "op quant tidak dikenal: '{op}' (ta_analyze/qlib_backtest/nautilus_backtest/openbb_data). \
             Untuk pencarian Valyu gunakan tool 'valyu' op 'search'."
        ),
    }
}

/// Tulis skrip Python dari params ke artifact dir, lalu jalankan terisolasi.
async fn run_python_script(params: &Value, ctx: &ToolCtx, kind: &str) -> Result<Value> {
    let python = quant_python().context("python3 tidak ditemukan")?;
    let script = params
        .get("script")
        .and_then(|v| v.as_str())
        .context("params.script wajib (kode Python)")?;
    // Guard kedua: tolak skrip yang memuat pola live-trading.
    let low = script.to_lowercase();
    for pat in [
        "live",
        "place_order",
        "submit_order",
        "binance",
        "interactivebrokers",
        "ibapi",
    ] {
        if low.contains(pat) {
            anyhow::bail!(
                "DITOLAK: skrip mengandung pola live-trading ('{pat}'). \
                 Hanya backtest simulasi / paper trading yang diizinkan."
            );
        }
    }
    let dir = ctx.artifact_dir()?;
    let path = dir.join(format!("quant_{kind}.py"));
    std::fs::write(&path, script)?;
    let out = super::run_cmd(
        &python,
        &[path.to_str().context("path invalid")?],
        Some(&dir),
        Duration::from_secs(1800),
    )
    .await?;
    Ok(json!({ "ok": true, "kind": kind, "script": path.to_string_lossy(), "output": out }))
}

/// Analisis multi-agent TradingAgents: ticker + tanggal → sinyal riset.
async fn ta_analyze(params: &Value, ctx: &ToolCtx) -> Result<Value> {
    let ticker = params
        .get("ticker")
        .and_then(|v| v.as_str())
        .context("params.ticker wajib")?;
    let date = params
        .get("date")
        .and_then(|v| v.as_str())
        .context("params.date wajib (YYYY-MM-DD)")?;
    let analysts = params
        .get("analysts")
        .and_then(|v| v.as_str())
        .unwrap_or("market,news,fundamentals");
    let script = format!(
        r#"from tradingagents.graph.trading_graph import TradingAgentsGraph
from tradingagents.default_config import DEFAULT_CONFIG
ta = TradingAgentsGraph(config=DEFAULT_CONFIG.copy())
_, decision = ta.propagate("{ticker}", "{date}")
print(decision)
print("[SINYAL RISET - BUKAN REKOMENDASI EKSEKUSI]")
"#
    );
    let dir = ctx.artifact_dir()?;
    let path = dir.join("ta_analyze.py");
    std::fs::write(&path, script)?;
    // Analis via env: TRADINGAGENTS_* sudah didukung framework.
    let python = quant_python().context("python3 tidak ditemukan")?;
    let _ = analysts; // documented for future CLI flag use
    let out = super::run_cmd(
        &python,
        &[path.to_str().context("path invalid")?],
        Some(&dir),
        Duration::from_secs(1800),
    )
    .await?;
    Ok(json!({ "ok": true, "ticker": ticker, "date": date, "signal": out }))
}

/// Data pasar via OpenBB SDK.
async fn openbb_data(params: &Value) -> Result<Value> {
    let ticker = params
        .get("ticker")
        .and_then(|v| v.as_str())
        .context("params.ticker wajib")?;
    let provider = params
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("yfinance");
    let script = format!(
        r#"from openbb import obb
df = obb.equity.price.historical("{ticker}", provider="{provider}").to_df()
print(df.tail(5).to_csv())
print("rows:", len(df))
"#
    );
    let python = quant_python().context("python3 tidak ditemukan")?;
    let out = super::run_cmd(&python, &["-c", &script], None, Duration::from_secs(300)).await?;
    Ok(json!({ "ok": true, "ticker": ticker, "provider": provider, "data": out }))
}
