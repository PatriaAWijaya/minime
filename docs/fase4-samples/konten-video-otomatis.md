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

- Render Veo3 asli via fal.ai (butuh `MINIME_FAL_KEY` / Keychain akun `fal`).
- Fallback HyperFrames lokal (kode ada; render nyata belum dijalankan di uji ini).
- Upload media + `publish` non-Meta via Blotato (butuh `MINIME_BLOTATO_KEY` /
  Keychain akun `blotato` + approval approved).
