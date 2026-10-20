# Spesifikasi `parse_rupiah(teks) -> int`

Fungsi mengubah teks nominal rupiah menjadi bilangan bulat.

- `"Rp1.250.000"` → `1250000` (awalan `Rp` opsional, titik = pemisah ribuan)
- `"75000"` → `75000`
- `"1,5 jt"` → `1500000` (akhiran `jt` = juta, koma = koma desimal)
- `"250 rb"` → `250000` (akhiran `rb` = ribu)
- Spasi di awal/akhir diabaikan; huruf besar/kecil diabaikan.
- Input tidak valid (teks acak, string kosong, angka negatif) → raise `ValueError`.
