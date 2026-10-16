//! Native notifications — works even when the Tauri UI is closed.
//! macOS: `osascript` display notification (no extra dependency).
//! Linux: `notify-send` when available, else stderr log.
//! Full macOS test happens on Patria's MacBook; on Linux we only verify
//! the code compiles and degrades gracefully.

/// Approval requested → notify Patria. Returns silently on best effort.
pub fn approval_requested(stage: &str, summary: &str) {
    let title = "Minime — Persetujuan dibutuhkan";
    let body = format!("Stage '{stage}': {summary}");
    send(title, &body);
}

/// Generic notification helper.
pub fn notify(title: &str, body: &str) {
    send(title, body);
}

fn send(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification {b} with title {t}",
            b = osa_string(body),
            t = osa_string(title)
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        let ok = std::process::Command::new("notify-send")
            .args([title, body])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            eprintln!("[notify] {title}: {body}");
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        eprintln!("[notify] {title}: {body}");
    }
}

#[cfg(target_os = "macos")]
fn osa_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
