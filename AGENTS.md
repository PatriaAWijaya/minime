# AGENTS.md — Minime v2

> Visi ditetapkan Patria 18 Okt 2026. Kode v1 diarsipkan di `arsip-v1/`.
> **Keputusan 18 Okt 2026: rekonstruksi OpenBot dengan ramuan sendiri** —
> UI/UX meniru OpenBot, kode 100% buatan sendiri (clean-room).
> Stack: **Tauri 2 + Svelte 5** (terbukti di v1).
> Referensi visual (JANGAN salin kode): `~/workspace/referensi/openbot-base/`.

---

## a. Visi (kata-kata Patria)

Minime v2 adalah **desktop app** seperti Hermes/OpenBot. Intinya: **membantu membuat
agen tambahan** yang membuat model AI pengguna **lebih efektif dan efisien**.
Model = pilihan pengguna (Patria: **Gemini 3.8** berbayar + model lokal gratis).

Fokus: **kebutuhan kreatif untuk bisnis berbasis data akurat**; terbuka untuk
keperluan lain.

## b. Roster agen (keputusan Patria)

Orkestrator: **Mimi — CIO (Chief Intelligent Officer)**, model Muse.

| # | Agen | Tanggung jawab |
|---|------|----------------|
| 1 | Kreatif | Rencana + produksi + distribusi materi kreatif via channel tersedia |
| 2 | Riset | Riset bisnis untuk pengambilan keputusan |
| 3 | Finance | Analisa finansial mikro/makro, saran finansial, bantu akuntansi |
| 4 | Bisnis Strategis | Perhitungan bisnis, brand, strategi marketing, sales & closing |
| 5 | Code Builder | Bantu membuat aplikasi tambahan bila diperlukan |

Agen setara senior manager; bisa akses tool (meta, postiz, pireel);
user bisa tambah agen sendiri. Model per agen bebas (berbayar/lokal gratis).

## c. Urutan build (keputusan Patria)

**UI/UX dulu** (klik-tayang) → fitur + skills 5 agen + tools → rilis **macOS + Windows**
dengan instalasi mudah dan intuitif.

## d. Aturan keras

1. Kredensial di OS keychain, tidak di git.
2. TIDAK ADA auto-posting Meta — distribusi hanya draft.
3. Deliverable user-facing → Google Drive folder MIMI.
4. Data sensitif tidak diupload ke internet.
5. UI Bahasa Indonesia (opsi: Inggris, Mandarin); lolos anti-pattern Impeccable.
6. Jangan salin kode OpenBot — tiru UI/UX-nya saja (clean-room).
7. Gate tiap fase: review + vonis Patria.

## e. Konvensi repo & koding

- Bahasa dokumen & UI: Indonesia. Bahasa kode & komentar: Inggris.
- Commit: `<tipe>: <ringkasan>` — feat/fix/docs/refactor/chore.
- Secret hanya via keychain/env; `git status` bersih sebelum commit.

## f. Konvensi docs/

- `docs/status.md` — log kronologis; entri baru di-APPEND.
- `docs/rule.md` — aturan bernomor; ubah hanya atas persetujuan Patria.
- `docs/TODO.md` — checklist per fase.
- `docs/PLAN.md` — rencana fase + arsitektur ringkas.
- `docs/arsitektur.md` — framework arsitektur.
- `docs/workflow.md` — tahapan workflow.
