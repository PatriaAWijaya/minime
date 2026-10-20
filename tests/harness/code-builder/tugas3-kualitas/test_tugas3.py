"""Tes perilaku untuk tugas3 — jalankan: python3 test_tugas3.py (exit 0 = lulus)."""
import sys

from laporan import buat_laporan


def test_format_ribuan():
    assert buat_laporan([1000000, 250000]) == "Total penjualan: Rp1.250.000"


def test_kosong():
    assert buat_laporan([]) == "Tidak ada penjualan."


def test_satu_item():
    assert buat_laporan([500]) == "Total penjualan: Rp500"


if __name__ == "__main__":
    semua = [v for k, v in sorted(globals().items()) if k.startswith("test_")]
    gagal = 0
    for t in semua:
        try:
            t()
            print(f"LULUS {t.__name__}")
        except AssertionError as e:
            gagal += 1
            print(f"GAGAL {t.__name__}: {e}")
    print(f"--- {len(semua) - gagal}/{len(semua)} lulus ---")
    sys.exit(1 if gagal else 0)
