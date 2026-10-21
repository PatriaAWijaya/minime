---
name: reverse-engineer-anything
description: Reverse engineering binary, paket, dan aplikasi yang sudah di-ship dengan REA — dekompilasi, perbandingan versi, dan verifikasi klaim terhadap artefak aktual.
owner_agent: code
version: 1.0.0
example-prompt: Analisis binary aplikasi ini dan jelaskan cara kerja fitur login-nya di level kode.
---

# reverse-engineer-anything

Skill reverse engineering milik agen Code Builder. Diadopsi dari `morluto/rea` (MIT) — pola dan instruksi diadaptasi untuk Minime, kode tidak disalin. Referensi penuh: `~/workspace/rea-morluto/.agents/skills/reverse-engineer-anything/`.

## Kapan dipakai

Gunakan skill ini ketika klaim bergantung pada **binary, paket, atau aplikasi yang sudah di-ship**: menjelaskan cara kerja fitur dari artefak jadi, membandingkan dua versi, mendekompilasi kode, atau memverifikasi perilaku runtime. JANGAN dipakai untuk analisis arsitektur repository source biasa — untuk itu pakai tool repository normal.

## Prasyarat

Skill ini hanya berisi instruksi; ia tidak otomatis meregistrasi MCP server REA atau menginstal engine analisis. REA tersedia via MCP (`rea-agents`) atau CLI (`npx -y rea-agents@latest ...`). Engine native (Hopper/Ghidra/IDA) hanya dibutuhkan untuk binary native; inspeksi JavaScript statis tidak butuh engine apapun.

## Workflow

1. **Cek tool.** Jika tool REA tersedia di sesi dan registrasinya tidak basi, langsung ke target — jangan jalankan diagnostik/setup berulang.
2. **Diagnosa blocker.** Jika tool tidak ada: `npx -y rea-agents@latest doctor --json`. Bedakan: registrasi hilang (repair konfigurasi), sesi butuh reconnect, atau provider spesifik hilang. Untuk repair: `npx -y rea-agents@latest setup --dry-run --json` dulu, tunjukkan rencana ke user, minta persetujuan, baru apply `--yes`.
3. **Fallback CLI** (tanpa setup MCP) untuk aplikasi JavaScript/ASAR:
   `npx -y rea-agents@latest analyze-javascript-application /absolute/path/ke/app --json`
4. **Evidence.** Baca `normalized_result`, kutip `evidence_id` sebagai referensi stabil. Jaga limitasi dan unknowns; gunakan `record_unknown` untuk pertanyaan terbuka. Untuk perbandingan versi pakai `compare_artifacts`/`compare_functions` dengan data yang kompatibel.

## Output

Laporan: temuan + evidence ID, limitasi analisis, dan hal yang masih belum diketahui beserta otoritas yang dibutuhkan untuk menjawabnya.

## Aturan

- Jangan ulangi panggilan yang sama tanpa perubahan setelah timeout.
- REA setup tidak menginstal Node.js, npm, Homebrew, Java, Ghidra, IDA, atau JADX — gunakan prasyarat yang ada.
- Referensi per target di repo referensi: `native-and-artifacts.md`, `javascript-applications.md`, `android-applications.md`, `runtime-observation.md`, `evidence-workflows.md`, `connection-and-recovery.md`.
