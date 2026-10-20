# Rekap Akhir — Minime v2 (20 Okt 2026, ±14:50 WIB)

> Rekap detail semua pekerjaan dari awal sampai saat ini, per fase.
> Penanda: ✅ selesai · 🟡 sebagian · ⏳ belum dikerjakan · ⛔ menunggu
> keputusan/aksi Patria · 🔒 ditangguhkan (parkir, disengaja).
> v1 (Fase 1–4 lolos, Fase 5 tidak selesai) diarsipkan di `arsip-v1/`;
> sejak 18 Okt 2026 semua pekerjaan adalah v2 clean-room.

---

## Fase 0 — Fondasi ✅ SELESAI

- v2 clean-room: Tauri 2 + Svelte 5; pola UI ditiru dari OpenBot,
  kode 100% sendiri (lisensi OpenBot = PolyForm Noncommercial).
- Repo `PatriaAWijaya/minime` PUBLIK; house rules di `AGENTS.md` repo.
- Keputusan besar: Minime untuk PUBLIK — koneksi ter-reset untuk user
  umum, tiap user memakai kredensialnya sendiri (OS Keychain);
  kredensial Patria hanya untuk pemakaian pribadinya.
- Framework kerja: protokol build @aabiizaa/@ficky.dev, UI rules
  @suwito.js, anti-pattern Impeccable, MeHarness (lihat bagian khusus).
- Roster: Mimi (orkestrator) + 5 agen (Kreatif, Riset, Finance, Bisnis
  Strategis, Code Builder); user bisa menambah agen.
- Tema: monokrom + aksen biru #4FA6F0, warna agen pastel, tema gelap
  "Dark Glass"; i18n ID (default) / EN / ZH paritas penuh.

## Tahap 1 — UI/UX ✅ SELESAI (Gate LOLOS, vonis Patria 19 Okt ~10:12)

- Layout aplikasi, sidebar server, 7 tab (Agents, Tugas, Jadwal,
  Library, Cost, Tools, Pengaturan), Kanban 5 kolom, kanvas Koneksi.
- Personalisasi agen (gender, usia, jabatan, hobi, suara), panel
  Penyedia Model, dropdown model, tema Sistem/Gelap/Terang.
- ⏳ **Avatar animasi final** — dibuat Patria sendiri (tidak blocking).

## Pengaturan — 10 kategori ✅ SELESAI & FUNGSI NYATA (20 Okt)

Ditulis ulang mengikuti referensi, semua berfungsi (audit 20 Okt):

1. **Umum** — bahasa (+ mode System), 10 warna logo, pintasan kirim
   terhubung ke composer chat, memori per agen, tema.
2. **Notifikasi** — preferensi notifikasi tersimpan & dipakai sistem.
3. **Suara** — pilihan model dikte (Parakeet/Whisper/ElevenLabs),
   API key ElevenLabs tersimpan di OS Keychain (nyata).
   ⚠️ Unduhan model masih simulasi — mesin STT riil = Fase Voice.
4. **Dynamic Island** — jendela island SUNGGUHAN (window terpisah,
   always-on-top, per monitor), posisi atas/bawah, haptik macOS,
   status live dari agen & antrean approval. Terverifikasi visual
   oleh Patria (screenshot 14:24 & 14:30).
5. **Penggunaan Komputer** — status izin macOS NYATA (Perekaman Layar
   & Aksesibilitas via preflight API resmi) + tombol buka panel
   Pengaturan Sistem.
6. **Profile** — nama/foto (unggah + crop), keluar dua langkah yang
   membersihkan identitas lokal, kartu sesi = data perangkat asli
   (command `device_info`). ⚠️ Sesi perangkat lain menunggu akun
   cloud Fase 6 — tidak difabrikasi.
7. **Tagihan** — Tambah server memanggil `addServer()` nyata
   (Cloud/Independen), badge Gratis, seksi Paket Minime.
8. **Koneksi Mobile** — QR ASLI (library `qrcode`, teruji valid),
   kode sekali-pakai 6 digit kedaluwarsa 2 menit + hitung mundur,
   tautan permanen (token persisten, bisa dicabut), salin tautan.
   ⚠️ Redeem di ponsel menunggu aplikasi mobile (Fase 6).
9. **Pembaruan** — cek rilis GitHub NYATA (command `check_update`,
   track Stable/Beta, banding semver teruji unit test), catatan rilis
   asli, mulai ulang nyata (`restart_app`), "Pasang saat santai"
   menolak saat agen bekerja, cek otomatis saat app dibuka.
   Status saat ini: belum ada rilis diterbitkan (API 404) — halaman
   menampilkannya jujur.
