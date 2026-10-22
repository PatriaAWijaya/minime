
## Revisi 22 Okt — Agent-Driven Flow

**Perintah Patria:** Cara kerja tetap dari chat agen.

### Alur Baru

```
User chat dengan agen (Mimi/Kreatif)
  → "Buatkan konten untuk [goal] dengan data [...]"
       ↓
Agen OTOMATIS jalankan pipeline:
  1. Generate script (LLM)
  2. Buat storyboard (scene list)
  3. Generate images (DALL-E / SD)
  4. Rakit draft (preview)
       ↓
DRAFT ditampilkan ke user
  → [Approve] → lanjut ke Penjadwalan
  → [Sesuaikan] → buka Editor
       ↓
EDITOR:
  - Video: UI macam CapCut (timeline, trim, teks, transisi)
  - Image: UI macam Canva (layer, teks, filter, crop)
       ↓
Hasil edit → kembali ke Draft → Approve → Jadwal
```

### Komponen Baru

1. **Draft Review UI** — tampilkan draft lengkap, tombol Approve / Sesuaikan
2. **Image Editor** — canvas-based: tambah teks, crop, filter, undo
3. **Video Editor** — timeline sederhana: trim, tambah teks overlay, preview
4. **Agent Integration** — chat command untuk trigger pipeline
