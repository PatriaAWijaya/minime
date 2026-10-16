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
