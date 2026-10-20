"""Tes untuk tugas2 — jalankan: python3 test_tugas2.py (exit 0 = semua lulus)."""
import sys

from parser import parse_rupiah


def test_rp_titik():
    assert parse_rupiah("Rp1.250.000") == 1250000


def test_tanpa_rp():
    assert parse_rupiah("75000") == 75000


def test_juta_koma():
    assert parse_rupiah("1,5 jt") == 1500000


def test_ribu():
    assert parse_rupiah("250 rb") == 250000


def test_spasi_dan_kapital():
    assert parse_rupiah("  RP2.000  ") == 2000


def _wajib_valueerror(teks):
    try:
        parse_rupiah(teks)
    except ValueError:
        return
    raise AssertionError(f"{teks!r} seharusnya raise ValueError")


def test_invalid_acak():
    _wajib_valueerror("abc")


def test_invalid_kosong():
    _wajib_valueerror("   ")


def test_invalid_negatif():
    _wajib_valueerror("-5000")


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
