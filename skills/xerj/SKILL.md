---
name: xerj
description: "Local AI search engine (single Rust binary) — index folder apapun (kode, docs, PDF) sekali, lalu tanya dan dapatkan passage + sitasi file:line. 40x lebih hemat token daripada grep. Diadaptasi dari xerj-org/xerj (Apache-2.0)."
owner_agent: riset
version: 1.0.0
example-prompt: Index repo referensi, lalu cari bagaimana mereka mengimplementasikan retry dengan backoff sebelum kita menulis kode sendiri.
---

# xerj

> Skill riset lokal milik **Riset** — mesin pencari untuk agen.
> Diadaptasi dari [`xerj-org/xerj`](https://github.com/xerj-org/xerj)
> (lisensi **Apache-2.0**): satu binary Rust (±61 MB), tanpa server eksternal.
> Referensi: `~/workspace/referensi/xerj/`.

## Kapan dipakai

Bila tugas butuh **menjawab dari korpus lokal yang besar** tanpa membakar
konteks: codebase referensi, dokumentasi proyek, arsip PDF, log.
Grep menarik file utuh ke konteks; xerj mengembalikan **passage yang tepat +
file:line**. Terukur: 2,7x lebih sedikit output token vs grep pada solve rate
yang sama (studi 8 task, 4 bahasa).

BUKAN untuk: pencarian web real-time (pakai `riset-web`), atau korpus yang
belum di-index.

## SOP

### 1. Instalasi (sekali, di MacBook Patria)

```sh
curl -fsSL https://xerj.org/get | sh
```

### 2. Jalankan node (per sesi kerja)

```sh
xerj --insecure --data-dir ~/.minime/xerj-data &
```

- Node bind `127.0.0.1` saja. Flag `--insecure` mematikan TLS+auth —
  **batas kepercayaan**: hanya untuk satu user di mesin dev sendiri.
  Jangan expose port ke jaringan.

### 3. Index sekali (autoindex)

```sh
xerj autoindex ~/referensi/proyek-acuan     # kode/docs/PDF — AST-aware
xerj autoindex ~/workspace/minime/docs      # docs Minime sendiri
```

Index bersifat resumable; ulangi setelah korpus berubah signifikan.

### 4. Tanya, jangan baca file

```sh
xerj search "bagaimana retry dengan backoff diimplementasikan"
# → passage berperingkat + file:line + kontrak fungsi

xerj def "nama_fungsi"
# → go-to-definition: file:line + signature (bila nama sudah diketahui)
```

### 5. Sintesis ber-sumber (kontrak Riset)

- Setiap klaim dari hasil xerj WAJIB dikutip: **file:line**.
- Bedakan: passage dari index (fakta) vs interpretasi agen.
- Bila hasil kosong → eskalasi, jangan mengarang.

### 6. Ukur penghematan

```sh
xerj gain   # laporan token yang dihemat — tercatat, bukan estimasi
```

## Pola reference-coding (utama)

1. Clone repo open-source yang paling dekat dengan yang dibangun:
   `git clone --depth 1 <url> ~/referensi/<nama>`.
2. `xerj autoindex ~/referensi/<nama>`.
3. `xerj search "bagaimana mereka menyelesaikan <masalah>"` SEBELUM menulis kode.
4. Tulis kode sendiri (jangan salin — hormati lisensi repo acuan).

## Quality bar

- Tidak ada file besar dibaca utuh bila xerj bisa menjawab (hemat token).
- Semua kutipan dari index memakai format file:line.
- Index di-refresh bila korpus acuan berubah.
- `xerj gain` dilaporkan berkala sebagai proof of work.

## Trigger eskalasi — WAJIB ke Patria bila

- Binary xerj belum terinstal di MacBook → minta instalasi (perintah Patria).
- Node tidak bisa start / port konflik.
- Hasil search kosong untuk pertanyaan yang seharusnya terjawab index.
- Ada permintaan meng-index data sensitif milik pihak ketiga tanpa izin.

## Proof of work

- Setiap `autoindex` + query penting tercatat di tabel `runs` (aturan #15).
- Output riset mencantumkan sitasi file:line dari xerj.
