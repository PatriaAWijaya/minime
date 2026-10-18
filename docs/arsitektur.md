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

## 2. Lapisan (usulan — stack menunggu verdict Patria)

```
┌──────────────────────────────────────────────┐
│ UI (meniru layout OpenBot)                   │
│ sidebar workflow | daftar agen | chat+delivery │
│ | monitor tugas berjalan                      │
└──────────────────┬───────────────────────────┘
                   │ IPC
┌──────────────────▼───────────────────────────┐
│ Backend: agen runtime, model router, SQLite   │
│ skills, tools (meta/postiz/pireel)           │
└──────────────────────────────────────────────┘
```

## 3. Roster agen (seed)

Mimi (CIO, Muse) orkestrasi: Kreatif, Riset, Finance, Bisnis Strategis, Code Builder.
Setiap agen: system prompt + skills + tools allowlist + quality bar + budget.

## 4. Bahasa UI

Indonesia (default), Inggris, Mandarin — pilihan di pengaturan.
