# Sampel uji skill: trading-agents

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `strategi-bisnis` dengan playbook stage-level `["trading-agents"]`.
> Bukti wiring: `PLAYBOOK AKTIF: trading-agents`.
> Status wiring: OK — TERBUKTI.
> Isi lengkap skill: `skills/trading-agents/SKILL.md`.

## Tugas
Analisis multi-perspektif saham BBCA untuk tanggal 2026-10-17

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `trading-agents` ter-inject ke prompt agen dan di-echo oleh mock provider.
Kualitas output nyata skill hanya bisa dinilai dengan LLM beneran.

```
Status task: done | Playbook terdeteksi di DB: True
(Ini adalah output deterministik dari mock provider untuk verifikasi pipeline.)
```

## Catatan uji
- Aturan keras: riset/analisis/backtest/paper-trading saja — DILARANG order live otomatis.
