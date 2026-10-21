//! Google Drive connector (Tahap 3.5) — deliverables go to the MIMI folder.
//!
//! Each Minime user connects their OWN Google account with their OWN OAuth
//! client ID (Google Cloud Console → OAuth client ID → Desktop app). No
//! Google credentials are bundled with Minime; tokens live in the user's
//! OS keychain (service `com.minime.app`, account `google-drive`).
//!
//! Scope is the least-privilege `drive.file`: the app sees only the files
//! and folders it created itself (including the MIMI folder it makes).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::oauth_local::{
    bind_callback, now_epoch, open_browser, pkce_challenge, random_b64url,
    url_encode,
};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const DRIVE_API: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_API: &str = "https://www.googleapis.com/upload/drive/v3";
const SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const MIMI_FOLDER: &str = "MIMI";
const KEYCHAIN_ACCOUNT: &str = "google-drive";
const KEYCHAIN_SERVICE: &str = "com.minime.app";

/// A Google OAuth client ID looks like `<id>.apps.googleusercontent.com`.
fn valid_client_id(id: &str) -> bool {
    let id = id.trim();
    !id.is_empty()
        && id.ends_with(".apps.googleusercontent.com")
        && id.len() > ".apps.googleusercontent.com".len()
        && !id.contains(char::is_whitespace)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBundle {
    access_token: String,
    refresh_token: String,
    expires_at: u64,
    client_id: String,
}

impl TokenBundle {
    fn needs_refresh(&self) -> bool {
        self.expires_at <= now_epoch() + 60
    }
}

fn read_bundle() -> Option<TokenBundle> {
    // File fallback (Patria 21 Okt: untuk impor token manual via SSH).
    if let Ok(home) = std::env::var("HOME") {
        let p = std::path::Path::new(&home).join(".minime").join("drive_tokens.json");
        if let Ok(raw) = std::fs::read_to_string(&p) {
            if let Ok(b) = serde_json::from_str::<TokenBundle>(&raw) {
                return Some(b);
            }
        }
    }
    if let Ok(entry) = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        if let Ok(raw) = entry.get_password() {
            if let Ok(b) = serde_json::from_str::<TokenBundle>(&raw) {
                return Some(b);
            }
        }
    }
    // Fallback for ad-hoc-signed binaries (Patria 21 Okt).
    let raw = crate::oauth_local::security_cli_read(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)?;
    serde_json::from_str(&raw).ok()
}

fn write_bundle(b: &TokenBundle) -> Result<(), String> {
    let raw = serde_json::to_string(b).map_err(|e| format!("encode failed: {e}"))?;
    // 1. keyring crate (may fail closed on ad-hoc-signed builds — Patria 21 Okt).
    let _ = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map(|e| e.set_password(&raw));
    // 2. Verify it persisted; fall back to `security` CLI.
    if read_bundle().is_some() {
        return Ok(());
    }
    crate::oauth_local::security_cli_write(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT, &raw)
        .map_err(|e| format!("keychain write failed ({e})"))?;
    // 3. Final verification — never report success silently.
    if read_bundle().is_some() {
        Ok(())
    } else {
        Err("keychain tidak menyimpan token".to_string())
    }
}

fn delete_bundle() -> Result<(), String> {
    match keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        Ok(e) => match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("keychain delete failed: {e}")),
        },
        Err(e) => Err(format!("keychain open failed: {e}")),
    }
}

/// True when the user has connected their Google account (for the runtime gate).
pub(crate) fn has_token() -> bool {
    read_bundle().is_some()
}

#[derive(Debug, Clone)]
struct PendingOAuth {
    verifier: String,
    state: String,
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    expires_at: u64,
}

#[derive(Clone)]
pub struct DriveState {
    pending: std::sync::Arc<Mutex<Option<PendingOAuth>>>,
    /// Last OAuth failure reason, surfaced to the UI (Patria 20 Okt).
    last_error: std::sync::Arc<Mutex<Option<String>>>,
    /// Tokens awaiting keychain write on the Tauri thread (Patria 21 Okt).
    pending_tokens: std::sync::Arc<Mutex<Option<TokenBundle>>>,
}

