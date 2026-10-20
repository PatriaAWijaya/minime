# MeHarness — Library Pelajaran (Improvement Library)

> Catatan putaran Loop Tuning MeHarness (`docs/meharness.md` §3): edit yang
> diusulkan, bukti probe, keputusan adopsi/tolak, dan pelajaran yang berlaku
> lintas putaran. Pelajaran yang terbukti di ≥3 putaran dipromosikan jadi
> aturan tetap.

---

## Putaran #1 — Code Builder (20 Okt 2026)

**Pertanyaan:** apakah edit kecil pada instruksi agen (komponen *skills /
tool guidance*) menaikkan kinerja terukur dibanding baseline?

**Set uji:** `tests/harness/code-builder/` — 3 tugas (fix bug, implementasi
dari spesifikasi, kualitas kode), penilaian verifikasi mesin (exit 0 =
lulus, tanpa nilai parsial). Sanity check: kode awal ketiga tugas GAGAL
semua tes sebelum diuji ke agen.

**Executor:** OpenCode 1.19.3 di MacBook (substrate yang sama dengan yang
didukung Harness²), model bawaan konfigurasi Patria. Yang berbeda antar-run
hanya `AGENTS.md` (varian instruksi). Batas 600 detik/run.

**Hasil probe** (semua sel valid; 1 sel diulang — lihat pelajaran infra):

| Varian | Tugas 1 | Tugas 2 | Tugas 3 | Lulus | Total | Ukuran instruksi |
|---|---|---|---|---|---|---|
| baseline | ✅ 47 dtk | ✅ 73 dtk | ✅ 196 dtk | 3/3 | 316 dtk | 444 B |
| **kandidat-umum** (loop verifikasi) | ✅ 36 dtk | ✅ 40 dtk | ✅ 68 dtk | **3/3** | **144 dtk** | 735 B |
| kandidat-spesifik (checklist kasus tepi) | ✅ 58 dtk | ✅ 56 dtk | ✅ 81 dtk | 3/3 | 195 dtk | 899 B |

Detail verifikasi per run: semua tes hijau penuh (tugas 1: 6/6, tugas 2:
8/8, tugas 3: 3/3 + cek kualitas lulus) — arsip di
`tests/harness/runs/2026-10-20-code-builder/HASIL.md`; log mentah di
MacBook (`~/harness-runs/`).

**Keputusan COMPOSE:**
- Akurasi: ketiga varian memenuhi standar zero-error (3/3). Tidak ada
  kandidat yang gugur karena error.
- Pembeda: kecepatan & kekompakan. Kandidat-umum **tercepat di setiap
  tugas** (urutan konsisten di 3 tugas independen, n=1 per sel) dan total
  54% lebih cepat dari baseline; kandidat-spesifik lebih lambat dan lebih
  gemuk.
- **ADOPSI: kandidat-umum** — seksi "Loop verifikasi (WAJIB)" masuk
  `skills/code-builder-playbook/SKILL.md` sebagai SOP langkah 7.
- Kandidat-spesifik DITOLAK untuk sekarang: checklist-nya tidak memberi
  keunggulan terukur di set uji ini (kemungkinan karena tugas belum cukup
  kejam pada kasus tepi). Bisa diuji ulang bila set uji diperberat.

**Pelajaran lintas putaran:**
1. Instruksi prosedural yang pendek ("jalankan tes setelah setiap
   perubahan; jangan selesai sebelum hijau") berdampak lebih besar daripada
   checklist pengetahuan yang lebih panjang. Kompak menang atas lengkap.
2. Tugas yang mudah membuat semua varian lulus akurasi — untuk putaran
   berikutnya set uji perlu tugas yang lebih sulit agar perbedaan kualitas
   terlihat, bukan hanya kecepatan.

**Pelajaran infrastruktur (di luar loop, tapi menentukan validitas):**
3. OpenCode paralel bertabrakan: 3 instance bersamaan → error
   `database is locked` (SQLite OpenCode). Satu run kandidat-spesifik
   tugas 1 gugur karena ini dan **diulang sendirian** (hasil rerun dipakai).
   Aturan: probe antar-varian dijalankan **berurutan**, atau isolasi data
   dir per run.
4. Jembatan MacBook (bridge HTTP) satu utas: `/exec` yang berjalan lama
   memblokir semua request lain. Pekerjaan panjang WAJIB dilepas sebagai
   proses lepas (nohup) + polling file hasil — pola yang dipakai
   `run-probe.sh`. Jangan jalankan tugas menit-an di foreground `/exec`.
