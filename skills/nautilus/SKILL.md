---
name: nautilus
description: Engine backtest & paper-trading production-grade (nautechsystems/nautilus_trader, LGPL-3.0) — simulasi event-driven deterministik. Backtest & paper saja; LIVE DITOLAK.
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: Backtest strategi EMA-cross AUD/USD di engine Nautilus (simulasi) — laporkan fill & biaya.
---

# nautilus

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> Sumber: `nautechsystems/nautilus_trader` (LGPL-3.0).
> ⚠️ **KEWAJIBAN LISENSI (produk komersil):** JANGAN static-link / vendor kode
> Nautilus ke binary proprietary Minime. Koneksi HANYA sebagai proses eksternal
> (`python3` subprocess menjalankan skrip backtest) — pola ini mematuhi LGPL.
> Cantumkan atribusi + teks lisensi di `docs/lisensi-pihak-ketiga.md`.
>
> 🚫 **LARANGAN KERAS — TIDAK BISA DITAWAR:**
> Skill ini HANYA untuk **backtest simulasi** dan **paper trading** (venue SIM / sandbox).
> **DILARANG menghubungkan ke venue live / broker sungguhan, DILARANG mengeksekusi
> order live dalam bentuk apa pun.** Adapter live (mis. Binance/IB) TIDAK BOLEH
> diaktifkan. Order live HANYA boleh dengan approval eksplisit Patria PER AKSI —
> dan tetap TIDAK melalui skill ini. Pelanggaran = eskalasi + hentikan task.

## Kapan dipakai

- Backtest strategi dengan simulasi order yang realistis (antrean, fee maker/taker,
  slippage, partial fill) — lebih teliti dari backtest vektor biasa.
- Paper trading di venue simulasi (SIM) untuk validasi perilaku strategi.
- Uji strategi event-driven (tick/bar) sebelum kesimpulan riset.

## SOP

1. **Venue HANYA simulasi.** Gunakan `Venue("SIM")` + `AccountType` margin/cash
   simulasi. JANGAN PERNAH memakai adapter live.
2. **Tulis skrip strategi** (Python): `BacktestEngine(BacktestEngineConfig(...))` →
   `add_venue` → `add_instrument` → `add_data` → `run()` → laporan.
   Contoh: `examples/backtest/fx_ema_cross_audusd_bars_from_ticks.py`.
3. **Jalankan via tool `quant` op `nautilus_backtest`** (subprocess terisolasi,
   timeout wajar). Skrip live-trading DITOLAK di level tool (lihat `tools/quant.rs`).
4. **Baca laporan dengan skeptis**: total PnL, win rate, max drawdown, jumlah trade,
   biaya. Sedikit trade (< 30) = tidak signifikan secara statistik.
5. **Sajikan sebagai HASIL SIMULASI** — bukan bukti profit masa depan.

## Toolbox

- Tool `quant` op `nautilus_backtest` (daemon) — readiness: `nautilus_trader` ter-install.
- Python (v1.x, pip): `from nautilus_trader.backtest.engine import BacktestEngine`;
  `BacktestEngineConfig(trader_id=TraderId('ID'))` → `add_venue` → `add_instrument` →
  `add_data` → `run()` → laporan. (Contoh repo upstream memakai API v2-dev —
  sesuaikan dengan versi ter-install.)
- Fee model eksplisit (maker/taker) — JANGAN backtest tanpa biaya.

## Quality bar

- Setiap hasil menyebut: venue (SIM), instrumen, periode data, fee model, jumlah trade.
- Laporan mencakup max drawdown + biaya total, bukan cuma PnL.
- Output berlabel **[HASIL SIMULASI — BUKAN JAMINAN]**.
- Tidak ada klaim tanpa menyebutkan asumsi data & biaya.

## Trigger eskalasi — WAJIB ke Patria bila

- Ada permintaan menghubungkan ke broker/venue live atau mengeksekusi order — TOLAK + eskalasi.
- Data historis tidak tersedia / kualitas buruk.
- Hasil tidak stabil saat parameter digeser sedikit (fragile).

## Proof of work

Skrip strategi, config venue, periode data, metrik (PnL/drawdown/trade/biaya),
dan label [HASIL SIMULASI] tercatat di DB.
