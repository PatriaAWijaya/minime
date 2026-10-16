---
name: openbb
description: Data pasar & riset investasi via OpenBB Platform SDK (paket PyPI `openbb`) — harga, fundamental, berita. Sumber data untuk analis, BUKAN eksekusi.
owner_agent: strategi-bisnis
version: 1.0.0
example-prompt: Ambil harga historis dan profil fundamental BBCA via OpenBB untuk bahan analisis.
---

# openbb

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> Sumber: OpenBB Platform — paket PyPI `openbb` (SDK; lisensi paket: cek `pip show openbb`).
> Repo `OpenBB-finance/agents-for-openbb` (MIT) sebagai referensi pola agen.
> ⚠️ Repo `experimental-openbb-platform-agent` TIDAK mencantumkan lisensi → JANGAN
> vendor kodenya; gunakan hanya sebagai referensi pola, koneksi via SDK/API.
>
> 🚫 **LARANGAN KERAS:** skill ini HANYA sumber data & riset. **DILARANG mengeksekusi
> order dalam bentuk apa pun.** Order live HANYA dengan approval eksplisit Patria PER AKSI.

## Kapan dipakai

- Ambil data pasar terstruktur: harga historis, profil emiten, laporan keuangan,
  indikator teknikal — untuk bahan analis (`trading-agents`) atau backtest (`qlib`).
- Riset cepat satu ticker: "apa yang diketahui pasar tentang X?"
- Gantikan scraping ad-hoc dengan data provider yang terstandarisasi.

## SOP

1. **Tentukan kebutuhan data dulu**: ticker, jenis (harga/fundamental/berita),
   periode, provider (default bawaan SDK; `yfinance` untuk harga gratis).
2. **Ambil via tool `quant` op `openbb_data`** (atau SDK langsung):
   `from openbb import obb; obb.equity.price.historical("BBCA", provider="yfinance")`.
3. **Validasi**: cek rentang tanggal & kelengkapan (data bolong → catat).
4. **Teruskan ke analis** — skill ini tidak menyimpulkan; ia memasok data.

## Toolbox

- Tool `quant` op `openbb_data` (daemon) — readiness: paket `openbb` ter-install.
- SDK: `obb.equity.price.historical`, `obb.equity.profile`, `obb.equity.fundamental.*`,
  `obb.news.*` (nama fungsi mengikuti versi SDK — cek `help(obb.equity)`).
- Provider data bisa diganti (yfinance gratis; provider lain butuh key via Keychain/env).

## Quality bar

- Setiap dataset menyebut: ticker, provider, rentang tanggal, jumlah baris.
- Data bolong/terlambat dicatat, bukan disembunyikan.
- Output berlabel sumber data.

## Trigger eskalasi — WAJIB ke Patria bila

- Data yang diminta butuh provider berbayar (key belum ada).
- Data terlihat aneh (split belum disesuaikan, ticker salah).
- Ada permintaan eksekusi order — TOLAK + eskalasi.

## Proof of work

Ticker, provider, rentang tanggal, ringkasan dataset, dan sumber tercatat di DB.
