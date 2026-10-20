# Minime — Definisi

> **Minime adalah AI Studio with purpose** — aplikasi desktop local-first yang
> mengubah satu orang menjadi satu tim produksi campaign kemanusiaan, sosial,
> dan lingkungan.

## Apa itu Minime

Satu orang + Minime = satu studio. Riset isu, strategi pesan, naskah, produksi
video, sampai distribusi campaign — dikerjakan tim agen AI yang persisten,
dipimpin seorang orkestrator (Mimi, CIO), dengan persetujuan manusia di setiap
langkah penting.

Semua berjalan **lokal di mesin pengguna** — data, riwayat chat, dan tugas
tersimpan di komputermu sendiri, bukan di server orang lain.

## Untuk apa

Minime dibangun untuk satu tujuan: **memproduksi campaign yang berdampak**
— kemanusiaan, sosial, lingkungan. Dari riset → strategi → naskah & konten →
produksi video → distribusi, dalam satu aplikasi.

Contoh pemakaian: kampanye donasi, edukasi publik, gerakan sosial, advokasi
lingkungan — dikerjakan tim kecil atau bahkan satu orang dengan daya dobrak
satu studio.

## Roster

| Agen | Peran |
|---|---|
| **Mimi** (CIO) | Orkestrator: terima permintaan → delegasikan → gabungkan → minta approval |
| **Kreatif** | Naskah, copywriting, storytelling, produksi & editing video |
| **Riset** | Riset isu, data, analisis |
| **Finance** | Anggaran campaign, analisis & backtest (dilarang order live otomatis) |
| **Bisnis Strategis** | Brand, strategi pesan, strategi distribusi |
| **Code Builder** | Aplikasi & landing page campaign |

## Alur produksi

**Video:** generate footage (Veo 3.1 via Gemini) → edit/rakit (Pireel) → QC
(skor 0–100) → approval manusia → distribusi.

**Konten:** riset → strategi → produksi (naskah/copywriting/desain) → QC →
approval → draft distribusi.

Setiap tahap tercatat; tidak ada yang lolos tanpa jejak.

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
   *draft*. Tidak pernah auto-posting dalam kondisi apa pun.
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
