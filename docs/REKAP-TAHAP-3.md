# REKAP DETAIL TAHAP 3 — Tools & Distribusi

> Disusun 21 Okt 2026. Status: kode selesai, gate menunggu vonis Patria.

## Ringkasan

Tahap 3 membangun lapisan tools Minime: registry 12 tool, approval gate,
draft-only Meta/Postiz, koneksi OAuth Pireel & Google Drive, dan runtime
integrasi yang memungkinkan agen memanggil tool nyata dengan gate kejujuran.

---

## 3.1 Tools Gateway ✅ SELESAI

**Yang dibangun:**
- Registry 12 tool: Meta, Postiz, Pireel, Google Drive, Slack, Discord,
  Telegram, GitHub, Notion, Canva, RuangAPI, Browser.
- 6 Tauri command: `list_tools`, `get_tool_status`, `set_tool_connected`,
  `get_agent_tools`, `set_agent_tools`, `agent_can_use_tool`.
- Allowlist per agen (diatur di menu Izin agen).
- ToolsView menampilkan status koneksi nyata dari backend.

**Verifikasi:** IPC cocok, tidak ada gimmick.

---

## 3.2 Approval Gate + Notifikasi Native ✅ SELESAI

**Yang dibangun:**
- Kolom "Butuh Approval" di Kanban.
- Keputusan: Setuju → Selesai, Revisi → Revisi, Tolak → Antrian.
- Notifikasi native macOS saat kartu butuh approval atau selesai.
- Permission `notification` di capabilities.

**Verifikasi:** alur keputusan berfungsi, notifikasi terkirim.

---

## 3.3 Meta & Postiz — Draft-Only ✅ SELESAI (dengan catatan)

**Yang dibangun:**
- `createSocialDraft()`: hasil agen menjadi DRAF di kolom Butuh Approval.
- Dialog Publikasikan: pilih platform (Meta/Postiz), judul, isi.
- Mode per platform: "Draft saja" (default) atau "Posting otomatis".
- **Consent gate** (21 Okt): autopost butuh persetujuan eksplisit pertama
  kali — dialog menampilkan 3 konsekuensi, user harus centang "Saya
  mengerti risikonya", baru tombol Aktifkan bisa ditekan. Persetujuan
  tersimpan per platform.

**Verifikasi kejujuran:**
- TERBUKTI tidak ada HTTP call ke Meta Graph API atau Postiz API di
  seluruh codebase. Draft-only terjamin struktural.
- Catatan: mode autopost saat ini menandai kartu sebagai "autopost" tanpa
  benar-benar memposting (integrasi posting API belum ada). UI sudah
  diperbaiki agar tidak menyesatkan.

---

## 3.4 Pireel ✅ SELESAI & TERHUBUNG

**Yang dibangun:**
- OAuth 2.1 PKCE melawan endpoint resmi `pireel.com` (terverifikasi hidup).
- Alur jujur: tukar kode → simpan token → BARU tampil sukses. Gagal =
  pesan error tampil di UI + tombol "Coba lagi".
- Token di Keychain (dengan fallback `security` CLI untuk binary
  ad-hoc-signed).
- `pireel_status`: panggil API asli `get_state`, klasifikasikan
  (live/offline/fresh).
- `pireel_create_project`: buat project via MCP + tombol "Buat project
  dummy" di kartu Pireel.

**Verifikasi:**
- Token ADA di Keychain (terverifikasi via `security`).
- API merespons valid; badge hijau "✓✓ Terhubung".
- Status "mode offline" = tidak ada sesi cloud aktif (wajar).

**Perbaikan penting (20–21 Okt):**
- 20 Okt: halaman sukses prematur → diperbaiki (tukar dulu).
- 21 Okt: keychain write dari thread mentah tidak persisten → dipindah ke
  thread Tauri + fallback `security` CLI.

---

## 3.5 Google Drive ✅ KODE SELESAI (menunggu Patria)

**Yang dibangun:**
- OAuth flow sama jujurnya dengan Pireel (client ID milik user).
- `drive_upload`: validasi → token → folder MIMI → multipart upload →
  file ID asli.
- Scope `drive.file` (hanya file yang dibuat aplikasi).

**Batasan jujur:** BELUM terhubung. Butuh OAuth Client ID tipe Desktop app
dari Google Cloud Console oleh Patria, lalu login di aplikasi.

---

## 3.6 Integrasi Runtime ✅ SELESAI

**Yang dibangun:**
- `tool_run` dengan 3 gate jujur:
  1. Allowlist → `not_allowed`
  2. Koneksi dicek ke Keychain (bukan status UI) → `not_connected`
  3. Tanpa implementasi → `tool_not_implemented`
- 3 dispatch arm nyata: `browser/fetch`, `pireel/create_project`,
  `drive/upload`.
- Kartu Kanban "Jalankan tool" → hasil ke Library → Butuh Approval.

**Verifikasi:** 32/32 tes Rust lolos, termasuk fetch internet betulan.

---

## Kriteria Gate

| Kriteria | Status |
|---|---|
| Draft-only terbukti | ✅ Struktural (tidak ada kode posting) |
| Pipeline E2E | 🟡 Komponen teruji; alur penuh belum uji UI |
| Konsistensi + performa | ✅ Audit #3 LOLOS |
| Vonis Patria | ⏳ Menunggu |

## Rekomendasi: LOLOS BERSYARAT

Syarat: uji E2E penuh via UI setelah vonis. Tidak ada gimmick di Tahap 3.
