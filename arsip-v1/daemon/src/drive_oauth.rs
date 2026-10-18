//! Google Drive OAuth 2.0 — authorization code flow dengan loopback redirect.
//! Scope: `drive.readonly` (sekali hubung, read-only).
//!
//! Alur:
//! 1. UI panggil IPC `drive.auth_url` → daemon bind 127.0.0.1:0, kembalikan URL
//!    otorisasi Google. UI membukanya di browser.
//! 2. Patria approve → Google redirect ke http://127.0.0.1:<port>/callback?code=…
//! 3. Daemon menukar code → token, menyimpan refresh token di OS Keychain
//!    (write-only; tidak pernah dibaca balik / di-log — rule #2, #19).
//! 4. Event `DriveAuth` memberitahu UI hasil akhirnya.
//!
//! Prasyarat: OAuth client bertipe "Desktop app" dibuat di Google Cloud Console
//! oleh Patria; `client_id` (+ `client_secret` bila ada) disimpan via IPC
//! `secrets.set` (lihat docs/drive-oauth.md). Flow browser penuh hanya bisa
//! diuji di MacBook Patria.

use crate::events::Event;
use crate::secrets;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;

pub const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const TOKENS_ACCOUNT: &str = "drive-oauth";
pub const CLIENT_ID_ACCOUNT: &str = "drive-client-id";
pub const CLIENT_SECRET_ACCOUNT: &str = "drive-client-secret";
/// Batas tunggu user menyelesaikan approve di browser.
const FLOW_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBlob {
    refresh_token: String,
    access_token: String,
    /// Unix timestamp kedaluwarsa access token.
    expires_at: i64,
    scope: String,
}

fn read_blob() -> Option<TokenBlob> {
    let raw = secrets::api_key(TOKENS_ACCOUNT)?;
    serde_json::from_str::<TokenBlob>(&raw).ok()
}

fn write_blob(blob: &TokenBlob) -> Result<()> {
    let raw = serde_json::to_string(blob).context("serialize token blob")?;
    secrets::set_api_key(TOKENS_ACCOUNT, &raw).map_err(|e| anyhow::anyhow!(e))
}

/// Percent-encode untuk query string OAuth.
fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Mulai flow OAuth: bind loopback listener, kembalikan URL otorisasi.
/// Penukaran code berjalan di background task; hasilnya via event `DriveAuth`.
pub async fn start_flow(tx: broadcast::Sender<Event>) -> Result<serde_json::Value> {
    let client_id = secrets::api_key(CLIENT_ID_ACCOUNT).ok_or_else(|| {
        anyhow::anyhow!(
            "drive-client-id belum diset — simpan via IPC secrets.set \
             (lihat docs/drive-oauth.md langkah 1)"
        )
    })?;
    if read_blob().is_some_and(|b| !b.refresh_token.is_empty()) {
        bail!("Drive sudah terhubung — putuskan dulu via drive.disconnect bila ingin hubung ulang");
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind loopback listener")?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let state = uuid::Uuid::new_v4().simple().to_string();

    let url = format!(
        "{AUTH_URL}?client_id={}&redirect_uri={}&response_type=code\
         &scope={}&access_type=offline&prompt=consent&state={}",
        enc(&client_id),
        enc(&redirect_uri),
        enc(DRIVE_SCOPE),
        enc(&state),
    );

    tokio::spawn(async move {
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(FLOW_TIMEOUT_SECS),
            wait_for_code(listener, &state),
        )
        .await;
        let outcome: Result<()> = match res {
            Err(_) => Err(anyhow::anyhow!(
                "waktu habis — user tidak menyelesaikan approve (10 menit)"
            )),
            Ok(Err(e)) => Err(e),
            Ok(Ok(code)) => exchange_and_store(&code, &redirect_uri).await,
        };
        let status = match &outcome {
            Ok(()) => "connected".to_string(),
            Err(e) => format!("gagal: {e:#}"),
        };
        if let Err(e) = &outcome {
            // Jangan log token; pesan error aman (tidak mengandung secret).
            eprintln!("[drive-oauth] {e:#}");
        }
        let _ = tx.send(Event::DriveAuth { status });
    });

    Ok(json!({ "url": url, "expires_in": FLOW_TIMEOUT_SECS }))
}