impl DriveState {
    pub fn new() -> Self {
        Self {
            pending: std::sync::Arc::new(Mutex::new(None)),
            last_error: std::sync::Arc::new(Mutex::new(None)),
            pending_tokens: std::sync::Arc::new(Mutex::new(None)),
        }
    }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Minime/0.1 (Tauri)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

async fn exchange_code(
    client_id: &str,
    client_secret: Option<&str>,
    redirect_uri: &str,
    verifier: &str,
    code: &str,
) -> Result<TokenBundle, String> {
    let client = http_client();
    let mut form = HashMap::new();
    form.insert("grant_type", "authorization_code");
    form.insert("code", code);
    form.insert("client_id", client_id);
    if let Some(secret) = client_secret {
        form.insert("client_secret", secret);
    }
    form.insert("redirect_uri", redirect_uri);
    form.insert("code_verifier", verifier);
    let raw = client
        .post(TOKEN_URL)
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?;
    let status = raw.status();
    let body = raw.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("token http {status}: {}", body.chars().take(300).collect::<String>()));
    }
    let resp: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| format!("token parse failed: {e}"))?;
    Ok(TokenBundle {
        access_token: resp.access_token,
        refresh_token: resp
            .refresh_token
            .ok_or_else(|| "no refresh token granted — retry with a fresh consent".to_string())?,
        expires_at: now_epoch() + resp.expires_in.unwrap_or(3600),
        client_id: client_id.to_string(),
    })
}

async fn refresh_bundle(bundle: &TokenBundle) -> Result<TokenBundle, String> {
    let client = http_client();
    let mut form = HashMap::new();
    form.insert("grant_type", "refresh_token");
    form.insert("client_id", bundle.client_id.as_str());
    form.insert("refresh_token", bundle.refresh_token.as_str());
    let resp: TokenResponse = client
        .post(TOKEN_URL)
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("refresh request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("refresh http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("refresh parse failed: {e}"))?;
    Ok(TokenBundle {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token.unwrap_or_else(|| bundle.refresh_token.clone()),
        expires_at: now_epoch() + resp.expires_in.unwrap_or(3600),
        client_id: bundle.client_id.clone(),
    })
}

async fn valid_access_token() -> Result<String, String> {
    let bundle = read_bundle().ok_or_else(|| "not_connected".to_string())?;
    if !bundle.needs_refresh() {
        return Ok(bundle.access_token);
    }
    let fresh = refresh_bundle(&bundle)
        .await
        .map_err(|_| "reauth_needed".to_string())?;
    write_bundle(&fresh).map_err(|_| "keychain_write".to_string())?;
    Ok(fresh.access_token)
}

fn serve_callback(
    listener: std::net::TcpListener,
    pending: PendingOAuth,
    pending_slot: std::sync::Arc<Mutex<Option<PendingOAuth>>>,
    error_slot: std::sync::Arc<Mutex<Option<String>>>,
    tokens_slot: std::sync::Arc<Mutex<Option<TokenBundle>>>,
) {
    // Patria 20 Okt: accept first, exchange the code, THEN reply — the page
    // must reflect the real outcome, never a premature "connected".
    let (stream, params) =
        match crate::oauth_local::accept_callback_raw(&listener, Duration::from_secs(600)) {
            Ok(v) => v,
            Err(e) => {
                *error_slot.lock().unwrap() = Some(e.clone());
                *pending_slot.lock().unwrap() = Some(PendingOAuth {
                    expires_at: 0,
                    ..pending.clone()
                });
                return;
            }
        };
    let outcome: Result<TokenBundle, String> = (|| {
        let code = params.get("code").cloned().unwrap_or_default();
        let cb_state = params.get("state").cloned().unwrap_or_default();
        if code.is_empty() {
            return Err(params
                .get("error")
                .cloned()
                .unwrap_or_else(|| "tidak ada kode otorisasi".to_string()));
        }
        if cb_state != pending.state {
            return Err("state mismatch".to_string());
        }
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("runtime: {e}"))?;
        let bundle = rt.block_on(exchange_code(
            &pending.client_id,
            pending.client_secret.as_deref(),
            &pending.redirect_uri,
            &pending.verifier,
            &code,
        ))?;
        Ok(bundle)
    })();
    let ok = outcome.is_ok();
    let msg = match &outcome {
        Ok(_) => "Otorisasi selesai — Google Drive terhubung.".to_string(),
        Err(e) => format!("Koneksi gagal: {e}"),
    };
    crate::oauth_local::reply_callback(
        stream,
        ok,
        "Google Drive terhubung",
        "Google Drive gagal terhubung",
        &msg,
    );
    let mut slot = pending_slot.lock().unwrap();
    if ok {
        *tokens_slot.lock().unwrap() = outcome.ok();
        *slot = None;
        *error_slot.lock().unwrap() = None;
    } else {
        *error_slot.lock().unwrap() = outcome.err();
        // Clear pending so the user can retry with a fresh code
        // (auth codes are single-use; retrying with a burned code = 400).
        *slot = None;
    }
}

