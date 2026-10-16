---
name: qlib
description: Platform riset quant AI Microsoft (MIT) — pipeline data → model ML → backtest → analisis portofolio. Riset & backtest saja, BUKAN eksekusi live.
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: Backtest strategi TopkDropout pada CSI300 2017-2020 dengan model GBDT — laporkan metrik risiko.
---

# qlib

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> Sumber: `microsoft/qlib` (MIT — aman untuk produk komersil; TIDAK di-vendor ke repo,
> dikoneksikan sebagai tool eksternal via `pip install pyqlib`).
>
> 🚫 **LARANGAN KERAS — TIDAK BISA DITAWAR:**
> Skill ini HANYA untuk riset quant, pelatihan model, dan backtest historis.
> **DILARANG membangun atau menjalankan eksekusi order trading otomatis/live dalam
> bentuk apa pun** (termasuk `online_srv` / paper/live serving qlib). Order live HANYA
> boleh dengan approval eksplisit Patria PER AKSI. Pelanggaran = eskalasi + hentikan task.

## Kapan dipakai

- Uji strategi berbasis sinyal/ML pada data historis (equity, multi-market).
- Bandingkan model (GBDT, LSTM, Transformer) pada universe & periode yang sama.
- Analisis portofolio: attribution, risk metrics (Sharpe, max drawdown, turnover).
- Validasi robustness: walk-forward / rolling sebelum kesimpulan apa pun.

## SOP

1. **Siapkan data dulu.** `qlib.init(provider_uri, region)`; unduh dataset bila perlu.
   Catat region & periode data di proof of work.
2. **Definisikan eksperimen sebagai KODE atau YAML** (`qrun task.yaml` atau
   `workflow_by_code.py`): dataset → model → strategi → executor → backtest.
3. **Jalankan via tool `quant` op `qlib_backtest`** (atau `qrun` langsung).
   Simpan config + seed agar eksperimen reproducible.
4. **Baca metrik dengan skeptis.** Wajib lapor: return, Sharpe, max drawdown,
   turnover, dan periode. Satu backtest bagus ≠ strategi bagus.
5. **Uji robustness minimal**: ganti periode ATAU universe sekali; bila hasil
   runtuh → laporkan sebagai overfit, bukan sebagai temuan.
6. **Sajikan sebagai HASIL BACKTEST** — selalu dengan disclaimer data historis
   ≠ jaminan masa depan. JANGAN jadikan dasar order tanpa approval Patria.

## Toolbox

- Tool `quant` op `qlib_backtest` (daemon) — readiness: `pyqlib` ter-install.
- CLI: `qrun <task.yaml>`; Python: `qlib.init()` → `R.start(experiment)` → fit → backtest.
- Dataset: `~/.qlib/qlib_data/` (region CN/US); unduh dari sumber komunitas bila perlu.
- Model bawaan: GBDT (LightGBM), LSTM/GRU, Transformer; strategi: TopkDropoutStrategy.
- Analisis: `SignalRecord`, `PortAnaRecord` (IC, Sharpe, drawdown, turnover).

## Quality bar

- Setiap hasil menyebut: universe, periode train/test, model, seed, biaya transaksi.
- Metrik risiko (max drawdown, Sharpe) SELALU menyertai metrik return.
- Ada uji robustness (periode/universe alternatif) atau dinyatakan belum diuji.
- Output berlabel **[HASIL BACKTEST — HISTORIS, BUKAN JAMINAN]**.
- Tidak ada klaim "profitable" tanpa menyebutkan asumsi biaya & slippage.

## Trigger eskalasi — WAJIB ke Patria bila

- Ada permintaan mengeksekusi/menjadwalkan order dari hasil backtest — TOLAK + eskalasi.
- Data tidak tersedia untuk universe/periode yang diminta.
- Hasil "terlalu bagus" (Sharpe > 3 tanpa penjelasan) — indikasi look-ahead/overfit.
- Butuh data berbayar untuk universe yang diminta.

## Proof of work

Config eksperimen, seed, periode, metrik (return/Sharpe/drawdown/turnover),
hasil uji robustness, dan label [HASIL BACKTEST] tercatat di DB.