10. **Penyedia Model** — blok provider asli dipertahankan utuh
    (koneksi, API key Keychain, probe status).

## Tahap 2 — Model Router ✅ SELESAI (Gate LOLOS, vonis Patria 20 Okt 11:59)

- Router Rust: Ollama, OpenCode, Muse, Gemini, DeepSeek (+ RouterKu,
  RuangAPI); fallback berjenjang. Gratis: Ollama → OpenCode;
  Berbayar: Muse → Gemini → GPT → Claude.
- Chat riil teruji via Ollama `qwen2.5:3b`; skills engine; SQLite
  (riwayat + usage); kredensial di OS Keychain.
- 9 nama model gratis OpenCode TERVERIFIKASI ASLI via katalog
  models.dev (audit 12:58 — bukan fiktif).
- Tab Cost memakai data usage nyata; 5 model Ollama online di MacBook.
- 🟡 **Eksekusi delegasi multi-agen otonom penuh** — alur UI-nya
  selesai (Tahap 2.5); eksekusi runtime otonom menjadi bagian
  Tahap 3.6.
- 🔒 **RouterKu BELUM aktif** — API key tersimpan, tetapi base URL
  gateway tidak diketahui & saldo 0. ⛔ Langkah Patria: tiket ke
  support RouterKu + top-up QRIS.
- ✅ **RuangAPI TERHUBUNG** — MCP resmi, 8 tools, kuota free 150/bln.

## Tahap 2.5 — Alur Kerja + Polish UI ✅ SELESAI

- Alur resmi: project di Tugas/folder Agen → brief + todo + goals di
  Mimi → Mimi menyusun delegasi → popup inisiasi project → agen
  melapor di Library → pantau di Kanban.
- Aturan: agen aktif default HANYA Mimi; agen lain aktif atas
  aktivasi user atau delegasi Mimi.
- `BriefPanel.svelte` + `initiateProject()`; Tools pasang/copot;
  lampiran chat (maks 5 file); menu bubble chat (Balas/Reaksi/Salin/
  Hapus); data dummy dihapus; folder tugas default kosong.

## Tahap 3 — Tools & Distribusi 🟡 SEBAGIAN (3 dari 6)

- ✅ **3.1 Tools Gateway** — registry 12 tool (meta, postiz, pireel,
  drive, slack, discord, telegram, github, notion, canva, ruangapi,
  browser), allowlist per agen, 6 IPC; gate terverifikasi unit test
  Rust (fail-closed: agen tak dikenal = nol akses).
- ✅ **3.2 Approval Gate + Notifikasi** — plugin notifikasi native;
  tombol Setuju/Revisi/Tolak di kartu Kanban kolom approval.
- ✅ **3.3 Meta & Postiz** — `createSocialDraft`, mode per platform
  tersimpan; DEFAULT = draft saja; autoposting hanya bila user
  mengaktifkan sendiri dengan peringatan risiko eksplisit
  (revisi aturan Patria 20 Okt 13:08).
- ⏳ **3.4 Pireel** — integrasi editor video (MCP + OAuth) belum
  dikerjakan.
- ⏳ **3.5 Google Drive folder MIMI** — upload deliverable belum
  dikerjakan.
- ⏳ **3.6 Integrasi Runtime** — agen memanggil tool dari chat,
  hasil → Library, progres → Kanban; belum dikerjakan.
- ⛔ **Gate Tahap 3** — menunggu 3.4–3.6 selesai (pipeline E2E) +
  vonis Patria.

## Tahap 4 — Rilis 🟡 SEBAGIAN

- 🟡 **4.1 Logo + Avatar** — logo/ikon aplikasi selesai; avatar 3D
  agen PERLU REBUILD (file lama di commit `8525762`).
- 🟡 **4.2 Build** — build DEBUG stabil & terverifikasi berkali-kali
  (app 52 MB + DMG 13,7 MB). Build RELEASE ✅ SELESAI 20 Okt sore:
  app 16,14 MB + DMG 6,91 MB (aarch64). Installer Windows `.exe`
  (GH Actions) ⏳ belum.
- ⏳ **4.3 Security Audit** — belum di-rebuild; CSP masih `null`;
  audit keamanan penuh menunggu.
- ⏳ **Installer UX + uji instalasi bersih** (Mac & Windows) — belum.

## Tahap 4.5 — Scoring Kinerja 🟡 BENCHMARK INTERNAL SELESAI

