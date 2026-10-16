---
name: manimgx
description: "Skill produksi video pembelajaran — tulis scene animasi Python (API kompatibel Manim CE, gaya 3Blue1Brown: matematika/sains) lalu render ke MP4 via engine ManimGX (Rust + wgpu). Alternatif html-anime untuk konten edukasi."
owner_agent: creative-director
version: 1.0.0
example-prompt: Buatkan video 20 detik yang menjelaskan mengapa luas lingkaran πr².
---

# manimgx

> Skill produksi milik **Creative Director** — alternatif `html-anime` khusus
> **video pembelajaran** (matematika, sains, konsep teknis).
> Berdasarkan [`academa-labs/manimgx`](https://github.com/academa-labs/manimgx)
> (lisensi **MIT**). Referensi: `~/workspace/referensi/manimgx/`
> (contoh: `examples/`, panduan agen: `skills/manimgx/SKILL.md` asli).
>
> Prinsip: **tulis scene Python (API Manim CE) → `manimgx inspect` (storyboard) →
> `manimgx render` → MP4.** Engine Rust + wgpu; tanpa GPU tetap jalan via
> software (lebih lambat).

## Kapan dipakai

Bila tugas butuh video yang **menjelaskan konsep** lewat animasi presisi:
rumus matematika, fisika, algoritma, visualisasi data bergerak, diagram
beranimasi. BUKAN untuk: bumper sinematik (pakai `html-anime`), footage
realistis, atau konten yang butuh model video generatif.

## SOP

### 1. Tulis scene Python
- `import manimgx as m`; satu `class NamaScene(m.Scene)` dengan method `construct`.
- Waktu berjalan lewat `self.play(...)` / `self.wait(...)`; `self.add(...)` hanya menaruh objek.
- Komposisi: `VGroup`, `next_to`, `arrange`, `to_edge` agar label mengikuti subjek.
- Teks: `m.Text` untuk prosa; `m.MathTex(r"...")` (raw string) untuk rumus.
- Animasi nilai berubah: `ValueTracker` + geometri turunan dari nilainya
  (jangan pakai hitungan frame — ganti fps tidak boleh mengubah gerak).
- Scene 3D: pakai `m.ThreeDScene`.
- Contoh kanonik: `~/workspace/referensi/manimgx/examples/circle_area.py`.

### 2. Inspeksi — storyboard dulu
```
manimgx inspect scene.py NamaScene
```
Lihat storyboard: perbaiki overlap/clipping sebelum render final.
Untuk 3D hanya storyboard (tanpa cek layout 2D).

### 3. Render draft → review → render final
```
manimgx render scene.py NamaScene -r 1280x720 --fps 30 -o draft.mp4
```
Tonton draft (gerakannya, bukan cuma still frame). Final:
`-r 1920x1080 --fps 60` (default bila diminta kualitas penuh).

### 4. QC → Approval → Deliverable
- QC menilai (rubrik 0–100) → Approval Patria → MP4 + source `.py` menjadi deliverable.
- Distribusi TETAP aturan #3: draft-only, tanpa auto-posting.

## Toolbox

- Tool **`manimgx`** Minime (`daemon/src/tools/manimgx.rs`): op `render`
  (`scene_file`, `scene`, `output`, `resolution`, `fps`) dan `inspect`.
- CLI langsung: `manimgx render|inspect` (butuh Python 3.13+; install:
  `uv tool install --python 3.13 manimgx`).
- Referensi API: `~/workspace/referensi/manimgx/examples/` (30+ contoh).

## Quality bar

- `inspect` tanpa masalah layout sebelum render final.
- Gerakan bermakna: tiap `play` mengilustrasikan satu ide; tidak ada objek
  menganggur tanpa tujuan.
- Label mengikuti subjeknya (tidak overlap, tidak kepotong di tepi).
- Ganti fps tidak mengubah gerak (updater pakai waktu/state, bukan frame).
- MP4 terverifikasi via ffprobe (durasi + resolusi sesuai permintaan).

## Trigger eskalasi — WAJIB ke Patria bila

- Konsep yang dijelaskan butuh validasi kebenaran (rumus/teori di luar
  pengetahuan umum) — jangan mengarang matematika/fisika.
- Render gagal 2x dengan error yang tidak jelas.
- Butuh durasi > 5 menit atau resolusi > 1080p (pertimbangkan waktu render).

## Proof of work

- Source `.py` scene tersimpan per task (editable, bisa di-render ulang).
- Storyboard `inspect` + MP4 tercatat sebagai deliverable di DB.
- Setiap LLM call tercatat di tabel `runs` (aturan #15).
- **Terverifikasi 17 Okt 2026**: `examples/circle_area.py` → MP4 nyata
  17.9 detik, 1280×720@30fps (software render, tanpa GPU) —
  `docs/bukti-fase4/manimgx-sample.mp4`.
