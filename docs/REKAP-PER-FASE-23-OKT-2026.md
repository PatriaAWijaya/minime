# Rekap Detail Per Fase/Tahap — Minime v2 (16 Okt – 23 Okt 2026)

**Disusun:** 23 Okt 2026 · **Perintah Patria:** rekap detail per fase/tahap

Legenda: ✅ selesai · ⏳ belum selesai · 🅿️ parkir · 🔑 butuh Patria · ❌ ditolak/dihapus

---

## FASE 0 — v1 (16–18 Okt) → DIARSIPKAN

### Yang dikerjakan (v1)

| # | Tugas | Status |
|---|---|---|
| 1 | Brainstorm arsitektur v1 → v2 → v3 | ✅ |
| 2 | Fase 1: fondasi (selesai, menunggu gate) | ✅ |
| 3 | Fase 2: orkestrasi inti + provider | ✅ |
| 4 | Fase 3: skills & knowledge | ✅ |
| 5 | Fase 4: selesai (menunggu gate) | ✅ |
| 6 | Skill oil-ui, html-anime, manimgx, valyu-finance | ✅ |
| 7 | Skill warehouse (agency-agents) | ✅ |
| 8 | Finance skills quant/trading | ✅ |
| 9 | Pipeline konten video otomatis | ✅ |
| 10 | Render video pindah ke Gemini Veo 3.1 | ✅ |
| 11 | Koneksi Meta langsung (hybrid, draft-only) | ✅ |
| 12 | fal.ai DIHAPUS, Blotato → Postiz | ✅ |
| 13 | Uji Instagram via Postiz berhasil | ✅ |
| 14 | Gate Fase 4: LOLOS BERSYARAT → Fase 5 | ✅ |

### Keputusan arsip (18 Okt 19:36)

Patria memutuskan bangun ulang total. v1 diarsipkan:
- Repo GitHub di-archive (read-only)
- Kode → `arsip-v1/` (history git tetap)
- Repo dipakai ulang untuk v2 (fresh start, clean-room)

**Sisa v1:** tidak ada yang belum selesai — semua sudah diarsipkan.

---

## TAHAP 1 — UI/UX (18–19 Okt) ✅ GATE LOLOS (19 Okt 10:12)

### Selesai

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | Scaffold Tauri 2 + Svelte 5 | Proyek fresh `app/` |
| 2 | Layout 4 kolom ala OpenBot | Sidebar \| agen \| chat+delivery \| monitor |
| 3 | 6 agen seed + badge + dot status | Kuning=bekerja, hijau=siap, merah=perhatian (dikunci) |
| 4 | 3 bahasa (ID/EN/ZH) | 790 key, paritas penuh |
| 5 | Navigasi: Pengaturan, Penggunaan, Peralatan | |
| 6 | Sidebar icon + 7 tab | Redesign 18 Okt malam (gate v2) |
| 7 | Voice command (mock) | Di semua chat agen |
| 8 | Personalisasi agen | Gender, usia, jabatan, hobi, suara |
| 9 | Tema monokrom + #4FA6F0 | Dark glass; dropdown WAJIB solid |
| 10 | Topbar mobile 2 baris | Fix ≤860px |
| 11 | Dokumen arsitektur | arsitektur.md, framework, workflow |
| 12 | Review Tahap 1 oleh Mimi | Selesai 18 Okt 21:55 |

### Belum selesai / parkir

| # | Item | Status | Keterangan |
|---|---|---|---|
| 1 | Avatar animasi | 🅿️ | Patria buat sendiri, tidak blocking |

---

## TAHAP 2 — Model Router (19–20 Okt) ✅ GATE LOLOS (20 Okt 11:59)

### Selesai

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | Trait provider + registry + error routing | Fondasi Rust |
| 2 | Secrets OS Keychain | + fallback `security` CLI (fix 21 Okt) |
| 3 | Ollama (gratis default) | Deteksi localhost:11434, list models, chat |
| 4 | OpenCode (fallback gratis) | CLI free tier via shell-out |
| 5 | Muse / Gemini / DeepSeek / RouterKu | HTTP OpenAI-compatible |
| 6 | Chain: Muse→Gemini→DeepSeek | **→ 23 Okt berubah jadi Meta-first** |
| 7 | Tauri commands | provider_probe, agent_chat, connect/disconnect, dsb |
| 8 | Frontend Penyedia Model | Status nyata + form API key (Keychain) |
| 9 | SQLite riwayat + usage | Tab Cost pakai data nyata |
| 10 | System prompt per agen + skills engine | Playbook disuntik otomatis |
| 11 | **5 model Meta terdaftar** (23 Okt) | Spark 1.3, Wonder 1.0, Image 1.0, Voice Transcribe 1.0, SAM 3.1 |
| 12 | **Chain Meta-first** (23 Okt) | Meta → Muse → Gemini → DeepSeek |
| 13 | **Rancangan final + vonis Wonder** (23 Okt) | docs/RANCANGAN-PRIORITAS-MODEL-META.md |

