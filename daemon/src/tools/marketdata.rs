//! Market data GRATIS: EDGAR (filings SEC emiten AS), FRED (data makro Federal
//! Reserve), quote harga via yfinance.
//!
//! Valyu tetap sebagai FALLBACK PREMIUM untuk retrieval proprietary yang dalam
//! (pencarian filings mendalam, dataset khusus) — lihat skill `valyu-finance`.
//!
//! Semua op HANYA untuk riset/analisis. Tidak ada eksekusi order dalam bentuk
//! apapun.
//!
//! - EDGAR: tanpa key; SEC mewajibkan header User-Agent yang jujur.
//! - FRED: butuh API key gratis (https://fred.stlouisfed.org/docs/api/api_key.html)
//!   via `secrets::api_key("fred")` (Keychain/env MINIME_FRED_KEY).
//! - yfinance: butuh Python + paket `yfinance` (`pip install yfinance`).
//! - Stooq DINONAKTIFKAN (18 Okt 2026): endpoint CSV kini di balik verifikasi
//!   JavaScript browser — tidak bisa diakses via HTTP langsung. yfinance
//!   menggantikannya untuk quote gratis.

use super::{ToolCtx, ToolStatus};
use crate::secrets;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// User-Agent wajib SEC (jujur, non-bot).
const SEC_UA: &str = "Minime/1.0 (alat riset pribadi; kontak: Patria A. Wijaya)";

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent(SEC_UA)
        .build()
        .context("gagal membuat HTTP client")
}

fn quant_python() -> Option<std::path::PathBuf> {
    super::find_bin("MINIME_QUANT_PYTHON", &["python3"], &[])
}

pub async fn readiness() -> ToolStatus {
    let not = |state: &str, detail: &str| {
        ToolStatus::not(
            super::TOOL_MARKETDATA,
            "Market Data",
            "http+python",
            state,
            detail,
            "Minta data ke agen Strategi Bisnis secara manual, atau pakai tool 'valyu' (fallback premium).",
        )
    };
    // 1. EDGAR: cek jangkauan ringan.
    let edgar_ok = client().map(|c| c).map(|c| async move {
        c.get("https://www.sec.gov/files/company_tickers.json")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    });
    let edgar_ok = match edgar_ok {
        Ok(fut) => fut.await,
        Err(_) => false,
    };
    // 2. yfinance via python.
    let yf_ok = match quant_python() {
        Some(py) => super::run_cmd(
            &py,
            &["-c", "import yfinance"],
            None,
            Duration::from_secs(60),
        )
        .await
        .is_ok(),
        None => false,
    };
    // 3. FRED key.
    let fred_key = secrets::api_key("fred");

    let mut parts = Vec::new();
    if edgar_ok {
        parts.push("EDGAR OK (tanpa key)".to_string());
    }
    if yf_ok {
        parts.push("yfinance OK".to_string());
    }
    match &fred_key {
        Some(_) => parts.push("FRED key OK".to_string()),
        None => parts.push("FRED: tanpa key".to_string()),
    }

    if !edgar_ok && !yf_ok && fred_key.is_none() {
        return not(
            super::STATE_UNREACHABLE,
            "EDGAR tidak terjangkau, yfinance tidak ter-install, dan FRED tanpa key.",
        );
    }
    let mut detail = parts.join("; ");
    if !yf_ok {
        detail.push_str(". quote butuh `pip install yfinance`");
    }
    if fred_key.is_none() {
        detail.push_str(". FRED butuh API key gratis di https://fred.stlouisfed.org/docs/api/api_key.html (Keychain akun 'fred' / env MINIME_FRED_KEY)");
    }
    ToolStatus::ready(
        super::TOOL_MARKETDATA,
        "Market Data",
        "http+python",
        &detail,
        "Minta data ke agen Strategi Bisnis secara manual, atau pakai tool 'valyu' (fallback premium).",
    )
}

pub async fn run(op: &str, params: &Value, _ctx: &ToolCtx) -> Result<Value> {
    match op {
        "quote" => quote(params).await,
        "edgar_ticker" => edgar_ticker(params).await,
        "edgar_facts" => edgar_facts(params).await,
        "edgar_filings" => edgar_filings(params).await,
        "fred" => fred(params).await,
        _ => anyhow::bail!(
            "op marketdata tidak dikenal: '{op}' (quote/edgar_ticker/edgar_facts/edgar_filings/fred). \
             Untuk retrieval proprietary yang dalam gunakan tool 'valyu' op 'search' (fallback premium)."
        ),
    }
}

