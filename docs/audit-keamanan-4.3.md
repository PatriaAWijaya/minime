# Audit Keamanan + Impeccable — Minime v2 (Tahap 4.3)

Tanggal: 19 Okt 2026. Cakupan: repo `PatriaAWijaya/minime` @ `bab5b62`
(frontend Svelte + backend Tauri/Rust). Mode: review terfokus (guidance),
bukan gate audit 6 fase penuh.

## Ringkasan

Tidak ditemukan vulnerability. 1 masalah build diperbaiki, 3 hardening note
dicatat. Impeccable: 1 temuan, dinilai false positive.

## Hasil pemeriksaan

### 1. Secret di git — BERSIH ✅
`git grep` pola `sk_|api_key|secret|password|token` di `*.ts/*.rs/*.json/*.svelte`:
hanya false positive (`input_tokens`/`output_tokens` = counter usage di db.rs).
Kredensial user memang dirancang masuk OS Keychain, tidak di kode.

### 2. WebDriver embedded — AMAN ✅
`tauri_plugin_wdio_webdriver::init()` digate `#[cfg(debug_assertions)]`
(main.rs:203) → tidak ikut di build rilis. Server remote-control tidak
terekspos ke user publik.

### 3. IPC commands (16) — AMAN ✅
- `provider_connect`/`provider_disconnect`: `provider_id` di-whitelist via
  `provider_account()` — tidak bisa tulis keychain account sembarangan.
- `agent_chat` → `Command::new(&bin).args([...])`: argumen terpisah, tanpa
  shell → tidak ada command injection. `bin` dari `find_opencode()`
  (env `OPENCODE_BIN` / path yang dikenal).
- `routerku_set_base_url`: validasi skema `http(s)`; hanya konfigurasi milik
  user di aplikasi desktop miliknya sendiri.
- `chat_history`: `limit` di-clamp 1..500.
- `send_notification`: teks polos via API notifikasi OS.
- SQL di `db.rs`: parameterized (`params!`), tanpa string interpolation.

### 4. Capabilities — MINIMAL ✅
Hanya `core:default`, `core:window:default`, `notification:default`.
Tidak ada akses fs/shell/http mentah dari frontend.

### 5. Database — AMAN ✅
SQLite di app data dir (`minime.db`), fallback memory bila tidak bisa dibuka.

### 6. Masalah build — DIPERBAIKI 🔧
`tauri.conf.json` `resources: ["../../skills"]` menunjuk `app/skills/` yang
tidak ada di repo → risiko gagal saat `tauri build`. Dibuatkan direktori +
README penjelasan (commit ini).

## Hardening notes (bukan vulnerability)

1. **CSP null** (`app.security.csp: null`). Umum untuk Tauri, tapi CSP dasar
   dengan `connect-src` untuk Ollama (localhost) + gateway akan menambah
   defense-in-depth. Perlu uji agar tidak memblokir koneksi provider.
2. **Supply chain belum terverifikasi**: `npm audit` diblokir policy registry
   (403); `cargo audit` belum dijalankan. Disarankan sebelum rilis publik.
3. **Impeccable**: 1 temuan `pulsing-dot` pada `.rec-dot` (ChatPanel) —
   dinilai FALSE POSITIVE: dot hanya render saat `recording=true`, berdampingan
   timer live + `aria-live="polite"`. Pulse memang menandai data live.

## Batasan audit ini

- Bukan gate audit 6 fase penuh (recon → adversarial → verifikasi independen).
- Tanpa eksekusi dinamis; tanpa fuzzing input IPC.
- Dependensi (npm/cargo) belum diaudit terhadap vulnerability database.
- Disarankan satu putaran gate audit penuh sebelum rilis publik pertama,
  sesuai rencana Tahap 4.3.