### Belum selesai / butuh Patria

| # | Item | Status | Keterangan |
|---|---|---|---|
| 1 | Runtime agen penuh | ⏳ | Chat → Mimi orkestrasi → eksekusi → output |
| 2 | RouterKu | 🔑 | Blocked: gateway unknown, saldo 0. Patria WA SAKA Support 6285928157869 |
| 3 | Meta API key | 🔑 | Untuk 5 model Meta / tunggu kredit hackathon $150 |

---

## TAHAP 3 — Tools & Distribusi (20–21 Okt) ✅ GATE LOLOS BERSYARAT (21 Okt 08:10)

### 3.1 Tools Gateway ✅ (20 Okt 13:00)

| # | Tugas |
|---|---|
| 1 | Pulihkan tools.rs (registry + ToolInfo) |
| 2 | 6 command: list_tools, get_tool_status, set_tool_connected, dsb |
| 3 | ToolsView → status koneksi nyata |
| 4 | Editor allowlist di menu Izin agen |

### 3.2 Approval + Notifikasi ✅ (20 Okt 13:10)

| # | Tugas |
|---|---|
| 1 | Permission notification + command send_notification |
| 2 | Kartu → kolom Butuh Approval/Selesai → notifikasi otomatis |
| 3 | Tombol Setuju/Revisi/Tolak di kartu approval |

### 3.3 Meta & Postiz Draft-Only ✅ (20 Okt 13:15)

| # | Tugas |
|---|---|
| 1 | createSocialDraft() — hasil = DRAFT, tidak pernah auto-post |
| 2 | Form draft di dialog Publikasikan |
| 3 | Gate koneksi: tombol mati bila tool belum terhubung |
| 4 | Direct post HANYA atas perintah eksplisit user |

### 3.4–3.6 Koneksi Nyata ✅

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | Pireel via OAuth 2.1 | Fix 21 Okt: token tidak tersimpan → sekarang nyata |
| 2 | Google Drive via OAuth | Client ID "Minime Desktop"; upload teruji |
| 3 | Runtime gateway → chat | Selesai |
| 4 | Consent gate autopost | Dialog 3 konsekuensi + checkbox wajib |

### Model Agen & Folder (20 Okt sore)

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | Tab Agen hanya agen AKTIF | Perintah Patria |
| 2 | Satu agen banyak folder, tidak 2x di folder sama | |
| 3 | Nonaktif → hilang dari Agen, tetap di Tools & Tugas | Fungsi dimatikan |
| 4 | Hapus permanen hanya via Tools | 6 bawaan diproteksi |
| 5 | Mimi = agen utama | Dikecualikan dari folder; node pertama di Koneksi |
| 6 | Folder = Project | Agen dalam folder otomatis diplot |
| 7 | Sinkronisasi folder↔Koneksi dua arah | + kolom resizable |
| 8 | Default bersih: hanya Mimi | Perintah Patria |

### Belum selesai

| # | Item | Status | Keterangan |
|---|---|---|---|
| 1 | Kurasi ECC gelombang 1 (7 skill) | ⏳ | MeHarness |
| 2 | Evaluasi AgentShield | ⏳ | Konsep belum dievaluasi |

---

## TAHAP 4 — Media Studio (21–23 Okt) 🔄 BERJALAN

### 4.1 Studio Gambar ✅

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | ImageEditor ala Doory | Clean-room; object/layer, 10 tools, 6 template |
| 2 | Upload & Edit via dialog native | Fix 23 Okt: HTML input dibuang (WebKit abaikan .click()), pakai rfd |
| 3 | Backend SD.cpp + Pixabay | generate_image, pixabay_search_images |
| 4 | 🔑 Konfirmasi visual Patria | ⏳ Belum dikonfirmasi |

### 4.2 Studio Suara ✅

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | VoiceStudio engine utama | Via API lokal 127.0.0.1:3900; override Patria (AGPL boleh via proses eksternal) |
| 2 | yovoice fallback | Kokoro-82M Q8 + VoxCPM2-Q8 |
| 3 | Voice cloning | Teruji |
| 4 | Voice editor native | Timeline multi-track |
| 5 | Audio mastering (FFmpeg loudnorm) | |

