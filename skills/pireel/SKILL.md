---
name: pireel
description: "AI video editor (alternatif CapCut) yang bisa disetir agen via MCP — transcript-aware cuts, captions, talking-head cleanup, multi-source assembly, export. Diadaptasi dari pireel/pireel (AGPL-3.0) — HANYA layanan eksternal."
owner_agent: creative-director
version: 1.0.0
example-prompt: Rapikan talking-head ini: buang jeda mati dan filler, pasang caption, lalu export 1080x1920.
---

# pireel

> Skill editing video AI milik **Creative Director**.
> Diadaptasi dari [`pireel/pireel`](https://github.com/pireel/pireel)
> (lisensi **AGPL-3.0**).
> Referensi: `~/workspace/referensi/pireel/`.
>
> ✅ **STATUS: PERMANEN** (vonis Patria, 18 Okt 2026 — uji 7/7 lolos,
> lihat `docs/uji-pireel.md`).
>
> ⚠️ **ATURAN LISENSI (setara postiz/OpenViking): AGPL-3.0 aman HANYA sebagai
> layanan eksternal** — via MCP cloud `pireel.com/api/studio/mcp` + OAuth.
> **DILARANG mem-vendor kode Pireel ke repo Minime.**

## Posisi di pipeline

```
Veo 3.1 / OpenCreator (generate) → PIREEL (edit/rakit) → QC → Approval
```

Pireel = **lapisan editing**, BUKAN generator. Ia tidak menggantikan Veo
(text-to-video) — ia merapikan dan merakit hasil generate + footage nyata.

**Arsitektur koneksi (hasil uji 18 Okt 2026):** MCP hanya tersedia via
**cloud** (pireel.com + OAuth akun free) — Studio lokal (OSS) tidak punya
endpoint MCP (frontend-only, "no backend wired"). Bridge MCP bekerja lewat
**tab studio yang terbuka** di browser. Pola sama seperti Viralin/postiz:
layanan eksternal via API, aman AGPL.

## Kapan dipakai

- Talking-head: buang jeda mati, filler, false start, retake — via transkrip.
- Caption terbaca otomatis, selaras dengan ucapan (+ terjemahan Indonesia).
- Rakit multi-source: video + gambar + audio + grafis → satu timeline.
- Potong klip panjang jadi shorts; varian multi-platform.
- Reframe, B-roll, musik latar.

BUKAN untuk: generasi footage dari nol (pakai Veo/OpenCreator), motion
graphics programatik (pakai HyperFrames/manimgx).

## SOP — Operasional

### 1. Prasyarat (sekali saja)

- Akun pireel.com paket **Free** ($0, tanpa kartu kredit, 20 kredit/hari).
- OAuth login untuk koneksi MCP.
- Tab studio pireel.com **terbuka** selama agen bekerja (bridge).

Gotcha MacBook: `corepack enable` butuh `sudo` (`sudo corepack enable`).

### 2. Hubungkan agen via MCP

Endpoint: `https://pireel.com/api/studio/mcp` (streamable HTTP).
Verifikasi dengan `get_state`:
- Composition snapshot → terhubung ke project.
- `OFFLINE MODE` → terhubung; edit project cloud langsung.
- `no cloud project` → terhubung; buat project di studio dulu.

### 3. Alur edit standar

1. `get_state` → snapshot komposisi (sekali per sesi).
2. `get_transcript` (granularity=segments) → baca ucapan.
3. `remove_silence` dulu untuk pacing; `remove_words` (granularity=words
   → wordIds) untuk filler/false start spesifik.
4. `set_captions` on:true → caption; `translations: {lang:"Indonesian",
   items:[{index,text}]}` untuk baris kedua Indonesia.
5. `export` action:start {resolution:1080, fps:30, format:"mp4"} → poll
   action:status sampai done → file ter-download via browser.

**Catatan:** posisi transkrip = source seconds (tidak bergeser saat
timeline di-cut); wordIds bergeser setelah `remove_words` → baca ulang
sebelum cut berikutnya.

## Scenario skills bawaan (referensi)

Di `packages/studio-engine/src/scenario-skills/content/`:
- `talking-head-edit` — edit berbasis ucapan (jangan samakan dengan
  auto-silence-removal; butuh tesis editorial).
- `montage-edit` — rakitan multi-source.
- `audio-and-music` — musik & desain suara.

## Billing (terverifikasi 18 Okt 2026)

- Orkestrasi agen (`get_state`, `remove_words`, `set_captions`, `export`)
  **gratis** — jalan di subscription agen sendiri, bukan kredit Pireel.
- Hanya tool berlabel `[CHARGES]` yang menagih kredit: `get_transcript`
  (transkripsi), `inspect_media` (analisis visual), generasi media.
- Uji penuh (transkrip 40 dtk + edit + caption + export) = **2 kredit**
  dari jatah 20/hari paket Free. $0 keluar.
- Produksi bervolume → pertimbangkan Pro ($17–20/bln). Keputusan Patria.

## Quality bar

- Setiap edit penting bisa di-inspeksi di Studio sebelum export.
- Caption Bahasa Indonesia selaras ucapan (cek QC).
- Tidak ada langkah manual yang terjebak di luar pipeline tanpa dokumentasi.
- Tool `[CHARGES]` tidak pernah dipakai tanpa approval Patria.

## Trigger eskalasi — WAJIB ke Patria bila

- Ada tool `[CHARGES]` yang perlu dipakai di luar batas wajar → approval dulu.
- Butuh upgrade ke paket berbayar → keputusan Patria.
- Hasil edit gagal / tidak sesuai brief → lapor + usulan.

## Proof of work

- Hasil uji lengkap di `docs/uji-pireel.md` (7/7, vonis PERMANEN 18 Okt 2026).
- Setiap job produksi: catat kredit terpakai di laporan.
