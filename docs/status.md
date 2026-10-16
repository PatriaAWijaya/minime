# status.md — Minime

> Log kronologis proyek. Entri baru di-APPEND di bawah dengan header tanggal.
> JANGAN edit entri lama (pelajaran 14 Okt 2026: pakai anchor akhir-entri yang unik).

---

## 2026-10-16 — Kickoff & arsitektur (brainstorm selesai, menunggu review Patria)

### Latar
- 16 Okt ~16:15 WIB: Patria perintahkan **reset total PENDOPO** — semua file di-trash, 7 cron dihapus, dashboard artifact + goal dihapus permanen. Kredensial `custom.gemini` di Secure Vault TIDAK dihapus (bisa dipakai ulang).
- 16 Okt ~19:05 WIB: diputuskan bangun **agen house** dengan dashboard realtime mengikuti diagram alur Patria.
- 16 Okt ~19:05–19:24 WIB: serangkaian keputusan (di bawah) → proyek dinamai **Minime**.

### Keputusan final (tidak diubah tanpa Patria)
1. **Target akhir**: aplikasi desktop seperti `nightly-labs/openbot` — local-first workspace untuk AI teammates persisten. **Bangun sendiri terinspirasi arsitekturnya (clean-room), BUKAN fork** — OpenBot berlisensi PolyForm Noncommercial.
2. **Nama proyek**: Minime.
3. **Platform**: desktop app langsung — **Tauri 2.x** (Rust backend + webview native). Target v1: dmg macOS untuk MacBook Patria.
4. **Alur kerja** (diagram Patria): Mimi (orkestrator) → Riset → Strategi Bisnis → 6 Playbook (Branding, Marketing, Selling, Operational, Finance, Code Builder) → Creative → 4 Playbook (Storytelling, Copywriting, Content, SMO) → Content Production (Image, Carousel, Reels) → Tools (Canva, Affinity, OpenCreator, VoiceStudio) → QC → Approval (Patria, manusia, wajib) → Deliverable Output. Cabang: Code Builder → UI/UX → QC; SMO → Page Optimization/Content Pilar/Planning/Distribution/Engagement → QC; feedback loop Deliverable → distribusi.
5. **LLM**: Gemini API (orkestrasi/riset/konten) + DeepSeek API (strategi/code). Kunci milik Patria, simpan di **macOS Keychain** — bukan di git, bukan plaintext.
6. **Ikon aplikasi**: robot biru dari upload Patria → `assets/app-icon.jpg` (220×220 JPEG).
7. **Avatar agen**: 15 ikon 3D (flat vector ditolak Patria 16 Okt) → `assets/avatars/`.
8. **Aturan keras**: kredensial tidak di git; TIDAK ADA auto-posting ke Meta (distribusi = draft sampai Patria approve manual); deliverable ke Drive folder MIMI; data sensitif tidak diupload; UI ikut `ui-rules-for-real-users.md` + lolos anti-pattern Impeccable; review gate tiap fase.
9. **Bahasa**: Indonesia untuk UI & dokumen user-facing; Inggris untuk kode & komentar.

### Artefak desain (16 Okt, tahap brainstorm)
- `AGENTS.md` — dibuat via `PROMPT-AGENTS.md` oleh subagent; **sudah diserahkan ke Patria untuk review** (status: menunggu vonis).
- `docs/status.md` (file ini), `docs/rule.md`, `docs/arsitektur.md` — ditulis subagent brainstorm 16 Okt ~19:25 WIB.
- File referensi Impeccable (`~/workspace/kantor-ai-agent/referensi/impeccable-skill.md`) sempat ikut ke-trash saat reset — **sudah dipulihkan** dari upstream github.com/pbakaus/impeccable (Apache 2.0).

### Status saat ini
- [x] Analisis kebutuhan (model/agen/tools/koneksi) — 16 Okt
- [x] Keputusan scope: bangun sendiri, nama Minime, desktop app Tauri
- [x] Prompt generator AGENTS.md (`PROMPT-AGENTS.md`)
- [x] AGENTS.md v1 — menunggu review Patria
- [x] Aset: app-icon + 15 avatar agen 3D
- [x] Brainstorm arsitektur: `docs/arsitektur.md` + rekomendasi fase — **menunggu review Patria**
- [ ] TODO.md / PLAN.md — dibuat SETELAH arsitektur + fase disetujui Patria
- [ ] Fase 1 build — belum mulai

### Berikutnya (butuh vonis Patria)
1. Review `AGENTS.md` — setuju / revisi.
2. Review `docs/arsitektur.md` + usulan 5 fase (dengan refinement) — setuju / revisi.
3. Setelah keduanya disetujui → tulis TODO.md + PLAN.md → mulai Fase 1.

---

## 2026-10-16 — Rewrite arsitektur v2: daemon + UI (menunggu review Patria)

### Keputusan final baru (tidak diubah tanpa Patria)
1. **Prinsip agen senior** (vonis Patria ~19:33 WIB): sedikit agen setara senior manager > banyak agen junior. Roster: 5 agen senior (Mimi/Orkestrator, Riset, Strategi Bisnis, Creative Director, QC) + Approval = Patria (manusia). Playbook menjadi skills milik agen senior. Kontrak senior: playbook/SOP + toolbox + proof of work + quality bar + eskalasi bila ragu.
2. **Agen tetap bekerja walau aplikasi ditutup** (permintaan Patria ~19:33 WIB) → arsitektur DUA KOMPONEN: `minime-daemon` (background service Rust: pemilik tunggal SQLite, eksekutor task, LLM, tools, scheduler, notifikasi native macOS) + aplikasi Tauri (dashboard UI sebagai client tipis via Unix socket). Daemon via launchd LaunchAgent (RunAtLoad + KeepAlive): start saat login, hidup walau UI di-quit. Quit app ≠ stop agents. Hanya daemon yang menulis DB (single-writer).
3. Perintah Patria ~19:34 WIB: **"buat ulang arsitekturnya"** → `docs/arsitektur.md` ditulis ulang DARI NOL (v1 satu-proses batal).

