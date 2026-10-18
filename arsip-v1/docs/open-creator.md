# OpenCreator — Kontrak Sidecar (untuk implementasi/packaging)

> Status: wiring daemon selesai. **Keputusan LLM: Gemini** (Patria, 17 Okt 2026 —
> bukan kunci OpenAI-compatible terpisah). Live test menunggu API key Gemini
> dari Patria + penyetelan provider Gemini di aplikasi OpenCreator (sekali,
> manual di MacBook: Settings → AI Services → Gemini).

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

OpenCreator memakai **kunci Gemini** (keputusan Patria 17 Okt 2026):

- **Daemon:** memvalidasi keberadaan kunci Gemini via Keychain akun `gemini`
  (atau env `MINIME_GEMINI_KEY` untuk dev) di readiness check — tanpa kunci,
  op ditolak dengan pesan jelas (bukan gagal diam-diam). Tidak ada lagi
  akun Keychain `open-creator` terpisah.
- **Aplikasi OpenCreator** (sidecar): penyetelan provider Gemini dilakukan
  **manual sekali** di MacBook — buka OpenCreator → Settings → AI Services →
  pilih Gemini → masukkan API key yang sama.

## Keputusan (sudah diputuskan Patria 17 Okt 2026)

~~1. Kunci LLM OpenAI-compatible untuk OpenCreator, ATAU~~ → **DITOLAK**
**2. Wiring Gemini** → **DIPILIH**. Uji 1 video menunggu kunci Gemini + instalasi
OpenCreator di MacBook.

Fallback tetap: video dirender via **HyperFrames** (lokal, tanpa kunci,
terverifikasi Fase 4) bila OpenCreator belum siap.
