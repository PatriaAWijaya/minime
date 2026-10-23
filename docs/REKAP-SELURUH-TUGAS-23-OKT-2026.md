# Rekap Seluruh Tugas Minime v2 — 16 s.d. 23 Okt 2026

**Disusun:** 23 Okt 2026 ~12:00 WIB · **Atas perintah Patria:** rekap detail semua tugas

Legenda: ✅ selesai · ⏳ belum selesai · 🅿️ ditangguhkan/diparkir · 🔑 butuh input Patria

---

## FASE 0 — Kickoff & Arsitektur (16 Okt)

| Tugas | Status |
|---|---|
| Brainstorm arsitektur v1 → v2 → v3 (daemon + UI + komersialisasi) | ✅ |
| Keputusan stack: Tauri 2 + Svelte 5, clean-room dari pola OpenBot | ✅ |
| Dokumen: arsitektur, framework agen, workflow, rule, plan | ✅ |

---

## TAHAP 1 — UI/UX (18–19 Okt) ✅ Gate LOLOS

| Tugas | Status | Perubahan penting |
|---|---|---|
| Scaffold proyek fresh (`app/`) | ✅ | |
| Layout 4 kolom ala OpenBot | ✅ | |
| 6 agen seed + badge avatar + dot status | ✅ | Dot kuning/hijau/merah dikunci |
| 3 bahasa (ID/EN/ZH), 790 key | ✅ | Paritas penuh |
| Sidebar icon + 7 tab | ✅ | Redesign 18 Okt malam |
| Voice command (mock) di chat agen | ✅ | |
| Personalisasi agen (gender/usia/jabatan/hobi/suara) | ✅ | |
| Tema: monokrom + aksen biru #4FA6F0, dark glass | ✅ | Dropdown WAJIB solid |
| Avatar animasi | 🅿️ | Dibuat ulang Patria sendiri, tidak blocking |

---

## TAHAP 2 — Model Router (19–20 Okt) ✅ Gate LOLOS

| Tugas | Status | Perubahan penting |
|---|---|---|
| Trait provider + registry + error routing (Rust) | ✅ | |
| Secrets: OS Keychain + fallback security CLI | ✅ | Fix 21 Okt pagi |
| Ollama (default gratis) → OpenCode (fallback) | ✅ | |
| HTTP OpenAI-compatible: Muse/Gemini/DeepSeek/RouterKu | ✅ | |
| Chain berbayar: Muse→Gemini→DeepSeek | ✅ | **→ 23 Okt: Meta-first** |
| SQLite: riwayat chat + usage metering | ✅ | Tab Cost pakai data nyata |
| System prompt per agen + skills engine | ✅ | |
| **Registrasi 5 model Meta** (23 Okt) | ✅ | Spark 1.3, Wonder 1.0, Image 1.0, Voice Transcribe 1.0, SAM 3.1 |
| **Chain Meta-first** (23 Okt) | ✅ | Meta → Muse → Gemini → DeepSeek |
| **Rancangan Meta-first final + vonis Wonder** (23 Okt) | ✅ | `docs/RANCANGAN-PRIORITAS-MODEL-META.md` |
| RouterKu | 🔑 | Blocked: gateway unknown, saldo 0 — Patria harus WA SAKA Support |
| Runtime agen: chat → Mimi orkestrasi → eksekusi | ⏳ | Belum diimplementasi penuh |

---

## TAHAP 3 — Tools & Distribusi (20–21 Okt) ✅ Gate LOLOS BERSYARAT

### 3.1–3.3 — Tools Gateway, Approval, Meta/Postiz

| Tugas | Status |
|---|---|
| Tools gateway Rust (registry + 6 command) | ✅ |
| Approval gate + notifikasi native | ✅ |
| Meta & Postiz draft-only (TIDAK ADA auto-posting) | ✅ |

### 3.4–3.6 — Koneksi Nyata

