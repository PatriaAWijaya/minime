# Rekap Lengkap Minime v2 — Semua Fase (0 → Akhir)
> Disusun 20 Okt 2026 11:56 WIB atas perintah Patria. Mencakup semua progres yang SUDAH dan BELUM dikerjakan di setiap tahap, plus rencana pekerjaan berikutnya.
> Gate tiap tahap = review + vonis Patria. Aturan tetap: akhir fase selalu cek konsistensi + performa (tidak boleh ada error lolos).

---

## FASE 0 — Persiapan & Fondasi ✅ SELESAI
### Sudah:
- Protokol build dengan AI (framework @aabiizaa + @ficky.dev, diadopsi 1 Okt 2026): AGENTS.md proyek, status.md, rule.md, TODO.md, PLAN.md
- Rule basic UI untuk semua build (readability, button min 44px, satu primary action, dll.)
- Keputusan stack: Tauri 2 + Svelte 5 (clean-room, pola UI OpenBot — kode 100% sendiri karena lisensi OpenBot PolyForm Noncommercial)
- v1 diarsipkan ke `arsip-v1/` (18 Okt 19:36), v2 dibangun bersih
- Repo GitHub `PatriaAWijaya/minime` PUBLIK (19 Okt ~09:48), push via Git Data API
- Prinsip inti ditetapkan: "compact tapi powerful" + "ringan tapi berperforma"
- Keputusan produk: aplikasi untuk PUBLIK — kredensial user sendiri via OS Keychain, tidak dibundel
- Roster: Mimi (orkestrator) + 5 agen (Kreatif, Riset, Finance, Bisnis Strategis, Code Builder)
### Belum:
- Tidak ada.

---

## TAHAP 1 — UI/UX Klik-Tayang ✅ SELESAI — Gate LOLOS (19 Okt ~10:12)
### Sudah:
- Scaffold Tauri 2 + Svelte 5, layout 3 kolom, sidebar icon-only
- 7 tab: Agents → Tugas → Jadwal → Library → Cost → Tools (+ Pengaturan)
- i18n 3 bahasa: Indonesia (default), Inggris, Mandarin — paritas penuh
- Tema monokrom + aksen biru #4FA6F0, warna pastel per agen, dot status (kuning/hijau/merah)
- Personalisasi agen: nama, judul, instruksi, gender, usia, jabatan, hobi, suara
- Voice command mock di chat agen
- View Tugas: Kanban 5 kolom (Antrian, Dikerjakan, Butuh Approval, Revisi, Selesai) + Koneksi (kanvas Svelte Flow)
- UI Penyedia Model: dropdown OpenCode/Ollama/Premium + tambah model custom
- Top bar: server dropdown, folder tugas, toggle panel
- Badge Tools warna brand asli (keputusan Patria)
### Belum:
- Avatar animasi final — dibuat ulang oleh Patria sendiri (tidak blocking)

---

## TAHAP 2 — Model Router + Runtime Agen 🟡 ~90% — Gate MENUNGGU vonis Patria
### Sudah:
- Router Rust: Ollama (default gratis) → OpenCode (fallback); berbayar: Muse → Gemini → GPT → Claude
- Chat riil via Ollama qwen2.5:3b — TERUJI (19 Okt 16:15)
- Skills engine (playbook disuntik ke system prompt per agen)
- SQLite: riwayat chat + usage metering (tab Cost pakai data nyata)
- System prompt per agen
- Kredensial via OS Keychain (keyring + fallback security CLI)
- IPC 8 command: get_app_info, agent_chat, chat_history, provider_probe, provider_connect, provider_disconnect, usage_summary, routerku_set_base_url
- RuangAPI TERHUBUNG (MCP, 8 tools, plan free)
- Audit model riil (20 Okt): 10 nama model fiktif diganti 8 model riil; fallback fiktif terakhir (glm-5.3-flash) diperbaiki
### Belum:
- Orkestrasi delegasi multi-agen via chat `[DELEGATE:agenId]` (pernah dibangun, ter-rollback) — fondasi UI sudah ada lewat tab Brief (lihat Tahap 2.5)
- Budget cap per agen (peringatan batas pemakaian)
- Form agen custom via UI ("Agen Baru")
- RouterKu: gateway belum diketahui + saldo 0 (menunggu support + top-up — langkah Patria)
- **Gate: uji 1 alur minta → delegasi → hasil + vonis Patria**

---

## TAHAP 2.5 — Alur Kerja Project + Polish UI (tambahan 20 Okt) ✅ SELESAI — dikonfirmasi Patria
### Sudah (semua atas perintah Patria 20 Okt):
- **Alur kerja resmi**: user buat project (Tugas/folder Agen) → brief + to-do + goals di Mimi → Mimi auto-delegasi → popup inisiasi → agen kerja mandiri & lapor ke Library → user pantau di Kanban
- Tab **Brief** di Tugas: form brief/goals, to-do list per agen, tombol Inisiasi + popup konfirmasi
- **Default hanya Mimi aktif** — agen lain aktif bila diaktifkan user atau menerima delegasi
- Menu Tools: tab Terpasang/Jelajahi ala Kanban untuk Agen, Aplikasi, Skill + fitur pasang/copot
- Chat: ikon **paperclip** (lampiran gambar/audio/video/dokumen, maks 5) untuk pengaya skill & analisa project
- Menu aksi per bubble chat: Balas, Reaksi emoji, Salin, Hapus
- Menu pengaturan agen (kolom kanan) semua terhubung: Backup, Memori, Keahlian, Berkas, Tabel, Rutinitas, Penggunaan, Publikasikan, Izin, Lanjutan
- Tema gelap jadi **Dark Glass** (frosted); semua dropdown/menu/popup SOLID agar teks terbaca; layer panel di atas window utama
- Tombol **X** penutup di panel model & dialog setting agen
- Project dummy & folder contoh DIHAPUS — aplikasi mulai kosong, user buat sendiri
### Belum:
- Uji project pertama yang asli (direncanakan Patria setelah semua build selesai — menjadi uji lolos/tidak aplikasi)

