# Framework Arsitektur — Minime v2

> UI/UX meniru OpenBot (referensi: `~/workspace/referensi/openbot-base/`,
> TIDAK menyalin kode — clean-room). Dibuat 18 Okt 2026.

---

## 1. Prinsip

1. **Agent builder, bukan roster tetap.** 6 agen seed (Mimi + 5); user bisa tambah sendiri.
2. **Model-agnostic.** Default Patria = Gemini 3.8; fallback model lokal gratis;
   user lain bebas daftar modelnya sendiri. API key di OS keychain.
3. **Data-grounded creative.** Output kreatif berpijak pada data/sumber terverifikasi.
4. **Local-first.** Data di mesin sendiri.
5. **Human-in-the-loop.** Approval wajib sebelum aksi eksternal.

## 2. Lapisan (ramuan sendiri: Tauri 2 + Svelte 5)

```
┌──────────────────────────────────────────────┐
│ UI Svelte 5 (tiru layout OpenBot)             │
│ sidebar workflow | daftar agen | chat+delivery │
│ | monitor tugas berjalan                      │
└──────────────────┬───────────────────────────┘
                   │ Tauri IPC (command)
┌──────────────────▼───────────────────────────┐
│ Backend Rust: agen runtime, model router      │
│ (Gemini 3.8 default), SQLite, skills, tools  │
└──────────────────────────────────────────────┘
```

Prinsip: thin client + backend Rust (pola terbukti v1).
Tidak ada daemon terpisah di v2 awal — backend menyatu di Tauri
(disederhanakan dari v1; persistensi via SQLite langsung).

## 3. Roster agen (seed)

Mimi (CIO, Muse) orkestrasi: Kreatif, Riset, Finance, Bisnis Strategis, Code Builder.
Setiap agen: system prompt + skills + tools allowlist + quality bar + budget.

## 4. Bahasa UI

Indonesia (default), Inggris, Mandarin — pilihan di pengaturan.
