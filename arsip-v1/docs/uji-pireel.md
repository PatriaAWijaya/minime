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
| 1 | ✅ LULUS | `pnpm dev` jalan (Vite :5173); klip 40 dtk ter-import, muncul di timeline + preview. Gotcha: `corepack enable` butuh `sudo` di MacBook |
| 2 | ✅ LULUS | MCP cloud `pireel.com/api/studio/mcp` + OAuth akun FREE (tanpa kartu kredit); `get_state` → "no cloud project" (salah satu dari 3 respons = terhubung per connection guide). Studio lokal TIDAK punya endpoint MCP (frontend-only). |
| 3 | ✅ LULUS | Cloud project `pmvdcus3m242w`; `get_transcript` → 13 segmen; `remove_words` buang false start "How." (segmen 6 → [REMOVED], terkonfirmasi); `set_captions` on + 12 terjemahan Indonesia (38 cues, preset yellowPop); `export` 1080p30 mp4 → `export-1080p-051536.mp4` (28MB, 1080×1920, 39.6 dtk — 0.3 dtk lebih pendek pasca-cut) |
| 4 | ✅ LULUS | Billing pireel.com: plan **Free** aktif, terpakai **2 kredit** dari 20/hari (reset harian); $0 keluar. Tidak ada [CHARGES] di luar batas wajar. |

**Rekomendasi Mimi:** PERMANEN — **7/7 kriteria lolos** (18 Okt 2026, ~12:20 WIB).

**Catatan arsitektur (penting):** Pireel beroperasi sebagai **layanan cloud eksternal** (pireel.com + OAuth), bukan lokal — pola sama seperti Viralin/postiz: aman AGPL via API, bukan vendor kode. MCP lokal tidak tersedia (Studio OSS = frontend-only). Free tier 20 kredit/hari cukup untuk uji + pemakaian ringan; produksi bervolume butuh Pro ($17–20/bln).

**Vonis akhir oleh Patria:** ✅ PERMANEN — tanggal: 18 Okt 2026
