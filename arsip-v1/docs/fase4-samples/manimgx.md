# Sampel uji skill: manimgx

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `creative-director` dengan playbook stage-level `["manimgx"]`.
> Bukti wiring: `PLAYBOOK AKTIF: manimgx`.
> Status wiring: OK — TERBUKTI.
> Tool `manimgx` ter-wire di daemon (`daemon/src/tools/manimgx.rs`): readiness ✅.
> Isi lengkap skill: `skills/manimgx/SKILL.md`.

## Tugas
Uji wiring skill manimgx

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `manimgx` ter-inject ke prompt agen dan di-echo oleh mock provider.

```
PLAYBOOK AKTIF: manimgx
```

## Uji nyata (bukti koneksi berjalan)

Render nyata `examples/circle_area.py` (CircleArea) via `manimgx` 0.1.1:
MP4 17.9 detik, 1280×720@30fps, 0.6 MB — `docs/bukti-fase4/manimgx-sample.mp4`
(+ frame `docs/bukti-fase4/manimgx-frame1.png`).
Catatan: tanpa GPU di VM ini → software render (lavapipe), 68 detik;
verifikasi penuh GPU di MacBook Patria.
