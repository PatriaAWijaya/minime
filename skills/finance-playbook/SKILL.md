---
name: finance-playbook
description: Playbook finance — struktur P&L, unit economics, cash flow & runway, proyeksi, pricing. Knowledge menunggu dokumen final Patria.
owner_agent: strategi-bisnis
version: 0.1.0
example-prompt: Buat proyeksi arus kas 12 bulan untuk produk digital.
---

# finance-playbook

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> ⚠️ **STATUS: struktur skill lengkap, KNOWLEDGE MENUNGGU DOKUMEN FINAL PATRIA.**
> Dokumen knowledge finance belum ada (Fase 3) — bagian bertanda [KNOWLEDGE PATRIA]
> adalah placeholder dan TIDAK BOLEH diisi dengan karangan. Sampai dokumen final tersedia,
> skill ini hanya menjalankan kerangka kerja generik di bawah dan menandai setiap angka
> sebagai asumsi yang wajib divalidasi manusia.

## Kapan dipakai

Analisis dan perencanaan keuangan: struktur biaya, unit economics, cash flow & runway,
proyeksi, break-even, keputusan pricing & investasi.

## SOP (kerangka kerja — berlaku apa pun isi knowledge final)

1. **Pisahkan fakta vs asumsi.** Setiap angka dalam output WAJIB berlabel:
   `[FAKTA]` (dari data yang diberikan user) atau `[ASUMSI — validasi Patria]`.
   Jangan pernah menyajikan asumsi seolah fakta.
2. **Unit economics dulu.** Hitung CAC, LTV, gross margin per unit/produk.
   Aturan: LTV ≥ 3x CAC sebelum scale (lihat `selling-playbook`).
3. **Cash flow, bukan cuma P&L.** Susun proyeksi arus kas bulanan (minimal 12 bulan):
   kas masuk (per sumber) → kas keluar (per kategori) → saldo akhir.
   "Profit itu opini, cash itu fakta."
4. **Runway & break-even.** Hitung burn rate, runway (bulan), dan titik impas.
   Tandai bulan kritis (saldo minimum).
5. **Skenario.** Selalu 3 skenario: pesimis / realistis / optimis — dengan pemicu
   yang membedakan ketiganya (bukan sekadar ±20% seragam).
6. **Keputusan.** Akhiri dengan rekomendasi + risiko terbesar + angka yang bila berubah
   akan membalik rekomendasi (analisis sensitivitas sederhana).

## Toolbox

- Template P&L bulanan (pendapatan per sumber; COGS; OPEX per kategori; EBITDA)
- Template arus kas 12 bulan + 13-week cash flow (detail: lihat `operational-playbook`)
- Unit economics: CAC, LTV, LTV:CAC, gross margin, payback period
- Break-even point; burn rate & runway
- 3 skenario (pesimis/realistis/optimis) + analisis sensitivitas
- [KNOWLEDGE PATRIA] Struktur & rasio khas yang dipakai Patria (menunggu dokumen final)

## Quality bar

- Nol angka tanpa label [FAKTA]/[ASUMSI].
- Setiap proyeksi punya 3 skenario + pemicu yang membedakannya.
- Bulan kritis kas teridentifikasi; rekomendasi menyebut risiko terbesar.
- Konsisten dengan `selling-playbook` (LTV:CAC) dan `operational-playbook` (13-week cash flow).
- [KNOWLEDGE PATRIA] Quality bar final menyusul dokumen.

## Trigger eskalasi — WAJIB ke Patria bila

- Setiap angka berlabel [ASUMSI] yang material terhadap keputusan (validasi manusia wajib).
- Keputusan investasi, pricing final, dan komitmen biaya tetap.
- Skenario pesimis menunjukkan runway < 6 bulan (krisis kas = eskalasi langsung).
- [KNOWLEDGE PATRIA] Batasan knowledge final menyusul dokumen.

## Proof of work

Setiap output WAJIB mencantumkan: tabel fakta vs asumsi, unit economics,
proyeksi kas 12 bulan (3 skenario), runway & bulan kritis, rekomendasi + risiko terbesar.
