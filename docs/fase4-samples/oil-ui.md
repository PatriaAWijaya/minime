# Sampel uji skill: oil-ui

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `strategi-bisnis` dengan playbook stage-level `["oil-ui"]`.
> Bukti wiring: `PLAYBOOK AKTIF: oil-ui`.
> Status wiring: OK — TERBUKTI.
> Isi lengkap skill: `skills/oil-ui/SKILL.md` (diadaptasi dari `oil-oil/oil-ui`, MIT).

## Tugas
Desainkan landing page web untuk aplikasi kasir UMKM

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `oil-ui` ter-inject ke prompt agen dan di-echo oleh mock provider.
Kualitas output nyata skill hanya bisa dinilai dengan LLM beneran.

```
[MOCK OUTPUT — model: mock-echo]
Ringkasan eksekusi mock untuk task ini. Konteks diterima: [{...}]
PLAYBOOK AKTIF: oil-ui
(Ini adalah output deterministik dari mock provider untuk verifikasi pipeline.)
```

## Catatan uji

- Saat uji ditemukan dan diperbaiki: frontmatter `SKILL.md` awal tidak valid
  (deskripsi tak dikutip mengandung `: `) sehingga loader melewatkan skill ini.
  Setelah dikutip, skill terdaftar (21 skills) dan ter-inject dengan benar.
- Update-check bawaan (`scripts/check_update.sh`) DINONAKTIFKAN di salinan
  Minime (aturan #12: aksi eksternal baru wajib konfirmasi Patria);
  prompt upsell oil-ui-pro dihapus dari alur.
- Dipakai berpasangan dengan `code-builder-playbook` untuk aplikasi web-based:
  eksplorasi 2–3 arah desain → bandingkan side-by-side → refine terhadap
  screenshot nyata dari built-in browser Minime.
