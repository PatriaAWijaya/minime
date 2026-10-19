# Analisis Bug Blinking + CPU 117% — Minime v2

**Tanggal:** 19 Okt 2026, 22:30 WIB
**Status:** BELUM TERATASI — dokumentasi untuk perbaikan lanjutan

---

## 1. Timeline

| Waktu | Kejadian |
|-------|----------|
| 19 Okt 16:15 | ✅ **TERAKHIR BERJALAN BAIK** — Chat via Ollama qwen2.5:3b BERHASIL (screenshot Patria) |
| 19 Okt 16:22 | Commit d94a87e: hapus debug eprintln (aman) |
| 19 Okt 18:19–18:43 | **Tahap 3 commits** — tools gateway, notifikasi, allowlist, orkestrasi, hapus dummy |
| 19 Okt 19:27–19:36 | Test WebDriver rewrite, notification permission fix |
| 19 Okt 19:50 | Commit 8525762: **avatar agen 3D + picker** |
| 19 Okt 19:52 | Commit b4e9b22: bundle config (dmg/nsis metadata) |
| 19 Okt 19:54 | Commit 0d5d89d: audit docs + skills/README.md |
| 19 Okt ~20:07 | ❌ **BLINKING DILAPORKAN** — jendela berkedip, tab tidak bisa dibuka |
| 19 Okt 20:09 | Commit 615c662: pill model reaktif (setelah blinking) |
| 19 Okt 20:28 | Commit f1ea693: beforeBuildCommand fix |
| 19 Okt 20:30 | Screenshot: avatar tampil BENAR, tapi blinking tetap |
| 19 Okt 20:48 | Activity Monitor: **117% CPU**, 29 threads |
| 19 Okt 21:35 | Fix avatar via Vite import (d651a06) — TIDAK MEMPERBAIKI |
| 19 Okt 22:01 | Sesi dihentikan atas perintah Patria |

---

## 2. Perubahan Kode Antara "Baik" (16:15) → "Rusak" (20:07)

### 2a. Aman (docs only)
- fa83d88, 86527f3, fb0604d, e142970, 0d5d89d (docs)
- 44b8362, 4880fa9 (rename Mimi→Muse di docs)
- d94a87e (hapus eprintln debug)

### 2b. Risiko RENDAH
- **d73b841**: hapus data dummy + skenario project — hanya hapus mock data
- **c983604**: hapus dummy library/jadwal/skill/template/cost — hanya hapus mock data
- **b56cd1e**: sederhanakan capabilities notification — JSON config
- **b4e9b22**: bundle metadata (publisher, dmg layout, nsis) — tidak affect runtime
- **34e21d9, 7d4d373**: test files — tidak ikut build produksi

### 2c. Risiko SEDANG — Kandidat Penyebab
- **b34e4f2** (18:24): Tools gateway — tambah `tools.rs` (176 baris), 6 IPC command baru, `ToolRegistry` dengan Mutex<HashMap>. *Catatan: kode terlihat bersih, tapi ini perubahan Rust terbesar.*
- **4ddc817** (18:25): Notifikasi native — tambah `tauri-plugin-notification`, 1 IPC command `send_notification`. *Catatan: plugin baru = surface area baru.*
- **186d9ae** (18:26): Allowlist + draft-only + Pireel — 68 baris store.ts
- **28674c5** (18:27): Framework Drive — 20 baris store.ts
- **4c0c1f6** (18:20): Orkestrasi via delegasi chat — logika delegasi `[DELEGATE:agenId]`
- **8525762** (19:50): Avatar 3D + picker — 215 baris AgentsView.svelte, snippet `avatarDisplay`, CSS animasi `avatar-bob`/`avatar-shake`, store `agentAvatarOverrides` + localStorage

---

## 3. Hipotesis yang Sudah Diuji & Dikesampingkan

