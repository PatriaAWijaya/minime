# Framework Workflow Video AI — Media Studio Minime

> Diadopsi 23 Okt 2026 atas perintah Patria. Pola ditiru dari MoneyPrinterTurbo
> (harry0703, MIT — ide & arsitektur saja, kode ditulis ulang clean-room di Rust).

## Alur Kerja — Terintegrasi ke Tab Media Studio

> Revisi 23 Okt 2026 (perintah Patria): Video AI BUKAN menu tersendiri,
> melainkan menyatu dengan workflow Media Studio yang sudah ada.

| Tahap Video AI | Tab Media Studio | Keterangan |
|----------------|------------------|------------|
| Topik | **Brief** (dulu "Input User") | Input topik/brief dari user atau agen Kreatif |
| Naskah | **Script** | Naskah narasi |
| Suara | **Studio Suara** | TTS via VoiceStudio → audio narasi |
| Footage | **Studio Video** § Footage | Stok (Pixabay/Pexels) atau AI text-to-video |
| Subtitle | **Studio Video** § Subtitle | Otomatis via Whisper |
| Render (dulu "Rakit") | **Studio Video** § Render | FFmpeg otomatis atau HyperFrames kustom |

### Command backend per tahap

| Tahap | Command backend |
|-------|-----------------|
| Suara | `voicestudio_tts` / `yovoice_tts` |
| Footage (stok) | `pixabay_search_videos`, `pexels_search_videos` |
| Footage (AI) | `text_to_video` (Wan 3.0 / LTX-Video via fal.ai) |
| Subtitle | `generate_subtitle` (Whisper) |
| Render | `assemble_video_ai` (FFmpeg) / `hyperframes_assemble` |

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

- UI: terintegrasi di `app/src/src/components/StudioView.svelte`
  - Tab "Brief" (dulu "Input User"), tab "Script", tab "Studio Suara" (tetap)
  - Tab "Studio Video": section Footage + Subtitle + Render (di bawah TimelineEditor)
- Backend: `app/src-tauri/src/providers.rs` (`assemble_video_ai`, `text_to_video`, `hyperframes_assemble`)
- Backend: `app/src-tauri/src/social.rs` (`pexels_search_videos`, `set_pexels_key`, `set_video_ai_key`)
- Skill agen: `skills/media-studio/SKILL.md` (pipeline Kreatif)
- Riset: `~/workspace/kantor-ai-agent/referensi/moneyprinterturbo.md`,
  `~/workspace/kantor-ai-agent/referensi/text-to-video-options.md`
