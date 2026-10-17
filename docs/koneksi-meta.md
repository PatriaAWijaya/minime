# Koneksi Meta langsung — panduan setup untuk Patria

> Tool `meta` di daemon Minime (IG/FB/Threads, DRAFT-ONLY) membutuhkan token Meta
> milikmu. Dokumen ini daftar langkah yang harus kamu lakukan — Mimi tidak bisa
> melakukannya untukmu (butuh login akun Meta-mu di developers.facebook.com).

## Yang perlu kamu siapkan

### 1. Buat Meta App (developers.facebook.com)

1. Buka [developers.facebook.com/apps](https://developers.facebook.com/apps), login
   dengan akun Facebook-mu.
2. **Create App** → pilih use case **Business** → beri nama mis. `Minime`.
3. Di dashboard app, tambahkan produk/izin:
   - **Instagram Graph API** (untuk IG @patriabicara dkk — akun harus **Bisnis/Kreator**, bukan personal)
   - **Facebook Login** + Pages API (untuk FB Page di portfolio 'Branding, Marketing & AI'-mu)
4. Untuk **Threads**: buat app terpisah (atau bagian app yang sama) dengan use case
   **"Access the Threads API"** — catat: akan ada 2 App ID; untuk Threads pakai
   **Threads App ID** + secret-nya.

### 2. Permission yang diminta (saat OAuth)

| Platform | Permission | Untuk apa |
|---|---|---|
| Instagram | `instagram_business_basic` | baca info akun |
| Instagram | `instagram_business_content_publish` | (cadangan — saat ini tidak dipakai; draft-only) |
| Facebook | `pages_show_list` | daftar FB Page |
| Facebook | `pages_manage_posts` | (cadangan — saat ini tidak dipakai; draft-only) |
| Facebook | `pages_read_engagement` | baca info Page |
| Threads | `threads_basic` | baca profil Threads |
| Threads | `threads_content_publish` | (cadangan — saat ini tidak dipakai; draft-only) |

Karena Minime **draft-only** (tidak pernah publish via API), permission publish di
atas bersifat cadangan untuk masa depan. Yang wajib jalan hari ini: yang `_basic`
+ `pages_show_list`.

### 3. App Review

- Selama yang memakai app adalah **kamu sendiri** (ditambahkan sebagai tester/admin
  di App Roles), permission langsung aktif **tanpa App Review**.
- App Review (2–4 minggu per submission) baru dibutuhkan bila app dipakai akun lain.

### 4. Ambil token & simpan ke Minime

1. Di Graph API Explorer (atau via OAuth flow), generate **user access token**
   berumur panjang dengan permission di atas.
2. Simpan ke Keychain via Minime (write-only, tidak pernah ditampilkan):
   - lewat UI: Settings → Secrets → `secrets.set` account `meta`, atau
   - `MINIME_META_TOKEN` di env (hanya untuk dev).
3. Verifikasi: `tools.status` → tool `meta` menjadi `ready`; `tools.run`
   tool=`meta` op=`accounts` menampilkan FB Page + IG Bisnis + Threads-mu
   (read-only).

### 5. Syarat akun

- Instagram: akun **Bisnis atau Kreator** (akun personal tidak bisa).
- Threads: akun **publik**.
- Media yang nanti dipublish manual harus bisa diakses publik (syarat Meta).

## Endpoint resmi yang dipakai tool `meta` (referensi)

Terverifikasi dari dokumentasi Meta (Okt 2026); tool saat ini hanya memakai yang
read-only (`accounts`), sisanya didokumentasikan untuk masa depan bila kamu
membuka kunci publish:

- **Instagram:** `POST /{ig-user-id}/media` (container: `media_type=REELS`,
  `video_url`, `caption`) → poll `status_code` → `POST /{ig-user-id}/media_publish`
  (`creation_id`). Limit 100 post/24 jam.
- **Facebook Pages:** `GET /{user-id}/accounts` → `POST /{page-id}/feed|photos|videos|video_reels`.
- **Threads:** `POST /{threads-user-id}/threads` → tunggu ±30 dtk →
  `POST /{threads-user-id}/threads_publish` (`creation_id`). Host: `graph.threads.com`.

## Status

- [x] Tool `meta` ter-wire di daemon (draft/accounts/readiness) — 17 Okt 2026
- [x] Uji wiring mock lolos (`docs/fase4-samples/konten-meta-langsung.md`)
- [x] Pipeline `konten-video-otomatis` v3: stage `distribusi-meta` (tool meta)
- [ ] Token Meta dari Patria (langkah 1–4 di atas)
- [ ] Live test `accounts` (verifikasi FB Page + IG + Threads terbaca)

## Catatan live test 17 Okt 2026 (Patria)

**Hasil:** user access token dari Graph API Explorer TIDAK bisa mengakses FB Page
untuk Business app, meskipun:
- 5 permission terpilih (pages_show_list, pages_read_engagement, instagram_basic,
  instagram_manage_insights, instagram_manage_contents)
- Dialog OAuth "Setujui semua Halaman saat ini dan di masa mendatang" dipilih
- Token valid (`me?fields=id,name` → "Patria A. Wijaya" benar)

Gejala: `me/accounts` → `{"data": []}`; query Page langsung by ID →
error 100/subcode 33 (no access). Penyebab: token user dari **Facebook Login for
Business** bersifat business-scoped — `/me/accounts` tidak mengembalikan Page
meski user mengelolanya via facebook.com/pages.

**Jalur yang benar untuk Business app: System User** (tidak expire, akses
langsung ke aset bisnis):
1. Business Settings → Users → System Users → buat "Minime" (Admin) ✅ dibuat 17 Okt
2. Assign aset (Page + IG) ke system user
3. Business Settings → Apps → tambahkan App ID Minime ke bisnis
4. System user → "Buat token" → pilih app + scope → token long-lived

**Catatan:** op `draft` tool `meta` TIDAK butuh token (file lokal saja).
Token hanya dibutuhkan op `accounts` (verifikasi read-only).
