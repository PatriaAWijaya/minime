//! Shared helpers for localhost OAuth 2.0 flows (desktop apps).
//!
//! Used by the Pireel and Google Drive connectors. Each Minime user
//! connects their OWN account; tokens land in their own OS keychain.
//! Nothing here holds credentials — only protocol mechanics.

use std::io::{Read, Write};
use std::time::Duration;

/// URL-safe base64 without padding.
pub fn b64url(bytes: &[u8]) -> String {
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

pub fn random_b64url(nbytes: usize) -> Result<String, String> {
    let mut buf = vec![0u8; nbytes];
    getrandom::getrandom(&mut buf).map_err(|e| format!("random failed: {e}"))?;
    Ok(b64url(&buf))
}

/// PKCE S256 code challenge for a verifier.
pub fn pkce_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    b64url(&Sha256::digest(verifier.as_bytes()))
}

pub fn url_encode(s: &str) -> String {
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

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn url_decode(s: &str) -> String {
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

/// Parse `?a=1&b=2` query pairs into a map.
pub fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    let mut m = std::collections::HashMap::new();
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
            m.insert(url_decode(k), url_decode(v));
        }
    }
    m
}

/// Open a URL in the user's default browser (cross-platform).
pub fn open_browser(url: &str) -> Result<(), String> {
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

fn callback_html(ok: bool, title_ok: &str, title_err: &str, message: &str) -> String {
    let title = if ok { title_ok } else { title_err };
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head>\
         <body style=\"font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:90vh\">\
         <div><h2>{title}</h2><p>{message}</p><p>Kamu bisa menutup tab ini dan kembali ke Minime.</p></div>\
         </body></html>",
    )
}

/// Bind 127.0.0.1 on an ephemeral port for the OAuth callback.
/// Returns the listener and the port.
pub fn bind_callback() -> Result<(std::net::TcpListener, u16), String> {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| format!("localhost bind failed: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("localhost addr failed: {e}"))?
        .port();
    Ok((listener, port))
}

/// Accept ONE callback connection (blocking, with deadline) and return the
/// open stream plus parsed query params — WITHOUT replying yet, so the
/// caller can finish the token exchange first and then serve a page that
/// reflects the real outcome (Patria 20 Okt: the old flow showed "connected"
/// before the exchange even ran).
pub fn accept_callback_raw(
    listener: &std::net::TcpListener,
    timeout: Duration,
) -> Result<
    (
        std::net::TcpStream,
        std::collections::HashMap<String, String>,
    ),
    String,
> {
    let _ = listener.set_nonblocking(true);
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let mut head = vec![0u8; 8192];
                let n = stream.read(&mut head).unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]).to_string();
                let line = head.lines().next().unwrap_or("");
                let path = line.split_whitespace().nth(1).unwrap_or("");
                let query = path.split('?').nth(1).unwrap_or("");
                let params = parse_query(query);
                return Ok((stream, params));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => return Err(format!("callback accept failed: {e}")),
        }
    }
    Err("callback timed out".to_string())
}

/// Reply to an accepted callback stream with the result page.
pub fn reply_callback(
    mut stream: std::net::TcpStream,
    ok: bool,
    title_ok: &str,
    title_err: &str,
    msg: &str,
) {
    let body = callback_html(ok, title_ok, title_err, msg);
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
}

pub fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64url_vectors() {
        assert_eq!(b64url(b""), "");
        assert_eq!(b64url(b"f"), "Zg");
        assert_eq!(b64url(b"fo"), "Zm8");
        assert_eq!(b64url(b"foo"), "Zm9v");
        assert_eq!(b64url(&[0xfb, 0xff]), "-_8");
    }

    #[test]
    fn pkce_matches_rfc7636_vector() {
        // RFC 7636 Appendix B.
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn url_roundtrip() {
        assert_eq!(url_decode("a+b%20c%2F"), "a b c/");
        assert_eq!(url_encode("a b/c"), "a%20b%2Fc");
    }

    #[test]
    fn query_parsing() {
        let m = parse_query("code=abc&state=xyz&empty=");
        assert_eq!(m["code"], "abc");
        assert_eq!(m["state"], "xyz");
    }
}
