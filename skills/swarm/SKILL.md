---
name: swarm
description: Fan-out N worker paralel lalu agregasi menjadi satu laporan — untuk coverage paralel, race pendekatan, dan eksplorasi.
owner_agent: mimi
version: 1.0.0
example-prompt: Swarm audit paralel: 3 worker periksa keamanan, performa, dan konsistensi UI hasil build ini.
---

# swarm

Skill orkestrasi paralel milik Mimi. Diadopsi dari `backnotprop/pstack` (skill `swarm`, MIT, karya poteto) — pola diadaptasi untuk Minime. Referensi penuh: `~/workspace/pstack-check/skills/swarm/`.

## Kapan dipakai

Ketika tugas bisa dipecah menjadi irisan independen (coverage paralel), ketika ingin mengadu beberapa pendekatan terhadap brief yang sama (race: `first pass`, `rank all`, `best-of`), atau untuk eksplorasi/gauntlet. BUKAN untuk tugas sekuensial yang hasilnya saling bergantung.

## Workflow

### Fase A: Frame
1. Nyatakan **done predicate**: artefak/laporan apa yang harus dihasilkan.
2. Pilih bentuk: **partisi** (irisan), **race** (brief identik), atau campuran. Untuk race, tetapkan aturan seleksi di awal.
3. Setiap worker yang menulis harus punya output path sendiri yang tidak bertabrakan.

### Fase B: Fan out
Luncurkan SEMUA worker sekaligus dengan brief yang **berdiri sendiri** (tujuan, cakupan, cara verifikasi, format laporan). Laporan worker: `PASS`, `ISSUES`, atau `BLOCKED` + bukti. Worker yang menemukan cacat mendaftar SEMUA yang bisa dibuktikan. Worker gugur → lanjut N-1, catat.

### Fase C: Aggregate
Buang hasil yang tidak memenuhi brief; spawn ulang sekali; gagal dua kali = **gap** (bukan pass). Coverage: semua irisan wajib ada hasil. Race: terapkan aturan seleksi Fase A. Jangan tempel dump mentah.

### Fase D: Report
Satu laporan konsolidasi: tabel hasil, one-liner issue, gap/dropout, aturan race bila dipakai.

## Aturan
- Brief worker mandiri — worker tidak melihat konteks induk.
- Done predicate + aturan seleksi ditetapkan SEBELUM fan-out.
- Tetapkan kapan N-1 cukup agar satu worker lambat tidak memblokir selamanya.
