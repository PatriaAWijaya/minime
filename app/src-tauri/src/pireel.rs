//! Pireel connection layer (Tahap 3.4).
//!
//! Pireel Studio exposes its MCP endpoint at
//! `https://pireel.com/api/studio/mcp` (streamable HTTP, stateless) and
//! authenticates third-party clients with standard OAuth 2.1:
//!   - Resource metadata:  `https://pireel.com/.well-known/oauth-protected-resource`
//!   - Server metadata:    `https://pireel.com/.well-known/oauth-authorization-server`
//!     (authorization / token / dynamic-client-registration endpoints,
//!      public clients via `token_endpoint_auth_method: none` + PKCE S256)
//!
//! Each Minime user connects their OWN Pireel account (Free tier: 20
//! credits/day). Tokens are stored in the OS keychain (service
//! `com.minime.app`, account `pireel`) and never bundled or committed.
//! `pireel_status` calls the free `get_state` tool only — it never
//! consumes credits.

use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MCP_URL: &str = "https://pireel.com/api/studio/mcp";
const RESOURCE_META_URL: &str = "https://pireel.com/.well-known/oauth-protected-resource";
const KEYCHAIN_ACCOUNT: &str = "pireel";
const KEYCHAIN_SERVICE: &str = "com.minime.app";
/// RFC 7636 test vector (used by unit tests).
#[cfg(test)]
const RFC7636_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
#[cfg(test)]
const RFC7636_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

// ---------------------------------------------------------------------------
// Small helpers (pure, unit-tested)
// ---------------------------------------------------------------------------

/// URL-safe base64 without padding.
fn b64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4 + 2) / 3);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (chunk.get(1).copied().unwrap_or(0) as u32) << 8
            | chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((n >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(n & 63) as usize] as char);
        }
    }
    out
}

fn random_b64url(nbytes: usize) -> Result<String, String> {
    let mut buf = vec![0u8; nbytes];
    getrandom::getrandom(&mut buf).map_err(|e| format!("random failed: {e}"))?;
    Ok(b64url(&buf))
}

fn sha256_b64url(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(input.as_bytes());
    b64url(&digest)
}

/// PKCE S256 code challenge for a verifier.
fn pkce_challenge(verifier: &str) -> String {
    sha256_b64url(verifier)
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Classify a `get_state` text snapshot into a short status code.
fn classify_state(text: &str) -> &'static str {
    if text.contains("OFFLINE MODE") {
        "offline"
    } else if text.contains("no cloud project") {
        "fresh"
    } else if text.contains("<composition_state>") || text.contains("composition_state") {
        "live"
    } else {
        "unknown"
    }
}

/// Extract `code` and `state` from a raw HTTP callback request head.
fn parse_callback_query(head: &str) -> Option<(String, String)> {
    let line = head.lines().next().unwrap_or("");
    let path = line.split_whitespace().nth(1).unwrap_or("");
    let query = path.split('?').nth(1).unwrap_or("");
    let mut code = None;
    let mut state = None;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("code"), Some(v)) => code = Some(url_decode(v)),
            (Some("state"), Some(v)) => state = Some(url_decode(v)),
            _ => {}
        }
    }
    match (code, state) {
        (Some(c), Some(s)) if !c.is_empty() => Some((c, s)),
        _ => None,
    }
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                    out.push((h * 16 + l) as char);
                    i += 3;
                    continue;
                }
                out.push('%');
                i += 1;
            }
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Token bundle (keychain)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBundle {
    access_token: String,
    refresh_token: String,
    /// Unix epoch seconds when the access token expires.
    expires_at: u64,
    /// Dynamic OAuth client id (needed for refresh).
    client_id: String,
    /// Token endpoint (cached from server metadata).
    token_endpoint: String,
}

impl TokenBundle {
    fn needs_refresh(&self) -> bool {
        self.expires_at <= now_epoch() + 60
    }
}

fn read_bundle() -> Option<TokenBundle> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT).ok()?;
    let raw = entry.get_password().ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_bundle(bundle: &TokenBundle) -> Result<(), String> {
    let raw = serde_json::to_string(bundle).map_err(|e| format!("encode failed: {e}"))?;
    keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|e| format!("keychain open failed: {e}"))?
        .set_password(&raw)
        .map_err(|e| format!("keychain write failed: {e}"))
}

fn delete_bundle() -> Result<(), String> {
    match keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("keychain delete failed: {e}")),
        },
        Err(e) => Err(format!("keychain open failed: {e}")),
    }
}

