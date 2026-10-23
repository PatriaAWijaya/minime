# Rancangan: Reprioritisasi Model AI Minime (Meta-first)

**Tanggal:** 23 Okt 2026 · **Status:** final, disetujui Patria — siap implementasi Fase A (26 Okt)
**Konteks:** Meta Global AI Developer Hackathon (26 Okt–4 Nov). Bobot juri "Use of Meta AI Tooling" = 25% — integrasi Meta Model API yang nyata adalah syarat bersaing, bukan pemanis.

## 1. Kondisi saat ini (dari `app/src-tauri/src/providers.rs`)

- Provider `muse` SUDAH ada: `id="muse"`, `base_url="https://api.meta.ai/v1"`, `key_account="muse"`, satu model `muse:muse-spark-1.3`.
- Chain default tier Berbayar: **Muse → Gemini → DeepSeek (→ RouterKu)**. Tier Gratis: Ollama → OpenCode.
- Klien generik hanya bisa `POST {base_url}/chat/completions` — belum ada endpoint image/transcribe/segment.
- Image gen saat ini: `generate_image()` → stable-diffusion.cpp lokal via `127.0.0.1:8080`.
- TTS: VoiceStudio (local API) → yovoice. Video gen: Veo (Gemini) → fal.ai.

## 2. Prioritas baru per kapabilitas

| Kapabilitas | Prioritas 1 (baru) | Prioritas 2 | Prioritas 3 | Catatan |
|---|---|---|---|---|
| Chat/reasoning agen | `muse:muse-spark-1.3` | `muse:muse-wonder-1.0` (power mode) | Muse → Gemini → DeepSeek | Wonder hanya untuk orkestrasi kompleks (Mimi), bukan default — hemat biaya ($4/$20 vs $1,25/$4,25) |
| Image (poster, carousel, thumbnail, storyboard visual) | `muse:muse-image-1.0` ($0,01/gbr) | SD.cpp lokal (gratis/offline) | — | Muse Image = OpenAI-compatible `/v1/images/generations` |
| Transkripsi/caption | `muse:muse-voice-transcribe-1.0` ($0,18/jam) | (existing bila ada) | — | Baru — kapabilitas baru |
| Segmentasi video | `muse:sam-3.1` | — (tanpa fallback) | — | Baru — kapabilitas baru untuk video editor |
| TTS/voiceover | VoiceStudio → yovoice (TETAP) | — | — | Meta TIDAK punya model TTS di katalog |
| Video generation | Veo → fal.ai (TETAP) | — | — | Meta TIDAK punya video gen di katalog |

Chain default tier Berbayar menjadi: **Meta → Muse → Gemini → DeepSeek (→ RouterKu)**.
Tier Gratis tetap: Ollama → OpenCode.

## 3. Peta ke pipeline Media Studio

```
Intake brief ──→ [Mimi/CIO: spark-1.3]
    → Script ──→ [Kreatif: spark-1.3]
    → Storyboard (multimodal) ──→ [Kreatif: spark-1.3]
    → Visual/poster/carousel ──→ [muse-image-1.0 → fallback SD.cpp]
    → Video ──→ [Veo → fal.ai] (tetap)
    → Voiceover ──→ [VoiceStudio → yovoice] (tetap)
    → Caption ──→ [muse-voice-transcribe-1.0] (baru)
    → Segmentasi/edit ──→ [sam-3.1] (baru)
    → QC → Scheduling
```

## 4. Perubahan arsitektur (`providers.rs` + sekitarnya)

### 4.1 Registri model provider `muse` diperluas
```rust
models: vec![
    ModelInfo { id: "muse:muse-spark-1.3", label: "Muse Spark 1.3", capabilities: [Chat] },
    ModelInfo { id: "muse:muse-wonder-1.0", label: "Muse Wonder 1.0", capabilities: [Chat] },
    ModelInfo { id: "muse:muse-image-1.0", label: "Muse Image 1.0", capabilities: [Image] },
    ModelInfo { id: "muse:muse-voice-transcribe-1.0", label: "Muse Voice Transcribe", capabilities: [Transcribe] },
    ModelInfo { id: "muse:sam-3.1", label: "SAM 3.1", capabilities: [Segment] },
]
```

### 4.2 Chain default baru
`chain_for()` tier Berbayar: **sudah Meta-first di kode** (`muse → gemini → deepseek → routerku`, terverifikasi 23 Okt 2026 di `providers.rs:647-652`). Tidak ada perubahan logika fallback yang diperlukan — hanya registrasi model yang diperluas (4.1).

