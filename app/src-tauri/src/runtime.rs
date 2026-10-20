//! Agent runtime — tool dispatch (Tahap 3.6).
//!
//! Connects the tool gateway to real work: an agent runs a tool from its
//! allowlist, the result flows to the Library, and the Kanban card moves
//! along Antrian → Dikerjakan → Butuh Approval → Selesai.
//!
//! Gates (all enforced, all honest):
//!   1. The tool must be on the agent's allowlist (`not_allowed`).
//!   2. Tools with `requires_auth` need a real user connection
//!      (`not_connected`) — checked against the keychain, not UI state.
//!   3. Tools without a real implementation fail as `tool_not_implemented`
//!      instead of pretending to work.

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::tools::ToolRegistry;

/// Arguments for a tool run (all strings; the tool parses what it needs).
pub type ToolArgs = HashMap<String, String>;

/// Outcome of a tool run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRunResult {
    pub ok: bool,
    /// Short human-readable summary (goes to the Library).
    pub summary: String,
    /// Machine-readable payload (JSON value).
    pub data: serde_json::Value,
}

impl ToolRunResult {
    fn ok(summary: String, data: serde_json::Value) -> Self {
        Self {
            ok: true,
            summary,
            data,
        }
    }
}

/// Extract (title, text) from HTML. Pure and unit-tested.
fn extract_text(html: &str) -> (String, String) {
    // Title: first <title>…</title>.
    let title = html
        .find("<title>")
        .and_then(|s| {
            html[s + 7..]
                .find("</title>")
                .map(|e| html[s + 7..s + 7 + e].trim().to_string())
        })
        .unwrap_or_default();

    // Strip head/scripts/styles, then all tags.
    let mut text = html.to_string();
    for tag in ["head", "script", "style", "noscript"] {
        loop {
            let lower = text.to_lowercase();
            let open = match lower.find(&format!("<{tag}")) {
                Some(i) => i,
                None => break,
            };
            let close = match lower[open..].find(&format!("</{tag}>")) {
                Some(i) => open + i + tag.len() + 3,
                None => break,
            };
            text.replace_range(open..close, " ");
        }
    }
    let mut plain = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                plain.push(' ');
            }
            _ if !in_tag => plain.push(ch),
            _ => {}
        }
    }
    // Decode a few common entities + collapse whitespace.
    let decoded = plain
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    // Cap at ~6000 chars so a run result stays manageable.
    let capped: String = collapsed.chars().take(6000).collect();
    (title, capped)
}

