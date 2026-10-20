# Scoring Kinerja — Tahap 4.5 (Benchmark Internal)

> Pengukuran pertama: 20 Okt 2026 sore, di MacBook Patria
> (Apple Silicon, aarch64) pada **build RELEASE** Minime 0.1.0.
> Gate rilis penuh tetap: benchmark ini + Lighthouse CI tiap rilis +
> satu audit konsultan performa independen sebelum rilis publik.

## Hasil Build Release (Tahap 4.2)

| Artefak | Debug | Release |
|---|---|---|
| Minime.app | 52,00 MB | **16,14 MB** |
| DMG (aarch64) | 13,71 MB | **6,91 MB** |

Build release pertama yang berhasil penuh (bundel `.app` + `.dmg`).

## Benchmark Internal — Build Release

| Metrik | Hasil | Catatan |
|---|---|---|
| Startup: proses tampil | 0,13–0,15 dtk | 3 run, konsisten |
| Startup: siap (CPU < 2%) | 0,2–3,3 dtk | run dingin ±3 dtk, hangat < 1 dtk |
| CPU idle | 0,3–0,7% | 5 sampel @ 5 dtk setelah 20 dtk jalan |
| RAM idle | 106–115 MB | didominasi WebView sistem |
| Chat (Ollama qwen2.5:3b) | total 2,0–2,3 dtk | jawaban pendek; load model ±0,01–0,03 dtk (model sudah resident) |
| Kecepatan generate model | 21–24 token/dtk | sisi model lokal; latensi IPC aplikasi di atasnya kecil |

## Masih Tersisa untuk Gate 4.5 Penuh

- [ ] Latensi IPC formal (panggilan command terukur, p50/p95)
- [ ] CPU/RAM saat aktif bekerja (agen menjalankan tugas nyata)
- [ ] Lighthouse CI terpasang sebagai gate otomatis tiap rilis
- [ ] Audit konsultan performa independen (satu kali, pra-rilis publik)
- [ ] Uji instalasi bersih dari DMG release (mesin/profil bersih)

## Vonis Sementara

Arah "ringan tapi berperforma" terbukti pada build release: bundel
kecil (DMG < 7 MB), startup sub-detik untuk tampil, idle nyaris nol.
RAM ±110 MB adalah lantai WebView, bukan kebocoran — stabil lintas
5 sampel dan sama dengan build debug.