---

## TAHAP 3 — Tools & Distribusi 🔴 PERLU REBUILD — Gate MENUNGGU
> Kode pernah selesai 19 Okt 18:24–18:43, lalu HILANG saat rollback 19 Okt 22:26. Harus dibangun ulang satu per satu sambil monitor.
### Pernah selesai (perlu dibangun ulang):
- 3.1 Tools gateway Rust: 12 tool, allowlist per agen
- 3.2 Approval gate: kartu setuju/revisi/tolak + notifikasi native
- 3.3 Meta & Postiz: draft-only (tidak pernah auto-post)
- 3.4 Direct post hanya atas perintah eksplisit user
- 3.5 Pireel: framework koneksi (cloud MCP + OAuth) sebagai lapisan editing video
- 3.6 Google Drive: framework upload deliverable ke folder MIMI
### Belum:
- Seluruh rebuild di atas
- **Gate: 1 pipeline end-to-end teruji + vonis Patria**

---

## TAHAP 4.1 — Logo & Avatar 🔴 PERLU REBUILD
### Sudah (arsip): logo final Patria + 6 avatar 3D hewan (singa=Mimi, rubah=Kreatif, gajah=Riset, anjing=Finance, kucing=Bisnis, monyet=Code Builder) — file tersimpan di riwayat git commit 8525762
### Belum:
- Restore avatar dari git + picker avatar di UI
- Animasi status avatar

## TAHAP 4.2 — Build & Bundle Rilis 🟡 SEBAGIAN
### Sudah:
- Build debug berjalan stabil di MacBook (CPU 0,2–4%, memori 54–120 MB, bundle 50 MB, DMG 13 MB)
- Konfigurasi pernah selesai: publisher "Patria A. Wijaya", id `id.minime.app`, DMG layout, NSIS currentUser (ter-rollback — perlu dipasang ulang)
### Belum:
- Eksekusi build **release** (DMG final + installer Windows .exe via GitHub Actions)
- Code signing + notarization macOS
- Auto-updater

## TAHAP 4.3 — Security Audit 🔴 PERLU REBUILD
### Sudah (arsip): audit pernah selesai — tidak ada vulnerability ditemukan
### Belum:
- Audit ulang pada kode terbaru
- Hardening CSP (masih `null`)

## TAHAP 4.5 — Scoring Kinerja ⏳ BELUM DIMULAI
### Belum:
- Benchmark internal: startup (hyperfine), RAM/CPU idle & aktif, ukuran bundle, latensi IPC, latensi chat
- Lighthouse CI sebagai gate tiap rilis
- Satu kali audit konsultan performa independen sebelum rilis publik
> Catatan: pengukuran kasar sudah berjalan tiap uji konsistensi (CPU/memori/bundle) dan selalu dalam batas sehat.

---

## RENCANA PEKERJAAN BERIKUTNYA (urutan prioritas)
1. **Selesaikan Tahap 2** — orkestrasi delegasi multi-agen (sambungkan tab Brief ke runtime chat), budget cap, form agen custom → **gate vonis Patria**
2. **Rebuild Tahap 3 satu per satu** — tools gateway → approval gate + notifikasi → Meta/Postiz draft-only → Pireel/Drive → **gate pipeline E2E + vonis Patria**
3. **Rebuild Tahap 4** — restore avatar 3D → konfigurasi bundle release → security audit ulang + CSP → eksekusi build release (DMG + .exe)
4. **Tahap 4.5 Scoring kinerja** — benchmark internal + Lighthouse CI + audit independen
5. **Uji lolos final** — Patria membuat project pertama yang asli end-to-end untuk menentukan aplikasi layak dipakai
6. **Fase komersil (sesuai visi awal)** — lisensi trial/berbayar + Stripe, login OTP email, Account API (Cloudflare Workers + D1), kupon admin — dieksekusi setelah uji lolos

---

## STATUS GATE SEKARANG
| Tahap | Status | Menunggu |
|---|---|---|
| Fase 0 | ✅ Selesai | — |
| Tahap 1 | ✅ Gate lolos | — |
| Tahap 2 | 🟡 ~90% | Vonis Patria + delegasi |
| Tahap 2.5 | ✅ Selesai (dikonfirmasi) | — |
| Tahap 3 | 🔴 Perlu rebuild | Rebuild + gate E2E |
| Tahap 4.1–4.3 | 🔴/🟡 Perlu rebuild | Rebuild |
| Tahap 4.5 | ⏳ Belum mulai | Setelah app stabil |
