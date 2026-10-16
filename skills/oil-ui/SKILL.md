---
name: oil-ui
description: "Skill desain UI untuk AI agent — dipakai code builder saat Patria butuh bikin aplikasi web-based: eksplorasi 2-3 arah desain yang benar-benar berbeda, bandingkan side-by-side, pilih satu, lalu refine terhadap screenshot nyata dari built-in browser Minime. Diadaptasi dari oil-ui (MIT)."
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: "Desainkan dashboard web untuk aplikasi kasir UMKM — 3 arah desain berbeda, lalu refine yang terpilih."
---

# oil-ui

> Skill UI milik **Strategi Bisnis**, pasangan `code-builder-playbook` — dipakai
> setiap kali Patria butuh **aplikasi web-based**.
>
> Diadaptasi dari [`oil-oil/oil-ui`](https://github.com/oil-oil/oil-ui)
> (lisensi **MIT**). Referensi asli (berbahasa Mandarin): `~/workspace/referensi/oil-ui/`.
> Prinsip inti yang diadopsi: **arah desain lahir dari produknya, bukan dari template
> bawaan model** — dan setiap keputusan visual harus bisa diverifikasi lewat screenshot nyata.
>
> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).

## Kapan dipakai

Bila tugas adalah **mendesain interface aplikasi web**: halaman baru, perbandingan
gaya, polish visual, rekreasi dari screenshot, atau review UI. BUKAN untuk:
logika bisnis, API, build/deploy, atau edit rutin yang perubahannya sudah
disebutkan eksplisit (ganti copy, hapus elemen bernama) — itu langsung kerjakan
tanpa proses desain.

## SOP

### 0. Pahami dulu (sebelum menggambar apa pun)

Jawab 3 hal secara tertulis:
1. **Kategori** produk ini apa (mis. dashboard kasir, landing SaaS, admin panel)?
2. **User datang ke layar utama untuk melihat/mengerjakan apa?** (satu kalimat)
3. **2-3 produk terbaik di kategori ini** — seperti apa layar utama + satu kontrol
   kuncinya? (ambil dari ingatan yang bisa dipertanggungjawabkan; bila bisa,
   screenshot via built-in browser Minime dan simpan di folder task)

### 1. Tetapkan tone → keputusan visual konkret

Tulis **satu kalimat north star**: kesan pertama apa yang harus ditangkap user,
dan seluruh halaman tunduk padanya.

Terjemahkan kata sifat kabur menjadi keputusan terukur (dilarang memakai
"premium", "clean", "modern" sebagai alasan tanpa angka):
- typeface spesifik + skala (display / body / label)
- whitespace: skala spacing yang dipakai
- warna: hex + peran (background, teks, aksi, status) — **satu accent color**
- radius, border, shadow: satu sistem, konsisten di semua kontrol

Keputusan ini WAJIB selaras dengan `docs/rule.md` #6 (UI rules Minime):
body 14–16px, judul section 32px bold, label di atas field, button min 44px,
satu primary action per section, hierarki via tipografi/spacing dulu.

### 2. Eksplorasi 2-3 arah BERBEDA → bandingkan side-by-side

- Buat **2-3 arah desain yang benar-benar berbeda** (bukan satu template ganti warna):
  bedakan di **struktur/bone**, typeface, warna, dan visual utama.
- Tulis tiap arah dalam 3-4 baris: kesan apa, layar utama seperti apa,
  font + warna apa, visual utama dari mana. Sampaikan ke Patria **dulu**,
  minta ia memilih — JANGAN langsung membangun ketiganya.
- Bila Patria bilang "kamu yang tentukan": bangun ketiganya sebagai halaman HTML
  mandiri (satu file per arah), screenshot ketiganya via built-in browser,
  tampilkan side-by-side, pilih satu dengan alasan tertulis.