// ---------------------------------------------------------------------------
// OAuth metadata + dynamic client registration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct ResourceMeta {
    authorization_servers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerMeta {
    authorization_endpoint: String,
    token_endpoint: String,
    registration_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RegisterRequest {
    client_name: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    token_endpoint_auth_method: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RegisterResponse {
    client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[derive(Debug, Clone)]
struct PendingOAuth {
    verifier: String,
    state: String,
    client_id: String,
    token_endpoint: String,
    redirect_uri: String,
    /// Seconds since epoch when this flow expires.
    expires_at: u64,
}

#[derive(Clone)]
pub struct PireelState {
    meta: std::sync::Arc<Mutex<Option<ServerMeta>>>,
    pending: std::sync::Arc<Mutex<Option<PendingOAuth>>>,
}

impl PireelState {
    pub fn new() -> Self {
        Self {
            meta: std::sync::Arc::new(Mutex::new(None)),
            pending: std::sync::Arc::new(Mutex::new(None)),
        }
    }

    fn pending_arc(&self) -> std::sync::Arc<Mutex<Option<PendingOAuth>>> {
        std::sync::Arc::clone(&self.pending)
    }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(25))
        .user_agent("Minime/0.1 (Tauri)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

async fn fetch_server_meta(state: &PireelState) -> Result<ServerMeta, String> {
    if let Some(m) = state.meta.lock().unwrap().clone() {
        return Ok(m);
    }
    let client = http_client();
    let res: ResourceMeta = client
        .get(RESOURCE_META_URL)
        .send()
        .await
        .map_err(|e| format!("metadata fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("metadata http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("metadata parse failed: {e}"))?;
    let auth_server = res
        .authorization_servers
        .first()
        .cloned()
        .ok_or_else(|| "no authorization server advertised".to_string())?;
    let meta_url = format!("{auth_server}/.well-known/oauth-authorization-server");
    let meta: ServerMeta = client
        .get(&meta_url)
        .send()
        .await
        .map_err(|e| format!("server metadata fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("server metadata http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("server metadata parse failed: {e}"))?;
    *state.meta.lock().unwrap() = Some(meta.clone());
    Ok(meta)
}

// ---------------------------------------------------------------------------
// JSON-RPC over the MCP endpoint (stateless streamable HTTP)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

async fn rpc_call(
    access_token: &str,
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let client = http_client();
    let req = RpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
    };
    let resp: RpcResponse = client
        .post(MCP_URL)
        .header("Accept", "application/json, text/event-stream")
        .bearer_auth(access_token)
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("mcp request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("mcp http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("mcp parse failed: {e}"))?;
    if let Some(err) = resp.error {
        return Err(format!("mcp error: {}", err.message));
    }
    resp.result.ok_or_else(|| "empty mcp result".to_string())
}

/// Call the free `get_state` tool and return its text summary.
async fn get_state_text(access_token: &str) -> Result<String, String> {
    // MCP handshake (stateless: initialize per call is fine).
    let _ = rpc_call(
        access_token,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "minime", "version": "0.1.0" }
        })),
    )
    .await?;
    let result = rpc_call(
        access_token,
        "tools/call",
        Some(serde_json::json!({ "name": "get_state", "arguments": {} })),
    )
    .await?;
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    if text.is_empty() {
        return Err("get_state returned no text".to_string());
    }
    Ok(text)
}

async fn refresh_access_token(
    token_endpoint: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenBundle, String> {
    let client = http_client();
    let mut form = HashMap::new();
    form.insert("grant_type", "refresh_token");
    form.insert("refresh_token", refresh_token);
    form.insert("client_id", client_id);
    let resp: TokenResponse = client
        .post(token_endpoint)
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
        refresh_token: resp.refresh_token.unwrap_or_else(|| refresh_token.to_string()),
        expires_at: now_epoch() + resp.expires_in.unwrap_or(3600),
        client_id: client_id.to_string(),
        token_endpoint: token_endpoint.to_string(),
    })
}

/// Ensure a usable access token (refresh when near expiry).
/// Returns the access token, or an error code for the UI.
async fn valid_access_token(state: &PireelState) -> Result<String, String> {
    let bundle = read_bundle().ok_or_else(|| "not_connected".to_string())?;
    if !bundle.needs_refresh() {
        return Ok(bundle.access_token);
    }
    let meta = fetch_server_meta(state).await?;
    let fresh = refresh_access_token(&bundle.token_endpoint, &bundle.client_id, &bundle.refresh_token)
        .await
        .map_err(|_| "reauth_needed".to_string())?;
    write_bundle(&fresh).map_err(|_| "keychain_write".to_string())?;
    Ok(fresh.access_token)
}

// ---------------------------------------------------------------------------
// System browser + localhost OAuth callback
// ---------------------------------------------------------------------------

/// Open a URL in the user's default browser (cross-platform).
fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    };
    #[cfg(target_os = "linux")]
    let mut cmd = std::process::Command::new("xdg-open");
    #[cfg(target_os = "macos")]
    cmd.arg(url);
    #[cfg(target_os = "linux")]
    cmd.arg(url);
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("could not open browser: {e}"))
}

