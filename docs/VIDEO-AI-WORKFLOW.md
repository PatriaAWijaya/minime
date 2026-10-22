# Framework Workflow Video AI — Media Studio Minime

> Diadopsi 23 Okt 2026 atas perintah Patria. Pola ditiru dari MoneyPrinterTurbo
> (harry0703, MIT — ide & arsitektur saja, kode ditulis ulang clean-room di Rust).

## Alur Kerja (6 Tahap)

```
Topik → Naskah → Suara → Footage → Subtitle → Rakit
```

| Tahap | Input | Output | Command backend |
|-------|-------|--------|-----------------|
| 1. Topik | Teks topik (dari user langsung atau via agen Kreatif) | Topik | — (UI) |
| 2. Naskah | Topik | Naskah narasi (teks) | — (UI, agen Kreatif via skill) |
| 3. Suara | Naskah | File audio narasi (MP3/WAV) | `voicestudio_tts` / `yovoice_tts` |
| 4. Footage | Keyword / prompt | Daftar file video | `pixabay_search_videos`, `pexels_search_videos`, `text_to_video` |
| 5. Subtitle | Audio narasi | File SRT / burn-in | `generate_subtitle` (Whisper) |
| 6. Rakit | Footage + audio + subtitle | MP4 final | `assemble_video_ai` / `hyperframes_assemble` |

## Jalur Footage (dipilih per tahap 4)

### Jalur A — Stok (default, gratis, tanpa GPU)
- **Pixabay** (`pixabay_search_videos`) — sudah ada, key di `minime.pixabay_key`
- **Pexels** (`pexels_search_videos`) — baru, gratis 200 req/jam, key di `minime.pexels_key`
- Pola MoneyPrinterTurbo: ekstrak keyword per segmen naskah → cari footage per keyword

### Jalur B — AI Text-to-Video (berbayar, API key milik user)
- **Wan 3.0** via fal.ai (`fal-ai/wan/v2.6/text-to-video`) — ±$0.10/detik, 720p
- **LTX-Video** via fal.ai (`fal-ai/ltx-video`) — ±$0.09/detik, 720p
- Key di keychain: `minime.wan_key`, `minime.ltx_key` (via `set_video_ai_key`)
- ❌ **Sora TIDAK tersedia** — OpenAI menutup Videos API permanen Sep 2026

### Jalur C — Self-host (power user, opsional)
- LTX-2.5 open weights, 32GB VRAM (FP8) — dokumentasi terpisah
- Pola: proses eksternal via API (seperti VoiceStudio/Pireel), tidak di-bundle

## Mode Perakitan (dipilih per tahap 6)

### Otomatis — FFmpeg (`assemble_video_ai`)
Gaya MoneyPrinterTurbo: concat footage → scale/crop ke aspek → tempel narasi
(`-shortest`) → burn subtitle. Cepat, konsisten, tanpa GPU.

### Kustom — HyperFrames (`hyperframes_assemble`)
Untuk user yang ingin merakit tampilan sendiri: komposisi HTML (teks animasi,
layout kustom) → render MP4 via HyperFrames CLI. Aset harus lokal
(correctness gate HyperFrames memblokir CDN).

## Prinsip

1. **Naskah → audio adalah master clock** — durasi video mengikuti durasi narasi
2. **Draft-only** — tidak ada auto-publish (aturan keras Minime)
3. **Kredensial milik user** — API key di OS Keychain, tidak ikut git
4. **Local-first** — Jalur A jalan 100% offline (setelah footage terunduh);
   TTS lokal (VoiceStudio/yovoice/arboTTS), subtitle Whisper lokal
5. **Biaya transparan** — Jalur B selalu tampilkan estimasi sebelum generate

## File Terkait

- UI: `app/src/src/components/VideoPipeline.svelte`
- Tab: Media Studio → "Video AI" (`StudioView.svelte`)
- Backend: `app/src-tauri/src/providers.rs` (`assemble_video_ai`, `text_to_video`, `hyperframes_assemble`)
- Backend: `app/src-tauri/src/social.rs` (`pexels_search_videos`, `set_pexels_key`, `set_video_ai_key`)
- Skill agen: `skills/media-studio/SKILL.md` (pipeline Kreatif)
- Riset: `~/workspace/kantor-ai-agent/referensi/moneyprinterturbo.md`,
  `~/workspace/kantor-ai-agent/referensi/text-to-video-options.md`
