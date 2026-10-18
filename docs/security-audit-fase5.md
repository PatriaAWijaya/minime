# Security Audit — Fase 5 (18 Okt 2026)

Mode: guidance (terfokus). Cakupan: daemon Rust (IPC, secrets, tools, OAuth) + frontend Svelte.

## Hasil: 2 temuan hardening (severity rendah), 0 vulnerability kritis

### ✅ Lolos
- **SQL injection**: semua query pakai parameter binding (`rusqlite::params!`); tidak ada `format!` untuk SQL.
- **Command injection**: semua `Command::new` pakai `.args([...])`, tanpa shell; input user tidak pernah masuk ke shell string.
- **OAuth Drive**: `state` random (UUID) divalidasi di callback; token di Keychain, tidak diekspos ke UI/IPC.
- **Secrets**: tidak ada hardcoded key di kode; `secrets.has` hanya kembalikan boolean (write-only pattern).
- **Frontend**: tidak ada `innerHTML`/`dangerouslySetInnerHTML`; Svelte auto-escape.
- **Drive upload**: best-effort, kegagalan tidak menggagalkan approval.

### ⚠️ Temuan 1: Unix socket tanpa permission eksplisit (rendah)
`daemon/src/ipc.rs` — `UnixListener::bind(sock)` tanpa `set_permissions(0o600)`.
Socket di `data_dir/minime.sock` mengandalkan umask. User lain di mesin yang sama
secara teori bisa konek dan memanggil IPC (termasuk `secrets.set`).
*Konteks*: single-user desktop app; risiko rendah tapi mudah diperbaiki.
*Rekomendasi*: set `0o600` setelah bind (Unix).

### ⚠️ Temuan 2: `secrets.set` tanpa allowlist akun (rendah)
`daemon/src/ipc.rs:335` — `account` bebas diisi apa saja; tidak ada validasi
terhadap daftar provider yang dikenal (`gemini`, `muse`, `deepseek`, `valyu`, `fred`, `postiz`).
*Konteks*: hanya bisa dieksploitasi bila penyerang sudah punya akses IPC lokal.
*Rekomendasi*: allowlist akun di `secrets.set`.

## Catatan
- Perbaikan butuh approval Patria (rule.md §2.2/§2.3).
- Audit ulang disarankan setelah perbaikan + sebelum rilis publik (Fase 6).
