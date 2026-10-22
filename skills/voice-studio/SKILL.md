---
name: voice-studio
description: Tool skill — VoiceStudio sebagai engine TTS Bahasa Indonesia via local API; fallback yovoice lalu edge-tts.
owner_agent: creative-director
version: 2.0.0
example-prompt: Buatkan voiceover Bahasa Indonesia untuk naskah ini.
---

# voice-studio

> Tool skill (bukan playbook). Dieksekusi via Tauri command backend (`voicestudio_*`).
> **Engine utama: VoiceStudio** (debpalash, AGPL-3.0, v0.5.6) via **local API saja** —
> proses terpisah di `127.0.0.1:3900`, TIDAK di-bundle ke Minime (pola sama seperti Pireel;
> keputusan Patria 22 Okt 2026 19:20, override penolakan AGPL 18:45).
> JANGAN sentuh kode VoiceStudio.

## Kapan dipakai

Bila tugas butuh voiceover/narasi audio (reels, explainer) dalam Bahasa Indonesia,
atau voice cloning dari sampel suara yang diizinkan.

## Rantai engine (otomatis, catat yang dipakai)

1. **VoiceStudio** — bila backend terjangkau (`voicestudio_status` → reachable).
   Backend HANYA jalan saat VoiceStudio.app dibuka; bila mati, UI menawarkan tombol "Buka VoiceStudio".
2. **yovoice** (fallback) — CLI lokal `~/.minime/yovoice/`, 100% offline.
3. **edge-tts** (fallback terakhir) — bila terinstal.

## Operasi (Tauri commands)

| command | params | hasil |
|---|---|---|
| `voicestudio_status` | — | JSON `{reachable, version, engine}` — engine = voice-studio / yovoice / none |
| `voicestudio_voices` | — | JSON array `[{voice_id, name, type, language}]` |
| `voicestudio_tts` | `text`, `voice`, `speed`, `outputName` | JSON `{path, engine}` — file WAV lokal; fallback otomatis ke yovoice |
| `voicestudio_clone_tts` | `text`, `refAudioPath`, `language`, `outputName` | JSON `{path, engine}` — via POST /generate (ref_audio); fallback yovoice `--reference` |
| `voicestudio_open` | — | Buka VoiceStudio.app (macOS) |

API VoiceStudio yang dipakai: `POST /v1/audio/speech` (OpenAI-compatible, response_format wav),
`GET /v1/audio/voices`, `POST /generate` (multipart, untuk cloning).

## Readiness

`voicestudio_status` → `reachable: true` = siap penuh. `engine: "yovoice"` = siap dengan fallback.
`engine: "none"` = tidak ada engine — eskalasi ke Patria.

## Quality bar

- Teks dibacakan apa adanya — jangan ubah naskah tanpa instruksi.
- Bahasa Indonesia: kirim `language: "id"` ke API.
- Voice cloning HANYA dari sampel suara yang user punya izinnya.

## Trigger eskalasi — WAJIB ke Patria bila

- Kualitas suara tidak layak siar untuk kebutuhan komersil.
- Tidak ada engine yang tersedia (VoiceStudio belum dibuka DAN yovoice belum terinstall).
- Model VoiceStudio belum terdownload (setup satu-kali di aplikasi VoiceStudio).

## Proof of work

Setiap pemanggilan mengembalikan `engine` yang dipakai — catat jujur di hasil.
