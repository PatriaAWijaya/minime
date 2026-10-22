---
name: unslop
description: Potong ciri tulisan AI dari caption, script, artikel, dan copywriting — katalog pola + teknik rewrite.
owner_agent: kreatif
version: 1.0.0
example-prompt: Unslop caption ini sebelum diposting.
---

# unslop

Skill kualitas tulisan milik agen Kreatif. Diadopsi dari `backnotprop/pstack` (skill `unslop`, MIT, karya poteto). Referensi penuh: `~/workspace/pstack-check/skills/unslop/`.

## Kapan dipakai

Setiap mereview/memoles tulisan sebelum dipublikasikan atau diserahkan: caption, script, artikel, copywriting. Jangan ubah makna — hanya bersihkan pola.

## Workflow

1. Pindai tulisan terhadap katalog pola di bawah.
2. Tulis ulang bagian yang kena. Pertahankan makna, samakan tone.
3. Uji akhir: jika kalimatnya bisa muncul tanpa perubahan di proyek lain, ia tidak mengatakan apa-apa — potong.

## Katalog Pola (ringkas)

- **Konten:** frasa -ing dangkal ("showcasing..."); atribusi kabur ("para ahli percaya...").
- **Bahasa:** kosakata AI (crucial, delve, landscape abstrak, vibrant); "berfungsi sebagai" → "adalah"; "bukan sekadar X, tapi Y" → langsung; rule of three yang dipaksakan; siklus sinonim; rentang palsu.
- **Gaya:** em dash (hindari total); bold berlebihan; daftar inline-header ("**Performa:** performa...") → prosa; title case → sentence case; emoji dekoratif.
- **Artefak chatbot:** "Semoga membantu!"; nada menjilat ("Pertanyaan bagus!").
- **Filler:** "demi untuk", "hal ini penting dicatat"; hedging berlapis; kesimpulan generik ("masa depan cerah").
- **Jargon:** kata benda metafor abstrak (paradigm, flywheel, north star) → kata konkret.
- **Lugas:** katakan fungsinya bukan rasanya; satu ide per kalimat; aktif; potong adverb ("berjalan cepat" → angka); kata polos ("memanfaatkan" → "pakai").

## Aturan

- Berlaku untuk tulisan agen sendiri SEBELUM diserahkan, bukan cuma tulisan user.
- Sejalan dengan anti-pattern Impeccable (rule Minime) — keduanya dipakai bersamaan.
