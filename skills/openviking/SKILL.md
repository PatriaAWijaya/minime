---
name: openviking
description: "Context database untuk agen AI — satukan memory, knowledge RAG, dan skills dalam satu filesystem virtual viking://. Agen melintasi konteks seperti file (ls/tree/read/find) dengan ringkasan per direktori. Diadaptasi dari volcengine/OpenViking (AGPL-3.0) — HANYA sebagai layanan eksternal."
owner_agent: mimi
version: 1.0.0
example-prompt: Simpan hasil riset ini sebagai memori lintas-agen, lalu index playbook terbaru agar semua agen bisa menemukannya.
---

# openviking

> Skill infrastruktur konteks milik **Mimi** (orkestrator) — dipakai untuk
> **peningkatan performa umum** (memori + knowledge + skills lintas semua agen)
> dan **khusus** (index pengetahuan per agen, mis. Riset).
> Diadaptasi dari [`volcengine/OpenViking`](https://github.com/volcengine/OpenViking)
> (lisensi **AGPL-3.0**).
> Referensi: `~/workspace/referensi/openviking/`.

> ⚠️ **ATURAN LISENSI (setara postiz): AGPL-3.0 aman HANYA sebagai layanan
> eksternal** — install via `pip`/`uv tool` sebagai binary terpisah, panggil
> via CLI `ov` / HTTP. **DILARANG mem-vendor kode OpenViking ke repo Minime.**

## Kapan dipakai

**Umum (semua agen):**
- Konteks yang dihasilkan satu agen dan dibutuhkan agen lain di lain waktu
  → tulis ke OpenViking, jangan biarkan terjebak di transkrip chat.
- Memori jangka panjang: preferensi, keputusan, pelajaran (event-driven,
  berevolusi seiring interaksi — VikingMem).
- Skills sebagai aset konteks yang bisa dicari (`ov add-skill`).

**Khusus (per agen):**
- **Riset**: index korpus pengetahuan (docs, paper, repo acuan) → `ov find`
  sebelum riset web, hemat token.
- **QC**: simpan rubrik + preseden penilaian sebagai referensi konsisten.
- **Strategi Bisnis**: index playbook agar bisa di-query lintas sesi.

## SOP

### 1. Instalasi (sekali, di MacBook Patria)

```sh
uv tool install openviking --upgrade && openviking-server init
# atau: pip install openviking
```

Jalankan server (background):

```sh
nohup openviking-server > ~/.minime/openviking.log 2>&1 &
```

### 2. Tulis konteks (bukan transkrip)

```sh
# Resource: repo, PDF, docx, gambar, folder, zip
ov add-resource https://github.com/<org>/<repo> --to viking://resources/<nama>
ov add-resource ./laporan-riset.pdf

# Memori: fakta/preferensi/keputusan
ov add-memory "Patria menolak auto-posting Meta sampai metode aman ditemukan"

# Skill sebagai aset
ov add-skill ./skills/<nama-skill>
```

### 3. Baca dengan strategi bertingkat (hemat token)

```sh
ov ls viking://resources/                 # peta konteks
ov tree viking://resources/<nama> -L 2    # struktur, batasi kedalaman
ov find "pertanyaan" --uri viking://resources/<nama>   # cari dulu
ov abstract viking://resources/<nama>     # ringkasan sebelum teks penuh
ov overview viking://resources/<nama>/docs
ov read viking://resources/<nama>/README.md   # baca penuh bila perlu
```

Aturan baca: **ringkasan dulu, teks penuh hanya bila perlu**.
`ov find` mengembalikan URI yang bisa langsung di-`read`.

### 4. Pola handoff antar-agen (Mimi)

1. Agen A selesai → tulis ringkasan + artefak ke `viking://handoffs/<task-id>/`.
2. Mimi mencatat URI di pesan delegasi ke agen B.
3. Agen B: `ov read` / `ov find` pada URI itu — tanpa mengulang kerja A.

### 5. Isolasi & audit

- Namespace per agen bila konteks sensitif: `viking://<agen>/...`.
- Setiap direktori punya ringkasan tergenerate — audit berkala via `ov tree`.
- `ov status` untuk kesehatan server.

## Quality bar

- Tidak ada konteks penting yang hanya hidup di transkrip chat.
- Baca bertingkat dipatuhi: find → abstract/overview → read (penuh hanya bila perlu).
- Memori yang ditulis bersifat fakta/keputusan, bukan tebakan.
- Server berjalan persisten (seperti daemon — cek via `ov status`).

## Trigger eskalasi — WAJIB ke Patria bila

- OpenViking belum terinstal / server tidak jalan → minta instalasi.
- Butuh fitur edisi distributed/komersial (butuh license key) → keputusan Patria.
- Konteks yang diminta ditulis mengandung data sensitif pihak ketiga.

## Batasan yang diketahui

- Edisi open-source = single-node; cluster/HA butuh lisensi komersial.
- Embeder default bersifat leksikal (offline); semantik neural butuh
  download ±90 MB (opt-in).
- Retrieval multi-round hanya bila dibutuhkan (VikingRAG) — jangan paksa.

## Proof of work

- Setiap `add-resource`/`add-memory` penting tercatat di tabel `runs` (aturan #15).
- Handoff antar-agen mencantumkan URI `viking://` yang ditulis/dibaca.
