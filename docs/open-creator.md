# OpenCreator — Kontrak Sidecar (untuk implementasi/packaging)

> Status Fase 4: wiring daemon selesai; **live test menunggu API key dari Patria**
> dan keputusan terbuka arsitektur §16.9 (kunci LLM OpenAI-compatible ATAU wiring Gemini).

## Model sidecar

Binary `open-creator` diletakkan **di samping binary `minime-daemon`**
(mis. `Minime.app/Contents/MacOS/open-creator`), atau path-nya di-set via
env `MINIME_OPENCREATOR_BIN`. Daemon tidak me-link library OpenCreator —
hanya memanggil CLI-nya (isolasi proses, pola yang sama dipakai OpenBot
untuk sidecar-nya).

## Kontrak CLI

```
open-creator --op <op> --params-json '<json>'
```

- stdout: **satu objek JSON** (boleh ada preamble manusia — daemon mencari `{` pertama).
- stderr: log manusia (diabaikan bila exit 0).
- Exit non-0 → dianggap gagal; 300 char pertama stderr jadi pesan error.

### op yang didukung daemon

| op | params (contoh) | hasil JSON (contoh) |
|---|---|---|
| `render` | `{"script": "...", "format": "9:16", "voice": "id"}` | `{"output_path": "/abs/path/video.mp4", ...}` |
| `voices` | `{}` | `{"voices": [...]}` |
| `script` | `{"brief": "..."}` | `{"script": "..."}` |

Bila sidecar mengembalikan `output_path`, daemon otomatis mendaftarkannya
sebagai `artifact_path` (artefak deliverable).

## Kredensial

API key via macOS Keychain: `secrets.set` dengan `account="open-creator"`
(atau env `MINIME_OPENCREATOR_KEY` untuk dev). Daemon memvalidasi keberadaan
kunci di readiness check — tanpa kunci, op ditolak dengan pesan jelas
(bukan gagal diam-diam).

## Keputusan yang masih dibutuhkan Patria (arsitektur §16.9)

1. Kunci LLM OpenAI-compatible untuk OpenCreator, ATAU
2. Wiring Gemini + uji 1 video.

Sampai diputuskan, video dirender via **HyperFrames** (fallback bawaan,
tanpa kunci, terverifikasi Fase 4).
