# Hasil Probe MeHarness — Putaran #1, Code Builder (20 Okt 2026)

Executor: OpenCode 1.19.3 (MacBook), model bawaan. Perbedaan antar-run hanya
`AGENTS.md`. Verifikasi mesin per tugas (VERIFY.sh, exit 0 = lulus).

| Varian | Tugas 1 (fix) | Tugas 2 (build) | Tugas 3 (kualitas) | Total |
|---|---|---|---|---|
| baseline | lulus, 47 dtk (6/6 tes) | lulus, 73 dtk (8/8) | lulus, 196 dtk (3/3 + kualitas) | 316 dtk |
| kandidat-umum | lulus, 36 dtk (6/6) | lulus, 40 dtk (8/8) | lulus, 68 dtk (3/3 + kualitas) | 144 dtk |
| kandidat-spesifik | lulus, 58 dtk (6/6) * | lulus, 56 dtk (8/8) | lulus, 81 dtk (3/3 + kualitas) | 195 dtk |

\* Run pertama kandidat-spesifik/tugas1 gugur oleh error infrastruktur
OpenCode (`database is locked` karena 3 instance paralel) — diulang
sendirian, hasil rerun yang dipakai.

Keputusan: adopsi kandidat-umum (lihat `docs/meharness-lessons.md`).
Log mentah per run tersimpan di MacBook: `~/harness-runs/<varian>/<tugas>/`
(opencode.log, verify.log, result.txt).
