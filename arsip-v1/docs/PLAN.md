# PLAN.md — Minime

> Rencana build per fase + arsitektur ringkas. Diperbarui setiap review gate.
> Detail penuh: `docs/arsitektur.md` (v3). Aturan: `docs/rule.md`. Status: `docs/status.md`.

## Ringkasan rencana

**Minime** = aplikasi desktop Tauri 2.x local-first untuk 5 agen AI senior yang persisten.
Dua komponen: `minime-daemon` (Rust headless — pemilik SQLite, eksekutor task, LLM, tools;
jalan via launchd, tetap hidup walau UI di-quit) + aplikasi Tauri (dashboard client tipis via IPC Unix socket).
Komersialisasi (Fase 6): Account API (OTP email), lisensi + Stripe, provider gratis/berbayar (default Muse),
release pipeline dengan code signing.

Bahasa: Indonesia (UI & dokumen), Inggris (kode & komentar).

## Arsitektur ringkas

```
minime-daemon (Rust)                    Aplikasi Tauri (dashboard)
├─ SQLite WAL (7 tabel, single-writer)  ├─ Proxy IPC → Tauri events
├─ Agent loader (agents/<id>/)          ├─ Views: Agen, Antrean, Berjalan,
├─ Pipeline runner (pipelines/*.yaml)   │         Persetujuan, Deliverable,
├─ FIFO executor per agen               │         Penggunaan, Pengaturan
├─ Provider trait (Fase 2: implementasi)├─ Menu konteks per-agen
├─ Tool runners (Fase 4)                └─ Panel browser mirror (Fase 4)
├─ Scheduler internal
├─ Notifikasi native macOS
└─ IPC: Unix socket (JSON-lines)
```

**Framework agen**: `agents/<id>/agent.yaml` + `system-prompt.md` → di-load daemon saat start → seed tabel `agents`.
Tambah agen = tambah folder (tanpa ubah kode). Skills di `skills/<nama>/SKILL.md` → tabel `skills`.

**Workflow**: `pipelines/minime-pipeline.yaml` (utama) + sub-pipeline SMO & Code Builder (opsional).
Runner: validasi DAG → eksekusi FIFO per stage → handoff via `messages` → terminal `approval` → record di `approvals`.

## Fase & kriteria review gate

| Fase | Fokus | Kriteria gate (demo nyata, bukan laporan) |
|---|---|---|
| 1 | Fondasi daemon + UI skeleton + framework agen/workflow | Daemon jalan, task dummy end-to-end via IPC, pipeline YAML tereksekusi, dashboard tampil 5 agen + status, **bukti quit UI ≠ stop daemon** |
| 2 | Orkestrasi inti + provider (Gemini, Muse) | 1 job nyata penuh (Riset→…→Approval) + tap approve Patria + notifikasi native |
| 3 | Skills & knowledge (10 playbook disuling) | Output tiap skill di-review Patria; DeepSeek jalan |
| 4 | Tools produksi + built-in browser | 1 reels/carousel jadi dari pipeline; SMO draft-only |
| 5 | Hardening & rilis v1 pribadi | dmg ad-hoc ter-install di MacBook Patria; audit keamanan + Impeccable lolos |
| 6 | Komersialisasi (paralel ≥Fase 2) | dmg ter-sign + login OTP + lisensi trial/berbayar berfungsi |

## Keputusan terbuka (tidak menghambat Fase 1)

1. Model harga (sekali-bayar vs langganan), lama trial, fitur terkunci lisensi, kuota LLM trial.
2. Verifikasi region Indonesia untuk Meta Model API (dibutuhkan Fase 2).
3. OpenCreator: kunci LLM OpenAI-compatible ATAU wiring Gemini + uji 1 video (Fase 4).
4. VoiceStudio: instalasi MacBook + uji suara BI (Fase 4).
5. OAuth Google Drive (Fase 3).

## Catatan lingkungan

- Build & verifikasi Fase 1 dilakukan di Linux (VM). `cargo build` daemon + `vite build` frontend terverifikasi di sini.
- Full `tauri build` (dmg) dan LaunchAgent hanya bisa diuji di **MacBook Patria** — langkah verifikasinya didokumentasikan di `docs/status.md` saat gate.
- Remote GitHub privat: TUNGGU konfirmasi Patria (dilarang buat tanpa izin).
