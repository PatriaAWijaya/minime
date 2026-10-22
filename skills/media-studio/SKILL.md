# Media Studio Pipeline Skill

Ketika user meminta pembuatan konten (contoh: "buatkan konten untuk kampanye donasi",
"buatkan poster untuk event"), jalankan pipeline ini OTOMATIS sampai level draft.

## Pipeline

### 1. Goal & Data
Tanyakan atau ekstrak dari chat:
- Tujuan konten (awareness, donasi, penjualan, edukasi)
- Target audiens
- Data mentah (teks, fakta, quotes)

### 2. Script
Buatkan script/naskah konten berdasarkan goal dan data.
Format: hook (3 detik) → isi → CTA.

### 3. Storyboard
Pecah script menjadi scene-scene:
- Setiap scene: deskripsi visual + narasi/VO + durasi (detik)
- Simpan ke localStorage key `minime.storyboard` sebagai JSON array:
  `[{id, duration, visual, narration}]`

### 4. Aset Visual
Untuk setiap scene, generate atau siapkan gambar:
- Gunakan tool `generate_image` dengan prompt dari deskripsi visual
- Atau minta user upload jika ada aset spesifik

### 5. Draft
Setelah semua siap, informasikan user:
"Draft selesai! Buka tab Studio untuk review. Kamu bisa Approve untuk lanjut ke penjadwalan, atau Sesuaikan dengan editor."

### 6. Approval → Jadwal
Jika user approve, arahkan ke tab Studio bagian Jadwal Konten.

## Aturan
- JANGAN auto-posting. Semua konten berhenti di draft.
- Hormati brand kit user (warna, font, tone) dari localStorage `minime.brand-kit`.
- Bahasa: Indonesia (default), ikuti preferensi user.
