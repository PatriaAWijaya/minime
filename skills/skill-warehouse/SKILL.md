---
name: skill-warehouse
description: "Gudang skill on-demand — 267 spesialis dari agency-agents (MIT). Mimi memuat modul hanya bila tugas butuh keahlian di luar skill inti; dilarang auto-load; gudang bukan agen baru."
owner_agent: mimi
version: 1.0.0
example-prompt: "Tugas ini butuh review kode mendalam — muat modul code-reviewer dari gudang."
---

# skill-warehouse — Gudang Skill On-Demand

> Kontrak agen senior. Gudang ini adalah "otot pinjaman", BUKAN agen baru.
> Prinsip Patria (rule #16): sedikit agen setara senior manager — menambah kemampuan =
> menambah skill/modul, BUKAN menambah agen. Roster tetap 5 agen + Approval.

Sumber: repo [`msitarzewski/agency-agents`](https://github.com/msitarzewski/agency-agents)
(MIT, 160k★, diakses 17 Okt 2026). Katalog: `skills-warehouse/INDEX.md`
(267 spesialis, 18 kategori). Referensi mentah: `~/workspace/referensi/agency-agents/`.
Modul yang sudah diadaptasi ke format Minime (8): `ui-designer`, `ux-researcher`,
`frontend-developer`, `code-reviewer`, `reality-checker`, `research-synthesist`,
`carousel-growth-engine`, `short-video-editing-coach`.

## Kapan dipakai (trigger — HANYA oleh Mimi)

Muat modul dari gudang bila SEMUA kondisi terpenuhi:

1. Tugas aktif membutuhkan keahlian spesialis yang **tidak tercakup** skill inti 5 agen
   (mis. review kode mendalam → `code-reviewer`; sintesis 20 sumber → `research-synthesist`;
   arsitektur carousel → `carousel-growth-engine`).
2. Modul yang relevan sudah ada di `skills-warehouse/` (teradaptasi ✅) ATAU kebutuhan
   cukup jelas untuk adaptasi cepat mengikuti `skills-warehouse/CONTRIBUTING.md`.
3. Tidak ada skill inti yang sudah menanganinya dengan baik (jangan duplikasi).

**DILARANG:** memuat modul "untuk jaga-jaga" / preemptive; memuat ke semua agen sekaligus;
menjadikan modul sebagai agen baru di roster; melewati approval untuk aksi eksternal.

## Cara memuat (on-demand per task)

1. Mimi membaca `skills-warehouse/<modul>/MODULE.md`.
2. Mimi meng-inject ke konteks task agen pelaksana: ringkasan SOP modul + quality bar +
   batasan Minime yang relevan (bukan seluruh file bila tidak perlu — hemat konteks).
3. Agen pelaksana mengikuti SOP modul sebagai "mode spesialis" DALAM task tersebut;
   setelah task selesai, mode berakhir — tidak ada state permanen, tidak ada agen baru.
4. Hasil task tetap melewati QC + Approval Patria seperti biasa.

## Atribusi (WAJIB)

Setiap modul adaptasi mencantumkan: lisensi MIT + nama repo asal + link + nama file sumber.
Bila output modul diedarkan ke pihak ketiga, atribusi ikut serta. Jangan hapus atribusi.

## Quality bar

- Modul yang dimuat harus yang paling spesifik untuk kebutuhan (bukan yang "kira-kira").
- Bila modul sumber mengandung perilaku yang melanggar rule Minime (mis. auto-publish),
  perilaku itu DINONAKTIFKAN di adaptasi (contoh: `carousel-growth-engine` = draft-only).
- Adaptasi baru mengikuti `skills-warehouse/CONTRIBUTING.md` dan ditandai ✅ di `INDEX.md`.

## Trigger eskalasi (ke Patria)

- Kebutuhan spesialis yang belum ada di gudang dan butuh adaptasi besar.
- Modul yang menyentuh area sensitif (publikasi, data pribadi, keamanan, biaya).

## Proof of work

- Catat di task: modul apa yang dimuat, kenapa, dan hasil penerapannya.
- Modul baru yang diadaptasi: file `MODULE.md` + ✅ di `INDEX.md` + commit.
