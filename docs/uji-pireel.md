# Rencana Uji: Pireel (18 Okt 2026)

> Keputusan Patria: siapkan untuk diuji; bila lancar → aplikasikan permanen.
> Lokasi uji: **MacBook Patria** (terpandu tahap demi tahap).
> Estimasi: 30–60 menit.

## Tujuan

Membuktikan Pireel bisa menjadi **lapisan editing** di pipeline Minime:
Veo/OpenCreator (generate) → Pireel (edit/rakit) → QC → Approval.

## Prasyarat

- [ ] Node.js 20+ terinstal (`node -v`)
- [ ] `ffmpeg` terinstal (`ffmpeg -version`) — untuk transkrip otomatis
- [ ] 1 klip talking-head mẫu (≤ 2 GB, MP4) — bisa pakai `docs/bukti-fase4/reels-minime.mp4`

## Tahap uji

### Tahap 1 — Instalasi & Studio jalan (10 menit)
```sh
git clone https://github.com/pireel/pireel.git ~/pireel
cd ~/pireel
corepack enable
pnpm install
pnpm dev
```
- [ ] `pnpm dev` jalan tanpa error; URL studio bisa dibuka di browser
- [ ] Bisa import 1 klip MP4 dan muncul di timeline

### Tahap 2 — Koneksi agen via MCP (10 menit)
```sh
npx skills add pireel/pireel-agent
```
- [ ] Ikuti connection guide sampai `get_state` mengembalikan snapshot
      (composition / OFFLINE MODE / no cloud project — ketiganya = terhubung)
- [ ] **Kritis**: bila koneksi butuh akun/OAuth berbayar di pireel.com →
      STOP, lapor ke Patria untuk keputusan (jangan bayar sendiri)

### Tahap 3 — Alur uji penuh (20 menit)
1. [ ] Import klip via agen (local file path) → project terdaftar
2. [ ] Transcript-based edit: "buang jeda mati dan kata filler" → timeline berubah
3. [ ] Caption Bahasa Indonesia terpasang dan selaras ucapan (cek manual 30 detik)
4. [ ] Export 1080x1920 → file MP4 bisa dibuka

### Tahap 4 — Billing check (5 menit)
- [ ] Tidak ada tool `[…CHARGES…]` tersentuh selama uji
- [ ] Catat bila ada prompt pembayaran / kredit

## Rubrik kelulusan

| # | Kriteria | Lolos bila |
|---|----------|-----------|
| 1 | Instalasi | `pnpm dev` jalan di MacBook tanpa error blocker |
| 2 | Koneksi MCP | `get_state` snapshot OK **tanpa** akun berbayar |
| 3 | Import | Klip lokal masuk project via agen |
| 4 | Edit transkrip | Jeda/filler terbuang, makna pembicara utuh |
| 5 | Caption | Bahasa Indonesia, selaras ucapan |
| 6 | Export | MP4 1080x1920 valid & bisa dibuka |
| 7 | Billing | Nol pemakaian `[CHARGES]` |

**Vonis:**
- 7/7 → **PERMANEN**: integrasikan ke pipeline (posisi: setelah generate, sebelum QC)
- 5–6/7 → **BERSYARAT**: perbaiki yang gagal, uji ulang bagian itu
- ≤4/7 → **DITOLAK**: abaikan, tetap dengan rantai Veo → HyperFrames

## Hasil uji (diisi saat eksekusi)

| Tahap | Hasil | Catatan |
|-------|-------|---------|
| 1 | ⬜ | |
| 2 | ⬜ | |
| 3 | ⬜ | |
| 4 | ⬜ | |

**Vonis akhir:** ⬜ PERMANEN / ⬜ BERSYARAT / ⬜ DITOLAK — oleh Patria, tanggal: ____
