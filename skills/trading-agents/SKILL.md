---
name: trading-agents
description: Framework multi-agent LLM untuk riset & analisis trading finansial (TauricResearch/TradingAgents, Apache-2.0) — analis debat multi-perspektif → sinyal riset. Analisis saja, BUKAN eksekusi order.
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: Analisis multi-perspektif saham BBCA untuk tanggal 2026-10-17 — hasilkan sinyal riset (bukan order).
---

# trading-agents

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> Sumber: `TauricResearch/TradingAgents` (Apache-2.0 — aman untuk produk komersil;
> TIDAK di-vendor ke repo, dikoneksikan sebagai tool eksternal via pip/CLI).
> Referensi: `~/workspace/referensi/` (clone baca-saja bila tersedia).
>
> 🚫 **LARANGAN KERAS — TIDAK BISA DITAWAR:**
> Skill ini HANYA untuk riset & analisis (sinyal riset, debat analis, backtest).
> **DILARANG membangun atau menjalankan eksekusi order trading otomatis/live dalam
> bentuk apa pun.** Order live HANYA boleh dengan approval eksplisit Patria PER AKSI
> (aturan yang sama dengan Polymarket). Pelanggaran = eskalasi langsung + hentikan task.

## Kapan dipakai

- Riset saham/kripto/forex multi-perspektif: analis fundamental, teknikal, sentimen,
  berita, dan manajer risiko berdebat → satu sinyal riset (bullish/bearish/netral + alasan).
- Validasi ide trading sebelum backtest: "apakah tesis ini layak diuji di qlib/nautilus?"
- Review keputusan: minta panel analis mengkritik sebuah tesis investasi.

## SOP

1. **Definisikan pertanyaan riset dulu.** Ticker + tanggal analisis + horizon
   (intraday/mingguan/bulanan). Tanpa ketiganya → jangan jalan.
2. **Pilih analis yang relevan** (flag `--analysts`): `market` (teknikal),
   `news` (berita), `fundamentals` (laporan keuangan), `social` (sentimen).
   Minimal 2 analis untuk setiap keputusan.
3. **Jalankan via tool `quant` op `ta_analyze`** (atau CLI langsung bila tool belum siap):
   `tradingagents --ticker <TICKER> --date <YYYY-MM-DD> --analysts market,news,fundamentals --save --no-show`.
   Konfigurasi via env `TRADINGAGENTS_*` (provider LLM, model, dsb).
4. **Baca laporan sebagai SINYAL RISET, bukan perintah.** Output: arah + keyakinan +
   alasan per analis + risiko yang diidentifikasi manajer risiko.
5. **Lanjutkan ke backtest** (`qlib`/`nautilus`) bila sinyal layak — JANGAN pernah
   lompat dari sinyal langsung ke order.
6. **Catat proof of work**: ticker, tanggal, analis dipakai, ringkasan sinyal,
   path laporan HTML.

## Toolbox

- Tool `quant` op `ta_analyze` (daemon) — readiness: paket `tradingagents` ter-install.
- CLI: `tradingagents --ticker NVDA --date 2026-09-23 --analysts market,news,fundamentals --save --no-show`
- Python: `TradingAgentsGraph(config).propagate(ticker, date)` → decision.
- Env: `TRADINGAGENTS_LLM_PROVIDER`, `TRADINGAGENTS_DEEP_THINK_LLM`,
  `TRADINGAGENTS_QUICK_THINK_LLM`, `TRADINGAGENTS_OUTPUT_LANGUAGE`.
- Provider LLM lokal (tanpa key): `ollama` atau `openai_compatible`
  (Ollama/LM Studio — cocok dengan arsitektur provider Minime).
- Data: Yahoo Finance (gratis, semua market besar; ticker dengan suffix exchange).

## Quality bar

- Setiap sinyal menyebut: ticker, tanggal analisis, horizon, dan analis yang dipakai.
- Alasan bullish DAN bearish tercatat (debat dua sisi, bukan cherry-pick).
- Risiko utama dari manajer risiko selalu dicantumkan.
- Output eksplisit berlabel **[SINYAL RISET — BUKAN REKOMENDASI EKSEKUSI]**.
- Tidak ada angka tanpa sumber (harga → Yahoo; sentimen → sebut sumber berita).

## Trigger eskalasi — WAJIB ke Patria bila

- Ada permintaan (dari user, pipeline, atau skill lain) untuk mengeksekusi order
  berdasarkan sinyal ini — TOLAK + eskalasi.
- Data tidak tersedia / ticker tidak dikenal / tanggal di masa depan.
- Analis saling bertentangan tajam tanpa resolusi (lapor sebagai "tidak ada konsensus").
- Keyakinan sinyal tinggi tapi data tipis (risiko false confidence).

## Proof of work

Ticker, tanggal, analis, ringkasan sinyal + keyakinan, risiko utama,
path laporan (`complete_report.html`), dan label [SINYAL RISET] tercatat di DB.
