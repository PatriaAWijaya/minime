# Set Tugas Uji — MeHarness

> Benchmark mini Minime (lihat `docs/meharness.md` §4). Dipakai Loop Tuning
> untuk menguji perubahan konfigurasi agen secara terukur: baseline vs
> kandidat, tugas sama, kondisi sama, penilaian oleh mesin.

## Aturan main

- **Penilaian = verifikasi mesin.** Tiap tugas punya `VERIFY.sh`; exit 0 =
  LULUS, selain itu GAGAL. Tidak ada nilai parsial (standar zero-error:
  tidak boleh ada error yang lolos).
- **Isolasi.** Setiap run bekerja di salinan segar folder tugas; file tes
  dan skrip verifikasi TIDAK boleh diubah agen (dicantumkan di prompt).
- **Keadilan.** Yang berbeda antar-run hanya file instruksi agen
  (`AGENTS.md` dari folder `varian/`). Model, executor (OpenCode), prompt
  tugas, dan batas waktu (600 detik) sama.
- **Kompak.** Varian instruksi dinilai juga dari ukurannya — edit yang
  diadopsi tidak boleh menggelembungkan instruksi tanpa bukti manfaat.

## Struktur

- `code-builder/tugas1-fix/` — perbaiki bug (tipe hasil, pembulatan, clamp)
- `code-builder/tugas2-build/` — implementasi dari spesifikasi (`parse_rupiah`)
- `code-builder/tugas3-kualitas/` — perbaiki perilaku + standar kualitas
  (tanpa print debug, tanpa default mutable, docstring, format ribuan)
- `varian/` — instruksi agen yang diuji: `baseline.md`,
  `kandidat-umum.md`, `kandidat-spesifik.md`

## Cara menjalankan (di MacBook, via executor OpenCode)

Lihat `run-probe.sh` — menyalin tiap tugas ke sandbox, memasang varian
sebagai `AGENTS.md`, menjalankan `opencode run`, lalu `VERIFY.sh`, dan
menulis `result.txt` per run (exit code executor, exit code verifikasi,
durasi).