/// Real web fetch for the `browser` tool (no credentials needed).
async fn browser_fetch(url: &str) -> Result<ToolRunResult, String> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("bad url (http/https only)".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36")
        .build()
        .map_err(|e| format!("client failed: {e}"))?;
    let html = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("http error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("read failed: {e}"))?;
    if html.len() > 2_000_000 {
        return Err("page too large".to_string());
    }
    let (title, text) = extract_text(&html);
    let shown_title = if title.is_empty() { url } else { &title };
    Ok(ToolRunResult::ok(
        format!("Dibaca: {shown_title} ({} karakter)", text.len()),
        serde_json::json!({ "url": url, "title": title, "text": text }),
    ))
}

/// Connection gate against the REAL credential stores (not UI mirrors).
fn check_connected(tool_id: &str, registry: &ToolRegistry) -> Result<(), String> {
    if !registry.requires_auth(tool_id) {
        return Ok(());
    }
    // Auth tools must prove a real user connection. drive/pireel check the
    // keychain; anything else without a dedicated check fails honestly.
    let connected = match tool_id {
        "drive" => crate::drive::has_token(),
        "pireel" => crate::pireel::has_token(),
        _ => return Err("not_connected".to_string()),
    };
    if connected {
        Ok(())
    } else {
        Err("not_connected".to_string())
    }
}

/// Dispatch one tool call for an agent. All gates enforced.
async fn dispatch(
    registry: &ToolRegistry,
    agent_id: &str,
    tool_id: &str,
    action: &str,
    args: &ToolArgs,
) -> Result<ToolRunResult, String> {
    if !registry.is_known(tool_id) {
        return Err("unknown_tool".to_string());
    }
    if !registry.can_use(agent_id, tool_id) {
        return Err("not_allowed".to_string());
    }
    check_connected(tool_id, registry)?;
    match (tool_id, action) {
        ("browser", "fetch") => {
            let url = args.get("url").cloned().unwrap_or_default();
            browser_fetch(&url).await
        }
        ("drive", "upload") => {
            let file_name = args.get("file_name").cloned().unwrap_or_default();
            let content_base64 = args.get("content_base64").cloned().unwrap_or_default();
            let mime_type = args.get("mime_type").cloned().unwrap_or_default();
            let r = crate::drive::drive_upload(file_name, content_base64, mime_type).await?;
            Ok(ToolRunResult::ok(
                format!("Di-upload ke folder MIMI: {} (id {})", r.name, r.id),
                serde_json::json!({ "id": r.id, "name": r.name }),
            ))
        }
        _ => Err("tool_not_implemented".to_string()),
    }
}

/// Tools with a real implementation, for one agent (pure helper).
fn implemented_for(registry: &ToolRegistry, agent_id: &str) -> Vec<String> {
    registry
        .allowed_tools(agent_id)
        .into_iter()
        .filter(|t| matches!(t.as_str(), "browser" | "drive"))
        .collect()
}

/// Allowed tool ids for an agent (drives the UI tool picker).
#[tauri::command]
pub fn runtime_allowed_tools(
    agent_id: String,
    registry: tauri::State<'_, ToolRegistry>,
) -> Vec<String> {
    // Only tools with a real implementation are offered.
    implemented_for(&registry, &agent_id)
}

/// Run a tool for an agent. Returns the result or a short error code.
#[tauri::command]
pub async fn tool_run(
    agent_id: String,
    tool_id: String,
    action: String,
    args: ToolArgs,
    registry: tauri::State<'_, ToolRegistry>,
) -> Result<ToolRunResult, String> {
    // Own the registry handle so the future is 'static.
    let reg = registry.inner().clone();
    dispatch(&reg, &agent_id, &tool_id, &action, &args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ToolRegistry {
        ToolRegistry::new()
    }

    #[test]
    fn extract_text_vectors() {
        let html = "<html><head><title>Hai</title><style>.x{}</style></head>\
                    <body><script>alert(1)</script><h1>Judul</h1><p>Isi&nbsp;teks</p></body></html>";
        let (title, text) = extract_text(html);
        assert_eq!(title, "Hai");
        assert_eq!(text, "Judul Isi teks");
    }

    #[test]
    fn extract_text_no_title() {
        let (title, text) = extract_text("<p>saja</p>");
        assert_eq!(title, "");
        assert_eq!(text, "saja");
    }

    #[tokio::test]
    async fn gate_not_allowed() {
        // code agent may not use pireel (allowlist).
        let reg = registry();
        let err = dispatch(&reg, "code", "pireel", "status", &ToolArgs::new())
            .await
            .expect_err("must be gated");
        assert_eq!(err, "not_allowed");
    }

    #[tokio::test]
    async fn gate_unknown_tool() {
        let reg = registry();
        let err = dispatch(&reg, "riset", "nope", "x", &ToolArgs::new())
            .await
            .expect_err("must fail");
        assert_eq!(err, "unknown_tool");
    }

    #[tokio::test]
    async fn gate_not_implemented() {
        // riset may use browser, but action "fly" has no implementation.
        let reg = registry();
        let err = dispatch(&reg, "riset", "browser", "fly", &ToolArgs::new())
            .await
            .expect_err("must fail");
        assert_eq!(err, "tool_not_implemented");
    }

    #[tokio::test]
    async fn gate_bad_url() {
        let reg = registry();
        let mut args = ToolArgs::new();
        args.insert("url".into(), "ftp://x".into());
        let err = dispatch(&reg, "riset", "browser", "fetch", &args)
            .await
            .expect_err("must fail");
        assert!(err.contains("bad url"), "got: {err}");
    }

    /// Full dispatch path against a LOCAL HTTP server: proves the real
    /// fetch + extraction works without external network or credentials.
    #[tokio::test]
    async fn dispatch_browser_fetch_local_server() {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("bind local server");
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let body = "<html><head><title>Uji Lokal</title></head><body><p>Halo dunia uji.</p></body></html>";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            if let Ok((mut s, _)) = listener.accept() {
                use std::io::Write;
                let mut head = [0u8; 1024];
                use std::io::Read;
                let _ = s.read(&mut head);
                let _ = s.write_all(resp.as_bytes());
            }
        });
        let reg = registry();
        let mut args = ToolArgs::new();
        args.insert("url".into(), format!("http://127.0.0.1:{port}/"));
        let r = dispatch(&reg, "riset", "browser", "fetch", &args)
            .await
            .expect("local fetch must work");
        assert!(r.ok);
        assert_eq!(r.data["title"], "Uji Lokal");
        assert!(r.data["text"].as_str().unwrap().contains("Halo dunia uji"));
    }

    #[test]
    fn allowed_tools_only_implemented() {
        // riset allowlist = browser + ruangapi; only browser is implemented.
        let reg = registry();
        assert_eq!(implemented_for(&reg, "riset"), vec!["browser".to_string()]);
    }

    #[test]
    fn check_connected_no_token() {
        // drive requires auth; no token in a clean keychain → not_connected.
        // (Skips hard assertion when a token happens to exist.)
        if crate::drive::has_token() {
            return;
        }
        let reg = registry();
        assert_eq!(
            check_connected("drive", &reg).expect_err("must fail"),
            "not_connected"
        );
        // browser needs no auth.
        assert!(check_connected("browser", &reg).is_ok());
    }

    /// Full dispatch E2E of the connection gate (Patria 20 Okt): an auth
    /// tool with no dedicated keychain check fails honestly at dispatch,
    /// deterministically — no keychain state involved.
    #[tokio::test]
    async fn dispatch_not_connected_e2e() {
        let reg = registry();
        let err = dispatch(&reg, "mimi", "slack", "status", &ToolArgs::new())
            .await
            .expect_err("must be gated");
        assert_eq!(err, "not_connected");
    }

    /// Full dispatch E2E against the real internet (Patria 20 Okt): proves
    /// browser/fetch works end-to-end without mocks or credentials.
    #[tokio::test]
    async fn dispatch_browser_fetch_real_internet() {
        let reg = registry();
        let mut args = ToolArgs::new();
        args.insert("url".into(), "https://example.com".into());
        let r = dispatch(&reg, "riset", "browser", "fetch", &args)
            .await
            .expect("real fetch must work");
        assert!(r.ok);
        assert!(r.data["title"].as_str().unwrap().contains("Example Domain"));
        assert!(!r.data["text"].as_str().unwrap().is_empty());
    }
}
