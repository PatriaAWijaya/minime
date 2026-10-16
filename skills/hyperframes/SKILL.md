---
name: hyperframes
description: Tool skill — HyperFrames untuk render MP4 lokal dari komposisi HTML; di-wire daemon Fase 4 (terverifikasi render nyata).
owner_agent: creative-director
version: 1.0.0
example-prompt: Render reels 9:16 dari 3 slide teks ini.
---

# hyperframes

> Tool skill (bukan playbook). Dieksekusi oleh `minime-daemon` via
> `npx hyperframes` (pola: skill `local_hyperframes`).
> **Terverifikasi Fase 4**: render reels 9:16 nyata di VM (draft, 1080x1920).

## Kapan dipakai

Bila tugas butuh video MP4 yang di-render lokal (reels, slideshow, explainer)
tanpa bergantung layanan eksternal/berbayar.

## Operasi (via `tools.run` / task `tool:`)

| op | params | hasil |
|---|---|---|
| `render` | `project_dir` (berisi index.html + hyperframes.json), `quality` (draft/looks/delivery), `output` | MP4 terverifikasi ffprobe → `artifact_path`, `duration_s`, `width`, `height` |
| `render-reels` | `title`, `slides[]` (`{kicker, heading, body, duration_s}`), `output`, `quality` | generate komposisi 9:16 → render → verifikasi → `artifact_path` |

Aturan komposisi (correctness gate): **semua aset lokal, tanpa CDN/JS eksternal**.
`render-reels` memakai slide statis (tanpa animasi) — paling robust.

## Readiness

`tools.status` → `hyperframes`: siap bila `npx hyperframes` + Chrome Headless
Shell (`npx hyperframes browser ensure`) + `ffmpeg` tersedia.
Temp render bisa GB-an → daemon mengarahkan `TMPDIR` ke data_dir.
Tidak siap → **fallback**: visual via Canva `image.generate`.

## Quality bar

- Teks min. 40px pada 1080x1920 agar terbaca di HP (UI rules).
- Verifikasi ffprobe wajib lolos (durasi ≈ total slide, 1080x1920) — gagal = render ulang.

## Trigger eskalasi — WAJIB ke Patria bila

- Render `delivery` terlalu lambat untuk volume produksi (butuh mesin khusus/cloud).

## Proof of work

Setiap render tercatat di `messages` (kind=`tool-run`) + file MP4 sebagai artefak.
