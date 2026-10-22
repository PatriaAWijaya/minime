# Media Studio Pipeline Skill

Media Studio adalah tempat agen Kreatif memproduksi aneka konten yang diinginkan user.
Hasil riset OTOMATIS dikembangkan agen Kreatif menjadi berbagai macam script produksi konten.
Masing-masing script yang diproduksi OTOMATIS diinput menjadi source untuk produksi media.

## Pipeline

### 1. Riset (input)
- Terima hasil riset dari agen Riset atau user (tempel teks).
- Simpan ke localStorage `minime.research` sebagai JSON array:
  `[{id, title, content, createdAt}]`
- Satu riset bisa menghasilkan BANYAK script.

### 2. Script Produksi (agen Kreatif)
Untuk SETIAP riset, kembangkan menjadi beberapa script sesuai kebutuhan:
- Jenis: reels, carousel, poster, story, thumbnail, artikel
- Format per script: hook (3 detik) → isi → CTA
- Hormati brand kit (warna, font, tone, template) dari `minime.brand-kit`
- Simpan ke localStorage `minime.scripts` sebagai JSON array:
  `[{id, researchId, title, kind, content, status: "draft", createdAt}]`

### 3. Produksi Media (otomatis per script)
Setiap script OTOMATIS menjadi source produksi:
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
- Satu riset → banyak script → setiap script diproduksi independen.
