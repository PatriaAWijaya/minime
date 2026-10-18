# Install postiz-app di MacBook (Apple Silicon)

Panduan ini untuk menginstall **postiz-app** (pengganti Blotato untuk
distribusi konten) sebagai layanan lokal di MacBook Patria. postiz-app
berlisensi **AGPL-3.0** — dipakai sebagai layanan eksternal via API,
tidak ditempel ke kode Minime (lihat `../alternatif-blotato.md` §4).

**Yang belum teruji:** instalasi nyata di MacBook menunggu Patria
menjalankannya. Semua image sudah terverifikasi mendukung Apple
Silicon (arm64) via registry API — tanpa download.

---

## 0. Kebutuhan

- Mac dengan chip Apple Silicon (M1/M2/M3/…), macOS terbaru
- Docker Desktop for Mac (versi Apple Silicon) — unduh di
  https://www.docker.com/products/docker-desktop/
- RAM: alokasikan **minimal 6–8 GB** untuk Docker Desktop
  (Settings → Resources → Memory). Stack postiz + Temporal butuh
  ±3–4 GB saat jalan.
- Koneksi internet (untuk download image pertama kali, ±2–3 GB)

## 1. Install Docker Desktop

1. Unduh Docker Desktop **untuk Apple Silicon** (jangan yang Intel).
2. Buka file `.dmg`, seret Docker ke Applications, jalankan.
3. Ikuti setup awal sampai ikon paus di menu bar menunjukkan
   "Docker Desktop is running".
4. Buka **Settings → Resources** → set Memory ke **8 GB**
   (atau minimal 6 GB), CPU 4. Apply & Restart.

## 2. Siapkan folder postiz

Salin folder `docs/riset/postiz-mac/` dari repo Minime ke MacBook,
misalnya ke `~/postiz/`. Isinya:

```
~/postiz/
├── docker-compose.yml   # definisi layanan (sudah disesuaikan untuk Mac)
├── .env.example         # contoh konfigurasi
├── dynamicconfig/       # konfigurasi Temporal (jangan diubah)
├── INSTALL_MAC.md       # panduan ini
└── INTEGRASI_MINIME.md  # cara menghubungkan ke Minime
```

## 3. Buat file `.env`

Di Terminal:

```bash
cd ~/postiz
cp .env.example .env
```

Lalu isi minimal dua hal ini di `.env`:

1. **JWT_SECRET** — buat string acak:
   ```bash
   openssl rand -hex 32
   ```
   Tempel hasilnya sebagai nilai `JWT_SECRET`.
2. **POSTGRES_PASSWORD** — boleh pakai default untuk pemakaian lokal,
   atau ganti dengan password sendiri.

Kredensial platform sosial (Facebook, TikTok, dsb.) bisa diisi
belakangan — lihat bagian 5.

## 4. Jalankan

```bash
cd ~/postiz
docker compose up -d
```

Tunggu ±1–3 menit (pertama kali: download image + migrasi database).
Cek status:

```bash
docker compose ps
```

Semua service harus `running`/`healthy`. Buka dashboard:

**http://localhost:4007**

Buat akun admin (registrasi pertama). Setelah akun admin jadi:

1. Di `.env`, ubah `DISABLE_REGISTRATION=false` menjadi
   `DISABLE_REGISTRATION=true` (agar orang lain tidak bisa daftar).
2. Restart: `docker compose up -d`

## 5. Hubungkan akun media sosial

Setiap platform butuh **developer app** yang dibuat di situs
resminya, lalu Client ID/Secret-nya dimasukkan ke `.env` dan
`docker compose up -d` lagi. Daftarnya:

| Platform | Buat app di | Env yang diisi |
|---|---|---|
| Facebook + Instagram | https://developers.facebook.com | FACEBOOK_APP_ID, FACEBOOK_APP_SECRET |
| Threads | (pakai Meta App yang sama) | THREADS_APP_ID, THREADS_APP_SECRET |
| YouTube | https://console.cloud.google.com | YOUTUBE_CLIENT_ID, YOUTUBE_CLIENT_SECRET |
| TikTok | https://developers.tiktok.com | TIKTOK_CLIENT_ID, TIKTOK_CLIENT_SECRET |
| X (Twitter) | https://developer.x.com | X_API_KEY, X_API_SECRET (+ X_URL) |
| LinkedIn | https://developer.linkedin.com | LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET |
| Pinterest | https://developers.pinterest.com | PINTEREST_CLIENT_ID, PINTEREST_CLIENT_SECRET |

Catatan:
- **Instagram wajib akun Bisnis/Kreator** dan terhubung ke Facebook Page.
- Redirect URI untuk OAuth: `http://localhost:4007` (atau domain
  yang dipakai bila diakses dari jaringan lain).
- Setelah env diisi, hubungkan akun lewat dashboard postiz:
  **Integrations → Connect** per platform, lalu login/otorisasi.

## 6. Buat API key untuk Minime

1. Di dashboard postiz, buka **Settings → API** (atau menu
   Public API / Integrations).
2. Salin API key pribadi — hex 64 char TANPA prefix (Settings → Agents). CATATAN: yang berawalan `pos_` adalah token OAuth App, bukan API key ini (koreksi 18 Okt 2026).
3. Simpan key ini baik-baik (hanya tampil sekali).
4. Cara menghubungkannya ke Minime: lihat `INTEGRASI_MINIME.md`.

## 7. Perintah sehari-hari

```bash
cd ~/postiz
docker compose up -d        # nyalakan
docker compose down         # matikan (data tetap tersimpan di volume)
docker compose ps           # lihat status
docker compose logs -f postiz   # lihat log bila ada masalah
docker compose pull && docker compose up -d   # update ke versi terbaru
```

Data (database, upload) tersimpan di Docker volume — aman walau
container dimatikan/dihapus.

## 8. Troubleshooting

**Port 4007 sudah dipakai**
→ Ganti mapping port di `docker-compose.yml` (`'4008:5000'`),
atau matikan aplikasi yang memakai port itu.

**Service tidak healthy / restart terus**
→ `docker compose logs -f <nama-service>` untuk lihat error.
Pertama kali jalan memang butuh 1–3 menit (migrasi DB).

**Mac terasa berat**
→ Pastikan Docker Desktop dapat 6–8 GB RAM. Bila masih berat,
service `temporal-elasticsearch` bisa dimatikan (lihat komentar di
`docker-compose.yml`) — belum teruji, lakukan bila perlu saja.

**Lupa password admin**
→ Sementara set `DISABLE_REGISTRATION=false`, restart, daftar akun
baru, lalu kembalikan ke `true`.

**"The design could not be generated" / error OAuth platform**
→ Biasanya redirect URI di developer app belum cocok dengan URL
postiz (`http://localhost:4007`). Samakan di kedua sisi.

---

*Lanjut ke `INTEGRASI_MINIME.md` untuk menghubungkan postiz ke Minime.*