/// Harga terakhir via yfinance (gratis, tanpa key).
async fn quote(params: &Value) -> Result<Value> {
    let ticker = params
        .get("ticker")
        .and_then(|v| v.as_str())
        .context("params.ticker wajib (cth: AAPL, BBCA.JK)")?;
    let period = params
        .get("period")
        .and_then(|v| v.as_str())
        .unwrap_or("1mo");
    let python = quant_python().context("python3 tidak ditemukan")?;
    let script = format!(
        r#"import yfinance as yf, json
h = yf.Ticker("{ticker}").history(period="{period}")
if h.empty:
    print(json.dumps({{"ok": False, "error": "tidak ada data"}}))
else:
    last = h.iloc[-1]
    print(json.dumps({{"ok": True, "ticker": "{ticker}",
        "last_close": round(float(last["Close"]), 4),
        "last_date": str(h.index[-1].date()),
        "rows": len(h)}}))
    print(h.tail(5).to_csv())
"#
    );
    let out = super::run_cmd(&python, &["-c", &script], None, Duration::from_secs(120)).await?;
    // Baris pertama = JSON ringkas; sisanya CSV.
    let first = out.lines().next().unwrap_or("");
    let summary: Value = serde_json::from_str(first).unwrap_or(json!({"raw": first}));
    Ok(json!({ "ok": true, "ticker": ticker, "summary": summary, "csv_tail": out }))
}

/// ticker → CIK via company_tickers.json SEC.
async fn cik_for_ticker(ticker: &str) -> Result<String> {
    let c = client()?;
    let v: Value = c
        .get("https://www.sec.gov/files/company_tickers.json")
        .send()
        .await
        .context("gagal mengunduh company_tickers.json")?
        .error_for_status()
        .context("SEC menolak permintaan ticker")?
        .json()
        .await
        .context("gagal parse company_tickers.json")?;
    let want = ticker.to_uppercase();
    let obj = v
        .as_object()
        .context("format company_tickers tak dikenal")?;
    for entry in obj.values() {
        if entry
            .get("ticker")
            .and_then(|t| t.as_str())
            .map(|t| t.to_uppercase())
            == Some(want.clone())
        {
            let cik_num = entry
                .get("cik_str")
                .and_then(|c| c.as_u64())
                .context("cik_str hilang")?;
            return Ok(format!("{cik_num:010}"));
        }
    }
    anyhow::bail!("ticker '{ticker}' tidak ditemukan di EDGAR")
}

/// Op: ticker → CIK + nama perusahaan.
async fn edgar_ticker(params: &Value) -> Result<Value> {
    let ticker = params
        .get("ticker")
        .and_then(|v| v.as_str())
        .context("params.ticker wajib")?;
    let cik = cik_for_ticker(ticker).await?;
    Ok(json!({ "ok": true, "ticker": ticker.to_uppercase(), "cik": cik }))
}

/// Op: company facts XBRL (cth concept: Revenues, NetIncomeLoss, Assets).
async fn edgar_facts(params: &Value) -> Result<Value> {
    let ticker = params
        .get("ticker")
        .and_then(|v| v.as_str())
        .context("params.ticker wajib")?;
    let concept = params
        .get("concept")
        .and_then(|v| v.as_str())
        .unwrap_or("Revenues");
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
    let cik = cik_for_ticker(ticker).await?;
    let c = client()?;
    let url = format!("https://data.sec.gov/api/xbrl/companyfacts/CIK{cik}.json");
    let v: Value = c
        .get(&url)
        .send()
        .await
        .context("gagal mengunduh companyfacts")?
        .error_for_status()
        .context("SEC menolak permintaan companyfacts")?
        .json()
        .await
        .context("gagal parse companyfacts")?;
    let facts = v
        .get("facts")
        .and_then(|f| f.get("us-gaap"))
        .and_then(|g| g.get(concept))
        .with_context(|| format!("concept '{concept}' tidak ada di us-gaap untuk {ticker}"))?;
    let units = facts
        .get("units")
        .and_then(|u| u.get("USD"))
        .and_then(|u| u.as_array())
        .context("tidak ada unit USD")?;
    // Ambil N terakhir (biasanya tahunan/kuartalan).
    let rows: Vec<Value> = units
        .iter()
        .rev()
        .take(limit)
        .map(|r| {
            json!({
                "val": r.get("val"),
                "end": r.get("end"),
                "fy": r.get("fy"),
                "fp": r.get("fp"),
                "form": r.get("form"),
                "filed": r.get("filed"),
            })
        })
        .collect();
    Ok(json!({
        "ok": true, "ticker": ticker.to_uppercase(), "cik": cik,
        "concept": concept, "unit": "USD", "count": rows.len(), "rows": rows,
        "sumber": format!("SEC EDGAR companyfacts (diakses {url})"),
    }))
}

