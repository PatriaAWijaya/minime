# Lisensi Pihak Ketiga — Komponen Quant/Trading

> Dokumen ini mencatat lisensi semua komponen eksternal yang dikoneksikan ke Minime
> untuk finance skills. Diperbarui: 17 Okt 2026.

## Tabel lisensi

| Komponen | Repo | Lisensi | Status komersil | Cara koneksi |
|---|---|---|---|---|
| TradingAgents | `TauricResearch/TradingAgents` | Apache-2.0 | ✅ Aman | Tool eksternal (pip/CLI/subprocess). Tidak di-vendor. |
| qlib | `microsoft/qlib` | MIT | ✅ Aman | Tool eksternal (`pip install pyqlib`). Tidak di-vendor. |
| NautilusTrader | `nautechsystems/nautilus_trader` | **LGPL-3.0** | ⚠️ Bersyarat | **HANYA sebagai proses eksternal** (`python3` subprocess). TIDAK static-link ke binary Minime. |
| OpenBB SDK | PyPI `openbb` (+ `OpenBB-finance/agents-for-openbb` MIT) | MIT (SDK) | ✅ Aman | Tool eksternal via SDK. |
| Valyu API | `yorkeccak/finance` (pola) | MIT (repo pola) | ✅ Aman | HTTP API eksternal (bukan kode). |

## Kewajiban LGPL-3.0 (NautilusTrader)

NautilusTrader berlisensi GNU Lesser General Public License v3.0.
Minime memenuhinya dengan cara:

1. **Tidak me-link**: kode NautilusTrader TIDAK dikompilasi menjadi bagian dari
   binary `minime-daemon`. Koneksi HANYA via subprocess (`python3` menjalankan
   skrip backtest yang mengimpor `nautilus_trader`) — proses terpisah.
2. **Tidak memodifikasi**: Minime tidak mengubah kode NautilusTrader.
3. **Atribusi**: Nautech Systems Pty Ltd — https://nautilustrader.io
4. **Teks lisensi**: salinan LGPL-3.0 tersedia di
   https://www.gnu.org/licenses/lgpl-3.0.en.html

## Catatan

- Tidak ada kode copyleft yang di-vendor ke repo Minime.
- Semua komponen di atas dipakai sebagai tool eksternal (proses/API terpisah).