- ✅ Benchmark internal di build RELEASE (20 Okt sore, dokumen
  `docs/SCORING-TAHAP-4.5.md`): startup tampil 0,13–0,15 dtk, siap
  0,2–3,3 dtk; CPU idle 0,3–0,7%; RAM idle 106–115 MB; chat Ollama
  qwen2.5:3b total 2,0–2,3 dtk (21–24 token/dtk); bundel release
  app 16,14 MB, DMG 6,91 MB.
- ⏳ Latensi IPC formal + CPU/RAM saat agen aktif bekerja.
- ⏳ Lighthouse CI sebagai gate tiap rilis.
- ⏳ Audit konsultan performa independen (satu kali, pra-rilis publik).

## MeHarness 🟡 SEBAGIAN

- ✅ Framework diadopsi (loop Propose–Probe–Compose; dokumen
  `docs/meharness.md` + §7 `docs/framework.md`); pilot Code Builder
  LULUS; set uji di `tests/harness/`; pelajaran di
  `docs/meharness-lessons.md`.
- ⏳ Kurasi ECC gelombang 1 (code-reviewer, build-error-resolver,
  e2e-runner, tdd-workflow, dll).
- ⏳ Evaluasi konsep AgentShield untuk Tahap 3/4.

## Fase Voice ⏳ DITANGGUHKAN

- ⏳ Mesin STT riil (Parakeet/Whisper lokal / ElevenLabs Realtime) —
  saat ini pilihan model + penyimpanan key selesai, unduhan model
  tersimulasi, mic chat masih mock.

## Fase 6 — Komersil ⏳ BELUM MULAI

- ⏳ Login OTP email 8 karakter (hash-only); lisensi trial/berbayar
  + Stripe; Account API (Cloudflare Workers + D1); kupon admin-only
  (1 kupon maks 5 user = Pro 1 tahun).
- ⏳ Installer per OS + auto-updater; code signing + notarization.
- ⏳ Aplikasi mobile (iOS/Android) — pasangan Koneksi Mobile.

## Parkir / Menunggu ⛔🔒

- 🔒 WebDriver AFK (uji GUI otomatis) — diparkir: incompatibility
  `tauri-service`; jalur verifikasi visual saat ini = screenshot
  Patria.
- ⛔ Hapus repo kosong `PatriaAWijaya/minime-v2` — manual oleh Patria
  (token GitHub tidak punya scope delete).
- ⛔ Tinjau ulang SEMUA tahap yang selesai sebelum launching
  (keputusan Patria 20 Okt 11:59) — terjadwal di akhir.
- ⛔ Uji final: Patria membuat project asli pertama end-to-end —
  setelah Gate Tahap 3 & 4.

## Audit Penuh Terakhir — 20 Okt ±14:55 ✅ LULUS

- cargo check 0 error 0 warning · cargo test 5/5 · vite build lolos
- IPC 24 command terdaftar / 23 dipakai frontend / 0 yatim
  (`agent_send_stub` memang backend-only)
- i18n 672 kunci × 3 bahasa, paritas penuh
- TODO/FIXME 0 · console.log 0 · dbg! 0 · git bersih, HEAD = remote

---

## Rencana Pengerjaan Berikutnya (urutan)

1. ✅ **Build release selesai** (20 Okt sore) — app 16,14 MB,
   DMG 6,91 MB; benchmark internal Tahap 4.5 sudah terukur
   (`docs/SCORING-TAHAP-4.5.md`).
2. **Tahap 3.4 Pireel** → **3.5 Drive folder MIMI** → **3.6 Integrasi
   Runtime** → uji pipeline E2E → **Gate Tahap 3 (vonis Patria)**.
3. **Rebuild Tahap 4**: avatar 3D (4.1) → security audit + CSP (4.3)
   → installer Windows `.exe` via GH Actions → uji instalasi bersih.
4. **Eksekusi delegasi multi-agen otonom** (sisa Tahap 2) sebagai
   bagian 3.6 — chat → delegasi → hasil, tanpa campur tangan manual.
5. **Tahap 4.5 scoring penuh** (benchmark internal ✅; tersisa
   latensi IPC formal, Lighthouse CI, audit independen).
6. **Tinjau ulang semua tahap** (keputusan 11:59) → **uji project
   asli pertama oleh Patria** → vonis launching.
7. **Fase 6 komersil** (OTP, lisensi/Stripe, Account API, kupon,
   auto-updater, signing) → **Fase Voice** (STT riil) → aplikasi
   mobile.

**Bergantung pada Patria (tidak bisa kukerjakan sendiri):**
vonis Gate Tahap 3 · tiket support + top-up RouterKu · avatar
animasi final · hapus repo `minime-v2` · uji project asli pertama.
