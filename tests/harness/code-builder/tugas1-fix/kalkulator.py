def hitung_diskon(harga, persen):
    """Hitung harga akhir setelah diskon `persen` (0-100).

    Hasil harus bilangan bulat (int); potongan dibulatkan ke bawah.
    Persen di atas 100 diperlakukan sebagai 100; persen negatif sebagai 0.
    """
    if persen < 0:
        persen = 0
    potongan = harga * persen / 100
    return harga - potongan