### Perubahan dokumen
- `docs/arsitektur.md` — rewrite penuh: diagram dua komponen, detail daemon (lifecycle launchd, crash recovery), UI client, protokol IPC Unix socket, skema 7 tabel, kontrak senior, pipeline YAML, alur QC→Approval + notifikasi native, packaging dmg + setup LaunchAgent, 5 fase disesuaikan.
- `AGENTS.md` — §b tech stack dua komponen, DoD +2 item (daemon tetap jalan setelah quit; start saat login; notifikasi approval saat UI tutup), §c catatan eksekusi di daemon, §d referensi rule #16, §f struktur repo (`daemon/` + `src-tauri/` thin client), §g 5 fase versi daemon.
- `docs/rule.md` — aturan #16 (prinsip agen senior), #17 (quit app ≠ stop agents), #18 (single-writer DB).

### Status saat ini
- Menunggu review Patria: `AGENTS.md` + `docs/arsitektur.md` (v2).
- Setelah disetujui → TODO.md + PLAN.md → Fase 1.

## 2026-10-16 — Rewrite arsitektur v3: komersialisasi (menunggu review Patria)

### Keputusan final baru (tidak diubah tanpa Patria)
1. **Target komersil** (~19:40 WIB): Minime menjadi aplikasi komersil yang di-download & di-install seperti `nightly-labs/openbot`. Mimi mempelajari mekanisme login + verifikasi email OpenBot dari repo publiknya (OTP 8 karakter via email, hash-only di D1, API di Cloudflare Workers, SMTP port 465/TLS, rate limit, Stripe billing) — **pola diadopsi, implementasi ditulis sendiri** (bukan fork).
2. **Distribusi**: installer per OS dari GitHub Releases — dmg (macOS), exe/msi (Windows), AppImage (Linux). **Code signing wajib** untuk rilis publik: Apple Developer $99/thn + notarization (tanpa ini Gatekeeper memblokir), sertifikat Windows (SmartScreen). **Auto-updater** bawaan Tauri (artifact ter-sign). v1 fokus macOS arm64; v1 pribadi boleh ad-hoc signing.
3. **Sistem akun**: login **OTP 8 karakter via email** — di DB hanya hash + expiry (tidak pernah plaintext); rate limit per email/IP/kirim ulang; kirim via SMTP (465/TLS) + fallback webhook POST `{email, code, expiresAt}`; mode dev boleh expose kode di response, **dilarang di production**; cron pembersih kode/session kadaluarsa; session token per perangkat. **Account API terpusat** (rekomendasi: Cloudflare Workers + D1, repo & deploy sendiri).
4. **Lisensi & billing**: akun ↔ lisensi (`trial`/`pro`/`lifetime`); app cek lisensi tiap start (cache + offline grace 30 hari — local-first tidak boleh mati karena WiFi putus); trial expired → fitur berbayar dikunci (data tetap bisa dibaca). **Stripe**: plan bulanan/tahunan, Customer Portal untuk upgrade/cancel mandiri, webhooks (renewal/gagal bayar/cancel).
5. **Sistem provider model** (~19:45 WIB): abstraction layer di daemon (trait: `list_models`, `complete_stream`, estimasi token). **Gratis**: deteksi server lokal Ollama/LM Studio (gratis & privat) + tier freemium. **Berbayar**: user tempel API key sendiri → Keychain. **Default berbayar = Muse** (~19:46 WIB; Meta Model API `ai.meta.com`, ±$1,25/1M input, ±$4,25/1M output, $20 kredit gratis akun baru) — CATATAN: verifikasi region Indonesia saat implementasi; fallback Gemini/DeepSeek. Provider lain: Anthropic, OpenAI, Gemini, DeepSeek. **Routing per-agen + fallback chain** + **metering** (token & estimasi biaya, kolom di tabel `runs`) + **budget cap** per agen/hari. UI: Settings → Providers, view Penggunaan.

### Perubahan dokumen
- `docs/arsitektur.md` — **rewrite penuh v3** (v2 batal): overview 4 komponen (daemon, UI client, account API, release pipeline); detail provider system (trait, routing, fallback, metering, budget cap); skema D1 account API (accounts, email_challenges [hash-only], sessions, licenses, stripe_customers); alur OTP end-to-end; alur lisensi & Stripe webhook; packaging + signing + auto-update; **6 fase** (Fase 6 komersialisasi, boleh paralel setelah Fase 2); keputusan bisnis terbuka (model harga, lama trial, fitur terkunci, kuota LLM trial, region Indonesia Meta API).
- `AGENTS.md` — §b (+provider system, account API, release pipeline), DoD +3 item (dmg ter-sign; login OTP; lisensi enforced), §g 6 fase (+Fase 6).
- `docs/rule.md` — aturan #19 (API key user hanya di Keychain), #20 (OTP hash-only + rate limit), #21 (signing wajib rilis publik), #22 (secret backend via secret manager).

### Status saat ini
- Menunggu review Patria: `AGENTS.md` + `docs/arsitektur.md` (v3).
- Setelah disetujui → TODO.md + PLAN.md → Fase 1.
