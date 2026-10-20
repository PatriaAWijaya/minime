# Minime — AI Studio with Purpose

Minime adalah AI studio dengan tujuan: aplikasi desktop local-first yang
mengubah satu orang menjadi satu tim produksi campaign — untuk kemanusiaan,
sosial, dan lingkungan.

Satu orang + Minime = satu studio. Riset isu, strategi pesan, naskah, produksi
video, sampai distribusi — dikerjakan tim agen AI, dengan persetujuan manusia
di setiap langkah penting. Semua berjalan di komputermu sendiri.

Definisi lengkap: [`DEFINISI.md`](DEFINISI.md)

## Untuk apa

Memproduksi campaign kemanusiaan, sosial, dan lingkungan — dari riset →
strategi → naskah & konten → produksi video → distribusi. Contoh: kampanye
donasi, edukasi publik, gerakan sosial, advokasi lingkungan.

## Cara kerja

**Orkestrator:** Mimi (CIO — Chief Intelligent Officer) — terima permintaan,
pecah jadi tugas, delegasikan ke agen yang tepat, gabungkan hasil, mintakan
persetujuanmu.

**5 agen spesialis:**

| Agen | Peran |
|---|---|
| Kreatif | Naskah, copywriting, storytelling, produksi & editing video |
| Riset | Riset isu, data, analisis |
| Finance | Anggaran campaign, analisis & backtest |
| Bisnis Strategis | Brand, strategi pesan, strategi distribusi |
| Code Builder | Membangun aplikasi & landing page campaign |

**Alur produksi video:** generate (Veo 3.1 via Gemini) → edit/rakit (Pireel) →
QC → approval manusia → distribusi (draft-only).

## Model AI

Pengguna memilih modelnya sendiri (model router):

- **Default: Muse** (via Meta Model API)
- **Fallback / agen tambahan:** Gemini, DeepSeek, model lokal gratis (Ollama),
  atau model custom

Setiap agen bisa memakai model berbeda. Penggunaan token & estimasi biaya
dicatat per agen; opsional budget cap harian.

## Prinsip keras

- **Approval manusia wajib** — setiap aksi eksternal berhenti di kartu
  persetujuan. Tidak ada yang jalan diam-diam.
- **Meta draft-only** — Instagram, Facebook, Threads: agen hanya membuat
  *draft*. Tidak pernah auto-posting dalam kondisi apa pun.
- **Kredensial milik pengguna** — API key & token di OS keychain
  masing-masing pengguna, tidak pernah di git, tidak dibundel ke aplikasi.
- **Deliverable milik pengguna** — hasil kerja masuk ke Google Drive
  masing-masing pengguna.
- **Local-first** — data, riwayat, dan tugas tersimpan di komputermu sendiri.
  Data sensitif tidak diunggah ke internet.
- **Keuangan aman** — riset, analisis, dan backtest saja; order trading live
  otomatis dilarang keras.

## Status

Tahap 1 (UI/UX klik-tayang): lolos gate ✅ — Tahap 2 (model router + runtime
agen): berjalan 🔄. Lihat `docs/workflow.md` dan `docs/status.md`.

## Riwayat

Kode v1 (Tauri + Rust daemon, 16–18 Okt 2026) diarsipkan di `arsip-v1/`.
Dibangun ulang dari nol dengan pendekatan berbeda (keputusan Patria, 18 Okt 2026).