/// Op: daftar filings terbaru (filter form opsional: 10-K, 10-Q, 8-K).
async fn edgar_filings(params: &Value) -> Result<Value> {
    let ticker = params
        .get("ticker")
        .and_then(|v| v.as_str())
        .context("params.ticker wajib")?;
    let form_filter = params
        .get("form")
        .and_then(|v| v.as_str())
        .map(|s| s.to_uppercase());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let cik = cik_for_ticker(ticker).await?;
    let c = client()?;
    let url = format!("https://data.sec.gov/submissions/CIK{cik}.json");
    let v: Value = c
        .get(&url)
        .send()
        .await
        .context("gagal mengunduh submissions")?
        .error_for_status()
        .context("SEC menolak permintaan submissions")?
        .json()
        .await
        .context("gagal parse submissions")?;
    let recent = v
        .get("filings")
        .and_then(|f| f.get("recent"))
        .context("tidak ada filings.recent")?;
    let get_arr = |k: &str| -> Vec<String> {
        recent
            .get(k)
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .map(|x| x.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    let forms = get_arr("form");
    let dates = get_arr("filingDate");
    let accs = get_arr("accessionNumber");
    let docs = get_arr("primaryDocument");
    let n = forms.len().min(limit * 3);
    let mut rows = Vec::new();
    for i in 0..n {
        if rows.len() >= limit {
            break;
        }
        if let Some(f) = &form_filter {
            if forms.get(i).map(|x| x.to_uppercase()) != Some(f.clone()) {
                continue;
            }
        }
        let acc_nodash = accs.get(i).map(|a| a.replace('-', "")).unwrap_or_default();
        rows.push(json!({
            "form": forms.get(i),
            "filingDate": dates.get(i),
            "accessionNumber": accs.get(i),
            "primaryDocument": docs.get(i),
            "url": format!("https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
                cik.trim_start_matches('0'), acc_nodash,
                docs.get(i).map(|s| s.as_str()).unwrap_or("")),
        }));
    }
    Ok(json!({
        "ok": true, "ticker": ticker.to_uppercase(), "cik": cik,
        "form_filter": form_filter, "count": rows.len(), "rows": rows,
    }))
}

/// Op: data deret waktu FRED (cth series_id: DGS10, FEDFUNDS, CPIAUCSL, UNRATE).
async fn fred(params: &Value) -> Result<Value> {
    let series_id = params
        .get("series_id")
        .and_then(|v| v.as_str())
        .context("params.series_id wajib (cth: DGS10, FEDFUNDS, CPIAUCSL)")?;
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(12);
    let key = secrets::api_key("fred").context(
        "FRED butuh API key gratis: https://fred.stlouisfed.org/docs/api/api_key.html — \
         simpan di Keychain akun 'fred' (atau env MINIME_FRED_KEY), lalu minta ke Patria bila belum ada.",
    )?;
    let c = client()?;
    let url = "https://api.stlouisfed.org/fred/series/observations";
    let v: Value = c
        .get(url)
        .query(&[
            ("series_id", series_id),
            ("api_key", key.as_str()),
            ("file_type", "json"),
            ("sort_order", "desc"),
            ("limit", &limit.to_string()),
        ])
        .send()
        .await
        .context("gagal memanggil FRED API")?
        .error_for_status()
        .context("FRED menolak permintaan (cek key / series_id)")?
        .json()
        .await
        .context("gagal parse respons FRED")?;
    let obs = v
        .get("observations")
        .and_then(|o| o.as_array())
        .context("tidak ada observations")?;
    let rows: Vec<Value> = obs
        .iter()
        .map(|o| json!({ "date": o.get("date"), "value": o.get("value") }))
        .collect();
    Ok(json!({
        "ok": true, "series_id": series_id.to_uppercase(),
        "count": rows.len(), "rows": rows, "sumber": "FRED (Federal Reserve Bank of St. Louis)",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ToolCtx {
        ToolCtx {
            data_dir: std::env::temp_dir().join("minime-test"),
            repo_dir: std::env::temp_dir().join("minime-test"),
            run_id: None,
            agent_id: "test".into(),
        }
    }

    // Live test EDGAR — hanya jalan eksplisit di mesin tanpa proxy intercept
    // (di VM sandbox gagal TLS karena proxy; endpoint terverifikasi via curl 18 Okt 2026).
    // Jalankan di MacBook: cargo test marketdata -- --ignored
    #[tokio::test]
    #[ignore]
    async fn edgar_ticker_aapl_live() {
        let v = run("edgar_ticker", &json!({"ticker": "AAPL"}), &ctx())
            .await
            .unwrap();
        assert_eq!(v["cik"], "0000320193");
    }

    #[tokio::test]
    #[ignore]
    async fn edgar_facts_revenues_live() {
        let v = run(
            "edgar_facts",
            &json!({"ticker": "AAPL", "concept": "Revenues", "limit": 2}),
            &ctx(),
        )
        .await
        .unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert!(!v["rows"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn edgar_filings_10k_live() {
        let v = run(
            "edgar_filings",
            &json!({"ticker": "AAPL", "form": "10-K", "limit": 2}),
            &ctx(),
        )
        .await
        .unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["rows"][0]["form"], "10-K");
    }

    #[tokio::test]
    async fn fred_tanpa_key_gagal_jujur() {
        if secrets::api_key("fred").is_none() {
            let e = run("fred", &json!({"series_id": "DGS10"}), &ctx())
                .await
                .unwrap_err();
            assert!(e.to_string().contains("FRED butuh API key"));
        }
    }

    #[tokio::test]
    async fn op_tak_dikenal_ditolak() {
        let e = run("order", &json!({}), &ctx()).await.unwrap_err();
        assert!(e.to_string().contains("tidak dikenal"));
    }
}
