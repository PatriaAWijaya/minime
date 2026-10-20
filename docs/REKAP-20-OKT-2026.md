# Rekap Tugas Minime v2 — 18–20 Okt 2026

> Disusun 20 Okt 2026 atas perintah Patria: "rekap detail semua tugas yang sudah dan belum kita kerjakan dari awal sampai saat ini disemua fase, serta rencana pengerjaan berikutnya."

---

## TAHAP 1 — UI/UX Klik-Tayang ✅ SELESAI

**Gate: LOLOS (vonis Patria 19 Okt ~10:12)**

### Sudah dikerjakan:
- 1.1 Scaffold: Tauri 2 + Svelte 5, struktur `app/`, konfigurasi dasar
- 1.2 Layout: sidebar icon-only, 7 tab (Agents→Tugas→Jadwal→Library→Cost→Tools), layout 3 kolom Agents
- 1.3 i18n: Indonesia (default), Inggris, Mandarin — paritas penuh
- 1.4 Tema: monokrom + aksen biru #4FA6F0, warna pastel per agen, 3 mode (Sistem/Gelap/Terang)
- 1.5 Personalisasi agen: gender, usia, jabatan, hobi, suara
- 1.6 Voice command mock di chat agen
- 1.7 View Tugas: Kanban 5 kolom + Koneksi (Svelte Flow canvas)
- 1.8 Penyedia Model: UI ala OpenBot, dropdown Ollama/OpenCode/Premium + model custom
- 1.9 Top bar: server dropdown, folder tugas, toggle panel agen

### Belum:
- Tidak ada — Tahap 1 selesai penuh.

---

## TAHAP 2 — Model Router + Skills 🟡 ~90%

**Gate: MENUNGGU vonis Patria**

### Sudah dikerjakan:
- 2.1 Model router (Rust): Ollama (default gratis), OpenCode (fallback), Muse/Gemini/DeepSeek (berbayar)
- 2.2 Agent runtime: 6 agen (Mimi + 5), chat real via Ollama qwen2.5:3b ✅ TERUJI 19 Okt 16:15
- 2.3 Skills engine: struktur skill per agen
- 2.4 SQLite: riwayat chat, usage metering, konfigurasi
- 2.5 System prompt per agen
- 2.6 Keychain: kredensial via OS Keychain (tidak di git)

### Belum:
- Orkestrasi delegasi multi-agen via chat `[DELEGATE:agenId]` (pernah dibangun, ter-rollback)
- Budget cap per agen (peringatan batas)
- Agen custom via UI (form "Agen Baru")
- **Gate: uji 1 alur minta → delegasi → hasil + vonis Patria**

---

## TAHAP 3 — Tools & Distribusi 🔴 PERLU REBUILD

**Status: Kode dihapus saat rollback 19 Okt 22:26. Perlu dibangun ulang satu per satu.**

### Pernah selesai (19 Okt 18:24–18:43), ter-rollback:
- 3.1 Tools gateway: Rust `tools.rs`, 12 tool, allowlist per agen
- 3.2 Meta draft-only: `createSocialDraft()`, tidak pernah auto-post
- 3.3 Postiz: penjadwalan sebagai draft
- 3.4 Pireel: framework koneksi cloud MCP + OAuth
- 3.5 Approval gate: kartu approval (setuju/revisi/tolak) + notifikasi native
- 3.6 Drive: framework upload ke folder MIMI
- 3.7 Direct post: `postSocialDraftDirect()` + tombol "Post langsung" (hanya atas perintah eksplisit Patria)

### Belum (setelah rebuild):
- Seluruh 3.1–3.7 perlu dibangun ulang bertahap sambil monitor
- **Gate: pipeline E2E jalan + vonis Patria**

---

## TAHAP 4 — Rilis 🔴 PERLU REBUILD SEBAGIAN

### 4.1 Logo + Avatar 🔴 Perlu rebuild
- Pernah selesai 19 Okt 19:46–20:00, ter-rollback
- Logo app dari Patria (mark "me" + wordmark "minime")
- 6 avatar 3D dari Patria (singa/rubah/gajah/anjing/kucing/monyet)
- Picker avatar + animasi status
- File avatar tersimpan di riwayat git (commit 8525762)

### 4.2 Build DMG + EXE 🟡 Config pernah ada, belum eksekusi
- Config pernah selesai (publisher, category, DMG layout, NSIS), ter-rollback
- Build debug BERHASIL di MacBook (50MB app, 13MB DMG)
- Build release BELUM dieksekusi

### 4.3 Security Audit + Impeccable 🔴 Perlu rebuild
- Pernah selesai 19 Okt ~20:05, ter-rollback
- Hasil: tidak ada vulnerability; CSP masih null (perlu hardening)

### 4.5 Scoring Kinerja ⏳ Belum dimulai
- Benchmark internal (startup, RAM/CPU, bundle, latensi IPC)
- Lighthouse CI gate tiap rilis
- Satu kali audit konsultan independen sebelum rilis publik

---

## BUG FIXES — 20 Okt Pagi ✅ SEMUA SELESAI

### Bug 1: CPU 117% + Blinking ✅ SELESAI (05:35)
- Penyebab: corrupted state/cache di MacBook (bukan kode)
- Solusi: pembersihan total MacBook + rebuild fresh
- Hasil: CPU normal 0.2–2.9%

### Bug 2: View Koneksi tidak tampil ✅ SELESAI (08:05)
- Error: `TypeError: {} is not iterable`
- Akar: Svelte 5 `nodes={sfNodes}` kirim store object ke SvelteFlow (bukan array)
- Fix: `bind:nodes={$sfNodes} bind:edges={$sfEdges}`
- Bonus: 4 fix defensif localStorage (Array.isArray + self-heal)

---

## RENCANA PENGERJAAN BERIKUTNYA

### Prioritas 1: Rebuild Tahap 3 (Tools & Distribusi)
1. Tools gateway Rust (12 tool + allowlist)
2. Approval gate + notifikasi native
3. Meta/Postiz draft-only
4. Pireel + Drive framework
5. **Gate: pipeline E2E + vonis Patria**

### Prioritas 2: Rebuild Tahap 4
1. Avatar 3D + picker (file sudah ada di git history)
2. Bundle config (DMG + NSIS)
3. Security audit ulang
4. Eksekusi build release

### Prioritas 3: Tahap 4.5 Scoring Kinerja
1. Benchmark internal
2. Lighthouse CI setup
3. Audit konsultan independen

### Prioritas 4: Selesaikan Tahap 2
1. Orkestrasi delegasi multi-agen
2. Budget cap
3. Agen custom UI
4. **Gate: vonis Patria**

---

## CATATAN PENTING
- Aturan: setiap akhir fase cek konsistensi + error (Patria 20 Okt 05:28)
- Prinsip: compact tapi powerful, ringan tapi berperforma
- Kredensial: OS Keychain, tidak di git
- Meta: draft-only sampai Patria perintahkan post
- Repo: PatriaAWijaya/minime, publik, push via Git Data API
