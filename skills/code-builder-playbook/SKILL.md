---
name: code-builder-playbook
description: Playbook code builder — penerjemah kebutuhan bisnis menjadi spesifikasi teknis; standar kualitas output development. Knowledge menunggu dokumen final Patria.
owner_agent: strategi-bisnis
version: 0.1.0
example-prompt: Tulis spesifikasi teknis untuk fitur antrean task.
---

# code-builder-playbook

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> ⚠️ **STATUS: struktur skill lengkap, KNOWLEDGE MENUNGGU DOKUMEN FINAL PATRIA.**
> Dokumen knowledge code builder belum ada (Fase 3) — bagian bertanda [KNOWLEDGE PATRIA]
> adalah placeholder dan TIDAK BOLEH diisi dengan karangan. Skill ini menerjemahkan
> kebutuhan bisnis menjadi spesifikasi yang siap dikerjakan developer/AI builder,
> mengikuti standar proyek Minime (`docs/rule.md`).

## Kapan dipakai

Bila tugas butuh output development: spesifikasi fitur, rancangan teknis, kriteria
penerimaan, atau review kualitas hasil kode/AI builder.

## SOP (kerangka kerja)

1. **Pahami kebutuhan bisnis dulu.** Tulis dalam 1 paragraf: masalah apa yang diselesaikan,
   untuk siapa, dan definisi "selesai" menurut user (bukan menurut teknis).
2. **Batasi scope.** Tulis yang eksplisit DI LUAR scope. Fitur tanpa batasan = scope creep.
3. **Spesifikasi teknis.** Untuk tiap fitur: input → proses → output; aturan bisnis;
   penanganan error; dependensi/komponen yang disentuh.
4. **Kriteria penerimaan (acceptance criteria).** Daftar cek yang bisa diuji ya/tidak.
   Tanpa ini, "selesai" tidak bisa diverifikasi.
5. **Standar kualitas Minime** (selaras `docs/rule.md`):
   - Rust: `cargo fmt` + `clippy` bersih; dilarang `unwrap()` di path produksi (gunakan `Result`).
   - Frontend: TypeScript strict; UI rules + anti-pattern Impeccable.
   - Tidak ada secret di kode/log/git; Bahasa Indonesia untuk user-facing, Inggris untuk kode/komentar.
   - Setiap LLM call yang memproses data user tercatat di tabel `runs`.
6. **Risiko & pertanyaan terbuka.** Daftar yang belum diketahui + keputusan yang butuh Patria.
7. **Loop verifikasi saat eksekusi (MeHarness — teruji putaran #1, 20 Okt 2026).**
   Bila agen atau builder yang dipakai agen menjalankan/mengubah kode: setelah
   SETIAP perubahan, jalankan tes yang relevan; bila gagal, baca pesan error,
   perbaiki akar masalahnya, ulangi. Dilarang menyatakan selesai sebelum run
   tes terakhir lulus semua. Bukti & metode: `docs/meharness-lessons.md`;
   set uji: `tests/harness/`.

## Toolbox

- Template spesifikasi: latar & tujuan → scope (in/out) → spesifikasi per fitur →
  acceptance criteria → standar kualitas → risiko & pertanyaan terbuka
- Standar proyek: `docs/rule.md`, `docs/arsitektur.md`
- Pola arsitektur Minime: daemon single-writer SQLite; IPC Unix socket JSON-lines;
  FIFO per agen; pipeline YAML; Keychain untuk secret
- [KNOWLEDGE PATRIA] Standar code builder final Patria (menunggu dokumen)

## Quality bar

- Spesifikasi bisa dikerjakan tanpa menebak (input/proses/output jelas per fitur).
- Acceptance criteria teruji ya/tidak; scope in/out eksplisit.
- Mematuhi standar `docs/rule.md` (fmt/clippy, no-unwrap, no-secret, UI rules, Impeccable).
- Risiko & pertanyaan terbuka terdaftar — tidak ada yang disembunyikan.

## Trigger eskalasi — WAJIB ke Patria bila

- Kebutuhan bisnis ambigu dan menebak akan mengubah arsitektur.
- Scope menyentuh kredensial, pembayaran, atau data sensitif (aturan keras rule.md).
- Keputusan teknologi/arsitektur yang belum ada di `docs/arsitektur.md`.
- Estimasi usaha melebihi budget/waktu yang disetujui.
- [KNOWLEDGE PATRIA] Batasan knowledge final menyusul dokumen.

## Proof of work

Setiap output WAJIB mencantumkan: ringkasan kebutuhan (1 paragraf), scope in/out,
spesifikasi per fitur, acceptance criteria, dan daftar risiko + pertanyaan terbuka.