| Tugas | Status | Perubahan penting |
|---|---|---|
| Pireel via OAuth 2.1 (nyata, bukan gimmick) | ✅ | Fix token tidak tersimpan 21 Okt |
| Google Drive via OAuth (upload nyata teruji) | ✅ | Client ID "Minime Desktop" |
| Runtime: gateway → chat | ✅ | |
| Consent gate autopost | ✅ | Dialog 3 konsekuensi + checkbox wajib |

### Model Agen & Folder (20 Okt sore, perintah Patria)

| Tugas | Status |
|---|---|
| Tab Agen hanya agen AKTIF | ✅ |
| Satu agen boleh banyak folder, tidak 2x di folder sama | ✅ |
| Nonaktif → hilang dari Agen, tetap di Tools & Tugas (fungsi mati) | ✅ |
| Hapus permanen hanya via Tools | ✅ |
| Mimi = agen utama, dikecualikan dari folder | ✅ |
| Koneksi: Mimi node pertama sebagai parent | ✅ |

---

## TAHAP 4 — Media Studio & Lanjutan (21–23 Okt, berjalan)

### Studio Gambar

| Tugas | Status | Perubahan penting |
|---|---|---|
| ImageEditor ala Doory (clean-room) | ✅ | Object/layer, 10 tools, 6 template |
| Upload & Edit via dialog native | ✅ | Fix 23 Okt: HTML input dibuang, pakai rfd |
| Backend: SD.cpp lokal + Pixabay | ✅ | |
| 🔑 Konfirmasi visual Patria | ⏳ | Upload & Edit belum dikonfirmasi Patria |

### Studio Suara

| Tugas | Status |
|---|---|
| VoiceStudio engine utama (via API lokal) | ✅ | Override Patria: AGPL boleh via proses eksternal |
| yovoice fallback | ✅ | |
| Voice cloning | ✅ | Teruji |
| Voice editor native (timeline multi-track) | ✅ | |

### Studio Video

| Tugas | Status | Perubahan penting |
|---|---|---|
| TimelineEditor (multi-track) | ✅ | |
| Video AI Pipeline 6 tahap (tiru MoneyPrinterTurbo) | ✅ | 23 Okt |
| Integrasi ke workflow Media Studio | ✅ | Tab "Video AI" dihapus, jadi section |
| Footage: Pixabay / Text to Video | ✅ | Icon dihapus atas perintah Patria |
| Subtitle: auto-subtitle | ✅ | Tanpa icon |
| Render (dulu "Rakit") + dropdown rasio & mode | ✅ | 9:16/16:9/4:5/1:1, FFmpeg/HyperFrames |
| Backend: assemble_video_ai, text_to_video, hyperframes | ✅ | Teruji dengan file nyata |
| UI key Pexels/Wan/LTX di Tools | ✅ | 23 Okt (hilangkan gimmick) |
| Sora | ❌ | DITOLAK: OpenAI tutup API permanen |
| Open-Sora | ❌ | DITOLAK: mati suri, butuh GPU 24GB+ |
| 🔑 Konfirmasi visual Patria | ⏳ | UI Studio Video baru |
| UI untuk render_slideshow, generate_subtitle | ⏳ | Backend ada, UI belum |

### Dynamic Island & Lainnya

| Tugas | Status | Perubahan penting |
|---|---|---|
| Dynamic Island | ✅ | Floating bawah menu bar, default NON-AKTIF |
| Koneksi Tools → Apps (status nyata) | ✅ | Toggle manual ditolak untuk tools nyata |
| Fix layar hitam (vite base './') | ✅ | Pelajaran tetap di AGENTS.md |
| Skill REA, pstack, marketplace | ✅ | |

---

## AUDIT (13x, semua LOLOS)

| # | Tanggal | Cakupan |
|---|---|---|
| 1–7 | 20–21 Okt | Fondasi, tools, koneksi |
| 8–9 | 22 Okt | Pasca Media Studio |
| 10 | 22 Okt | Zero-error gate Voice Editor |
| 11 | 23 Okt | Layar hitam + konsistensi |
| 12 | 23 Okt | ImageEditor + Video AI Pipeline |
| 13 | 23 Okt | 5 model Meta + chain Meta-first |

