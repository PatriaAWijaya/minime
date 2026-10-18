# Panduan Developer App per Platform — postiz-app di MacBook

> Untuk postiz-app v2.25.1 di MacBook Patria (http://localhost:4007).
> Setiap platform butuh **satu developer app** yang didaftarkan di portal resmi
> platform tersebut. Hasilnya (ID + secret) ditempel ke file `.env` postiz,
> lalu container di-restart.
>
> Status dokumen: disusun 17 Okt 2026 dari dokumentasi resmi postiz
> (docs.postiz.com) + kode provider postiz. **Instalasi postiz-nya sudah
> terverifikasi jalan; langkah OAuth per platform BELUM dicoba live** —
> bagian yang belum teruji ditandai ⚠️.

---

## 0. Pola umum (berlaku untuk semua platform)

1. Buka portal developer platform → buat app baru.
2. Daftarkan **redirect URI** postiz (daftar lengkap di §1).
3. Aktifkan produk/fitur yang dibutuhkan + tambahkan scope yang diminta.
4. Salin **Client ID / App ID** dan **Client Secret / App Secret**.
5. Tempel ke `.env` di folder `~/postiz-mac/postiz-mac/` (lihat pemetaan §1).
6. Restart: `docker compose up -d` (dari folder yang sama).
7. Di dashboard postiz (http://localhost:4007): **Add Channel** → pilih
   platform → login/otorisasi akun → selesai.

**Aturan main (rule #3 Minime):** di postiz selalu buat sebagai **draft /
terjadwal** — tidak ada publish otomatis tanpa approval Patria. Untuk
Meta (IG/FB/Threads), jalur utama tetap tool `meta` Minime (draft-only);
koneksi postiz untuk Meta bersifat cadangan.

---

## 1. Redirect URI & pemetaan env (referensi cepat)

Postiz selalu memakai pola `http://localhost:4007/integrations/social/<nama>`.
Daftarkan **persis** seperti di bawah di setiap developer app.

| Platform | Redirect URI yang didaftarkan | Env di `.env` |
|---|---|---|
| Facebook Page | `http://localhost:4007/integrations/social/facebook` | `FACEBOOK_APP_ID`, `FACEBOOK_APP_SECRET` |
| Instagram (via FB Business) | `http://localhost:4007/integrations/social/instagram` | *(pakai FACEBOOK_APP_* yang sama)* |
| Threads | `https://redirectmeto.com/http://localhost:4007/integrations/social/threads` | `THREADS_APP_ID`, `THREADS_APP_SECRET` |
| YouTube | `http://localhost:4007/integrations/social/youtube` | `YOUTUBE_CLIENT_ID`, `YOUTUBE_CLIENT_SECRET` |
| TikTok | ⚠️ butuh HTTPS — lihat §5 (butuh tunnel) | `TIKTOK_CLIENT_ID`, `TIKTOK_CLIENT_SECRET` |
| LinkedIn | `http://localhost:4007/integrations/social/linkedin` | `LINKEDIN_CLIENT_ID`, `LINKEDIN_CLIENT_SECRET` |
| X (Twitter) | `http://localhost:4007/integrations/social/x` | `X_API_KEY`, `X_API_SECRET` |
| Pinterest | `http://localhost:4007/integrations/social/pinterest` | `PINTEREST_CLIENT_ID`, `PINTEREST_CLIENT_SECRET` |
| Mastodon (opsional) | `http://localhost:4007/integrations/social/mastodon` | `MASTODON_CLIENT_ID`, `MASTODON_CLIENT_SECRET`, `MASTODON_URL` |

Catatan Threads: Meta mewajibkan redirect URI ber-HTTPS, jadi postiz
melewatkannya lewat layanan `redirectmeto.com` (terverifikasi dari kode
provider postiz — bukan karangan). Daftarkan URL lengkap di atas apa adanya.

---

## 2. Urutan prioritas yang disarankan

| Urutan | Platform | Kenapa | Estimasi |
|---|---|---|---|
| 1 | **Meta** (FB Page + IG + Threads) | Inti audiens: @patriabicara, @ini.patria, @gelombangkebaikan.id, FB Page, Threads | 30–45 mnt |
| 2 | **YouTube** | Channel YouTube Patria sudah ada | 20–30 mnt |
| 3 | **TikTok** | Jangkauan besar, tapi ada hambatan HTTPS + audit (lihat §5) | 30–45 mnt + tunggu audit |
| 4 | **LinkedIn** | Profesional; cocok untuk konten brand strategy | 20–30 mnt (+ formulir Advertising API) |
| 5 | **X (Twitter)** | Pelengkap | 15–25 mnt |
| 6 | **Pinterest** | Niche visual | 20–30 mnt (+ tunggu approval app) |
| 7 | Mastodon | Opsional | 10–15 mnt |

Tidak harus sekaligus — kerjakan sesuai prioritas, satu per satu.

---

## 3. Meta — Facebook Page + Instagram + Threads (satu akun developer)

Ringkasan dari `docs/koneksi-meta.md` (dokumen lengkapnya tetap jadi acuan),
disesuaikan untuk postiz.

### 3a. Facebook Page + Instagram (SATU app yang sama)

1. Buka https://developers.facebook.com/apps → login akun Facebook-mu →
   **Create App**.
2. Di layar pilihan use case: pilih **Other** (di bagian "Looking for
   something else?") → tipe **Business** → isi nama (mis. `Minime`) → Create.
3. Di dashboard app → **App Settings → Basic**: salin **App ID** dan
   **App Secret** (klik Show, masukkan password FB bila diminta).
4. Tambahkan produk **Facebook Login**: di pengaturannya, isi
   **Valid OAuth Redirect URIs** dengan dua URL ini (satu per baris):
   - `http://localhost:4007/integrations/social/facebook`
   - `http://localhost:4007/integrations/social/instagram`
5. Permission/scope yang diminta postiz (dari dokumentasi resmi postiz):
   `pages_show_list`, `business_management`, `pages_manage_posts`,
   `pages_manage_engagement`, `pages_read_engagement`, `read_insights`.
   Catatan postiz: **untuk pemakaian pribadi, permission lanjutan ini TIDAK
   wajib** — koneksi tetap jalan.
6. **App Review TIDAK perlu** selama yang memakai hanya kamu sendiri
   (kamu otomatis admin). Permission langsung aktif.
7. Mode app boleh tetap **Development** untuk pemakaian sendiri. (Baru ubah
   ke **Live** bila nanti kontennya harus terlihat publik untuk orang lain.)
8. Tempel ke `.env`:
   ```
   FACEBOOK_APP_ID=<App ID>
   FACEBOOK_APP_SECRET=<App Secret>
   ```

**Syarat akun:** Instagram harus **Bisnis atau Kreator** (bukan personal) dan
terhubung ke FB Page-mu. Bila koneksi gagal dengan error "Insufficient
developer role", tambahkan akun IG sebagai tester di App Roles → terima
undangan dari pengaturan Instagram.

### 3b. Threads (app TERPISAH)

1. Di developers.facebook.com → **Create App** baru → pilih use case
   **"Access the Threads API"** → isi nama (mis. `Minime Threads`).
2. Bila diminta detail bisnis dan error "no business portfolios": aktifkan
   **2FA** di akun Meta-mu (business.facebook.com settings), lalu ulangi.
3. Di dashboard app → buka **"Access the Threads API"** → tambahkan scope
   `threads_basic` dan `threads_content_publish` (dari dokumentasi postiz).
4. Buka tab **Settings**: salin **Threads App ID** dan **Threads App Secret**
   (⚠️ yang dipakai yang **Threads** App ID, bukan App ID biasa; secret-nya
   32 karakter).
5. Di kolom redirect URI, isi **persis**:
   `https://redirectmeto.com/http://localhost:4007/integrations/social/threads`
   lalu **klik URL-nya** agar aktif (form tidak tersimpan kalau tidak diklik).
   Kolom Uninstall/Delete Callback URL boleh diisi URL yang sama.
6. Kembali ke Dashboard → **Finish customization** → "Yes I'm finished"
   (API tidak jalan sebelum ini selesai).
7. Tambahkan akun Threads-mu sebagai tester: **App Roles → Roles → Testers →
   Add People** → pilih peran **Threads Tester** → masukkan username Threads-mu.
8. Buka threads.net → Settings → Website permissions → tab **Invites** →
   terima undangan app.
9. Tempel ke `.env`:
   ```
   THREADS_APP_ID=<Threads App ID>
   THREADS_APP_SECRET=<Threads App Secret>
   ```

**Syarat akun:** akun Threads harus **publik**.

---

## 4. YouTube (Google Cloud OAuth)

1. Buka https://console.cloud.google.com/apis/credentials → login akun
   Google-mu → **Create Project** (beri nama mis. `Minime`).
2. **Create Credentials → OAuth client ID** → tipe aplikasi **Web application**.
3. Di **Authorized redirect URIs**, tambahkan:
   `http://localhost:4007/integrations/social/youtube`
   (Google mengizinkan localhost tanpa HTTPS — terverifikasi dari dokumentasi
   postiz.)
4. Salin **Client ID** dan **Client Secret** yang muncul.
5. **Configure Consent Screen** (OAuth consent screen): isi nama app, email;
   tambahkan **dirimu sendiri sebagai Test User**.
6. Aktifkan API: menu **Enabled APIs & Services → Enable APIs and Services** →
   cari dan Enable **YouTube Data API v3** (disarankan postiz: sekalian
   **YouTube Analytics API** dan **YouTube Reporting API**).
7. Tempel ke `.env`:
   ```
   YOUTUBE_CLIENT_ID=<Client ID>
   YOUTUBE_CLIENT_SECRET=<Client Secret>
   ```

⚠️ Catatan jujur: karena app belum terverifikasi Google, saat OAuth Google
akan menampilkan layar peringatan "unverified app" — itu normal untuk
pemakaian sendiri; lanjutkan saja (kamu adalah test user). ⚠️ Belum dicoba
live.

---

## 5. TikTok ⚠️ (paling ribet — baca dulu sebelum mulai)

TikTok **menolak redirect URI http://** — wajib HTTPS. Karena postiz di
MacBook jalan di `http://localhost:4007`, koneksi TikTok **tidak bisa
langsung**; butuh tunnel HTTPS (mis. ngrok) atau domain + reverse proxy.
Dokumentasi resmi postiz menegaskan hal yang sama.

**Jalur yang disarankan (tunnel ngrok):**

1. Install ngrok (https://ngrok.com) → jalankan:
   `ngrok http 4007` → dapat URL mis. `https://abc123.ngrok.io`.
2. Buka https://developers.tiktok.com/apps → login → **Connect an app**
   (buat app baru, nama mis. `MyPostiz`).
3. Tambahkan produk **Login Kit** dan **Content Posting API**; di Content
   Posting API aktifkan **Direct Post**.
4. Tambahkan scope (dari dokumentasi resmi postiz): `user.info.basic`,
   `video.create`, `video.publish`, `video.upload`, `user.info.profile`.
5. Daftarkan redirect URI: `https://abc123.ngrok.io/integrations/social/tiktok`
   (pakai URL ngrok-mu, bukan localhost).
6. Isi **Terms of Service** dan **Privacy Policy** — wajib di domain publik
   ber-HTTPS (syarat TikTok).
7. Salin **Client key** (16 karakter) dan **Client secret** (32 karakter).
8. Tempel ke `.env`:
   ```
   TIKTOK_CLIENT_ID=<Client key>
   TIKTOK_CLIENT_SECRET=<Client secret>
   ```
9. Karena redirect URI sekarang pakai ngrok, ubah juga di `.env`/
   docker-compose agar `FRONTEND_URL` = URL ngrok-mu, lalu restart.

**Batasan penting (dari dokumentasi resmi postiz + TikTok):**
- Sebelum app lolos **audit** TikTok, postingan via Direct Post **dipaksa
  privat (SELF_ONLY)** dan dibatasi ±5 user per 24 jam. Audit = proses
  terpisah dari app review, bisa makan hari–minggu.
- Mode **Sandbox** tersedia untuk testing sebelum approval.
- TikTok mengambil media via URL publik HTTPS — file lokal tidak bisa.

**TikTok Business (opsional):** app terpisah untuk akun bisnis; kredensialnya
ke `TIKTOK_BUSINESS_CLIENT_ID` / `TIKTOK_BUSINESS_CLIENT_SECRET`. Lewati dulu
kecuali kamu butuh posting ke akun TikTok Business.

---

## 6. LinkedIn

1. Buka https://developer.linkedin.com → **Create app** → isi nama, LinkedIn
   Page (boleh Page pribadimu), logo.
2. Di tab **Products**, tambahkan:
   - **Share on LinkedIn**
   - **Sign In with LinkedIn using OpenID Connect**
   - **Advertising API** — ⚠️ wajib isi **formulir request**-nya; menurut
     dokumentasi postiz, tanpa ini **token tidak bisa di-refresh**.
3. Di tab **Auth**, pastikan daftar **OAuth 2.0 scopes** memuat semuanya
   (dari dokumentasi resmi postiz):
   `openid`, `profile`, `w_member_social`, `r_basicprofile`,
   `rw_organization_admin`, `w_organization_social`, `r_organization_social`.
   Empat terakhir dari Advertising API; `w_member_social` dari Share on
   LinkedIn; `openid`/`profile` dari Sign In.
   > Bila ada scope yang hilang, LinkedIn TIDAK menggagalkan login — ia
   > menerbitkan token tanpa scope itu, lalu postiz menolak koneksi dengan
   > error `NotEnoughScopes`. Cek tab Auth dulu bila ini terjadi.
4. Di **Authorized redirect URLs**, tambahkan:
   `http://localhost:4007/integrations/social/linkedin`
   (untuk LinkedIn Page pakai `.../linkedin-page` — kredensialnya sama).
5. Salin **Client ID** dan **Client Secret** dari tab Auth.
6. Tempel ke `.env`:
   ```
   LINKEDIN_CLIENT_ID=<Client ID>
   LINKEDIN_CLIENT_SECRET=<Client Secret>
   ```

---

## 7. X (Twitter)

Postiz memakai **OAuth 1.0a** untuk X (terverifikasi dari kode provider
postiz) — yang dibutuhkan hanya API Key + API Secret, bukan OAuth 2.0.

1. Buka https://developer.x.com → buat **Project** lalu **App** di dalamnya
   (nama mis. `Minime`).
2. Di pengaturan app → **User authentication settings** → **Edit**:
   - Aktifkan **OAuth 1.0a**.
   - **App permissions** = **Read and Write**.
   - **Callback URL**: `http://localhost:4007/integrations/social/x`
     (⚠️ bila portal menolak localhost, gunakan tunnel HTTPS seperti §5).
   - Type of App: **Web App**.
3. Di **Keys and tokens**: salin **API Key** (Consumer Key) dan
   **API Key Secret** (Consumer Secret — hanya tampil sekali).
4. Tempel ke `.env`:
   ```
   X_API_KEY=<API Key>
   X_API_SECRET=<API Key Secret>
   ```
   (`X_URL` di `.env.example` boleh dikosongkan — postiz otomatis pakai
   `http://localhost:4007`.)

⚠️ Catatan jujur: X menerapkan limit sesuai tier akun developer-mu — cek
kuota posting di portal developer.x.com. ⚠️ Belum dicoba live.

---

## 8. Pinterest

1. **Syarat:** akun Pinterest harus **Business** (https://business.pinterest.com).
2. Buka https://developers.pinterest.com → **Create app** → isi data →
   **tunggu app di-approve** (bisa makan waktu; dari dokumentasi postiz).
3. Salin **App ID** dan **App secret key**.
4. Daftarkan redirect URI:
   `http://localhost:4007/integrations/social/pinterest`
   (⚠️ bila portal menolak http/localhost, gunakan tunnel HTTPS seperti §5 —
   belum terverifikasi live.)
5. Tempel ke `.env`:
   ```
   PINTEREST_CLIENT_ID=<App ID>
   PINTEREST_CLIENT_SECRET=<App secret key>
   ```

---

## 9. Mastodon (opsional)

Tidak butuh developer portal — app didaftarkan langsung di instance
Mastodon-mu:

1. Buka instance-mu (default `https://mastodon.social`) → **Preferences →
   Development → New Application**.
2. Isi nama (mis. `Minime`), redirect URI:
   `http://localhost:4007/integrations/social/mastodon`,
   centang scope **read** dan **write** → Submit.
3. Salin **Client key** dan **Client secret**.
4. Tempel ke `.env`:
   ```
   MASTODON_CLIENT_ID=<Client key>
   MASTODON_CLIENT_SECRET=<Client secret>
   MASTODON_URL=https://mastodon.social
   ```
   (Ganti `MASTODON_URL` bila memakai instance lain.)

---

## 10. Setelah kredensial diisi — restart & hubungkan

1. Di Terminal, dari folder `~/postiz-mac/postiz-mac/`:
   ```
   docker compose up -d
   ```
   (Env baru hanya terbaca saat container start ulang.)
2. Buka http://localhost:4007 → **Add Channel** → pilih platform →
   ikuti proses login/otorisasi di tab yang terbuka.
3. **Verifikasi berhasil:** channel muncul di daftar **Channels** (panel kiri)
   dengan nama akunmu. Klik channel-nya — tidak boleh ada error merah.
4. **Uji nyata (disarankan):** buat 1 postingan **draft** (jangan publish)
   ke 1 platform → cek statusnya di Calendar postiz.

### Troubleshooting umum

| Gejala | Kemungkinan penyebab |
|---|---|
| Platform tidak muncul di Add Channel | Env var belum terisi / salah nama → cek `.env`, restart lagi |
| Gagal saat OAuth: "redirect_uri mismatch" | Redirect URI di portal tidak persis sama (cek huruf per huruf, termasuk `http` vs `https` dan port) |
| Threads: error saat authorize | Redirect URI belum "diklik aktif" di settings, atau akun belum terima invite tester |
| LinkedIn: `NotEnoughScopes` | Ada scope hilang di tab Auth → lengkapi sesuai §6 |
| TikTok: tidak bisa connect sama sekali | Wajar tanpa HTTPS — pakai tunnel (§5) |
| Google: layar "unverified app" | Normal untuk pemakaian sendiri → lanjutkan |

---

## 11. Yang belum terverifikasi live (jujur)

- Seluruh flow OAuth per platform di atas **belum dicoba** — disusun dari
  dokumentasi resmi postiz + kode provider postiz, bukan dari percobaan nyata.
- Perilaku tiap portal bisa berubah (nama tombol, syarat approval).
- Bila satu platform menolak `http://localhost:4007`, solusi umumnya sama:
  pasang tunnel HTTPS (ngrok / Cloudflare Tunnel), daftarkan URL tunnel
  sebagai redirect URI, dan sesuaikan `FRONTEND_URL` di docker-compose.

Dokumen terkait: `INSTALL_MAC.md` (instalasi), `INTEGRASI_MINIME.md`
(menghubungkan ke Minime), `../alternatif-blotato.md` (evaluasi postiz),
`../../koneksi-meta.md` (detail koneksi Meta langsung untuk tool `meta`).
