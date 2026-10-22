# Media Studio Pipeline Skill

Media Studio adalah tempat agen Kreatif memproduksi aneka konten yang diinginkan user.

PRINSIP: User menambahkan beberapa hal yang diinginkan sebagai input.
Otomasi SEPENUHNYA ditangani oleh agen Kreatif — user tidak perlu mengerjakan
langkah manual apapun setelah input diberikan.

## Pipeline (otomatis penuh oleh agen Kreatif)

### 1. Input User
- Baca semua input dari localStorage `minime.inputs`:
  `[{id, kind, title, content, createdAt}]`
- Jenis input: riset, goal/tujuan, preferensi, materi mentah, lainnya.
- User boleh menambah banyak input; agen menggabungkan semuanya sebagai brief.

### 2. Script Produksi (otomatis)
Untuk SETIAP input (atau gabungannya), kembangkan menjadi beberapa script:
- Jenis: reels, carousel, poster, story, thumbnail, artikel
- Format per script: hook (3 detik) → isi → CTA
- Hormati brand kit (warna, font, tone, template) dari `minime.brand-kit`
- Simpan ke localStorage `minime.scripts`:
  `[{id, researchId, title, kind, content, status: "draft", createdAt}]`

### 3. Produksi Media (otomatis per script)
Setiap script OTOMATIS menjadi source produksi tanpa campur tangan user:
- **Storyboard**: pecah script per baris/paragraf menjadi scene
  → simpan ke `minime.storyboard`:
  `[{id, duration, visual, narration, dialogue, action, notes, shotType}]`
- **Gambar**: generate via tool `generate_image` dengan prompt dari deskripsi visual scene
- **Video**: render slideshow dari gambar via `render_slideshow`
- **Voiceover**: generate via `generate_voiceover` (arboTTS, Bahasa Indonesia)
- **Subtitle**: via `generate_subtitle`
- Update status script: `draft` → `in_production` → `done`

### 4. Draft & Review
Setelah semua siap, informasikan user:
"Draft selesai! Buka tab Studio untuk review. Kamu bisa Approve untuk lanjut ke penjadwalan, atau Sesuaikan dengan editor."

### 5. Approval → Jadwal
Jika user approve, arahkan ke tab Studio bagian Jadwal Konten.

## Aturan
- JANGAN auto-posting. Semua konten berhenti di draft.
- Hormati brand kit user dari localStorage `minime.brand-kit`.
- Bahasa: Indonesia (default), ikuti preferensi user.
- User cukup beri input; SEMUA langkah setelahnya otomatis oleh agen Kreatif.
