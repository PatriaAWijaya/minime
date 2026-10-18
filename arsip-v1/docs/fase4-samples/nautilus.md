# Sampel uji skill: nautilus

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `strategi-bisnis` dengan playbook stage-level `["nautilus"]`.
> Bukti wiring: `PLAYBOOK AKTIF: nautilus`.
> Status wiring: OK — TERBUKTI.
> Isi lengkap skill: `skills/nautilus/SKILL.md`.

## Tugas
Backtest simulasi strategi EMA-cross AUD/USD

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `nautilus` ter-inject ke prompt agen dan di-echo oleh mock provider.
Kualitas output nyata skill hanya bisa dinilai dengan LLM beneran.

```
Status task: done | Playbook terdeteksi di DB: True
(Ini adalah output deterministik dari mock provider untuk verifikasi pipeline.)
```

## Catatan uji
- Aturan keras: riset/analisis/backtest/paper-trading saja — DILARANG order live otomatis.
