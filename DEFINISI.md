# Minime — Definisi

> **Minime adalah aplikasi desktop local-first untuk AI teammates yang persisten**
> — tim agen AI senior yang bekerja untukmu, di komputermu, dengan persetujuanmu.

## Apa itu Minime

Minime mengubah satu aplikasi desktop menjadi sebuah tim kerja AI: dipimpin
seorang orkestrator (Muse, CIO), didukung lima agen spesialis senior. Kamu
memberi tugas dalam bahasa sehari-hari; Muse memecahnya, mendelegasikan ke
agen yang tepat, menggabungkan hasilnya, dan memintakan persetujuanmu sebelum
sesuatu yang penting terjadi.

Semua berjalan **lokal di mesin pengguna** — data, riwayat chat, dan tugas
tersimpan di komputermu sendiri, bukan di server orang lain.

## Roster

| Agen | Peran |
|---|---|
| **Muse** (CIO) | Orkestrator: terima permintaan → delegasikan → gabungkan → minta approval |
| **Kreatif** | Konten, copywriting, storytelling, produksi & editing video |
| **Riset** | Riset mendalam, data, analisis |
| **Finance** | Keuangan bisnis, analisis & backtest (dilarang order live otomatis) |
| **Bisnis Strategis** | Brand, marketing, selling, operasional, strategi |
| **Code Builder** | Membangun aplikasi & UI |

## Model AI

Pengguna memilih modelnya sendiri (model router):

- **Default: Muse** (via Meta Model API)
- **Fallback / agen tambahan:** model lain — Gemini, DeepSeek, model lokal
  gratis (Ollama), atau model custom

Setiap agen bisa memakai model berbeda. Penggunaan token & estimasi biaya
dicatat per agen; opsional budget cap harian.

## Prinsip keras (tidak bisa ditawar)

1. **Approval manusia wajib** — setiap aksi eksternal (kirim, publish,
   transaksi) berhenti di kartu persetujuan. Tidak ada yang jalan diam-diam.
2. **Meta draft-only** — Instagram, Facebook, Threads: agen hanya membuat
   *draft*. Auto posting menunggu persetujuan
3. **Kredensial milik pengguna** — API key & token disimpan di OS keychain
   masing-masing pengguna, tidak pernah di git, tidak pernah dibundel ke
   aplikasi. Bila Minime dipakai publik, setiap user menghubungkan akunnya
   sendiri (Google Drive, Meta, dsb.) — bukan memakai milik pembuat.
4. **Deliverable milik pengguna** — hasil kerja masuk ke Google Drive
   masing-masing pengguna.
5. **Local-first** — tidak ada ketergantungan pada server pusat agar aplikasi
   tetap jalan. Data sensitif tidak diunggah ke internet.
6. **Keuangan aman** — riset, analisis, dan backtest saja; order trading live
   otomatis dilarang keras.

## Arsitektur singkat

- **Desktop app:** Tauri 2 (Rust) + Svelte 5 — macOS & Windows
- **Bahasa UI:** Indonesia (default), Inggris, Mandarin
- **Prinsip UI:** compact but powerful — intuitif untuk pemula, minimal coding

## Status

Tahap 1 (UI/UX klik-tayang): **lolos gate** ✅
Tahap 2 (model router nyata + runtime agen): **berjalan** 🔄

## Riwayat singkat

Minime v1 (arsitektur Tauri + Rust daemon, 16–18 Okt 2026) diarsipkan di
`arsip-v1/`. Kode ini dibangun ulang dari nol dengan pendekatan berbeda
(keputusan Patria, 18 Okt 2026).