### 4.3 Studio Video ✅ (sebagian)

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | TimelineEditor multi-track | V1/A1/T1 |
| 2 | Video AI Pipeline 6 tahap | Tiru MoneyPrinterTurbo (23 Okt) |
| 3 | Integrasi ke workflow | Tab "Video AI" dihapus → jadi section Footage/Subtitle/Render |
| 4 | Footage: Pixabay / Text to Video | Icon dihapus (perintah Patria) |
| 5 | Subtitle: auto-subtitle | Tanpa icon |
| 6 | Render + dropdown rasio & mode | 9:16/16:9/4:5/1:1; FFmpeg/HyperFrames |
| 7 | Backend: assemble_video_ai, text_to_video, hyperframes_assemble | Teruji file nyata |
| 8 | UI key Pexels/Wan/LTX | Hilangkan gimmick (23 Okt) |
| 9 | Sora | ❌ DITOLAK: OpenAI tutup API permanen Sep 2026 |
| 10 | Open-Sora | ❌ DITOLAK: mati suri 18 bulan, butuh VRAM 52-60GB, CUDA-only |
| 11 | 🔑 Konfirmasi visual Patria | ⏳ UI baru belum dikonfirmasi |
| 12 | UI render_slideshow & generate_subtitle | ⏳ Backend ada, UI belum |

### 4.4 Lainnya ✅

| # | Tugas | Detail perubahan |
|---|---|---|
| 1 | Dynamic Island | Floating bawah menu bar; default NON-AKTIF (koreksi 23 Okt) |
| 2 | Koneksi Tools → Apps | Status nyata via real_connected(); toggle manual ditolak |
| 3 | Fix layar hitam | Akar: vite.config.ts tanpa base './'; pelajaran tetap |
| 4 | Skill REA / pstack / marketplace | Untuk Code Builder, Kreatif, Mimi |

### Belum selesai Tahap 4

| # | Item | Status |
|---|---|---|
| 1 | STT nyata (masih mock) | ⏳ |
| 2 | AI clipper | ⏳ Smoke test gagal (konflik faster-whisper 1.2.1 vs huggingface_hub) |
| 3 | Wire defaultModelSpec & premiumEndpoint | ⏳ Masih state lokal |
| 4 | Tahap 4.5: latensi IPC, Lighthouse CI, audit independen | ⏳ |

---

## META-FIRST (23 Okt) ✅ FONDASI SELESAI

| # | Tugas | Status |
|---|---|---|
| 1 | Rancangan final | ✅ docs/RANCANGAN-PRIORITAS-MODEL-META.md |
| 2 | Vonis Wonder = power mode opsional | ✅ Setuju (3-5x lebih mahal dari Spark) |
| 3 | Registrasi 5 model Meta | ✅ |
| 4 | Chain Meta-first | ✅ |
| 5 | Endpoint image/transcribe/segment | ⏳ Fase B–C hackathon |
| 6 | Budget tracker $150 di tab Cost | ⏳ Fase D hackathon |

---

## RINGKASAN STATUS PER FASE

| Fase/Tahap | Status | Selesai | Belum/Parkir |
|---|---|---|---|
| Fase 0 (v1) | 🗄️ Diarsipkan | 14 | 0 |
| Tahap 1 (UI/UX) | ✅ Gate lolos | 12 | 1 🅿️ |
| Tahap 2 (Router) | ✅ Gate lolos | 13 | 1 ⏳, 2 🔑 |
| Tahap 3 (Tools) | ✅ Gate lolos bersyarat | 18 | 2 ⏳ |
| Tahap 4 (Media) | 🔄 Berjalan | 17 | 4 ⏳, 2 🔑 |
| Meta-first | ✅ Fondasi selesai | 4 | 2 ⏳ |

**Total audit:** 13x, semua LOLOS, nol error.

---

## RENCANA BERIKUTNYA

### Prioritas 1: Hackathon Meta (26 Okt – 4 Nov)

| Fase | Tanggal | Tugas |
|---|---|---|
| A | 26–27 Okt | Uji E2E chat dengan Meta key; Spark 1.3 default |
| B | 28–29 Okt | Endpoint muse-image-1.0 + muse-voice-transcribe-1.0 |
| C | 30–31 Okt | SAM 3.1 untuk video editor; Wonder power mode |
| D | 1–3 Nov | Budget tracker $150; hardening; rekam demo |
| Submit | 4 Nov 21:00 ET | Tenggat |

### Prioritas 2: Setelah hackathon

1. Runtime agen penuh (delegasi multi-agen Mimi)
2. UI slideshow & subtitle generator
3. Perbaiki AI clipper (pin dependensi)
4. Tahap 4.5 (IPC latency, Lighthouse CI, audit independen)
5. Tinjau ulang total → vonis launching → Fase 6 (komersialisasi)

### Menunggu Patria (9 item)

Pixabay key · Pexels key · fal.ai key · Meta key · Postiz · Meta Page · 2 konfirmasi visual · RouterKu (WA SAKA) · Avatar 3D
