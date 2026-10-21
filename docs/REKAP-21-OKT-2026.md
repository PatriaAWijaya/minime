# REKAP LENGKAP — Minime v2 (18 Okt – 21 Okt 2026)

> Disusun 21 Okt 2026 ~13:45 atas perintah Patria.
> 303 commit. Sumber: `docs/status.md` (88+ entri), `docs/TODO.md`, `docs/REKAP-MENYELURUH.md`.

---

## BAGIAN 1: PEKERJAAN YANG SUDAH SELESAI

### Fase 0 — Fondasi ✅ (18 Okt)
- Keputusan stack: **Tauri 2 + Svelte 5**, clean-room dari pola UI OpenBot (kode 100% sendiri)
- Repo publik `PatriaAWijaya/minime`, struktur `app/` + `docs/`
- Aturan keras: kredensial di OS Keychain, tidak di git, no auto-posting Meta, no live-trading
- House rules: `AGENTS.md`, `docs/rule.md` (hanya diubah atas persetujuan Patria)

### Tahap 1 — UI/UX ✅ GATE LOLOS (vonis Patria 19 Okt ~10:12)
- Layout 4 kolom (sidebar | agen | chat+delivery | monitor)
- 6 agen seed (Mimi + Kreatif, Riset, Finance, Bisnis Strategis, Code Builder)
- 3 bahasa (ID/EN/ZH), 761 kunci i18n
- Tema monokrom + aksen biru #4FA6F0; warna pastel per agen; dot status terkunci
- Kanban 5 kolom (Antrian, Dikerjakan, Butuh Approval, Revisi, Selesai)
- Koneksi (kanvas Svelte Flow ala n8n), 10 kategori Pengaturan fungsional
- 30 skills di Marketplace tab Tools
- Topbar mobile 2 baris, badge Tools warna brand asli

### Tahap 2 — Model Router ✅ GATE LOLOS (vonis Patria 20 Okt 11:59)
- Provider: **Ollama** (default gratis) → **OpenCode** fallback; berbayar: **Muse → Gemini → DeepSeek**
- Keychain + fallback `security` CLI; SQLite riwayat + usage metering
- 9 nama model gratis terverifikasi asli via models.dev
- Router per-agen dengan fallback otomatis; Tauri commands: `provider_probe`, `agent_chat`, `provider_connect/disconnect`
- System prompt per agen + skills engine

### Tahap 2.5 — Alur Kerja + Polish ✅
- Model agen aktif (tambah via '+', nonaktif → hilang dari list tapi tetap di Tools & Tugas)
- Folder = project; Mimi sebagai parent kanvas Koneksi otomatis
- Satu agen boleh di banyak folder, tidak 2x di folder sama

### Tahap 3 — Tools & Distribusi ✅ GATE LOLOS BERSYARAT (vonis Patria 21 Okt ~08:10)
- **3.1 Tools Gateway**: 12 tool registry, 6 command, allowlist per agen
- **3.2 Approval Gate**: Setuju → Selesai, Revisi → Revisi, Tolak → Antrian + notifikasi native
- **3.3 Meta/Postiz draft-only TERBUKTI struktural** (tidak ada HTTP call ke Meta/Postiz di codebase) + consent gate autopost (dialog 3 konsekuensi + checkbox wajib)
- **3.4 Pireel TERHUBUNG nyata**: 5 command, OAuth jujur, token di Keychain, 20 kredit/hari (akun Free)
- **3.5 Google Drive TERHUBUNG** (21 Okt ~12:55): OAuth end-to-end selesai, token valid, uji upload file berhasil
- **3.6 Runtime**: 3 dispatch arm nyata (browser/fetch, pireel/create_project, drive/upload) + 3 gate jujur

