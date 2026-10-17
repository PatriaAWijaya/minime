---
name: valyu-finance
description: "Skill riset finansial — query data institutional-grade (harga pasar live, SEC filings, berita, riset akademik) via Valyu Search API. Diadaptasi dari yorkeccak/finance (MIT). Butuh VALYU_API_KEY."
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: Cari data pendapatan kuartalan 3 bank BUMN terbesar + sentimen beritanya minggu ini.
---

# valyu-finance

> Skill riset finansial milik **Strategi Bisnis** — pasangan `finance-playbook`.
> Diadaptasi dari [`yorkeccak/finance`](https://github.com/yorkeccak/finance)
> (lisensi **MIT**): chat interface di atas **satu search API** (Valyu).
> Referensi: `~/workspace/referensi/yorkeccak-finance/`.
>
> **POSISI (keputusan Patria, 18 Okt 2026): Valyu adalah FALLBACK PREMIUM.**
> Selalu coba lapisan GRATIS dulu via tool `marketdata`
> (EDGAR/FRED/yfinance) dan `quant` op `openbb_data`. Naik ke Valyu hanya bila
> data tidak ada di sumber gratis ATAU butuh retrieval proprietary yang dalam
> (dataset khusus Valyu, pencarian filings mendalam, berita premium).
> Setiap query tercatat biayanya (`cost`) — laporkan ke Patria bila mendekati
> batas wajar.
>
> **Aturan keras (keputusan Patria, 17 Okt 2026): riset/analisis saja —
> DILARANG order trading live otomatis.** Skill ini hanya membaca data;
> tidak ada eksekusi order dalam bentuk apapun.

## Kapan dipakai

Bila tugas butuh **data finansial berkualitas institusional**: harga pasar
live, laporan keuangan emiten (SEC 10-K/10-Q), insider trading, berita &
sentimen pasar, riset akademik keuangan, data paten. BUKAN untuk: eksekusi
trading, rekomendasi beli/jual personal (bukan nasihat investasi —
selalu tulis disclaimer), atau data yang tidak perlu kesegaran real-time
(pakai riset web biasa bila cukup).

## SOP

### 1. Rumuskan query yang presisi
- Satu query = satu pertanyaan (< 400 karakter). Contoh:
  `"net income growth Bank Mandiri Q3 2026"`, bukan `"bank bagus"`.
- Pilih `search_type`: `all` (default), `web`, `news`, `proprietary`
  (dataset khusus Valyu: SEC filings, data pasar).

### 2. Query via tool `valyu`
```
tool: valyu, op: search
params: { query, search_type, max_num_results (1–20) }
```
- Mulai dari 10 hasil; persempit dengan `included_sources` bila perlu.
- Tiap query tercatat (biaya per query terlihat di respons `cost`).

### 3. Sintesis dengan sumber
- Setiap klaim angka WAJIB dikutip: sumber + tanggal + URL.
- Bedakan fakta (dari hasil) vs interpretasi (dari agen) — tulis eksplisit.
- Angka yang konflik antar sumber → tampilkan keduanya, jangan pilih diam-diam.

### 4. QC → Approval → Deliverable
- Output: ringkasan riset Bahasa Indonesia + tabel angka + daftar sumber.
- Disclaimer standar: "Riset untuk edukasi, bukan rekomendasi investasi."

## Toolbox

- Tool **`valyu`** Minime (`daemon/src/tools/valyu.rs`): op `search`
  (`query`, `search_type`, `max_num_results`).
- Kredensial: Keychain `com.minime.app` akun `valyu` (override env `MINIME_VALYU_KEY`
  untuk dev) — aturan #2/#19.
  Gratis $10 di https://platform.valyu.ai — minta ke Patria bila belum ada.

## Quality bar

- Setiap angka material bersumber (URL + tanggal akses).
- Query presisi; tidak ada query generik yang hasilnya sampah.
- Fakta vs opini dipisahkan secara eksplisit.
- Disclaimer edukasi selalu tercantum.
- Biaya API terpantau (tidak ada query boros berulang).

## Trigger eskalasi — WAJIB ke Patria bila

- `tools.status` menunjukkan valyu `not_configured` → minta API key ke Patria
  (jangan cari/beli sendiri).
- Hasil pencarian kosong/konflik untuk angka yang krusial.
- Ada permintaan yang mengarah ke eksekusi trading/order — TOLAK dan eskalasi.
- Biaya query mendekati batas wajar (lapor sebelum lanjut).

## Proof of work

- Setiap query + respons tercatat di tabel `runs` (aturan #15).
- Deliverable: dokumen riset + daftar sumber lengkap.
- **Status 17 Okt 2026**: tool ter-wire, readiness `not_configured`
  (menunggu VALYU_API_KEY dari Patria via Keychain); uji live menyusul
  setelah key tersedia.
