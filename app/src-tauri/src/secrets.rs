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

/// File fallback TERENKRIPSI: ~/.minime/keys/<account>.enc (mode 600).
/// Enkripsi AES-256-GCM, kunci = SHA256(machine_id + "minime-key-v1").
/// Jauh lebih aman dari plaintext: file dicuri pun tidak bisa dibaca
/// tanpa machine yang sama. Bukan pengganti Keychain (tetap utama),
/// tapi fallback andal untuk binary ad-hoc.
/// File lama .key (plaintext) tetap dibaca untuk migrasi, lalu dihapus.
fn keys_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::Path::new(&h).join(".minime").join("keys")
    })
}

fn machine_key() -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    // Identitas mesin: hostname + username + HOME (stabil per mesin)
    if let Ok(v) = std::env::var("USER") {
        h.update(v.as_bytes());
    }
    if let Ok(v) = std::env::var("HOME") {
        h.update(v.as_bytes());
    }
    // Hostname via perintah (opsional, abaikan bila gagal)
    if let Ok(out) = std::process::Command::new("hostname").output() {
        h.update(&out.stdout);
    }
    h.update(b"minime-key-v1");
    h.finalize().into()
}

fn file_read(account: &str) -> Option<String> {
    // 1. Coba file terenkripsi dulu
    if let Some(k) = file_read_enc(account) {
        return Some(k);
    }
    // 2. Migrasi: file plaintext lama (.key) → baca, enkripsi, hapus
    let p = keys_dir()?.join(format!("{account}.key"));
    let c = std::fs::read_to_string(&p).ok()?;
    let c = c.trim().to_string();
    if c.is_empty() {
        return None;
    }
    // Migrasi otomatis ke format terenkripsi
    let _ = file_write(account, &c);
    let _ = std::fs::remove_file(&p);
    Some(c)
}

fn file_read_enc(account: &str) -> Option<String> {
    use aes_gcm::{Aes256Gcm, Key, KeyInit, aead::{Aead, generic_array::GenericArray}};
    let p = keys_dir()?.join(format!("{account}.enc"));
    let data = std::fs::read(p).ok()?;
    if data.len() < 12 {
        return None;
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(&machine_key());
    let cipher = Aes256Gcm::new(key);
    let nonce = GenericArray::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
    let s = String::from_utf8(plaintext).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn file_write(account: &str, key: &str) -> Result<(), String> {
    use aes_gcm::{Aes256Gcm, Key, KeyInit, aead::{Aead, AeadCore, OsRng, generic_array::GenericArray}};
    let dir = keys_dir().ok_or("HOME tidak ditemukan")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir gagal: {e}"))?;
    let key_bytes = Key::<Aes256Gcm>::from_slice(&machine_key());
    let cipher = Aes256Gcm::new(key_bytes);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, key.as_bytes())
        .map_err(|e| format!("enkripsi gagal: {e}"))?;
    let mut data = nonce.to_vec();
    data.extend_from_slice(&ciphertext);
    let p = dir.join(format!("{account}.enc"));
    std::fs::write(&p, data).map_err(|e| format!("tulis file gagal: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
    }
    // Hapus file plaintext lama bila ada
    let _ = std::fs::remove_file(dir.join(format!("{account}.key")));
    Ok(())
}

/// Remove an API key for `account` from the OS keychain (disconnect).
/// Juga hapus file fallback (terenkripsi maupun plaintext lama).
pub fn delete_api_key(account: &str) -> Result<(), String> {
    if let Some(dir) = keys_dir() {
        let _ = std::fs::remove_file(dir.join(format!("{account}.enc")));
        let _ = std::fs::remove_file(dir.join(format!("{account}.key")));
    }
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
