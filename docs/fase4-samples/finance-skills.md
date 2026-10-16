# Sampel Finance Skills (mock)

> ⚠️ **CATATAN JUJUR:** File ini BUKAN output beneran analisis/backtest.
> Ini bukti bahwa wiring skill → playbook TERBUKTI via tugas mock
> (`PLAYBOOK AKTIF: <nama-skill>` ter-inject ke agen). Uji koneksi Python asli
> dicatat di `docs/status.md` (qlib/TradingAgents/nautilus TERBUKTI import+init di VM).

## Tugas: `fin-tc-trading-agents`

Perintah mock: "Jalankan analisis multi-agen saham BBCA..."
Hasil: `PLAYBOOK AKTIF: trading-agents` ✅

## Tugas: `fin-tc-qlib`

Perintah mock: "Jalankan workflow qlib: alpha158 + LightGBM, backtest IHSG 2024..."
Hasil: `PLAYBOOK AKTIF: qlib` ✅

## Tugas: `fin-tc-nautilus`

Perintah mock: "Jalankan backtest NautilusTrader venue SIM..."
Hasil: `PLAYBOOK AKTIF: nautilus` ✅

## Tugas: `fin-tc-openbb`

Perintah mock: "Ambil harga historis BBCA.JK via openbb..."
Hasil: `PLAYBOOK AKTIF: openbb` ✅

## Aturan keras (berlaku untuk semua)

**DILARANG order trading live otomatis.** Hanya riset, analisis, backtest,
paper-trading. Order live HANYA dengan approval eksplisit Patria per aksi.
Penolakan live-trading diimplementasikan di `daemon/src/tools/quant.rs`.
