# Google Drive OAuth — Minime (read-only, sekali hubung)

> Scope: `https://www.googleapis.com/auth/drive.readonly` — aplikasi hanya BISA MEMBACA Drive, tidak bisa menulis/menghapus. Token disimpan di OS Keychain (write-only), tidak pernah di-log atau dikembalikan via IPC.

## Langkah sekali setup (dilakukan Patria, di MacBook)

### 1. Buat OAuth client di Google Cloud Console
1. Buka https://console.cloud.google.com/ → pilih/buat project (mis. "Minime").
2. **APIs & Services → Library** → aktifkan **Google Drive API**.
3. **APIs & Services → OAuth consent screen** → User type **External** → isi nama aplikasi "Minime" → **Test users**: tambahkan email Patria (selama status Testing, hanya test user yang bisa approve).
4. **APIs & Services → Credentials → Create Credentials → OAuth client ID** → Application type **Desktop app** → Create.
5. Catat **Client ID** (dan **Client secret** bila ditampilkan).

### 2. Simpan kredensial ke Keychain Minime
Via IPC (atau nanti via UI Settings → Providers):
```json
{"id": 1, "method": "secrets.set", "params": {"account": "drive-client-id", "value": "<CLIENT_ID>"}}
{"id": 2, "method": "secrets.set", "params": {"account": "drive-client-secret", "value": "<CLIENT_SECRET>"}}
```
Cek: `{"method": "secrets.has", "params": {"account": "drive-client-id"}}` → `{"has": true}` (nilai tidak pernah ditampilkan).

### 3. Hubungkan
1. Panggil IPC: `{"method": "drive.auth_url"}` → dapat `{"url": "https://accounts.google.com/...", "expires_in": 600}`.
2. Buka URL di browser → login akun Google Patria → **Allow** akses read-only Drive.
3. Browser redirect ke `http://127.0.0.1:<port>/callback` → halaman "Minime terhubung ke Google Drive ✅".
4. Daemon menukar code → refresh token tersimpan di Keychain; UI menerima event `minime://drive-auth` dengan `status: "connected"`.

### 4. Verifikasi
- `{"method": "drive.status"}` → `{"connected": true, "scope": "https://www.googleapis.com/auth/drive.readonly", ...}`.
- `{"method": "drive.disconnect"}` → token dihapus dari Keychain (`{"disconnected": true}`).

## Catatan teknis
- Flow memakai **loopback redirect** (`127.0.0.1:<port acak>/callback`) + token `state` anti-CSRF; tidak ada server publik.
- Timeout 10 menit menunggu approve; setelah itu URL kedaluwarsa — panggil `drive.auth_url` lagi.
- Access token di-refresh otomatis oleh daemon saat kedaluwarsa (`drive_oauth::access_token()`); refresh token tidak pernah keluar dari Keychain.
- Bila Google menampilkan warning "unverified app" (wajar untuk OAuth client baru berstatus Testing): klik **Advanced → Go to Minime (unsafe)** — hanya untuk client milik sendiri.
- Wiring tool Drive (baca folder MIMI untuk upload deliverable) masuk Fase 4/5; fondasi auth selesai di Fase 3 ini.

## Batasan uji di Linux (lingkungan build saat ini)
Kode terkompilasi (`cargo build`) dan alur logika terverifikasi; **flow browser penuh (langkah 3) hanya bisa diuji di MacBook Patria** karena butuh browser + Keychain macOS.