// ---------------------------------------------------------------------------
// Drive API
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FileList {
    files: Option<Vec<DriveFile>>,
}

#[derive(Debug, Deserialize)]
struct DriveFile {
    id: String,
    #[allow(dead_code)]
    name: Option<String>,
}

/// Find the MIMI folder (created by this app) or create it.
async fn ensure_mimi_folder(access_token: &str) -> Result<String, String> {
    let client = http_client();
    let q = format!("name='{MIMI_FOLDER}' and mimeType='application/vnd.google-apps.folder' and trashed=false");
    let list: FileList = client
        .get(format!("{DRIVE_API}/files"))
        .bearer_auth(access_token)
        .query(&[("q", q.as_str()), ("fields", "files(id)"), ("pageSize", "1")])
        .send()
        .await
        .map_err(|e| format!("folder lookup failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("folder lookup http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("folder lookup parse failed: {e}"))?;
    if let Some(f) = list.files.and_then(|mut v| v.pop()) {
        return Ok(f.id);
    }
    let created: DriveFile = client
        .post(format!("{DRIVE_API}/files"))
        .bearer_auth(access_token)
        .json(&serde_json::json!({
            "name": MIMI_FOLDER,
            "mimeType": "application/vnd.google-apps.folder",
        }))
        .send()
        .await
        .map_err(|e| format!("folder create failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("folder create http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("folder create parse failed: {e}"))?;
    Ok(created.id)
}

/// Build a multipart/related body for a file upload.
fn multipart_body(boundary: &str, metadata: &str, mime: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(metadata.len() + data.len() + 256);
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(metadata.as_bytes());
    body.extend_from_slice(format!("\r\n--{boundary}\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Type: {mime}\r\n\r\n").as_bytes());
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct DriveOAuthPhase {
    /// "pending" | "connected" | "idle"
    pub state: String,
    /// Last failure reason, if the flow failed.
    pub error: Option<String>,
}

/// Start Google OAuth with the user's own client ID. Opens the system
/// browser; returns the authorization URL as a fallback link.
#[tauri::command]
pub async fn drive_connect_url(
    state: tauri::State<'_, DriveState>,
    client_id: String,
    client_secret: Option<String>,
) -> Result<String, String> {
    let s = state.inner().clone();
    let client_id = client_id.trim().to_string();
    if !valid_client_id(&client_id) {
        return Err("invalid_client_id".to_string());
    }
    if s.pending.lock().unwrap().is_some() {
        return Err("oauth already in progress".to_string());
    }
    let (listener, port) = bind_callback()?;
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let verifier = random_b64url(32)?;
    let oauth_state = random_b64url(16)?;
    let pending = PendingOAuth {
        verifier: verifier.clone(),
        state: oauth_state.clone(),
        client_id: client_id.clone(),
        client_secret: client_secret.map(|x| x.trim().to_string()).filter(|x| !x.is_empty()),
        redirect_uri: redirect_uri.clone(),
        expires_at: now_epoch() + 600,
    };
    let auth_url = format!(
        "{AUTH_URL}?response_type=code&client_id={cid}&redirect_uri={ru}&scope={sc}&state={st}&code_challenge={cc}&code_challenge_method=S256&access_type=offline&prompt=consent",
        cid = url_encode(&client_id),
        ru = url_encode(&redirect_uri),
        sc = url_encode(SCOPE),
        st = url_encode(&oauth_state),
        cc = url_encode(&pkce_challenge(&verifier)),
    );
    *s.pending.lock().unwrap() = Some(pending.clone());
    *s.last_error.lock().unwrap() = None;
    let slot = std::sync::Arc::clone(&s.pending);
    let error_slot = std::sync::Arc::clone(&s.last_error);
    let tokens_slot = std::sync::Arc::clone(&s.pending_tokens);
    std::thread::spawn(move || {
        serve_callback(listener, pending, slot, error_slot, tokens_slot)
    });
    let _ = open_browser(&auth_url);
    Ok(auth_url)
}

#[tauri::command]
pub fn drive_oauth_poll(state: tauri::State<'_, DriveState>) -> DriveOAuthPhase {
    let mut write_error: Option<String> = None;
    if let Some(bundle) = state.pending_tokens.lock().unwrap().take() {
        match write_bundle(&bundle) {
            Ok(()) => {
                if read_bundle().is_none() {
                    write_error = Some("keychain tidak menyimpan token".to_string());
                }
            }
            Err(e) => write_error = Some(e),
        }
        if let Some(e) = write_error.clone() {
            *state.last_error.lock().unwrap() = Some(e);
        }
    }
    let pending = state.pending.lock().unwrap().clone();
    let mut error = state.last_error.lock().unwrap().clone();
    if let Some(e) = write_error.clone() {
        error = Some(e);
    }
    let st = match pending {
        None => {
            if read_bundle().is_some() {
                "connected"
            } else {
                "idle"
            }
        }
        Some(p) if p.expires_at == 0 || p.expires_at < now_epoch() => "idle",
        Some(_) => "pending",
    };
    let st = if st == "connected" && write_error.is_some() {
        "idle"
    } else {
        st
    };
    DriveOAuthPhase {
        state: st.to_string(),
        error,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DriveStatus {
    pub connected: bool,
    /// "not_connected" | "connected" | "reauth_needed" | "network_error"
    pub detail: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct About {
    user: Option<AboutUser>,
}

#[derive(Debug, Deserialize)]
struct AboutUser {
    #[allow(dead_code)]
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

#[tauri::command]
pub async fn drive_status() -> Result<DriveStatus, String> {
    let token = match valid_access_token().await {
        Ok(t) => t,
        Err(code) => {
            return Ok(DriveStatus {
                connected: false,
                detail: code,
                email: None,
            })
        }
    };
    let client = http_client();
    let about: Result<About, String> = client
        .get(format!("{DRIVE_API}/about"))
        .bearer_auth(&token)
        .query(&[("fields", "user(emailAddress,displayName)")])
        .send()
        .await
        .map_err(|_| "network_error".to_string())?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                "reauth_needed".to_string()
            } else {
                "network_error".to_string()
            }
        })?
        .json()
        .await
        .map_err(|_| "network_error".to_string());
    match about {
        Ok(a) => Ok(DriveStatus {
            connected: true,
            detail: "connected".to_string(),
            email: a.user.and_then(|u| u.email_address),
        }),
        Err(detail) => Ok(DriveStatus {
            connected: false,
            detail,
            email: None,
        }),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DriveUploadResult {
    pub id: String,
    pub name: String,
}

/// Upload a deliverable (base64 content) into the MIMI folder.
#[tauri::command]
pub async fn drive_upload(
    file_name: String,
    content_base64: String,
    mime_type: String,
) -> Result<DriveUploadResult, String> {
    let file_name = file_name.trim().to_string();
    if file_name.is_empty() || file_name.len() > 200 {
        return Err("bad file name".to_string());
    }
    let data = b64_decode(&content_base64).ok_or_else(|| "bad base64 content".to_string())?;
    if data.len() > 25 * 1024 * 1024 {
        return Err("file too large (max 25 MB)".to_string());
    }
    let token = valid_access_token().await?;
    let folder_id = ensure_mimi_folder(&token).await?;
    let boundary = random_b64url(16)?;
    let metadata = serde_json::json!({
        "name": file_name,
        "parents": [folder_id],
        "mimeType": if mime_type.trim().is_empty() { "text/plain" } else { mime_type.trim() },
    })
    .to_string();
    let mime = if mime_type.trim().is_empty() {
        "text/plain"
    } else {
        mime_type.trim()
    };
    let body = multipart_body(&boundary, &metadata, mime, &data);
    let client = http_client();
    let created: DriveFile = client
        .post(format!("{UPLOAD_API}/files?uploadType=multipart"))
        .bearer_auth(&token)
        .header(
            "Content-Type",
            format!("multipart/related; boundary={boundary}"),
        )
        .header("Content-Length", body.len().to_string())
        .body(body)
        .send()
        .await
        .map_err(|e| format!("upload failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("upload http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("upload parse failed: {e}"))?;
    Ok(DriveUploadResult {
        id: created.id,
        name: file_name,
    })
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let clean: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.is_empty() || clean.len() % 4 != 0 {
        return None;
    }
    let val = |b: u8| -> Option<u32> {
        match b {
            b'A'..=b'Z' => Some((b - b'A') as u32),
            b'a'..=b'z' => Some((b - b'a' + 26) as u32),
            b'0'..=b'9' => Some((b - b'0' + 52) as u32),
            b'+' | b'-' => Some(62),
            b'/' | b'_' => Some(63),
            b'=' => Some(64), // padding marker
            _ => None,
        }
    };
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    for chunk in clean.chunks(4) {
        let mut pad = 0;
        let mut vals = [0u32; 4];
        for (i, &b) in chunk.iter().enumerate() {
            let v = val(b)?;
            if v == 64 {
                pad += 1;
                vals[i] = 0;
            } else {
                if pad > 0 {
                    return None; // data after padding
                }
                vals[i] = v;
            }
        }
        if pad > 2 {
            return None;
        }
        let triple = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6) | vals[3];
        out.push((triple >> 16) as u8);
        if pad < 2 {
            out.push((triple >> 8) as u8);
        }
        if pad < 1 {
            out.push(triple as u8);
        }
    }
    Some(out)
}

#[tauri::command]
pub fn drive_disconnect() -> Result<(), String> {
    delete_bundle()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_id_validation() {
        assert!(valid_client_id(
            "123456789012-abc123def456.apps.googleusercontent.com"
        ));
        assert!(!valid_client_id(""));
        assert!(!valid_client_id("not-a-client-id"));
        assert!(!valid_client_id(".apps.googleusercontent.com"));
        assert!(!valid_client_id(
            "123.apps.googleusercontent.com.evil.com"
        ));
        assert!(!valid_client_id("12 34.apps.googleusercontent.com"));
    }

    #[test]
    fn multipart_body_shape() {
        let body = multipart_body("BOUND", r#"{"name":"a.txt"}"#, "text/plain", b"hi");
        let s = String::from_utf8(body).unwrap();
        assert!(s.starts_with("--BOUND\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n"));
        assert!(s.contains("{\"name\":\"a.txt\"}"));
        assert!(s.contains("\r\n--BOUND\r\nContent-Type: text/plain\r\n\r\nhi\r\n--BOUND--\r\n"));
    }

    #[test]
    fn b64_decode_vectors() {
        assert_eq!(b64_decode("Zm9v").unwrap(), b"foo");
        assert_eq!(b64_decode("Zg==").unwrap(), b"f");
        assert_eq!(b64_decode("Zm8=").unwrap(), b"fo");
        assert_eq!(b64_decode("!!!!"), None);
        assert_eq!(b64_decode("abc"), None);
    }

    /// Without a keychain token the status must fail cleanly as
    /// "not_connected" — and make no network request.
    #[tokio::test]
    async fn status_without_token_is_not_connected() {
        if read_bundle().is_some() {
            return;
        }
        let s = drive_status().await.expect("status never fails hard");
        assert!(!s.connected);
        assert_eq!(s.detail, "not_connected");
    }

    #[tokio::test]
    async fn upload_without_token_fails_cleanly() {
        if read_bundle().is_some() {
            return;
        }
        let err = drive_upload("t.txt".into(), "aGk=".into(), "text/plain".into())
            .await
            .expect_err("must fail without token");
        assert_eq!(err, "not_connected");
    }
}
