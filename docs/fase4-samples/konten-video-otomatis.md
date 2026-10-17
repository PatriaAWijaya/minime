# Uji wiring pipeline `konten-video-otomatis` (mock)

> **CATATAN JUJUR:** ini bukti wiring MOCK (`MINIME_MOCK_LLM=1`), bukan produksi
> video AI beneran. LLM diganti mock provider; `veo3.render` mengembalikan hasil
> mock eksplisit (tidak memanggil fal.ai, tidak membuat MP4); `blotato.draft`
> membuat bundle draft lokal (upload dilewati — API key belum diset).
> Render Veo3 asli dan upload Blotato menunggu API key Patria via Keychain.

## Perintah uji

```bash
MINIME_DATA_DIR=/tmp/minime-videotest MINIME_MOCK_LLM=1 ./daemon/target/debug/minime-daemon
# via IPC (JSON-lines di minime.sock):
{"id":1,"method":"pipelines.run","params":{"pipeline":"konten-video-otomatis"}}
```

## Hasil (17 Okt 2026)

| Stage | Agen | Hasil |
|---|---|---|
| skrip | creative-director | mock OK — `PLAYBOOK AKTIF: storytelling, copywriting, content` |
| render | creative-director (tool `veo3.render`) | mock OK — `{"mock":true, ...}` eksplisit |
| qc | qc | mock OK — `SKOR: 80` |
| distribusi | creative-director (tool `blotato.draft`) | OK — 9 bundle draft lokal dibuat |

- Run berhenti di **`waiting-approval`** pada stage `distribusi` (gate Patria).
- 1 approval `pending` tercatat dengan **`qc_score=80`** (diambil dari output
  stage QC via `qc_score_from_any_stage` — perbaikan runner di commit ini).
- 9 file draft: `drafts/blotato-draft-{instagram,youtube,tiktok,facebook,threads,twitter,linkedin,bluesky,pinterest}-<ts>.json`
  — semua `status: draft`; bundle IG/FB/Threads mencantumkan
  "DITOLAK di kode: publikasi ke Meta hanya manual oleh Patria (rule #3)".

## Guard keamanan terverifikasi via `tools.run`

1. `blotato.publish` platform `instagram` → **DITOLAK** ("publikasi ke 'instagram'
   diblokir di kode (rule #3)").
2. `blotato.publish` platform `youtube` tanpa approval valid → **DITOLAK**
   ("approval tidak ditemukan" / bukan 'approved').
3. `tools.status` → `veo3` dan `blotato` terdaftar, `ready=false`,
   `state=not_configured` dengan pesan jujur meminta key ke Patria.

## Yang belum teruji (butuh kunci / MacBook)

- Render Veo 3.1 asli via Gemini API (butuh Keychain akun `gemini` / `MINIME_GEMINI_KEY`).
- Render Veo3 asli via fal.ai (butuh `MINIME_FAL_KEY` / Keychain akun `fal`) — kini fallback.
- Upload media + `publish` non-Meta via Blotato (butuh `MINIME_BLOTATO_KEY` /
  Keychain akun `blotato` + approval approved).

---

## 17 Okt 2026 — Rantai backend: Gemini API (primer) → fal.ai → HyperFrames

Perintah Patria: primer pindah dari fal.ai ke **Gemini API (Veo 3.1)**.

**Endpoint terverifikasi** (docs resmi Google AI for Developers,
`ai.google.dev/gemini-api/docs/veo`):
- `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:predictLongRunning`
  (header `x-goog-api-key`, body `{"instances":[{"prompt"}],"parameters":{...}}`)
  → `{"name": "operations/..."}` → `GET {BASE}/{name}` → cek `.done`
  → `.response.generateVideoResponse.generatedSamples[0].video.uri` → unduh MP4.
- Model IDs (Gemini API): `veo-3.1-fast-generate-preview` (default tool),
  `veo-3.1-generate-preview`. ID `-001` adalah Vertex AI, bukan Gemini API.
- Parameters: `aspectRatio` 16:9/9:16; `durationSeconds` "4"/"6"/"8"
  (wajib "8" untuk 1080p); `resolution` 720p/1080p; `personGeneration`
  "allow_all" (text-to-video). Latensi 11 dtk–6 mnt (docs).

**Uji wiring mock** (`MINIME_MOCK_LLM=1`, daemon terisolasi) — LULUS:
- `tools.status` → `veo3` label "Veo 3.1", `ready=false`, `state=not_configured`,
  detail jujur menyebut key Gemini DAN key fal.ai.
- `tools.run veo3.render` (mock) → `{"mock":true, "backend_chain":["gemini","fal","hyperframes"]}`.
- `blotato.publish` instagram → **tetap DITOLAK** (guard tidak tersentuh).

**Uji rantai fallback nyata** (tanpa mock, tanpa key) — LULUS (perilaku jujur):
- `gemini` → skipped (key belum diset) → `fal` → skipped (key belum diset) →
  `hyperframes` → dicoba, gagal jujur (ruang /tmp habis — 0,2 GB; lingkungan,
  bukan bug) → error akhir memuat SELURUH `attempts`. Tidak ada kegagalan diam-diam.

**Bug ditemukan & diperbaiki saat uji:** `fallback_hyperframes` panic
(index out of range) untuk prompt < 24 kata — `start` tidak di-clamp ke
panjang slice. Sudah diperbaiki (clamp `start` juga).

**Perkiraan biaya** (harga publik Google, BUKAN janji): klip 8 dtk 1080p
tier Fast ≈ $0,12/dtk → **≈ $0,96/klip**. Tercatat sebagai `est_cost_usd`
di hasil render + di `docs/status.md`.

**Panduan routing produksi video** (keputusan Patria: Veo & OpenCreator paralel):
ditambahkan sebagai §E di `skills/content/SKILL.md` — Veo untuk footage
generatif sinematik/b-roll AI; OpenCreator untuk naskah → video + voiceover
Indonesia (menunggu kunci); HyperFrames/manimgx fallback lokal tanpa kunci.
