---
name: finance-playbook
description: Playbook finance — P&L, unit economics, cash flow & runway, proyeksi, pricing + toolbox quant (TradingAgents, qlib, Nautilus, OpenBB, Valyu). Riset & analisis saja.
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: Buat proyeksi arus kas 12 bulan untuk produk digital + analisis risiko pasar.
---

# finance-playbook

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> Playbook ini punya DUA sisi: (A) keuangan bisnis (P&L, unit economics, cash flow)
> dan (B) toolbox quant untuk riset pasar (4 framework + Valyu). Knowledge bisnis
> final [KNOWLEDGE PATRIA] masih menunggu dokumen Patria — bagian itu TIDAK BOLEH
> diisi karangan.
>
> 🚫 **LARANGAN KERAS — TIDAK BISA DITAWAR:**
> Semua aktivitas quant di skill ini HANYA untuk riset, analisis, backtest, dan
> paper trading. **DILARANG membangun atau menjalankan eksekusi order trading
> otomatis/live dalam bentuk apa pun.** Order live HANYA boleh dengan approval
> eksplisit Patria PER AKSI (aturan yang sama dengan Polymarket).
> Pelanggaran = eskalasi langsung + hentikan task.

## Kapan dipakai

- **Keuangan bisnis**: struktur biaya, unit economics, cash flow & runway, proyeksi,
  break-even, keputusan pricing & investasi.
- **Riset pasar/quant**: analisis multi-perspektif (`trading-agents`), backtest
  strategi ML (`qlib`), simulasi order realistis (`nautilus`), data pasar (`openbb`,
   `valyu-finance`).

## SOP — A. Keuangan bisnis

1. **Pisahkan fakta vs asumsi.** Setiap angka WAJIB berlabel:
   `[FAKTA]` (dari data user) atau `[ASUMSI — validasi Patria]`.
2. **Unit economics dulu.** CAC, LTV, gross margin. Aturan: LTV ≥ 3x CAC sebelum scale.
3. **Cash flow, bukan cuma P&L.** Proyeksi arus kas bulanan 12 bulan; tandai bulan kritis.
   "Profit itu opini, cash itu fakta."
4. **Runway & break-even.** Burn rate, runway (bulan), titik impas.
5. **3 skenario** (pesimis/realistis/optimis) dengan pemicu yang membedakannya.
6. **Keputusan**: rekomendasi + risiko terbesar + analisis sensitivitas.
7. [KNOWLEDGE PATRIA] Struktur & rasio khas Patria (menunggu dokumen final).

## SOP — B. Riset quant (pilih framework sesuai kebutuhan)

| Kebutuhan | Skill | Pola |
|---|---|---|
| Sinyal riset multi-perspektif (analis debat) | `trading-agents` | ticker + tanggal → analis → sinyal [SINYAL RISET] |
| Backtest strategi ML / riset quant | `qlib` | `qrun`/kode → metrik + uji robustness [HASIL BACKTEST] |
| Simulasi order realistis / paper trading | `nautilus` | venue SIM → backtest engine [HASIL SIMULASI] |
| Data pasar terstruktur | `openbb` | SDK → harga/fundamental/berita |
| Data GRATIS lapis pertama | tool `marketdata` | lihat § Routing data di bawah |
| Filings, berita, sentimen (butuh key) |  `valyu-finance` | search API → temuan bersumber [DATA VALYU] |

Alur yang benar: data (`marketdata` gratis dulu → `openbb` → `valyu-finance`
fallback premium) → sinyal (`trading-agents`) → validasi (`qlib`/`nautilus`) →
**BERHENTI di analisis**. Tidak ada lompatan ke order.

## Routing data — GRATIS dulu, Valyu fallback premium (keputusan Patria 18 Okt 2026)

| Kebutuhan data | Sumber GRATIS (coba dulu) | Fallback PREMIUM |
|---|---|---|
| Harga saham/terkini | tool `marketdata` op `quote` (yfinance) | — |
| Laporan keuangan emiten AS (10-K/10-Q/8-K) | tool `marketdata` op `edgar_filings` / `edgar_facts` (SEC EDGAR, tanpa key) | tool `valyu` op `search` bila butuh pencarian filings yang dalam |
| Data makro (suku bunga, inflasi, CPI, unemployment) | tool `marketdata` op `fred` (butuh key gratis FRED) | — |
| Data pasar terstruktur umum | tool `quant` op `openbb_data` | — |
| Retrieval proprietary yang dalam (dataset khusus Valyu, pencarian filings mendalam, berita premium) | — | tool `valyu` op `search` **[DATA VALYU — BERBAYAR]** |

Aturan: **selalu coba lapisan gratis dulu**. Naik ke Valyu hanya bila (a) data
tidak ada di sumber gratis, atau (b) butuh kedalaman proprietary (dataset
khusus/berita premium). Setiap pemakaian Valyu tercatat biayanya (`cost` di
respons) — laporkan ke Patria bila mendekati batas wajar.

## Toolbox

- Template P&L bulanan, arus kas 12 bulan + 13-week, unit economics, break-even.
- Tool `quant` (daemon): `ta_analyze`, `qlib_backtest`, `nautilus_backtest`,
  `openbb_data` + tool `valyu` op `search` + tool **`marketdata`**
  (`quote`/`edgar_ticker`/`edgar_facts`/`edgar_filings`/`fred`) — masing-masing
  dengan readiness check. Stooq DINONAKTIFKAN (endpoint CSV kini di balik
  verifikasi JS; digantikan yfinance).
- [KNOWLEDGE PATRIA] Struktur & rasio khas Patria (menunggu dokumen final).

## Quality bar

- Nol angka tanpa label [FAKTA]/[ASUMSI].
- Setiap proyeksi: 3 skenario + pemicu; bulan kritis teridentifikasi.
- Setiap hasil quant berlabel jenisnya ([SINYAL RISET]/[HASIL BACKTEST]/[HASIL SIMULASI]/[DATA VALYU]).
- Tidak ada rekomendasi eksekusi tanpa approval Patria.
- [KNOWLEDGE PATRIA] Quality bar final menyusul dokumen.

## Trigger eskalasi — WAJIB ke Patria bila

- Setiap angka [ASUMSI] yang material; keputusan investasi/pricing final.
- Skenario pesimis menunjukkan runway < 6 bulan.
- ADA PERMINTAAN EKSEKUSI ORDER (dari user/pipeline/skill lain) — TOLAK + eskalasi.
- Data/key tidak tersedia untuk framework yang dibutuhkan.
- [KNOWLEDGE PATRIA] Batasan knowledge final menyusul dokumen.

## Proof of work

Tabel fakta vs asumsi, unit economics, proyeksi 12 bulan (3 skenario), runway;
untuk quant: framework dipakai, parameter, hasil berlabel, dan pernyataan eksplisit
"TIDAK ADA ORDER DIEKSEKUSI" tercatat di DB.
