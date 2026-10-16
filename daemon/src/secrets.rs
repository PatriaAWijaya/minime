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

/// Store an API key into the OS keychain (used by IPC `secrets.set`).
/// Write-only by design: there is no read path exposed to the UI.
pub fn set_api_key(account: &str, key: &str) -> Result<(), String> {
    let entry =
        keyring::Entry::new(SERVICE, account).map_err(|e| format!("keyring unavailable: {e}"))?;
    entry
        .set_password(key.trim())
        .map_err(|e| format!("keyring write failed: {e}"))
}

/// Delete a stored secret (used by IPC `drive.disconnect`).
pub fn delete_api_key(account: &str) -> Result<(), String> {
    let entry =
        keyring::Entry::new(SERVICE, account).map_err(|e| format!("keyring unavailable: {e}"))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        // Hilang = sudah terputus; anggap sukses agar idempotent.
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("keyring delete failed: {e}")),
    }
}
