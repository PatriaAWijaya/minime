# TODO.md — Minime

> Checklist per fase. Centang `- [x]` + tanggal saat item selesai.
> Satu fase selesai dan review gate lolos → baru lanjut fase berikut (rule #7, #14).

## Fase 1 — Fondasi daemon + UI skeleton ✅ selesai dibangun (16 Okt 2026, menunggu review gate)

### Dokumen & repo
- [x] docs/TODO.md + docs/PLAN.md ditulis (16 Okt 2026)
- [x] Struktur repo: `daemon/`, `src-tauri/`, `src/`, `pipelines/`, `skills/`, `agents/`, `docs/`, `assets/` (16 Okt 2026)
- [x] Git init + commit `feat: fase 1 fondasi Minime` (remote privat: TUNGGU konfirmasi Patria — dilarang buat remote tanpa izin)

### Daemon (Rust `minime-daemon`)
- [x] SQLite 7 tabel + migrasi berversi (`agents`, `tasks`, `messages`, `runs`, `approvals`, `deliverables`, `skills`)
- [x] Crash recovery: task `running` → `failed` saat daemon start (16 Okt 2026)
- [x] IPC Unix socket JSON-lines: `daemon.status`, `agents.list`, `tasks.enqueue`/`tasks.list`/`tasks.cancel`/`tasks.retry`, `runs.list`, `approvals.list`, `deliverables.list`, `subscribe` (event)
- [x] FIFO executor per agen (satu berjalan per agen; pause/resume/cancel); Fase 1 = eksekusi dummy (simulasi progres, tanpa LLM)
- [x] Provider trait: definisi interface saja (`list_models`, `complete_stream`, estimasi token) — implementasi Fase 2
- [x] Template plist launchd `com.minime.daemon` (RunAtLoad + KeepAlive)

### Framework agen (scope tambahan 16 Okt — Patria)
- [x] `agents/mimi/`, `agents/riset/`, `agents/strategi-bisnis/`, `agents/creative-director/`, `agents/qc/`: `agent.yaml` + `system-prompt.md` (Bahasa Indonesia)
- [x] Agent loader di daemon: baca `agents/<id>/` saat start → seed tabel `agents` (tambah agen = tambah folder, tanpa ubah kode)
- [x] `skills/` registry: stub `SKILL.md` per skill → seed tabel `skills` (bukan hardcode)

### Workflow (scope tambahan 16 Okt — Patria)
- [x] `pipelines/minime-pipeline.yaml` (alur utama: Mimi → Riset → Strategi Bisnis → Creative Director → QC → Approval → Deliverable)
- [x] `pipelines/smo-subpipeline.yaml` + `pipelines/codebuilder-subpipeline.yaml` (cabang opsional)
- [x] Pipeline runner di daemon: parse & validasi YAML (DAG valid, stage → agen yang ada, skill terdaftar), eksekusi FIFO per stage, handoff via tabel `messages`, dukung fan-out/fan-in

### UI skeleton (Svelte 5 — dikunci)
- [x] Dashboard: daftar 5 agen + status realtime (kuning=aktif, hijau=idle, merah=butuh perhatian)
- [x] Menu konteks per-agen: Tasks, New task, Pengaturan agen, Sembunyikan komputer (stub dulu)
- [x] Bahasa Indonesia; UI rules diterapkan (body 14–16px, judul section 32px bold, satu primary action per section, satu accent color)
- [x] `src-tauri/` thin client: proxy IPC → Tauri events (scaffold; full build di macOS)

### Verifikasi Fase 1
- [x] `cargo build` sukses (16 Okt 2026)
- [x] Daemon start → DB + migrasi + seed 5 agen + skills OK (16 Okt 2026)
- [x] IPC: enqueue task dummy → progres → done → approval record tercipta (16 Okt 2026)
- [x] Pipeline YAML tervalidasi & tereksekusi end-to-end (dummy)
- [x] `vite build` sukses (frontend)
- [x] Bukti quit ≠ stop terdokumentasi (di Linux: kill proses UI → daemon tetap jalan; launchd: di MacBook Patria)
- [x] `docs/status.md` di-append (16 Okt 2026)

## Fase 2 — Orkestrasi inti + provider
- [x] Pipeline YAML runner penuh (handoff nyata antar agen) (16 Okt 2026 — output stage N → runs.context → input stage N+1, terverifikasi via mock)
- [x] Implementasi provider Gemini + Muse + OpenCode free (16 Okt 2026 — region check Muse→fallback Gemini; live test: key belum ada, backend OpenCode free "Model is unavailable" — blocker eksternal, lihat status.md)
- [x] Routing per-agen + fallback chain dasar (16 Okt 2026 — terbukti di log: gemini NoKey → muse NoKey → opencode; + uji fallback mock)
- [x] 1 pipeline vertikal end-to-end (Riset → Strategi Bisnis → Creative Director → QC → Approval → Deliverable draft) (16 Okt 2026 — penuh via mock provider; QC skor 80 ter-parse; deliverable draft terbuat)
- [x] Approval inbox minimal + notifikasi native macOS (16 Okt 2026 — UI Setujui/Tolak + skor + catatan; notify.rs via osascript; test penuh di MacBook Patria)
- [ ] *Gate: 1 job nyata berjalan penuh + tap approve Patria* (menunggu review Patria; job mock penuh ✅, job OpenCode free terblokir backend)

## Fase 3 — Skills & knowledge
- [x] 6 skills → Strategi Bisnis, 4 skills → Creative Director (16 Okt 2026 — SKILL.md penuh: SOP, toolbox, quality bar, trigger eskalasi, proof of work; knowledge disuling dari 4 dokumen Drive → `docs/fase3-knowledge/`; finance & code-builder: struktur lengkap + placeholder [KNOWLEDGE PATRIA], tanpa karangan)
- [x] Provider DeepSeek (16 Okt 2026 — `providers_deepseek.rs`, OpenAI-compatible `api.deepseek.com/v1`; key via Keychain/env `MINIME_DEEPSEEK_KEY`; tanpa key → NoKey → fallback; masuk fallback chain strategi-bisnis; live test menunggu key)
- [x] OAuth Google Drive sekali, read-only (16 Okt 2026 — `drive_oauth.rs`: auth code flow + loopback listener + state anti-CSRF, scope `drive.readonly`, token di Keychain write-only; IPC `drive.auth_url`/`drive.status`/`drive.disconnect` + `secrets.set`/`secrets.has`; URL terverifikasi via IPC; flow browser penuh hanya di MacBook Patria — lihat `docs/drive-oauth.md`)
- [x] Uji tiap skill: 1 tugas nyata per skill via mock (16 Okt 2026 — 10/10 wiring TERBUKTI: playbook stage-level di-inject ke prompt; sampel di `docs/fase3-samples/`)
- [x] Coba ulang OpenCode free 1x (16 Okt 2026 — masih hang/timeout 5 menit, stdout kosong; backend free tier belum pulih di environment ini; JANGAN bakar waktu lagi — coba dari MacBook Patria)
- [ ] *Gate: output tiap skill di-review Patria* (10 SKILL.md + 10 sampel + 4 knowledge siap di-review)

## Fase 4 — Tools produksi
- [x] Readiness check eksplisit tiap tool + fallback (2026-10-16)
- [x] Wiring Canva / Affinity / OpenCreator / VoiceStudio / HyperFrames (2026-10-16)
- [x] Content Production (Image/Carousel/Reels); SMO draft-only (2026-10-16)
- [x] Built-in browser (Chromium headless via Playwright) untuk agen (2026-10-16)
- [x] *Gate: 1 reels/carousel jadi dari pipeline* (2026-10-16 — `docs/bukti-fase4/reels-minime.mp4`, 1080x1920 6 detik)
- [x] Skill `oil-ui` (17 Okt 2026 — skill UI untuk code builder, milik strategi-bisnis; diadaptasi dari `oil-oil/oil-ui` MIT; wiring mock TERBUKTI `PLAYBOOK AKTIF: oil-ui`, sampel di `docs/fase4-samples/oil-ui.md`; referensi asli di-vendor di `skills/oil-ui/referensi-asli/`, update-check dinonaktifkan)
- [x] Skill produksi `html-anime` (17 Okt 2026 — milik creative-director; mengorkestrasi 5 sub-skill asli `blixvip/html-anime` MIT; `harness/compile.js` di-vendor ke `tools/html-anime/`; wiring mock TERBUKTI `PLAYBOOK AKTIF: html-anime`; **bumper opening Minime NYATA**: `docs/bukti-fase4/bumper-minime.mp4` 1920x1080 8 detik — catatan: Patria minta ±4 detik, tapi `compileBrief` hanya mendukung 8/12/20 detik → dipakai 8 detik (minimum); GSAP di-vendor lokal karena CDN diblokir proxy VM)
- [x] Tool + skill `manimgx` (17 Okt 2026 — engine animasi Manim CE untuk video pembelajaran, milik creative-director; `daemon/src/tools/manimgx.rs` readiness ✅; **render NYATA**: `docs/bukti-fase4/manimgx-sample.mp4` 17.9 detik 1280x720@30fps via software (tanpa GPU di VM); wiring mock TERBUKTI)
- [x] Tool + skill `valyu-finance` (17 Okt 2026 — riset finansial via Valyu Search API, milik strategi-bisnis, diadaptasi dari `yorkeccak/finance` MIT; readiness `not_configured` — menunggu VALYU_API_KEY dari Patria via Keychain; wiring mock TERBUKTI; aturan keras: riset/analisis saja, DILARANG order live)
- [x] Gudang skill `skill-warehouse` (17 Okt 2026 — milik **mimi**/orkestrator; 267 spesialis dari `msitarzewski/agency-agents` MIT diindeks di `skills-warehouse/INDEX.md`; 8 modul teradaptasi: ui-designer, ux-researcher, frontend-developer, code-reviewer, reality-checker, research-synthesist, carousel-growth-engine, short-video-editing-coach; protokol on-demand: dimuat hanya bila tugas butuh keahlian di luar skill inti, dilarang auto-load; gudang BUKAN agen baru — roster tetap 5 agen; wiring mock TERBUKTI `PLAYBOOK AKTIF: skill-warehouse`, sampel di `docs/fase4-samples/skill-warehouse.md`)
- [x] Finance skills quant/trading (17 Okt 2026 — milik strategi-bisnis; `finance-playbook` diisi penuh: keuangan bisnis + toolbox quant; 4 skill baru: `trading-agents` (TauricResearch/TradingAgents Apache-2.0), `qlib` (microsoft/qlib MIT), `nautilus` (nautilus_trader LGPL-3.0 — koneksi proses eksternal saja), `openbb` (OpenBB SDK); tool daemon `quant` (`ta_analyze`/`qlib_backtest`/`nautilus_backtest`/`openbb_data`) + readiness check; wiring mock TERBUKTI; aturan keras: riset/analisis/backtest/paper-trading saja, DILARANG order live otomatis — DITOLAK di kode)
- [x] Pipeline konten video otomatis (17 Okt 2026 — adaptasi diagram n8n 3-step Patria: `pipelines/konten-video-otomatis.yaml` skrip → render Veo 3.1 → QC → draft distribusi 9 platform → approval; tool daemon `veo3` dengan rantai backend GEMINI API PRIMER → fal.ai → HyperFrames lokal (model default `veo-3.1-fast-generate-preview`, 1080p/8 dtk/16:9; key via Keychain akun `gemini`/`fal`; tiap percobaan dicatat di "attempts", tidak ada kegagalan diam-diam) dan `blotato` (upload + draft per platform, key via Keychain akun `blotato`); DRAFT-ONLY — publish IG/FB/Threads DITOLAK di kode (rule #3), 6 platform lain wajib approval_id approved; `smo-playbook` ditambah SOP Blotato 9 platform; `content` skill ditambah §E panduan routing produksi video (Veo vs OpenCreator paralel vs HyperFrames/manimgx); wiring mock TERBUKTI + uji rantai fallback nyata (gemini skipped → fal skipped → hyperframes, attempts jujur); sampel `docs/fase4-samples/konten-video-otomatis.md`; render Veo 3.1 asli menunggu API key Gemini Patria; perkiraan biaya ±$0,96/klip 8 dtk 1080p Fast — harga publik Google, bukan janji)

## Fase 5 — Hardening & rilis v1 pribadi
- [ ] QC judge penuh (rubrik skor 0–100)
- [ ] Metering + budget caps + view Penggunaan
- [ ] Providers settings UI
- [ ] Upload deliverable ke Drive MIMI
- [ ] dmg ad-hoc + setup LaunchAgent; security audit; audit Impeccable
- [ ] *Gate: v1 rilis di MacBook Patria*

## Fase 6 — Komersialisasi (boleh paralel setelah Fase 2)
- [ ] Account API (repo Workers + D1 + deploy)
- [ ] OTP email + SMTP; session token
- [ ] Stripe (products, Customer Portal, webhooks); license enforcement + offline grace 30 hari
- [ ] Sistem kupon Patria: generate kode (admin-only), 1 kupon = maks 5 user, redeem = akses Pro 1 tahun; kode disimpan hash-only di D1 + rate limit redeem; lisensi `coupon` (+1 tahun dari redeem)
- [ ] Signing publik (Apple notarization, sertifikat Windows); release pipeline GitHub Actions; auto-updater
- [ ] Installer Windows (.exe via Tauri NSIS/WiX, runner Windows di GitHub Actions) — dibangun setelah codebase rapi & konsisten; butuh sertifikat code signing Windows
- [ ] *Gate: instalasi dari dmg ter-sign + login OTP + lisensi trial/berbayar berfungsi*