| # | Hipotesis | Hasil |
|---|-----------|-------|
| 1 | Plugin WebDriver menyebabkan loop | ❌ Dikesampingkan — plugin ada sejak 13:32, app normal 16:15 |
| 2 | `refreshProviders()` probe Ollama spam IPC | ❌ Dikesampingkan — test dengan probe dimatikan tetap 118% CPU |
| 3 | Avatar `<img src="/avatars/...">` picu loop URL scheme | ❌ Dikesampingkan — fix via Vite import tidak memperbaiki |
| 4 | PNG corrupt | ❌ Dikesampingkan — semua 6 PNG valid (256×256 RGBA) |
| 5 | CSS animation `avatar-bob` picu reflow loop | ❌ Dikesampingkan — tidak ada loop di kode animasi |
| 6 | `setInterval`/polling di frontend | ❌ Dikesampingkan — tidak ada setInterval di kode |
| 7 | localStorage corrupt | Belum diuji |
| 8 | `chatAgent = $derived({...})` buat object baru tiap render | Belum dikonfirmasi — tapi ChatPanel effect punya guard |

---

## 4. Bukti Kuat dari Sample Proses

Dua sample Activity Monitor (PID 2196 & 2876):
- **65–69% CPU** di `WebKit::WebPageProxy::startURLSchemeTask`
- → `WebURLSchemeHandlerCocoa::platformStartTask`
- → Tauri `url_scheme_handler::start_task` (lib.rs)
- → `create_webview` → `prepare_pending_webview`
- → Tauri membuat HTTP Request/Response objects

**Interpretasi:** WebView membanjiri Tauri dengan request URL scheme. Di Tauri 2, `invoke()` dari JS memakai `fetch()` via custom URL scheme — jadi ini kemungkinan **IPC spam** (JS memanggil `invoke()` dalam loop), BUKAN asset loading.

**Yang aneh:** `prepare_pending_webview` dipanggil dari URL scheme handler — ini tidak normal untuk serving asset biasa.

---

## 5. Kemungkinan Penyebab (Belum Teruji)

### A. IPC Loop dari Frontend
Sesuatu di JS memanggil `invoke()` berulang-ulang. Kandidat:
- `isTauri()` → `invoke("get_app_info")` — tapi ada cache `tauriAvailable`
- `loadChatHistory()` → `invoke("chat_history")` — dipanggil dari ChatPanel onMount + $effect
- `chatAgent = $derived({ ...agent, name: displayName })` — object baru tiap derivasi bisa picu re-render ChatPanel berulang

### B. Bug di Tauri 2 Asset Protocol
`prepare_pending_webview` dari scheme handler mengindikasikan Tauri salah menangani request. Mungkin terkait:
- Path `frontendDist: "../src/dist"` di bundled app
- Interaksi `tauri-plugin-notification` atau `tauri-plugin-wdio-webdriver` dengan scheme handler
- Bug Tauri 2.x di macOS 26.6.2 (Tahoe)

### C. Data Corrupt
- localStorage `minime.avatarOverrides` atau state lain corrupt
- SQLite `minime.db` corrupt → query hang → retry loop

### D. Kombinasi
Tahap 3 (Rust) + Avatar (frontend) berinteraksi — misalnya ToolsView memanggil `list_tools` dalam loop saat registry kosong.

---

## 6. Langkah Perbaikan yang Disarankan

1. **Logging Rust**: Tambah `eprintln!` di setiap `#[tauri::command]` untuk identifikasi command yang di-spam. Jalankan dari terminal, lihat output.
2. **Web Inspector**: Buka devtools di debug build (klik kanan → Inspect), cek tab Network untuk URL yang di-request berulang.
3. **Bisect**: Revert ke commit sebelum Tahap 3 (sebelum b34e4f2), test apakah masih blinking. Jika tidak, re-apply satu per satu.
4. **Minimal repro**: Comment-out seluruh `<ChatPanel>` dari AgentsView, rebuild, test. Jika CPU normal, masalah di ChatPanel.
5. **Clear data**: Hapus `~/Library/WebKit/id.minime.app/`, `~/Library/Application Support/id.minime.app/`, dan localStorage.
6. **Cek Tauri issues**: Cari `prepare_pending_webview` + `url_scheme_handler` di GitHub tauri-apps/tauri.

---

## 7. Catatan Penting

- Patria sempat test dengan **build basi** (/Applications/Minime.app dari 20:22, sebelum fix avatar). Selalu pastikan test dengan build terbaru dari `target/`.
- Ada 3 icon Minime di Launchpad (kemungkinan cache) — hanya 1 app di /Applications.
- SSH ke MacBook DOWN sejak 19 Okt 18:17 — semua test harus manual oleh Patria.
- MacBook sempat overheat karena CPU 117% — jangan biarkan app jalan lama saat test.
