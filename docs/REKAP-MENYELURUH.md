# REKAP MENYELURUH — Minime v2 (18 Okt – 21 Okt 2026)

> Disusun 21 Okt 2026 atas perintah Patria. 255 commit, 88 entri log.

---

## BAGIAN 1: YANG SUDAH SELESAI

### Fase 0 — Fondasi ✅
- Keputusan stack: Tauri 2 + Svelte 5 (clean-room dari pola UI OpenBot)
- Repo publik `PatriaAWijaya/minime`, struktur `app/` + `docs/`
- Aturan keras: kredensial di Keychain, no auto-posting, no live-trading

### Tahap 1 — UI/UX ✅ GATE LOLOS (vonis 19 Okt)
- Layout 4 kolom, 6 agen seed, 3 bahasa (ID/EN/ZH)
- Tema monokrom + #4FA6F0, warna pastel per agen, dot status terkunci
- Kanban 5 kolom, Koneksi (Svelte Flow), 10 kategori Pengaturan fungsional
- 30 skills di Marketplace tab Tools

### Tahap 2 — Model Router ✅ GATE LOLOS (vonis 20 Okt)
- Provider: Ollama (default gratis) → OpenCode fallback; berbayar: Muse → Gemini → DeepSeek
- Keychain + fallback `security` CLI, SQLite riwayat + usage, skills engine
- 9 nama model gratis terverifikasi asli via models.dev

### Tahap 2.5 — Alur Kerja + Polish ✅
- Model agen aktif, folder = project, Mimi sebagai parent kanvas otomatis

### Tahap 3 — Tools & Distribusi ✅ GATE LOLOS BERSYARAT (vonis 21 Okt)
- **3.1** Tools Gateway: 12 tool registry, 6 command, allowlist per agen
- **3.2** Approval Gate: Setuju/Revisi/Tolak + notifikasi native
- **3.3** Meta/Postiz draft-only TERBUKTI struktural + consent gate autopost
- **3.4** Pireel TERHUBUNG nyata (OAuth jujur, token di Keychain)
- **3.5** Google Drive: kode lengkap (upload multipart nyata)
- **3.6** Runtime: 3 dispatch arm + 3 gate jujur

### Tahap 4 — Rilis (sebagian besar selesai)
- Security audit bersih, CSP ketat, release bundle 16MB self-contained
- Avatar 3D 6 hewan + logo + ikon app baru terpasang
- Tombol ganti avatar diperbaiki (picker berfungsi)
- 6x audit penuh LOLOS (terakhir #6, 21 Okt ~10:35)

---

## BAGIAN 2: BELUM SELESAI / TERTANGGUH

| # | Item | Status | Kenapa tertangguh |
|---|------|--------|-------------------|
| 1 | Avatar animasi (Tahap 1) | ⏳ | Patria mau buat sendiri; tidak blocking |
| 2 | RouterKu | 🚫 | Gateway URL tidak diketahui + saldo 0; menunggu tiket support + top-up QRIS oleh Patria |
| 3 | Google Drive terhubung | ⏳ | Menunggu OAuth Client ID (Desktop app) dari Google Cloud Console oleh Patria |
| 4 | Windows installer (.exe) | ⏳ | Workflow GH Actions siap di `docs/`; token tanpa scope workflow → Patria pasang manual |
| 5 | Pipeline E2E penuh via UI | ⏳ | Komponen teruji satuan; butuh sesi uji klik manual oleh Patria |
| 6 | Tahap 4.5 sisa | ⏳ | Latensi IPC formal, Lighthouse CI, audit independen — belum dikerjakan |
| 7 | MeHarness ECC gelombang 1 | ⏳ | Kurasi 7 skill belum dimulai |
| 8 | AgentShield | ⏳ | Evaluasi konsep belum dimulai |
| 9 | STT nyata (Fase Voice) | ⏳ | Masih simulasi; butuh integrasi Whisper/API |
| 10 | WebDriver AFK | 🅿️ | Diparkir atas keputusan Patria |
| 11 | Runtime multi-agen penuh | ⏳ | Chat → Mimi orkestrasi → eksekusi paralel belum penuh |

---

## BAGIAN 3: RENCANA PENGERJAAN BERIKUTNYA

### Prioritas 1 — Bisa dikerjakan sekarang (tanpa Patria)
1. **Tahap 4.5**: ukur latensi IPC formal, siapkan Lighthouse CI config
2. **MeHarness**: kurasi ECC gelombang 1 (7 skill → format Minime)
3. **E2E test script**: siapkan skenario uji klik untuk Patria

### Prioritas 2 — Menunggu input Patria
4. Google Drive Client ID → uji upload ke folder MIMI
5. Pasang workflow Windows manual → trigger build .exe
6. Tiket support + top-up RouterKu → aktifkan provider
7. Sesi uji E2E + uji klik avatar picker oleh Patria
8. Keputusan avatar animasi final

### Prioritas 3 — Setelah semua di atas
9. Audit independen (konsultan performa)
10. Tinjau ulang semua tahap → uji project asli pertama
11. Vonis launching oleh Patria
12. Fase 6 komersil (installer, lisensi, Stripe)

---

*Catatan: item Prioritas 2 tidak bisa dipercepat dari sisi Mimi — bola ada di tangan Patria.*
