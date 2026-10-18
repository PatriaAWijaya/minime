# Uji wiring tool `meta` — koneksi Meta langsung (mock)

> **CATATAN JUJUR:** ini bukti wiring MOCK (`MINIME_MOCK_LLM=1`), bukan koneksi
> Meta beneran. Token Meta belum diset; op `accounts` mengembalikan data mock;
> op `draft` membuat bundle lokal tanpa memanggil Graph API/Threads API.
> Live test (verifikasi akun + draft dari pipeline nyata) menunggu token Meta
> dari Patria — lihat `docs/koneksi-meta.md`.

## Perintah uji

```bash
MINIME_DATA_DIR=/tmp/meta-test MINIME_MOCK_LLM=1 MINIME_REPO_DIR=~/workspace/minime \
  ./daemon/target/debug/minime-daemon
# via IPC (JSON-lines di minime.sock):
{"method":"tools.status","params":{}}
{"method":"tools.run","params":{"tool":"meta","op":"draft","agent_id":"creative-director",
  "params":{"caption":"Uji draft Meta","hashtags":["minime"],
  "platforms":["instagram","facebook","threads"]}}}
{"method":"tools.run","params":{"tool":"meta","op":"publish",
  "params":{"platform":"instagram"}}}
```

## Hasil (17 Okt 2026)

| Uji | Hasil |
|---|---|
| `tools.status` memuat `meta` | ✅ PASS (`not_configured` — jujur, token belum diset) |
| `meta` op `draft` 3 platform | ✅ PASS — 3 bundle lokal: `drafts/meta-draft-{instagram,facebook,threads}-<ts>.json` (caption + media + hashtag + panduan posting manual) |
| `meta` op `publish` | ✅ DITOLAK di kode (`DITOLAK: op 'publish' menyerupai publikasi — auto-posting ke Meta dilarang (rule #3)`) |
| `meta` op `post` (varian) | ✅ DITOLAK di kode |
| `meta` op `accounts` (mock) | ✅ PASS — read-only, data mock |

## Arsitektur

Hybrid (keputusan Patria 17 Okt 2026): tool `meta` melayani Instagram/Facebook/Threads
(koneksi langsung Graph API + Threads API); tool `blotato` tetap melayani 6 platform
non-Meta (YouTube, TikTok, Twitter/X, LinkedIn, Bluesky, Pinterest). Pipeline
`konten-video-otomatis` v3: stage `distribusi-meta` (tool meta) → stage
`distribusi-nometa` (tool blotato) → approval Patria. Semua DRAFT-ONLY.
