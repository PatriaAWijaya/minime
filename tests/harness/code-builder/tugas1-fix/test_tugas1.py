"""Tes untuk tugas1 — jalankan: python3 test_tugas1.py (exit 0 = semua lulus)."""
import sys

from kalkulator import hitung_diskon


def test_dasar():
    assert hitung_diskon(100000, 10) == 90000


def test_tipe_int():
    hasil = hitung_diskon(99999, 33)
    assert isinstance(hasil, int), f"hasil harus int, dapat {type(hasil).__name__}"


def test_bulat_ke_bawah():
    # 99999 * 33 / 100 = 32999.67 -> potongan 32999 -> hasil 67000
    assert hitung_diskon(99999, 33) == 67000


def test_clamp_atas():
    assert hitung_diskon(100000, 150) == 0


def test_persen_negatif():
    assert hitung_diskon(100000, -5) == 100000


def test_nol():
    assert hitung_diskon(0, 50) == 0


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
