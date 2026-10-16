# Minime

Aplikasi desktop local-first untuk 5 agen AI senior yang persisten — terinspirasi pola
arsitektur `nightly-labs/openbot`, implementasi clean-room (bukan fork).

- **Daemon** (`daemon/`): Rust headless — pemilik SQLite, eksekutor task FIFO, pipeline runner, IPC Unix socket. Jalan via launchd, tetap hidup walau UI di-quit.
- **UI** (`src/` Svelte 5 + `src-tauri/`): dashboard client tipis.
- **Agen**: `agents/<id>/agent.yaml` + `system-prompt.md` (tambah agen = tambah folder).
- **Workflow**: `pipelines/*.yaml`.

Dokumen: `AGENTS.md` (aturan main) · `docs/arsitektur.md` (desain v3) · `docs/PLAN.md` · `docs/TODO.md`.

Status: Fase 1 — fondasi (dalam pengerjaan, menunggu review gate Patria).
