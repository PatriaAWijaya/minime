# Uji wiring tool `postiz` (mock) — pengganti Blotato

> **CATATAN JUJUR:** ini bukti wiring MOCK (`MINIME_MOCK_LLM=1`), bukan kirim
> nyata ke postiz-app. HTTP ke `http://localhost:4007` dilewati; `draft`
> membuat bundle draft lokal; `schedule` mengembalikan hasil mock eksplisit
> setelah approval terverifikasi di DB. Kirim nyata menunggu MacBook Patria
> (postiz-app + API key `pos_...` di Keychain).

## Perintah uji

```bash
MINIME_DATA_DIR=/tmp/minime-postiztest MINIME_REPO_DIR=~/workspace/minime \
  MINIME_MOCK_LLM=1 ./daemon/target/debug/minime-daemon
# via IPC (JSON-lines di minime.sock):
{"id":1,"method":"tools.status","params":{}}
{"id":2,"method":"tools.run","params":{"tool":"postiz","op":"draft","params":{"platforms":["youtube","tiktok"],"caption":"Uji mock Minime","hashtags":["#minime"]}}}
```

## Hasil (17 Okt 2026) — 10/10 LULUS

| # | Kasus | Hasil |
|---|---|---|
| 1 | `tools.status` | `postiz` terdaftar, `ready=false`, `state=not_configured` (jujur: key belum diset di VM); `blotato` HILANG dari daftar |
| 2 | `postiz.draft` youtube+tiktok (mock) | OK — 2 bundle lokal `drafts/postiz-draft-{youtube,tiktok}-<ts>.json`, `status: draft`, `postiz_id: mock-post-*` |
| 3 | `postiz.draft` instagram | **DITOLAK** — "platform Meta — distribusi Meta hanya via tool `meta` (rule #3)" |
| 4 | `postiz.schedule` youtube tanpa `approval_id` | **DITOLAK** — "`params.approval_id` wajib" |
| 5 | `postiz.schedule` youtube + approval `approved` (mock-appr-001) | OK — `{"mock":true, "platform":"youtube", "scheduled_at":"2026-10-18T10:00:00+07:00"}` |
| 5b | `postiz.schedule` youtube + approval tak dikenal | **DITOLAK** — "approval 'tidak-ada' tidak ditemukan" |
| 6 | `postiz.schedule` instagram + approval approved | **DITOLAK** — tetap diblokir (Meta) walau approval ada |
| 7 | `postiz.accounts` tanpa key | Error jujur — "API key postiz belum diset" (tidak gagal diam-diam) |
| 8 | op `publish` (tak dikenal) | **DITOLAK** — "op postiz tidak dikenal: 'publish' (draft/schedule/accounts/list)" |
| 9 | `postiz.draft` platform `twitter` | OK — dinormalisasi jadi `x` (identifier provider postiz) |

## Endpoint postiz yang dipakai (terverifikasi dari kode postiz-app)

- `GET /public/v1/is-connected` → readiness check (key valid + layanan hidup)
- `GET /public/v1/integrations` → petakan identifier provider → integration id (untuk `accounts` + resolusi channel saat draft/schedule)
- `POST /public/v1/posts` type `draft` → op `draft`; type `schedule` + `date` → op `schedule`
- `GET /public/v1/posts` → op `list`
- Catatan: Public API postiz **tidak punya** PATCH draft→schedule; op `schedule` membuat post terjadwal dari bundle draft (perilaku ini dicatat di kode).

## Yang belum teruji (butuh MacBook Patria)

- Kirim draft nyata: `POST /public/v1/posts` ke `http://localhost:4007` dengan Bearer `pos_...` asli + 1 channel terhubung (butuh API key di Keychain + developer app per platform).
- Resolusi integration id nyata via `GET /public/v1/integrations`.
- Validasi `type: "schedule"` oleh server postiz (settings per provider).
