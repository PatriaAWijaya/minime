---
name: html-anime
description: Skill produksi — ubah prompt scene menjadi film anime pendek yang digambar dalam HTML/SVG lalu di-render ke MP4 via HyperFrames, tanpa model video. Mengorkestrasi 5 sub-skill (direct, cel, timing, camera, render).
owner_agent: creative-director
version: 1.0.0
example-prompt: Buatkan bumper opening 8 detik untuk Minime — robot biru minimalis, 1920x1080.
---

# html-anime

> Skill produksi milik **Creative Director**. Berdasarkan `blixvip/html-anime`
> (lisensi **MIT** — di-vendor dengan atribusi, lihat `tools/html-anime/ATRIBUSI.md`).
>
> Prinsip: **prompt scene → shot sheet → agen menggambar frame dalam HTML/SVG →
> render HyperFrames → MP4.** Tidak memakai model video generatif — gambarnya
> ditulis sebagai kode oleh agen, deterministic (file sama → frame sama).

## Kapan dipakai

Bila tugas butuh video pendek bergaya anime yang digambar dengan kode:
bumper opening, intro/outro channel, motion logo, atau short animasi
(8, 12, atau 20 detik; 16:9 atau 9:16). BUKAN untuk: footage realistis,
rekaman orang asli, atau konten yang butuh model video generatif.

## SOP

### 1. Tulis brief scene
- Satu logline: adegan + verb aksi (verb = sakuga shot). Maks 400 karakter.
- Tentukan: durasi (**8, 12, atau 20 detik** — tidak ada durasi lain),
  aspek (**16:9** = 1920×1080, **9:16** = 1080×1920), color script
  (`indigo` | `eclipse` | `noon` | `festival`), dan dialog opsional
  (maks 180 karakter, satu bahasa, tidak diterjemahkan).

### 2. Bangun shot sheet — `compileBrief`
- Jalankan (Node, tanpa dependensi):
  `node -e "import('./tools/html-anime/compile.js').then(m => console.log(m.compileBrief({logline, seconds, aspect, script, dialogue}).markdown))"`
- Sheet berisi: runtime, satu sakuga shot (shot termahal), holds, color script
  TERTUTUP (hanya hex di sheet yang boleh dipakai), dan drawing budget.
- Tampilkan sheet ke Patria bila ia meminta review sebelum menggambar.

### 3. Gambar film — ikuti 5 sub-skill BERURUTAN
Sub-skill di-vendor di `skills/html-anime/sub-skills/` (masing-masing pemilik
satu subjek — baca sesuai urutan, jangan diringkas ulang):
1. `html-anime-direct.md` — sheet adalah film; putuskan shot & serahkan.
2. `html-anime-cel.md` — cat: SVG/HTML flat, satu hard shadow, tanpa airbrush.
3. `html-anime-timing.md` — 24fps on twos; sakuga 8 gambar, lainnya 3.
4. `html-anime-camera.md` — komposisi 16:9/9:16, cut on action.
5. `html-anime-render.md` — tulis `films/<slug>/index.html` sebagai komposisi
   HyperFrames (timeline GSAP paused & sinkron, cut = color dip, tanpa
   `Math.random()`/`Date.now()`).
- Acuan bentuk file: `~/workspace/referensi/html-anime/films/roof/index.html`
  (hanya bentuk file & color-dip cut — JANGAN jadikan karakter/plot/palet).

### 4. Gate lint — WAJIB lolos sebelum render
Dari direktori film:
```
npx hyperframes lint
npx hyperframes validate
```
Keduanya harus pass. Shot times harus menjumlah tepat ke runtime.

### 5. Render → MP4
- Render via tool **`hyperframes`** Minime (sudah ter-wire Fase 4).
- Simpan MP4 + 1–2 frame PNG sebagai deliverable.

### 6. QC → Approval → Deliverable
- QC menilai (rubrik 0–100) → Approval Patria → MP4 menjadi deliverable.
- Distribusi (posting) TETAP mengikuti aturan #3: draft-only, tanpa auto-posting.

## Toolbox

- `tools/html-anime/compile.js` — `compileBrief` (Node, tanpa dependensi).
- `skills/html-anime/sub-skills/*.md` — 5 sub-skill asli (MIT, di-vendor).
- Tool `hyperframes` Minime — render MP4 lokal (`daemon/src/tools/hyperframes.rs`).
- Referensi bentuk file: `~/workspace/referensi/html-anime/films/roof/index.html`.

## Quality bar

- Shot times menjumlah TEPAT ke runtime; tepat SATU sakuga shot.
- Hanya hex dari color script sheet — hex baru = gagal.
- Deterministik penuh: tanpa `Math.random()`, `Date.now()`, baca jam, atau
  `async`/`setTimeout` di timeline; file sama → frame sama.
- Cut memakai color dip; karakter tidak fade-in dari opacity 0.
- `hyperframes lint` + `hyperframes validate` lolos.
- Visual mengikuti identitas yang diminta (untuk Minime: bersih-minimalis,
  selaras UI rules — tanpa gradient ungu-biru, tanpa Inter untuk segalanya).

## Trigger eskalasi — WAJIB ke Patria bila

- Brief menyentuh merek/karya berhak cipta (gaya anime spesifik, karakter existing).
- Dialog/konten sensitif, atau butuh arahan pilihan bahasa.
- Butuh durasi di luar 8/12/20 detik atau aspek di luar 16:9/9:16.
- Ragu soal color script yang paling cocok untuk identitas brand.
- `lint`/`validate` gagal dan penyebabnya tidak jelas setelah 2x percobaan.

## Proof of work

- Shot sheet (markdown) tersimpan per task.
- File `films/<slug>/index.html` + log `lint`/`validate` tersimpan.
- MP4 + frame PNG tercatat sebagai deliverable di DB.
- Setiap LLM call tercatat di tabel `runs` (aturan #15).
