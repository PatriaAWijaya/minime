//! Lightweight file logger for diagnostics.
//!
//! Writes timestamped entries to `~/.minime/logs/minime-YYYY-MM-DD.log`
//! with levels: info / warn / error. Exposed to the frontend via
//! Tauri commands so UI errors leave a trace when users report bugs.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn logs_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME tidak ditemukan".to_string())?;
    let dir = PathBuf::from(home).join(".minime").join("logs");
    fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat folder log: {}", e))?;
    Ok(dir)
}

fn log_file() -> Result<PathBuf, String> {
    let dir = logs_dir()?;
    // Use chrono-free date: rely on SystemTime formatting via a simple approach.
    // We format with std only to avoid new dependencies.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    // Approximate YYYY-MM-DD via days since epoch (UTC). Good enough for rotation.
    let days = now.as_secs() / 86400;
    // 1970-01-01 was a Thursday (day 4 of week starting Monday=1..Sunday=7 → 4).
    // We compute a proleptic date with a simple civil-from-days algorithm.
    let (y, m, d) = civil_from_days(days as i64 + 719468); // days since 0000-03-01
    Ok(dir.join(format!("minime-{:04}-{:02}-{:02}.log", y, m, d)))
}

/// Convert days since 0000-03-01 to (year, month, day). Howard Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (h, min, s) = ((now / 3600) % 24, (now / 60) % 60, now % 60);
    format!("{:02}:{:02}:{:02}Z", h, min, s)
}

/// Write a log entry. `level` must be one of: info, warn, error.
#[tauri::command]
pub fn app_log(level: String, target: String, message: String) -> Result<(), String> {
    let level = level.to_lowercase();
    if !["info", "warn", "error"].contains(&level.as_str()) {
        return Err(format!("Level log tidak valid: {}", level));
    }
    let path = log_file()?;
    let line = format!("[{}] [{}] [{}] {}\n", timestamp(), level.to_uppercase(), target, message);
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Gagal membuka file log: {}", e))?;
    f.write_all(line.as_bytes())
        .map_err(|e| format!("Gagal menulis log: {}", e))?;
    Ok(())
}

/// Read the last `lines` entries from today's log file (or all if fewer).
#[tauri::command]
pub fn read_logs(lines: usize) -> Result<Vec<String>, String> {
    let path = log_file()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Gagal membaca log: {}", e))?;
    let all: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let n = lines.clamp(1, 2000);
    Ok(if all.len() > n {
        all[all.len() - n..].to_vec()
    } else {
        all
    })
}

/// Delete log files older than `keep_days` (default 14).
#[tauri::command]
pub fn prune_logs(keep_days: u64) -> Result<usize, String> {
    let dir = logs_dir()?;
    let keep_secs = keep_days.max(1) * 86400;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut removed = 0;
    let entries = fs::read_dir(&dir).map_err(|e| format!("Gagal membaca folder log: {}", e))?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = modified.duration_since(std::time::UNIX_EPOCH) {
                    if now.saturating_sub(age.as_secs()) > keep_secs && fs::remove_file(&p).is_ok() {
                        removed += 1;
                    }
                }
            }
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_known_date() {
        // 2026-01-01 is day 20454 since Unix epoch; +719468 → days since 0000-03-01
        let (y, m, d) = civil_from_days(20454 + 719468);
        assert_eq!((y, m, d), (2026, 1, 1));
        // 1970-01-01 itself
        let (y0, m0, d0) = civil_from_days(719468);
        assert_eq!((y0, m0, d0), (1970, 1, 1));
    }

    #[test]
    fn log_file_has_date_in_name() {
        let p = log_file().unwrap();
        let name = p.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("minime-"));
        assert!(name.ends_with(".log"));
    }
}
