# Sampel uji skill: valyu-finance

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `strategi-bisnis` dengan playbook stage-level `["valyu-finance"]`.
> Bukti wiring: `PLAYBOOK AKTIF: valyu-finance`.
> Status wiring: OK — TERBUKTI.
> Tool `valyu` ter-wire di daemon (`daemon/src/tools/valyu.rs`).
> Isi lengkap skill: `skills/valyu-finance/SKILL.md`.
> Diadaptasi dari `yorkeccak/finance` (MIT); referensi: `~/workspace/referensi/yorkeccak-finance/`.

## Tugas
Uji wiring skill valyu-finance

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `valyu-finance` ter-inject ke prompt agen dan di-echo oleh mock provider.

```
PLAYBOOK AKTIF: valyu-finance
```

## Status koneksi (jujur)

Tool `valyu` readiness: `not_configured` — **menunggu API key Valyu dari Patria**
(gratis $10 di https://platform.valyu.ai, simpan di Keychain akun `valyu` per aturan #2/#19).
Uji live (query nyata ke Valyu API) menyusul setelah key tersedia.
Aturan keras: skill ini hanya riset/analisis — DILARANG order trading live otomatis.
