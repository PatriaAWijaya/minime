def buat_laporan(penjualan=[]):
    total = 0
    for p in penjualan:
        total = total + p
    print("DEBUG total:", total)
    return "Total penjualan: Rp" + str(total)
