//! Secret resolution for model providers.
//!
//! Lookup order per provider `account` (e.g. "muse", "gemini", "deepseek",
//! "routerku"):
//!   1. `MINIME_<ACCOUNT>_KEY` env var (explicit dev override).
//!   2. OS keychain entry (service `com.minime.app`, account name).
//!   3. macOS fallback: Apple's `security` CLI (works for ad-hoc-signed
//!      binaries where the keyring crate fails closed).
//!
//! Public builds ship with NO credentials bundled: every user connects
//! their own keys, which land in their own OS keychain. Patria's personal
//! keys live in his MacBook keychain and are never part of the repo.
//!
//! NEVER hardcode, commit, or log key material.

const SERVICE: &str = "com.minime.app";

/// Resolve an API key for `account`. Returns None when not configured —
/// the router then skips the provider and falls through to the next one.
pub fn api_key(account: &str) -> Option<String> {
    // 1. Explicit dev override.
    let env_name = format!("MINIME_{}_KEY", account.to_uppercase().replace('-', "_"));
    if let Ok(v) = std::env::var(&env_name) {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    // 1b. File fallback (~/.minime/keys/<account>.key) — untuk binary ad-hoc
    // yang Keychain-nya tidak stabil (write OK, read gagal).
    if let Some(k) = file_read(account) {
        return Some(k);
    }
    // 2. OS keychain.
    if let Some(k) = keyring_read(account) {
        return Some(k);
    }
    // 3. macOS `security` CLI fallback.
    #[cfg(target_os = "macos")]
    if let Some(k) = security_cli_read(account) {
        return Some(k);
    }
    None
}

/// Store an API key for `account` in the OS keychain. Called when the user
/// connects a provider from the app's UI.
pub fn set_api_key(account: &str, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("empty key".to_string());
    }
    // Selalu tulis backup ke file dulu (anti-gagal).
    file_write(account, key)?;
    // Lalu coba Keychain (boleh gagal, file sudah aman).
    let _ = keyring::Entry::new(SERVICE, account)
        .map_err(|e| format!("keychain open failed: {e}"))
        .and_then(|e| e.set_password(key).map_err(|e| format!("keychain write failed: {e}")));
    Ok(())
}

/// File fallback: ~/.minime/keys/<account>.key (mode 600).
fn keys_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::Path::new(&h).join(".minime").join("keys")
    })
}

fn file_read(account: &str) -> Option<String> {
    let p = keys_dir()?.join(format!("{account}.key"));
    let c = std::fs::read_to_string(p).ok()?;
    let c = c.trim().to_string();
    if c.is_empty() { None } else { Some(c) }
}

fn file_write(account: &str, key: &str) -> Result<(), String> {
    let dir = keys_dir().ok_or("HOME tidak ditemukan")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir gagal: {e}"))?;
    let p = dir.join(format!("{account}.key"));
    std::fs::write(&p, key).map_err(|e| format!("tulis file gagal: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Remove an API key for `account` from the OS keychain (disconnect).
pub fn delete_api_key(account: &str) -> Result<(), String> {
    match keyring::Entry::new(SERVICE, account) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("keychain delete failed: {e}")),
        },
        Err(e) => Err(format!("keychain open failed: {e}")),
    }
}

fn keyring_read(account: &str) -> Option<String> {
    let entry = keyring::Entry::new(SERVICE, account).ok()?;
    let pw = entry.get_password().ok()?;
    let pw = pw.trim().to_string();
    if pw.is_empty() {
        None
    } else {
        Some(pw)
    }
}

#[cfg(target_os = "macos")]
fn security_cli_read(account: &str) -> Option<String> {
    let out = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            SERVICE,
            "-a",
            account,
            "-w",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let pw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if pw.is_empty() {
        None
    } else {
        Some(pw)
    }
}
