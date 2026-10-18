//! Secret resolution: macOS Keychain (crate `keyring`) with env-var override
//! for dev/testing on Linux. NEVER hardcode, commit, or log keys.
//!
//! Lookup order per provider `account`:
//!   1. `MINIME_<ACCOUNT>_KEY` env var (explicit dev override; e.g. MINIME_GEMINI_KEY)
//!   2. OS keychain entry (service `com.minime.app`, account name)
//!   3. None → provider reports "no key" and routing falls through to the next
//!      provider in the agent's fallback chain.

const SERVICE: &str = "com.minime.app";

/// Resolve an API key for `account` (e.g. "gemini", "muse", "deepseek").
pub fn api_key(account: &str) -> Option<String> {
    // 1. Explicit dev override.
    let env_name = format!("MINIME_{}_KEY", account.to_uppercase().replace('-', "_"));
    if let Ok(v) = std::env::var(&env_name) {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    // 2. OS keychain (macOS Keychain / Linux Secret Service / Windows Credential Manager).
    if let Some(k) = keyring_read(account) {
        return Some(k);
    }
    // 3. macOS fallback: Apple's `security` CLI. The keyring crate's
    // SecKeychain* calls fail closed (NoEntry on read, silent drop on write)
    // for our ad-hoc-signed binary, while `security` works fine
    // (Gate 4 finding, 18 Okt 2026).
    #[cfg(target_os = "macos")]
    if let Some(k) = security_cli_read(account) {
        return Some(k);
    }
    None
}

/// Read via the keyring crate; None on any failure.
fn keyring_read(account: &str) -> Option<String> {
    match keyring::Entry::new(SERVICE, account) {
        Ok(entry) => match entry.get_password() {
            Ok(pw) => {
                let pw = pw.trim().to_string();
                if pw.is_empty() {
                    None
                } else {
                    Some(pw)
                }
            }
            Err(_) => None,
        },
        Err(_) => None,
    }
}

/// macOS only: read via `security find-generic-password` (Apple-signed, trusted).
#[cfg(target_os = "macos")]
fn security_cli_read(account: &str) -> Option<String> {
    let out = std::process::Command::new("security")
        .args(["find-generic-password", "-s", SERVICE, "-a", account, "-w"])
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

/// Store an API key into the OS keychain (used by IPC `secrets.set`).
/// Write-only by design: there is no read path exposed to the UI.
pub fn set_api_key(account: &str, key: &str) -> Result<(), String> {
    let key = key.trim();
    #[cfg(target_os = "macos")]
    {
        // Prefer the `security` CLI: keyring's SecKeychainAddGenericPassword
        // silently drops writes for ad-hoc-signed binaries (Gate 4, 18 Okt 2026).
        return security_cli_write(account, key);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|e| format!("keyring unavailable: {e}"))?;
        entry
            .set_password(key)
            .map_err(|e| format!("keyring write failed: {e}"))
    }
}

/// macOS only: write via `security add-generic-password -U` (update if exists).
#[cfg(target_os = "macos")]
fn security_cli_write(account: &str, key: &str) -> Result<(), String> {
    let status = std::process::Command::new("security")
        .args([
            "add-generic-password",
            "-U",
            "-s",
            SERVICE,
            "-a",
            account,
            "-w",
            key,
        ])
        .status()
        .map_err(|e| format!("security CLI unavailable: {e}"))?;
    if !status.success() {
        return Err("security add-generic-password failed".to_string());
    }
    Ok(())
}

/// Delete a stored secret (used by IPC `drive.disconnect`).
pub fn delete_api_key(account: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("security")
            .args(["delete-generic-password", "-s", SERVICE, "-a", account])
            .status()
            .map_err(|e| format!("security CLI unavailable: {e}"))?;
        // Hilang = sudah terputus; anggap sukses agar idempotent.
        if status.success() || security_cli_read(account).is_none() {
            return Ok(());
        }
        return Err("security delete-generic-password failed".to_string());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|e| format!("keyring unavailable: {e}"))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            // Hilang = sudah terputus; anggap sukses agar idempotent.
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("keyring delete failed: {e}")),
        }
    }
}