---

## PEKERJAAN BELUM SELESAI / DITANGGUHKAN

### 🔑 Butuh input/keputusan Patria

| # | Item | Detail |
|---|---|---|
| 1 | API key Pixabay | Input ulang via Tools → Apps (key lama tak tersimpan) |
| 2 | API key Pexels | Via Tools → Apps (gratis, 200 req/jam) |
| 3 | API key fal.ai | Untuk Wan 3.0 / LTX-Video |
| 4 | Meta API key | Untuk 5 model Meta (atau tunggu kredit hackathon $150) |
| 5 | Postiz URL + key | Via Tools → Apps |
| 6 | Meta Page ID + token | Via Tools → Apps |
| 7 | Konfirmasi visual | Upload & Edit + UI Studio Video baru |
| 8 | RouterKu | WA SAKA Support 6285928157869: provisioning + top-up QRIS min Rp20rb |
| 9 | Avatar 3D | Menunggu keputusan arah desain Patria |

### ⏳ Belum dikerjakan

| # | Item | Detail |
|---|---|---|
| 1 | Runtime agen penuh | Chat → Mimi orkestrasi → eksekusi → output (delegasi multi-agen) |
| 2 | UI render_slideshow & generate_subtitle | Backend ada & teruji, UI belum |
| 3 | AI clipper | Repo di ~/.minime/clipper; smoke test gagal (konflik faster-whisper) → pin versi |
| 4 | Wire defaultModelSpec & premiumEndpoint | Masih state lokal di frontend |
| 5 | Budget tracker $150 di tab Cost | Dari rancangan Meta-first (Fase D hackathon) |
| 6 | Endpoint image/transcribe/segment Meta | Dari rancangan (Fase B–C hackathon) |
| 7 | MeHarness ECC gelombang 1 | 7 skill belum dikurasi |
| 8 | AgentShield | Evaluasi konsep belum dilakukan |
| 9 | STT nyata | Masih mock |
| 10 | Tahap 4.5 | Latensi IPC formal, Lighthouse CI, audit independen |

### 🅿️ Diparkir (jangan sentuh tanpa perintah)

| # | Item | Alasan |
|---|---|---|
| 1 | meMeet | Perintah Patria: stop total |
| 2 | WebDriver AFK | Diparkir atas "lanjut ke tahap 4" |
| 3 | Avatar animasi | Patria buat sendiri |
| 4 | Windows installer | Workflow siap, butuh copy manual (token tanpa scope workflow) |

---

## RENCANA BERIKUTNYA

### Hackathon Meta (26 Okt – 4 Nov) — PRIORITAS UTAMA

| Fase | Tanggal | Isi |
|---|---|---|
| A — Chat default | 26–27 Okt | ✅ Fondasi sudah siap (5 model terdaftar, chain Meta-first). Uji E2E dengan key |
| B — Image + transcribe | 28–29 Okt | Endpoint `muse-image-1.0` + `muse-voice-transcribe-1.0`; fallback SD.cpp |
| C — Segment + power | 30–31 Okt | `sam-3.1` untuk video editor; Wonder sebagai power mode opsional |
| D — Hardening + demo | 1–3 Nov | Budget tracker $150 di tab Cost; audit; rekam demo |
| Submit | 4 Nov | Tenggat 9:00 PM ET |

### Setelah hackathon

1. Runtime agen penuh (delegasi multi-agen)
2. UI slideshow & subtitle generator
3. AI clipper (perbaiki dependensi)
4. Tahap 4.5 → tinjau ulang total → vonis launching → Fase 6 (komersialisasi)

---

**Catatan:** Semua perubahan tercatat di `docs/status.md` (122 entri) dan pasangan SHA di `~/AGENTS.md`.
