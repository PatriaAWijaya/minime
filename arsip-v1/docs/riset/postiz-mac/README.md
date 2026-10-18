# Paket instalasi postiz-app untuk MacBook

Paket ini menyiapkan **postiz-app** (pengganti Blotato) untuk diinstall
di MacBook Patria. Dibuat 17 Okt 2026 atas perintah Patria
("masukkan di mac").

| File | Fungsi |
|---|---|
| `docker-compose.yml` | Definisi layanan, disesuaikan untuk Apple Silicon (semua image terverifikasi arm64) |
| `.env.example` | Contoh konfigurasi — salin jadi `.env` dan isi |
| `dynamicconfig/` | Konfigurasi Temporal (bawaan resmi, jangan diubah) |
| `INSTALL_MAC.md` | Panduan install langkah per langkah (Bahasa Indonesia) |
| `INTEGRASI_MINIME.md` | Cara menghubungkan postiz ke Minime + pola draft → approval |

**Mulai dari:** `INSTALL_MAC.md`

**Status jujur:** instalasi nyata di MacBook belum teruji (menunggu
Patria). Yang terverifikasi: sintaks YAML valid, semua 9 image
mendukung linux/arm64 (dicek via registry API tanpa download),
endpoint API `/public/v1` terkonfirmasi dari kode sumber postiz-app
v1.47.0.
