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
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::oauth_local::{
    bind_callback, now_epoch, open_browser, pkce_challenge, random_b64url,
    url_encode,
};

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

fn write_bundle(bundle: &TokenBundle) -> Result<(), String> {
    let raw = serde_json::to_string(bundle).map_err(|e| format!("encode failed: {e}"))?;
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
        Ok(entry) => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("keychain delete failed: {e}")),
        },
        Err(e) => Err(format!("keychain open failed: {e}")),
    }
}

/// True when the user has connected their Pireel account (for the runtime gate).
pub(crate) fn has_token() -> bool {
    read_bundle().is_some()
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
    /// Last OAuth failure reason, surfaced to the UI (Patria 20 Okt).
    last_error: std::sync::Arc<Mutex<Option<String>>>,
    /// Tokens from a completed exchange, awaiting keychain write on the
    /// Tauri thread (Patria 21 Okt: keychain writes from a raw spawned
    /// thread do not persist on macOS).
    pending_tokens: std::sync::Arc<Mutex<Option<TokenBundle>>>,
}

impl PireelState {
    pub fn new() -> Self {
        Self {
            meta: std::sync::Arc::new(Mutex::new(None)),
            pending: std::sync::Arc::new(Mutex::new(None)),
            last_error: std::sync::Arc::new(Mutex::new(None)),
            pending_tokens: std::sync::Arc::new(Mutex::new(None)),
        }
    }

    fn pending_arc(&self) -> std::sync::Arc<Mutex<Option<PendingOAuth>>> {
        std::sync::Arc::clone(&self.pending)
    }

    fn error_arc(&self) -> std::sync::Arc<Mutex<Option<String>>> {
        std::sync::Arc::clone(&self.last_error)
    }

    fn tokens_arc(&self) -> std::sync::Arc<Mutex<Option<TokenBundle>>> {
        std::sync::Arc::clone(&self.pending_tokens)
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
async fn valid_access_token() -> Result<String, String> {
    let bundle = read_bundle().ok_or_else(|| "not_connected".to_string())?;
    if !bundle.needs_refresh() {
        return Ok(bundle.access_token);
    }
    // The bundle carries its own client_id + token endpoint from registration.
    let fresh = refresh_access_token(&bundle.token_endpoint, &bundle.client_id, &bundle.refresh_token)
        .await
        .map_err(|_| "reauth_needed".to_string())?;
    write_bundle(&fresh).map_err(|_| "keychain_write".to_string())?;
    Ok(fresh.access_token)
}

// ---------------------------------------------------------------------------
// System browser + localhost OAuth callback
// ---------------------------------------------------------------------------

/// Handle one OAuth callback connection, exchange the code, store tokens.
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
        // Synchronous token exchange inside a blocking thread. The bundle is
        // NOT written here: keychain writes from a raw spawned thread do not
        // persist on macOS (Patria 21 Okt) — the poller writes it on the
        // Tauri thread instead.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("runtime: {e}"))?;
        rt.block_on(exchange_code(&pending, &code))
    })();
    let ok = outcome.is_ok();
    let msg = match &outcome {
        Ok(_) => "Otorisasi selesai — akun Pireel terhubung.".to_string(),
        Err(e) => format!("Koneksi gagal: {e}"),
    };
    crate::oauth_local::reply_callback(
        stream,
        ok,
        "Pireel terhubung",
        "Pireel gagal terhubung",
        &msg,
    );
    // Publish the final state for the poller: success stashes the bundle
    // for the Tauri-thread keychain write; failure marks it expired.
    let mut slot = pending_slot.lock().unwrap();
    if ok {
        *tokens_slot.lock().unwrap() = outcome.ok();
        *slot = None;
        *error_slot.lock().unwrap() = None;
    } else {
        *error_slot.lock().unwrap() = outcome.err();
        *slot = Some(PendingOAuth {
            expires_at: 0,
            ..pending.clone()
        });
    }
}

async fn exchange_code(pending: &PendingOAuth, code: &str) -> Result<TokenBundle, String> {
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
    Ok(TokenBundle {
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
    /// Last failure reason, if the flow failed.
    pub error: Option<String>,
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

    let (listener, port) = bind_callback()?;
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
    *s.last_error.lock().unwrap() = None;
    let pending_slot = s.pending_arc();
    let error_slot = s.error_arc();
    let tokens_slot = s.tokens_arc();
    std::thread::spawn(move || {
        serve_callback(listener, pending, pending_slot, error_slot, tokens_slot)
    });

    // Best effort: also return the URL so the UI can show a fallback link.
    let _ = open_browser(&auth_url);
    Ok(auth_url)
}

/// Poll the OAuth flow state.
#[tauri::command]
pub fn pireel_oauth_poll(state: tauri::State<'_, PireelState>) -> PireelOAuthState {
    // Keychain write happens HERE (Tauri thread), never in the spawned
    // callback thread (Patria 21 Okt).
    let mut write_error: Option<String> = None;
    if let Some(bundle) = state.pending_tokens.lock().unwrap().take() {
        match write_bundle(&bundle) {
            Ok(()) => {
                // Verify it actually persisted.
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
    // A stashed bundle that failed to persist must not read as "connected".
    let st = if st == "connected" && write_error.is_some() {
        "idle"
    } else {
        st
    };
    PireelOAuthState {
        state: st.to_string(),
        error,
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
pub async fn pireel_status() -> Result<PireelStatus, String> {
    let token = match valid_access_token().await {
        Ok(t) => t,
        Err(code) => {
            return Ok(PireelStatus {
                connected: false,
                detail: code,
            })
        }
    };
    Ok(match get_state_text(&token).await {
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
    })
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
    use crate::oauth_local::{now_epoch, parse_query};

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
    fn callback_params_parsing() {
        let m = parse_query("code=abc123&state=xyz");
        assert_eq!(m["code"], "abc123");
        assert_eq!(m["state"], "xyz");
        let m = parse_query("error=access_denied");
        assert!(!m.contains_key("code"));
    }

    /// Without a keychain token the status must fail cleanly as
    /// "not_connected" — and make no network request. Skips the assertion
    /// (vacuous pass) when a token happens to exist in the keychain.
    #[tokio::test]
    async fn status_without_token_is_not_connected() {
        if read_bundle().is_some() {
            return;
        }
        let s = pireel_status().await.expect("status never fails hard");
        assert!(!s.connected);
        assert_eq!(s.detail, "not_connected");
    }
}
