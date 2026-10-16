---
name: voice-studio
description: Tool skill — VoiceStudio untuk voiceover TTS Bahasa Indonesia; di-wire daemon Fase 4 (fallback edge-tts).
owner_agent: creative-director
version: 1.0.0
example-prompt: Buatkan voiceover Bahasa Indonesia untuk naskah ini.
---

# voice-studio

> Tool skill (bukan playbook). Dieksekusi oleh `minime-daemon` via REST.
> **Instalasi di MacBook Patria masih pending** (panduan: `docs/voice-studio.md`).
> Selama belum siap → **fallback otomatis ke `edge-tts` CLI** (bila terinstal),
> dan hasilnya dicatat jujur sebagai fallback.

## Kapan dipakai

Bila tugas butuh voiceover/narasi audio (reels, explainer) dalam Bahasa Indonesia.

## Operasi (via `tools.run` / task `tool:`)

| op | params | hasil |
|---|---|---|
| `tts` | `text` (wajib), `lang` (default `id`), `voice` | file MP3 lokal → `artifact_path` + `engine` (voice-studio / edge-tts fallback) |

## Readiness

`tools.status` → `voice-studio`: siap bila service terjangkau DAN kunci tersedia
(`secrets.set` account=`voice-studio`). Tidak siap → fallback edge-tts.

## Quality bar

- Teks dibacakan apa adanya — jangan ubah naskah tanpa instruksi.
- Bahasa Indonesia: pastikan voice `id-ID-*` bila pakai edge-tts.

## Trigger eskalasi — WAJIB ke Patria bila

- Kualitas suara tidak layak siar untuk kebutuhan komersil.
- Perlu instalasi VoiceStudio di MacBook (minta Patria ikuti `docs/voice-studio.md`).

## Proof of work

Setiap pemanggilan tercatat di `messages` (kind=`tool-run`), termasuk engine yang dipakai.