/// Tunggu satu koneksi callback, parse `code` + validasi `state`.
async fn wait_for_code(listener: tokio::net::TcpListener, expected_state: &str) -> Result<String> {
    let (mut stream, _) = listener.accept().await.context("accept callback")?;
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await.context("read callback")?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let request_line = req.lines().next().unwrap_or("");
    // "GET /callback?code=...&state=... HTTP/1.1"
    let path = request_line
        .split_whitespace()
        .nth(1)
        .context("request line tidak valid")?;
    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut code: Option<String> = None;
    let mut state: Option<String> = None;
    let mut error: Option<String> = None;
    for kv in query.split('&') {
        let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
        match k {
            "code" => code = Some(decode(v)),
            "state" => state = Some(decode(v)),
            "error" => error = Some(decode(v)),
            _ => {}
        }
    }
    let body = if error.is_none() && code.is_some() && state.as_deref() == Some(expected_state) {
        "<html><body style='font-family:sans-serif;padding:40px'>\
         <h2>Minime terhubung ke Google Drive ✅</h2>\
         <p>Kamu bisa menutup tab ini dan kembali ke aplikasi Minime.</p></body></html>"
    } else {
        "<html><body style='font-family:sans-serif;padding:40px'>\
         <h2>Gagal menghubungkan ❌</h2>\
         <p>State tidak cocok atau otorisasi ditolak. Coba lagi dari aplikasi Minime.</p></body></html>"
    };
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes()).await;

    if let Some(e) = error {
        bail!("user menolak otorisasi: {e}");
    }
    if state.as_deref() != Some(expected_state) {
        bail!("state tidak cocok — kemungkinan serangan CSRF, dibatalkan");
    }
    code.context("tidak ada authorization code di callback")
}

fn decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.bytes();
    while let Some(b) = it.next() {
        if b == b'%' {
            let hi = it.next().unwrap_or(b'0');
            let lo = it.next().unwrap_or(b'0');
            let v = |c: u8| match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => 0,
            };
            out.push((v(hi) * 16 + v(lo)) as char);
        } else if b == b'+' {
            out.push(' ');
        } else {
            out.push(b as char);
        }
    }
    out
}

/// Tukar authorization code → token, simpan refresh token di Keychain.
async fn exchange_and_store(code: &str, redirect_uri: &str) -> Result<()> {
    let client_id = secrets::api_key(CLIENT_ID_ACCOUNT).context("client_id hilang")?;
    let client_secret = secrets::api_key(CLIENT_SECRET_ACCOUNT).unwrap_or_default();
    let client = crate::provider::http_client();
    let mut form = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri.to_string()),
    ];
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret));
    }
    let resp = client.post(TOKEN_URL).form(&form).send().await?;
    let status = resp.status();
    let val: serde_json::Value = resp.json().await.context("token response bukan JSON")?;
    if !status.is_success() {
        let msg = val
            .get("error_description")
            .or_else(|| val.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("token exchange gagal");
        bail!("google: {msg}");
    }
    let refresh_token = val
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if refresh_token.is_empty() {
        bail!(
            "Google tidak mengembalikan refresh_token (butuh access_type=offline + prompt=consent)"
        );
    }
    let access_token = val
        .get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let expires_in = val
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    let now = chrono::Utc::now().timestamp();
    write_blob(&TokenBlob {
        refresh_token,
        access_token,
        expires_at: now + expires_in,
        scope: DRIVE_SCOPE.to_string(),
    })?;
    Ok(())
}

/// Status koneksi (tanpa membaca ulang token ke caller — hanya metadata).
pub fn status() -> serde_json::Value {
    match read_blob() {
        Some(b) if !b.refresh_token.is_empty() => json!({
            "connected": true,
            "scope": b.scope,
            "access_token_expires_at": b.expires_at,
            "has_client_id": secrets::api_key(CLIENT_ID_ACCOUNT).is_some(),
        }),
        _ => json!({
            "connected": false,
            "has_client_id": secrets::api_key(CLIENT_ID_ACCOUNT).is_some(),
        }),
    }
}

/// Putuskan koneksi: hapus token dari Keychain.
pub fn disconnect() -> Result<()> {
    secrets::delete_api_key(TOKENS_ACCOUNT).map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

/// Access token yang valid (refresh otomatis bila kedaluwarsa).
/// Dipakai wiring tool Drive di Fase 4+; tidak mengekspos token ke UI/IPC.
pub async fn access_token() -> Result<String> {
    let blob = read_blob().context("Drive belum terhubung — jalankan drive.auth_url dulu")?;
    let now = chrono::Utc::now().timestamp();
    if now < blob.expires_at - 60 && !blob.access_token.is_empty() {
        return Ok(blob.access_token);
    }
    // Refresh.
    let client_id = secrets::api_key(CLIENT_ID_ACCOUNT).context("client_id hilang")?;
    let client_secret = secrets::api_key(CLIENT_SECRET_ACCOUNT).unwrap_or_default();
    let client = crate::provider::http_client();
    let mut form = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", blob.refresh_token.clone()),
        ("client_id", client_id),
    ];
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret));
    }
    let resp = client.post(TOKEN_URL).form(&form).send().await?;
    let status = resp.status();
    let val: serde_json::Value = resp.json().await.context("refresh response bukan JSON")?;
    if !status.is_success() {
        let msg = val
            .get("error_description")
            .or_else(|| val.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("refresh gagal");
        bail!("google: {msg} — hubungkan ulang via drive.auth_url");
    }
    let access_token = val
        .get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if access_token.is_empty() {
        bail!("Google tidak mengembalikan access_token");
    }
    let expires_in = val
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    write_blob(&TokenBlob {
        access_token: access_token.clone(),
        expires_at: chrono::Utc::now().timestamp() + expires_in,
        ..blob
    })?;
    Ok(access_token)
}
