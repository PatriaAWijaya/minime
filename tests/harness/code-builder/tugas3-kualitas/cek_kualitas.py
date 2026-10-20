"""Cek kualitas laporan.py — exit 0 = lulus semua cek.

Cek: (1) tidak ada print debug di laporan.py, (2) fungsi buat_laporan
punya docstring, (3) tidak ada default argument mutable.
"""
import inspect
import sys

import laporan

sumber = inspect.getsource(laporan)
gagal = []

if "print(" in sumber:
    gagal.append("masih ada print( di laporan.py")
if not (laporan.buat_laporan.__doc__ or "").strip():
    gagal.append("buat_laporan belum punya docstring")
if "penjualan=[]" in sumber.replace(" ", ""):
    gagal.append("masih ada default argument mutable penjualan=[]")

for g in gagal:
    print(f"GAGAL kualitas: {g}")
if not gagal:
    print("LULUS semua cek kualitas")
sys.exit(1 if gagal else 0)