### 4.3 Endpoint baru di luar chat/completions
- `OpenAiCompatProvider::image(prompt, size)` → `POST {base_url}/v1/images/generations` → simpan file lokal, kembalikan path (pola sama seperti `generate_image()` SD.cpp agar UI tidak berubah).
- `OpenAiCompatProvider::transcribe(audio_path)` → endpoint transkripsi Meta → teks + timestamp.
- `OpenAiCompatProvider::segment(image/video_frame, prompt)` → `sam-3.1` → mask/box.
- Masing-masing dengan fallback chain sendiri (image → SD.cpp; transcribe → gagal jujur bila tak ada fallback).

### 4.4 Budget tracker ($150 kredit hackathon)
- Setiap request Meta dicatat: estimasi biaya = token × tarif (Spark 1.3: $1,25/$4,25; Wonder: $4/$20; Image: $0,01/gbr; Transcribe: $0,18/jam).
- Tampil di tab Cost: "Meta credit terpakai (estimasi)" + sisa. Peringatan di 80% ($120).
- Token usage diambil dari response API bila tersedia; bila tidak, estimasi dari panjang input/output.

### 4.5 Kredensial
- `key_account="muse"` SUDAH ada di `secrets.rs` — tidak perlu infrastruktur baru. UI Penyedia Model tambah kolom input key Meta.
- Aturan Patria tetap: key di Keychain, tidak di git; user publik wajib isi sendiri.

## 5. Fase implementasi (10 hari hackathon)

| Fase | Tanggal | Isi |
|---|---|---|
| A — Chat default | 26–27 Okt | Registri model Meta diperluas; chain default Meta-first; Spark 1.3 jadi default chat; uji E2E |
| B — Image + transcribe | 28–29 Okt | `muse-image-1.0` untuk Studio (fallback SD.cpp); transcribe untuk caption otomatis |
| C — Segment + power | 30–31 Okt | `sam-3.1` dasar untuk video editor; `wonder-1.0` sebagai opsi power mode Mimi |
| D — Hardening + demo | 1–3 Nov | Budget tracker UI, uji konsistensi + performa (nol error), rekam demo video |
| Submit | 4 Nov | Tenggat 9:00 PM ET |

## 6. Risiko & mitigasi

| Risiko | Mitigasi |
|---|---|
| Kredit $150 / API key hackathon belum diterima saat kickoff | Integrasi tetap jalan dengan key sendiri; tier `contributor` ($0,10/$0,20) sebagai opsi hemat eksplisit |
| Rate limit Meta | Fallback chain otomatis + backoff; tidak ada retry buta |
| Tier contributor murah tapi data boleh dipakai training | Default = Standard; contributor hanya bila user memilih eksplisit di UI |
| Spark Open (bila dipakai) mewajibkan notice "Built with Muse" | Tidak dipakai sebagai default; Glimmer 30B (Apache 2.0) untuk kebutuhan open-weight |
| SSE non-standar Spark (diketahui dari dokumentasi) | Tangani di client HTTP saat implementasi; uji dengan request nyata |

## 7. Yang TIDAK berubah

- Kredensial Patria tidak dibundel ke build publik.
- TTS tetap VoiceStudio → yovoice; video gen tetap Veo → fal.ai.
- Aturan build: fungsi benar-benar bekerja, nol error lolos, hasil dicatat ke log.
- Keputusan ini khusus prioritas model — tidak mengubah arsitektur agen, tema UI, atau model folder.

## 8. Vonis: Wonder sebagai power mode opsional (bukan default)

**Keputusan: SETUJU dengan usulan Patria — Wonder opsional, Spark default.**

Alasan:

1. **Biaya 3–5x lipat.** Wonder ($4/$20 per 1 jt token) vs Spark 1.3 ($1,25/$4,25). Dengan budget hackathon $150, Wonder sebagai default akan menghabiskan kredit 3–5x lebih cepat — berisiko kehabisan di tengah demo.
2. **Spark cukup untuk mayoritas tugas agen.** Chat, reasoning, penulisan naskah, dan orkestrasi standar tidak membutuhkan kapabilitas Wonder. Overkill = pemborosan.
3. **Power mode tepat guna.** Wonder diaktifkan eksplisit hanya untuk orkestrasi kompleks Mimi (multi-step reasoning, planning lintas agen) — di sinilah selisih kualitasnya membayar selisih harganya.
4. **Selaras prinsip hemat biaya.** Default hemat, upgrade sadar-biaya — pola yang sama dengan tier Gratis (Ollama) vs Berbayar.

Implementasi: toggle "Power mode (Wonder)" di Pengaturan → Model / per-agen; default OFF. Budget tracker menampilkan pemakaian terpisah Spark vs Wonder agar Patria bisa melihat trade-off biaya secara nyata.