fn callback_html(ok: bool, message: &str) -> String {
    let title = if ok { "Pireel terhubung" } else { "Pireel gagal terhubung" };
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head>\
         <body style=\"font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:90vh\">\
         <div><h2>{title}</h2><p>{message}</p><p>Kamu bisa menutup tab ini dan kembali ke Minime.</p></div>\
         </body></html>",
        title = title,
        message = message
    )
}

/// Handle one OAuth callback connection, exchange the code, store tokens.
fn serve_callback(
    listener: std::net::TcpListener,
    pending: PendingOAuth,
    pending_slot: std::sync::Arc<Mutex<Option<PendingOAuth>>>,
) {
    // Single attempt with a deadline; the frontend polls `pireel_oauth_poll`.
    let deadline = std::time::Instant::now() + Duration::from_secs(600);
    while std::time::Instant::now() < deadline {
        let _ = listener.set_nonblocking(true);
        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let mut head = vec![0u8; 8192];
                let n = match stream.read(&mut head) {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let head = String::from_utf8_lossy(&head[..n]).to_string();
                let outcome: Result<(), String> = (|| {
                    let (code, cb_state) =
                        parse_callback_query(&head).ok_or("missing code/state")?;
                    if cb_state != pending.state {
                        return Err("state mismatch".to_string());
                    }
                    // Synchronous token exchange inside a blocking thread.
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|e| format!("runtime: {e}"))?;
                    rt.block_on(exchange_code(&pending, &code))
                })();
                let (ok, msg) = match &outcome {
                    Ok(_) => (true, "Akun Pireel-mu terhubung ke Minime.".to_string()),
                    Err(e) => (false, format!("Gagal menukar kode otorisasi: {e}")),
                };
                let body = callback_html(ok, &msg);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                use std::io::Write;
                let _ = stream.write_all(resp.as_bytes());
                // Publish the final state for the poller: success clears the
                // slot (bundle is in the keychain); failure marks it expired.
                let mut slot = pending_slot.lock().unwrap();
                if outcome.is_ok() {
                    *slot = None;
                } else {
                    *slot = Some(PendingOAuth {
                        expires_at: 0,
                        ..pending.clone()
                    });
                }
                return;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(_) => break,
        }
    }
    // Timeout: clear the pending flow so a new one can start.
    *pending_slot.lock().unwrap() = None;
}

