# Minime

Aplikasi desktop local-first untuk AI teammates yang persisten — tim agen AI
senior yang bekerja untukmu, di komputermu, dengan persetujuanmu.

Definisi lengkap: [`DEFINISI.md`](DEFINISI.md)

**Orkestrator:** Mimi (CIO — Chief Intelligent Officer)

**5 agen utama:** Kreatif · Riset · Finance · Bisnis Strategis · Code Builder

**Model:** default Muse; model lain (Gemini, DeepSeek, Ollama lokal, custom)
untuk fallback atau agen tambahan.

## Status

Tahap 1 (UI/UX klik-tayang): lolos gate ✅ — Tahap 2 (model router + runtime agen): berjalan 🔄.
Lihat `docs/workflow.md` dan `docs/status.md`.

## Aturan keras

- Tidak ada auto-posting Meta (draft-only)
- Kredensial di OS keychain masing-masing pengguna, tidak di git
- Deliverable → Google Drive masing-masing pengguna
- UI Bahasa Indonesia (default)
- Dilarang order trading live otomatis

## Riwayat

Kode v1 (Tauri + Rust daemon, 16–18 Okt 2026) diarsipkan di `arsip-v1/`.
Dibangun ulang dari nol dengan pendekatan berbeda (keputusan Patria, 18 Okt 2026).
