---
name: pireel
description: "AI video editor (alternatif CapCut) yang bisa disetir agen via MCP — transcript-aware cuts, captions, talking-head cleanup, multi-source assembly, export. Diadaptasi dari pireel/pireel (AGPL-3.0) — HANYA layanan eksternal."
owner_agent: creative-director
version: 0.1.0
example-prompt: Rapikan talking-head ini: buang jeda mati dan filler, pasang caption, lalu export 1080x1920.
---

# pireel

> Skill editing video AI milik **Creative Director**.
> Diadaptasi dari [`pireel/pireel`](https://github.com/pireel/pireel)
> (lisensi **AGPL-3.0**).
> Referensi: `~/workspace/referensi/pireel/`.
>
> ⚠️ **STATUS: UJI COBA** (keputusan Patria, 18 Okt 2026).
> Permanen hanya bila lolos `docs/uji-pireel.md`.
>
> ⚠️ **ATURAN LISENSI (setara postiz/OpenViking): AGPL-3.0 aman HANYA sebagai
> layanan eksternal** — jalankan Studio terpisah (`pnpm dev`), daemon/agent
> berkomunikasi via MCP. **DILARANG mem-vendor kode Pireel ke repo Minime.**

## Posisi di pipeline (bila lolos uji)

```
Veo 3.1 / OpenCreator (generate) → PIREEL (edit/rakit) → QC → Approval
```

Pireel = **lapisan editing**, BUKAN generator. Ia tidak menggantikan Veo
(text-to-video) — ia merapikan dan merakit hasil generate + footage nyata.

## Kapan dipakai

- Talking-head: buang jeda mati, filler, false start, retake — via transkrip.
- Caption terbaca otomatis, selaras dengan ucapan.
- Rakit multi-source: video + gambar + audio + grafis → satu timeline.
- Potong klip panjang jadi shorts; varian multi-platform.
- Reframe, B-roll, musik latar.

BUKAN untuk: generasi footage dari nol (pakai Veo/OpenCreator), motion
graphics programatik (pakai HyperFrames/manimgx).

## SOP — Uji coba (MacBook Patria)

### 1. Jalankan Studio

```sh
git clone https://github.com/pireel/pireel.git ~/pireel
cd ~/pireel
corepack enable
pnpm install
pnpm dev
```

Buka URL yang muncul di terminal.

### 2. Hubungkan agen via MCP

```sh
npx skills add pireel/pireel-agent
```

Ikuti connection guide; verifikasi dengan memanggil `get_state`:
- Composition snapshot → terhubung ke tab studio live.
- `OFFLINE MODE` → terhubung; edit project cloud langsung.
- `no cloud project` → terhubung; import video / buat project dulu.

### 3. Alur uji 1 (kriteria kelulusan)

1. Import 1 klip talking-head lokal (≤ 2 GB; transkrip otomatis bila `ffmpeg` ada).
2. Minta agen: buang jeda mati + filler via transkrip.
3. Minta agen: pasang caption Bahasa Indonesia.
4. Export 1080x1920 → periksa hasil manual di QC.

**Lolos bila**: keempat langkah jalan tanpa error blocker, output bisa
dibuka dan caption selaras. Lihat `docs/uji-pireel.md` untuk rubrik penuh.

## Scenario skills bawaan (referensi)

Di `packages/studio-engine/src/scenario-skills/content/`:
- `talking-head-edit` — edit berbasis ucapan (jangan samakan dengan
  auto-silence-removal; butuh tesis editorial).
- `montage-edit` — rakitan multi-source.
- `audio-and-music` — musik & desain suara.

## Billing

- Orkestrasi agen jalan di subscription agen sendiri, **bukan** kredit Pireel.
- Hanya tool berlabel `[…CHARGES…]` (generasi gambar/video, LLM fallback)
  yang menagih kredit Pireel → hindari selama uji coba.

## Quality bar (bila permanen)

- Setiap edit penting bisa di-inspeksi di Studio sebelum export.
- Caption Bahasa Indonesia selaras ucapan (cek QC).
- Tidak ada langkah manual yang terjebak di luar pipeline tanpa dokumentasi.
- Billing `[CHARGES]` tidak pernah tersentuh tanpa approval Patria.

## Trigger eskalasi — WAJIB ke Patria bila

- Studio tidak bisa jalan di MacBook (error install/build).
- Koneksi MCP butuh akun/OAuth berbayar di pireel.com → keputusan Patria.
- Ada tool `[CHARGES]` yang perlu dipakai → approval dulu.
- Hasil uji gagal → lapor + usulan, jangan paksa permanen.

## Proof of work

- Hasil tiap langkah uji dicatat di `docs/uji-pireel.md` + tabel `runs`.
- Vonis permanen/tidak hanya dari Patria setelah rubrik lolos.
