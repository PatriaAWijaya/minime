# TODO.md — Minime

> Checklist per fase. Centang `- [x]` + tanggal saat item selesai.
> Satu fase selesai dan review gate lolos → baru lanjut fase berikut (rule #7, #14).

## Fase 1 — Fondasi daemon + UI skeleton ⏳ berjalan (16 Okt 2026)

### Dokumen & repo
- [x] docs/TODO.md + docs/PLAN.md ditulis (16 Okt 2026)
- [ ] Struktur repo: `daemon/`, `src-tauri/`, `src/`, `pipelines/`, `skills/`, `agents/`, `docs/`, `assets/`
- [ ] Git init + commit `feat: fase 1 fondasi Minime` (remote privat: TUNGGU konfirmasi Patria — dilarang buat remote tanpa izin)

### Daemon (Rust `minime-daemon`)
- [ ] SQLite 7 tabel + migrasi berversi (`agents`, `tasks`, `messages`, `runs`, `approvals`, `deliverables`, `skills`)
- [ ] Crash recovery: task `running` → `failed` saat daemon start
- [ ] IPC Unix socket JSON-lines: `daemon.status`, `agents.list`, `tasks.enqueue`/`tasks.list`/`tasks.cancel`/`tasks.retry`, `runs.list`, `approvals.list`, `deliverables.list`, `subscribe` (event)
- [ ] FIFO executor per agen (satu berjalan per agen; pause/resume/cancel); Fase 1 = eksekusi dummy (simulasi progres, tanpa LLM)
- [ ] Provider trait: definisi interface saja (`list_models`, `complete_stream`, estimasi token) — implementasi Fase 2
- [ ] Template plist launchd `com.minime.daemon` (RunAtLoad + KeepAlive)

### Framework agen (scope tambahan 16 Okt — Patria)
- [ ] `agents/mimi/`, `agents/riset/`, `agents/strategi-bisnis/`, `agents/creative-director/`, `agents/qc/`: `agent.yaml` + `system-prompt.md` (Bahasa Indonesia)
- [ ] Agent loader di daemon: baca `agents/<id>/` saat start → seed tabel `agents` (tambah agen = tambah folder, tanpa ubah kode)
- [ ] `skills/` registry: stub `SKILL.md` per skill → seed tabel `skills` (bukan hardcode)

### Workflow (scope tambahan 16 Okt — Patria)
- [ ] `pipelines/minime-pipeline.yaml` (alur utama: Mimi → Riset → Strategi Bisnis → Creative Director → QC → Approval → Deliverable)
- [ ] `pipelines/smo-subpipeline.yaml` + `pipelines/codebuilder-subpipeline.yaml` (cabang opsional)
- [ ] Pipeline runner di daemon: parse & validasi YAML (DAG valid, stage → agen yang ada, skill terdaftar), eksekusi FIFO per stage, handoff via tabel `messages`, dukung fan-out/fan-in

### UI skeleton (Svelte 5 — dikunci)
- [ ] Dashboard: daftar 5 agen + status realtime (kuning=aktif, hijau=idle, merah=butuh perhatian)
- [ ] Menu konteks per-agen: Tasks, New task, Pengaturan agen, Sembunyikan komputer (stub dulu)
- [ ] Bahasa Indonesia; UI rules diterapkan (body 14–16px, judul section 32px bold, satu primary action per section, satu accent color)
- [ ] `src-tauri/` thin client: proxy IPC → Tauri events (scaffold; full build di macOS)

### Verifikasi Fase 1
- [ ] `cargo build` sukses
- [ ] Daemon start → DB + migrasi + seed 5 agen + skills OK
- [ ] IPC: enqueue task dummy → progres → done → approval record tercipta
- [ ] Pipeline YAML tervalidasi & tereksekusi end-to-end (dummy)
- [ ] `vite build` sukses (frontend)
- [ ] Bukti quit ≠ stop terdokumentasi (di Linux: kill proses UI → daemon tetap jalan; launchd: di MacBook Patria)
- [ ] `docs/status.md` di-append

## Fase 2 — Orkestrasi inti + provider
- [ ] Pipeline YAML runner penuh (handoff nyata antar agen)
- [ ] Implementasi provider Gemini + Muse (verifikasi region Indonesia; fallback Gemini)
- [ ] Routing per-agen + fallback chain dasar
- [ ] 1 pipeline vertikal end-to-end nyata (Riset → Strategi Bisnis → Creative Director → QC → Approval → Deliverable draft)
- [ ] Approval inbox minimal + notifikasi native macOS
- [ ] *Gate: 1 job nyata berjalan penuh + tap approve Patria*

## Fase 3 — Skills & knowledge
- [ ] 6 skills → Strategi Bisnis, 4 skills → Creative Director (knowledge disuling dari 4 dokumen Drive)
- [ ] Provider DeepSeek
- [ ] OAuth Google Drive sekali (baca)
- [ ] *Gate: output tiap skill di-review Patria*

## Fase 4 — Tools produksi
- [ ] Readiness check eksplisit tiap tool + fallback
- [ ] Wiring Canva / Affinity / OpenCreator / VoiceStudio / HyperFrames
- [ ] Content Production (Image/Carousel/Reels); SMO draft-only
- [ ] Built-in browser (Chromium headless via Playwright) untuk agen
- [ ] *Gate: 1 reels/carousel jadi dari pipeline*

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
- [ ] Signing publik (Apple notarization, sertifikat Windows); release pipeline GitHub Actions; auto-updater
- [ ] *Gate: instalasi dari dmg ter-sign + login OTP + lisensi trial/berbayar berfungsi*