**Uji perbedaan** (sebelum diserahkan): tutup nama tiap arah, lihat wireframe-nya
saja — bila tidak terasa beda, arahnya gagal, ulangi. Arah yang mirip produk
referensi (font + warna + visual utama sama, cuma ganti nama) DILARANG.

### 3. Bangun arah terpilih → refine terhadap screenshot NYATA

- Bangun halaman penuh dari arah terpilih (HTML + CSS; JS seperlunya, tanpa build step).
- **Loop review memakai built-in browser Minime** (sudah ter-wire di Fase 4:
  screenshot + DOM): screenshot tiap iterasi, periksa di ukuran baca 100%:
  - hierarki: tiap area punya SATU hal terpenting; sisanya mundur (ukuran/warna/posisi)
  - tidak ada teks terpotong / overlap / kontras gagal
  - **anti-pattern Impeccable dilarang**: font Inter untuk segalanya, gradient
    ungu-biru, kartu-di-dalam-kartu, teks abu di background berwarna, hitam/abu murni
- Contoh data harus seperti data nyata (angka tidak bulat-bulat, nama bervariasi,
  tanggal realistis) — dilarang placeholder "Lorem ipsum" / "Contoh".

### 4. Review independen (bila memungkinkan)

- Bila ada agen/executor terpisah yang bisa melihat gambar: minta ia menilai
  screenshot final (skala 1-10) + daftar masalah berurutan dari yang paling berdampak.
- Tidak ada reviewer independen → lakukan self-review dengan checklist yang sama,
  dan tulis eksplisit "tanpa review independen" di catatan.

### 5. Kurangi (subtractive pass) → serahkan

- Tiap layar **satu tokoh utama**, sisanya diam. Hapus penjelasan/elemen yang
  dihapusnya tidak mengurangi pemahaman.
- Serahkan: file HTML final + screenshot final + catatan desain singkat
  (arah terpilih + alasan, yang diubah, yang sengaja tidak dibuat).

## Toolbox

- `browser` (built-in browser Minime, Fase 4): screenshot halaman, baca DOM —
  dipakai untuk loop review langkah 3 dan bukti visual.
- `code-builder-playbook`: menerjemahkan kebutuhan bisnis → spesifikasi
  sebelum desain dimulai (langkah 0).
- File HTML mandiri per arah desain (tanpa build step; CSS variabel untuk
  token desain: warna, spacing, radius, typeface).

## Quality bar

Output DITOLAK bila salah satu terjadi:
- Arah-arah desain tidak bisa dibedakan tanpa membaca labelnya.
- Keputusan visual tidak tertulis (alasan "terlihat bagus" tanpa angka/hex).
- Screenshot final belum diperiksa di 100% / ada teks terpotong / kontras gagal.
- Melanggar `docs/rule.md` #6 (UI rules) atau mengandung anti-pattern Impeccable.
- Ada auto-posting / publikasi ke layanan eksternal tanpa persetujuan Patria
  (aturan keras #3 — desain hanya berupa file lokal/draft).

## Trigger eskalasi — WAJIB ke Patria bila

- Patria memberi referensi/brand guideline yang bertentangan dengan arah yang
  sedang dikerjakan → tanya dulu, jangan menimpa diam-diam.
- Butuh membeli/mengunduh font/gambar berlisensi, atau memakai layanan berbayar.
- Ragu apakah suatu arah "terlalu berani" vs "terlalu aman" untuk brand-nya.
- Review independen memberi skor < 6 dan masalahnya struktural (bukan polish).

## Proof of work

Setiap task desain mencatat di DB (via runs): brief, 3 jawaban langkah 0,
arah-arah yang dieksplorasi + arah terpilih beserta alasan, path screenshot
tiap iterasi, hasil review (skor bila ada), dan path file HTML final.
File kerja (HTML, screenshot, catatan) disimpan di workspace agen.
Deliverable user-facing → Google Drive folder MIMI (aturan #4).
