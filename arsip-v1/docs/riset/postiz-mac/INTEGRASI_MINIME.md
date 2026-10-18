# Menghubungkan postiz-app ke Minime

Setelah postiz-app berjalan di MacBook (lihat `INSTALL_MAC.md`),
Minime memanggilnya sebagai **layanan eksternal via API** — pola yang
sama seperti nautilus_trader (kode postiz tidak ditempel ke Minime,
menjaga batasan lisensi AGPL-3.0).

## 1. Informasi koneksi

| Item | Nilai |
|---|---|
| Base URL API publik | `http://localhost:4007/public/v1` |
| Autentikasi | Header `Authorization: Bearer <API_KEY>` |
| Format API key | hex 64 char TANPA prefix (dashboard postiz → Settings → Agents). Yang berawalan `pos_` adalah token OAuth App, BUKAN API key pribadi (koreksi 18 Okt 2026) |

## 2. Simpan kredensial (di MacBook Patria)

JANGAN tulis API key di file repo. Simpan di macOS Keychain:

```bash
# Simpan URL + key (ganti dengan nilai aslimu)
security add-generic-password -s minime-postiz -a url -w "http://localhost:4007"
security add-generic-password -s com.minime.app -a postiz -w   # tempel API key hex 64 char
```

Untuk development/testing sementara, boleh pakai env:

```bash
export MINIME_POSTIZ_URL="http://localhost:4007"
export MINIME_POSTIZ_KEY="<hex-64-char>"
```

> Nama env final (`MINIME_POSTIZ_URL`, `MINIME_POSTIZ_KEY`) adalah
> konvensi yang disiapkan untuk wiring daemon Minime tahap berikut.
> Daemon **belum** membaca keduanya — wiring tool `postiz` menyusul
> setelah instalasi di MacBook terbukti jalan.

## 3. Pola draft → approval → schedule (rule #3)

Alur distribusi Minime yang wajib dijaga (aturan no-auto-post):

```
Minime (agen)                postiz-app                  Patria
    |                            |                          |
    |-- POST /posts -------------->|                          |
    |   { type: "draft",          |                          |
    |     caption, media,         |                          |
    |     channels: [...] }       |                          |
    |                            |-- draft tersimpan -------->|
    |                            |   (tampil di dashboard)  |
    |<-- draft_id ----------------|                          |
    |                                                    |
    |   [Patria review di approval inbox Minime ATAU      |
    |    dashboard postiz → tekan Setujui]                |
    |                                                    |
    |-- PATCH /posts/{id} -------->|                          |
    |   { status: "schedule",     |                          |
    |     scheduled_at: ... }     |                          |
    |                            |-- publish sesuai jadwal -->|
```

Poin penting:
- Agen Minime **hanya boleh** membuat `draft` (`type: "draft"`).
- Perubahan status draft → schedule **hanya** setelah approval
  Patria tercatat (di approval inbox Minime).
- Tidak ada pemanggilan publish langsung dari kode Minime.

## 4. Uji koneksi manual (curl)

```bash
# 1. Cek API hidup
curl -s -H "Authorization: Bearer $MINIME_POSTIZ_KEY" \
  http://localhost:4007/public/v1/integrations | head -c 300

# 2. Buat draft (contoh — sesuaikan payload dengan docs postiz)
curl -s -X POST -H "Authorization: Bearer $MINIME_POSTIZ_KEY" \
  -H "Content-Type: application/json" \
  -d '{"type":"draft","caption":"Tes draft dari Minime"}' \
  http://localhost:4007/public/v1/posts
```

Dokumentasi API interaktif (Swagger) tersedia di dashboard postiz
bila diaktifkan — payload `POST /posts` cukup kompleks (caption per
channel, media, dsb.); gunakan wizard di dashboard untuk melihat
contoh payload yang benar.

## 5. Checklist sebelum wiring daemon

- [x] postiz-app jalan di MacBook (`docker compose ps` semua healthy) — 17 Okt 2026
- [ ] 1 akun platform terhubung (mis. Threads atau Instagram)
- [x] API key (hex 64 char) dibuat dan tersimpan di Keychain `com.minime.app` / akun `postiz` (18 Okt 2026; koreksi format)
- [ ] 1 draft berhasil dibuat via curl dan tampil di dashboard
- [x] ~~Blotato **tetap dipertahankan** sampai draft nyata via postiz terbukti~~ —
      DIGANTI 17 Okt 2026 (perintah Patria "ganti blotato dengan postiz"):
      tool daemon `postiz` sudah menggantikan `blotato` TOTAL
      (`daemon/src/tools/postiz.rs`; `blotato.rs` dihapus, pulih via git history)

Wiring tool `postiz` di daemon Minime SELESAI dikerjakan (gantikan `blotato`
untuk 6 platform non-Meta; jalur Meta langsung tetap dipertahankan).
Live test (kirim draft nyata via API) menunggu: 1 channel terhubung +
API key Patria di Keychain MacBook.