### Tahap 4 — Rilis (sebagian besar selesai)
- Security audit bersih, CSP ketat, release bundle 16MB self-contained
- 6x audit penuh LOLOS (terakhir #6, 21 Okt ~10:35): cargo check 0/0, test 32/32, vite build ✅, gimmick scan 0, IPC 35-37 cocok
- Avatar 3D 6 hewan terpasang (Mimi=kucing, Kreatif=rubah, Riset=gajah, Finance=anjing, Bisnis Strategis=singa, Code Builder=monyet)
- Logo sidebar diganti ke ikon biru "me"; app icon diganti ke biru "me"
- Tombol ganti avatar diperbaiki (picker 6 hewan berfungsi)
- Benchmark internal: startup 0,13–0,15 dtk, CPU idle 0–0,7%, RAM 15–115 MB

### Perbaikan Bug Penting (21 Okt)
- **OAuth keychain**: keyring crate fails closed untuk binary ad-hoc-signed → fallback `security` CLI
- **OAuth retry 400**: pending tidak dibersihkan setelah gagal → retry pakai kode basi → fix: bersihkan pending saat gagal
- **client_secret missing**: Google mewajibkan secret bahkan untuk Desktop app → tambah field Client Secret opsional + buat Client ID Desktop baru via browser
- **Drive token import**: keychain SSH terblokir → fallback file `~/.minime/drive_tokens.json`

---

## BAGIAN 2: BELUM SELESAI / TERTANGGUH

| # | Item | Status | Catatan detail kenapa tertangguh |
|---|------|--------|----------------------------------|
| 1 | Avatar animasi (Tahap 1) | ⏳ Ditangguhkan | Patria memutuskan membuat sendiri; tidak blocking untuk rilis. Tidak ada deadline. |
| 2 | RouterKu aktif | 🚫 Blocked | Key API ada di Secure Vault, tapi: (a) base URL gateway tidak diketahui, (b) saldo 0. Menunggu Patria: tiket support ke RouterKu + top-up QRIS. Mimi tidak bisa mempercepat — bola di tangan Patria. |
| 3 | Windows installer (.exe) | 🚫 Blocked | Workflow GH Actions siap di `docs/build-windows.yml.README`. Token GitHub `custom.github` TIDAK punya scope `workflow` → push otomatis file `.github/workflows/` gagal 404. Menunggu Patria: copy manual file workflow ke repo, ATAU reconnect token dengan scope workflow. |
| 4 | Tahap 4.5 — latensi IPC formal | ⏳ Belum dikerjakan | Benchmark kasar sudah (startup, CPU, RAM). Pengukuran latensi IPC per-command yang formal belum dilakukan. Bisa dikerjakan Mimi tanpa Patria. |
| 5 | Tahap 4.5 — Lighthouse CI | ⏳ Belum dikerjakan | Config Lighthouse CI gate tiap rilis belum dibuat. Bisa dikerjakan Mimi tanpa Patria. |
| 6 | Tahap 4.5 — audit independen | ⏳ Belum dikerjakan | Audit konsultan performa independen satu kali pra-rilis publik. Butuh Patria: cari & bayar konsultan. |
| 7 | MeHarness ECC gelombang 1 | ⏳ Belum dimulai | Kurasi 7 skill (code-reviewer, build-error-resolver, e2e-runner, tdd-workflow, ai-regression-testing, agent-eval, agent-self-evaluation) ke format Minime belum dimulai. Bisa dikerjakan Mimi. |
| 8 | AgentShield evaluasi | ⏳ Belum dimulai | Evaluasi konsep pemindai konfigurasi agen untuk Tahap 3/4 belum dimulai. Bisa dikerjakan Mimi. |
| 9 | STT nyata (Fase Voice) | ⏳ Ditangguhkan | Voice command masih mock/simulasi. Butuh integrasi Whisper atau API STT. Tidak blocking untuk rilis awal; dijadwalkan Fase Voice terpisah. |
| 10 | WebDriver AFK | 🅿️ Diparkir | Diparkir atas keputusan Patria 21 Okt (setelah incompatibilitas tauri-service). Disebut sekali, jangan dikejar. |
| 11 | Runtime multi-agen penuh | ⏳ Belum penuh | Chat → Mimi orkestrasi → eksekusi paralel antar-agen belum terimplementasi penuh. Dispatch arm ada (3 arm), tapi orkestrasi multi-agen belum lengkap. |
| 12 | Pipeline E2E via UI | ⏳ Menunggu Patria | Komponen teruji satuan (test 32/32). Butuh sesi uji klik manual oleh Patria: brief → delegasi → tool → approval → hasil di Library. Tidak bisa diverifikasi via SSH (GUI terblokir). |
| 13 | Uji klik avatar picker | ⏳ Menunggu Patria | Kode selesai & terverifikasi di dist, tapi butuh konfirmasi visual Patria bahwa picker berfungsi saat diklik. |
| 14 | Uji tombol dummy Pireel | ⏳ Menunggu Patria | Tombol "Buat project dummy" belum diuji klik oleh Patria. |

---

## BAGIAN 3: RENCANA PENGERJAAN BERIKUTNYA

### Prioritas 1 — Dikerjakan Mimi sekarang (tanpa Patria)
1. **Tahap 4.5 latensi IPC**: ukur latensi per Tauri command secara formal, catat baseline
2. **Lighthouse CI**: buat config gate performa tiap rilis
3. **MeHarness ECC gelombang 1**: kurasi 7 skill ke format Minime
4. **AgentShield**: evaluasi konsep pemindai konfigurasi agen
5. **Skenario E2E**: siapkan skrip uji klik langkah-demi-langkah untuk sesi Patria

### Prioritas 2 — Menunggu input/aksi Patria
6. **RouterKu**: tiket support + top-up QRIS → Mimi aktifkan provider
7. **Windows installer**: copy manual workflow ATAU reconnect token → trigger build .exe
8. **Sesi uji E2E**: Patria uji klik alur penuh (brief → delegasi → tool → approval → Library)
9. **Uji klik**: avatar picker + tombol dummy Pireel
10. **Keputusan avatar animasi**: Patria buat sendiri atau batalkan

### Prioritas 3 — Setelah Prioritas 1+2 tuntas
11. **Audit independen**: Patria cari konsultan performa → audit pra-rilis
12. **Tinjau ulang semua tahap**: verifikasi konsistensi menyeluruh (aturan Patria 20 Okt)
13. **Uji project asli pertama**: jalankan satu project nyata end-to-end
14. **Vonis launching**: keputusan Patria
15. **Fase 6 komersil**: installer per OS + auto-updater, code signing, login OTP, lisensi + Stripe, Account API, kupon admin

---

*Prioritas 2 tidak bisa dipercepat dari sisi Mimi — bola ada di tangan Patria.*
*Prioritas 1 dikerjakan dalam sesi-sesi berikutnya atas perintah Patria.*
