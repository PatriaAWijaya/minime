# Sampel uji skill: html-anime

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `creative-director` dengan playbook stage-level `["html-anime"]`.
> Bukti wiring: `PLAYBOOK AKTIF: html-anime`.
> Status wiring: OK — TERBUKTI.
> Isi lengkap skill: `skills/html-anime/SKILL.md` (orkestrator 5 sub-skill di-vendor MIT).

## Tugas
Buatkan bumper opening 8 detik untuk Minime

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `html-anime` ter-inject ke prompt agen dan di-echo oleh mock provider.
Kualitas output nyata skill hanya bisa dinilai dengan LLM beneran.

```
Hasil penilaian QC terhadap output tahap produksi:
- Kebenaran & sumber: cukup (20/30)
- Kesesuaian brief: baik (22/25)
- Quality bar konten: baik (20/25)
- Kesiapan produksi: baik (18/20)
Total: 80/100 → LOLOS ke Approval dengan catatan ringan.
SKOR: 80
ALASAN: Output sesuai brief dan memenuhi quality bar dengan catatan minor.
PLAYBOOK AKTIF: html-anime
```

## Uji nyata (acceptance test Patria)

Lihat `docs/bukti-fase4/bumper-minime.mp4` — bumper opening Minime nyata
(brief → compileBrief → shot sheet → frame HTML → render HyperFrames → MP4),
dikerjakan sepenuhnya memakai workflow skill ini.