async fn exchange_code(pending: &PendingOAuth, code: &str) -> Result<(), String> {
    let client = http_client();
    let mut form = HashMap::new();
    form.insert("grant_type", "authorization_code");
    form.insert("code", code);
    form.insert("redirect_uri", pending.redirect_uri.as_str());
    form.insert("client_id", pending.client_id.as_str());
    form.insert("code_verifier", pending.verifier.as_str());
    let resp: TokenResponse = client
        .post(&pending.token_endpoint)
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("token http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("token parse failed: {e}"))?;
    write_bundle(&TokenBundle {
        access_token: resp.access_token,
        refresh_token: resp
            .refresh_token
            .ok_or_else(|| "no refresh token granted (offline_access scope missing)".to_string())?,
        expires_at: now_epoch() + resp.expires_in.unwrap_or(3600),
        client_id: pending.client_id.clone(),
        token_endpoint: pending.token_endpoint.clone(),
    })
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// OAuth login state for the frontend poller.
#[derive(Debug, Clone, Serialize)]
pub struct PireelOAuthState {
    /// "pending" | "connected" | "idle"
    pub state: String,
}

/// Start the Pireel OAuth flow: register a client, open the system browser
/// at the authorization URL, and listen on localhost for the callback.
/// Returns the authorization URL (also opened automatically).
#[tauri::command]
pub async fn pireel_connect_url(state: tauri::State<'_, PireelState>) -> Result<String, String> {
    // Own the state so the future is 'static.
    let s = state.inner().clone();
    // One flow at a time.
    if s.pending.lock().unwrap().is_some() {
        return Err("oauth already in progress".to_string());
    }
    let meta = fetch_server_meta(&s).await?;
    let reg_url = meta
        .registration_endpoint
        .clone()
        .ok_or_else(|| "server does not support dynamic client registration".to_string())?;

    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| format!("localhost bind failed: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("localhost addr failed: {e}"))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    // Dynamic client registration (RFC 7591, public client).
    let client = http_client();
    let reg: RegisterResponse = client
        .post(&reg_url)
        .json(&RegisterRequest {
            client_name: "Minime".to_string(),
            redirect_uris: vec![redirect_uri.clone()],
            grant_types: vec!["authorization_code".to_string(), "refresh_token".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "none".to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("client registration failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("registration http error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("registration parse failed: {e}"))?;

    let verifier = random_b64url(32)?;
    let challenge = pkce_challenge(&verifier);
    let oauth_state = random_b64url(16)?;
    let pending = PendingOAuth {
        verifier,
        state: oauth_state.clone(),
        client_id: reg.client_id,
        token_endpoint: meta.token_endpoint.clone(),
        redirect_uri: redirect_uri.clone(),
        expires_at: now_epoch() + 600,
    };

    let auth_url = format!(
        "{ep}?response_type=code&client_id={cid}&redirect_uri={ru}&scope={sc}&state={st}&code_challenge={cc}&code_challenge_method=S256",
        ep = meta.authorization_endpoint,
        cid = url_encode(&pending.client_id),
        ru = url_encode(&redirect_uri),
        sc = url_encode("openid profile email offline_access"),
        st = url_encode(&oauth_state),
        cc = url_encode(&challenge),
    );

    *s.pending.lock().unwrap() = Some(pending.clone());
    let pending_slot = s.pending_arc();
    std::thread::spawn(move || serve_callback(listener, pending, pending_slot));

    // Best effort: also return the URL so the UI can show a fallback link.
    let _ = open_browser(&auth_url);
    Ok(auth_url)
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Poll the OAuth flow state.
#[tauri::command]
pub fn pireel_oauth_poll(state: tauri::State<'_, PireelState>) -> PireelOAuthState {
    let pending = state.pending.lock().unwrap().clone();
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
    PireelOAuthState {
        state: st.to_string(),
    }
}

/// Connection status shown on the Pireel card.
#[derive(Debug, Clone, Serialize)]
pub struct PireelStatus {
    pub connected: bool,
    /// "not_connected" | "live" | "offline" | "fresh" | "unknown" |
    /// "reauth_needed" | "network_error"
    pub detail: String,
}

#[tauri::command]
pub async fn pireel_status(state: tauri::State<'_, PireelState>) -> PireelStatus {
    let s = state.inner().clone();
    let token = match valid_access_token(&s).await {
        Ok(t) => t,
        Err(code) => {
            return PireelStatus {
                connected: false,
                detail: code,
            }
        }
    };
    match get_state_text(&token).await {
        Ok(text) => PireelStatus {
            connected: true,
            detail: classify_state(&text).to_string(),
        },
        Err(e) => PireelStatus {
            connected: false,
            detail: if e.contains("401") || e.contains("unauthorized") {
                "reauth_needed".to_string()
            } else {
                "network_error".to_string()
            },
        },
    }
}

/// Disconnect: remove tokens from the keychain.
#[tauri::command]
pub fn pireel_disconnect() -> Result<(), String> {
    delete_bundle()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_matches_rfc7636_vector() {
        assert_eq!(pkce_challenge(RFC7636_VERIFIER), RFC7636_CHALLENGE);
    }

    #[test]
    fn b64url_vectors() {
        assert_eq!(b64url(b""), "");
        assert_eq!(b64url(b"f"), "Zg");
        assert_eq!(b64url(b"fo"), "Zm8");
        assert_eq!(b64url(b"foo"), "Zm9v");
        assert_eq!(b64url(&[0xfb, 0xff]), "-_8");
        assert_eq!(b64url(&[0xff, 0xff, 0xff]), "____");
    }

    #[test]
    fn bundle_refresh_logic() {
        let fresh = TokenBundle {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at: now_epoch() + 3600,
            client_id: "c".into(),
            token_endpoint: "t".into(),
        };
        assert!(!fresh.needs_refresh());
        let stale = TokenBundle {
            expires_at: now_epoch(),
            ..fresh.clone()
        };
        assert!(stale.needs_refresh());
    }

    #[test]
    fn state_classification() {
        assert_eq!(classify_state("OFFLINE MODE\nno tab"), "offline");
        assert_eq!(classify_state("no cloud project"), "fresh");
        assert_eq!(classify_state("<composition_state>{}"), "live");
        assert_eq!(classify_state("weird"), "unknown");
    }

    #[test]
    fn callback_query_parsing() {
        let head = "GET /callback?code=abc123&state=xyz HTTP/1.1\r\nHost: x\r\n\r\n";
        assert_eq!(
            parse_callback_query(head),
            Some(("abc123".to_string(), "xyz".to_string()))
        );
        let bad = "GET /callback?error=access_denied HTTP/1.1\r\n\r\n";
        assert_eq!(parse_callback_query(bad), None);
    }

    #[test]
    fn url_roundtrip() {
        assert_eq!(url_decode("a+b%20c%2F"), "a b c/");
        assert_eq!(url_encode("a b/c"), "a%20b%2Fc");
    }
}
