# Instalasi xerj + OpenViking (MacBook Patria)

> Dry run TERVERIFIKASI di VM Linux 18 Okt 2026 dini hari:
> xerj v1.0.0-rc.96 (install → node → autoindex 46 file → retrieval OK);
> openviking 0.5.0 (pip install → CLI OK).
> Langkah MacBook di bawah identik, kecuali server init yang interaktif.

## xerj (5 menit)

```sh
curl -fsSL https://xerj.org/get | sh
```

Bila `xerj: command not found` setelahnya:

```sh
export PATH="$HOME/.local/bin:$PATH"   # tambah permanen ke ~/.zshrc
```

Jalankan node (tiap sesi kerja; bind `127.0.0.1` saja):

```sh
xerj --insecure --data-dir ~/.minime/xerj-data &
```

> Flag `--insecure` mematikan TLS+auth — batas kepercayaan: hanya untuk
> satu user di mesin sendiri. Jangan expose port ke jaringan.

Index folder referensi (sekali; ulangi bila korpus berubah):

```sh
xerj autoindex ~/referensi/nama-proyek
xerj autoindex ~/workspace/minime/docs
```

Pakai:

```sh
xerj search "pertanyaan bahasa natural"   # kode: passage + file:line
xerj def "nama_fungsi"                   # go-to-definition
xerj gain                                # laporan token yang dihemat
```

Catatan: `xerj search` bersifat definition-first (untuk kode). Untuk korpus
murni dokumen, query via API Elasticsearch-kompatibel di `localhost:9200`.

## OpenViking (10 menit)

```sh
pip install openviking
# alternatif: uv tool install openviking --upgrade
ov language en
openviking-server init     # interaktif — ikuti wizard
```

Jalankan server background:

```sh
nohup openviking-server > ~/.minime/openviking.log 2>&1 &
```

Hubungkan CLI, lalu verifikasi:

```sh
ov config    # interaktif — arahkan ke server lokal
ov status
```

Coba tulis & baca konteks pertama:

```sh
ov add-memory "Patria menolak auto-posting Meta sampai metode aman ditemukan"
ov find "auto-posting" --uri viking://
```

## Setelah instalasi

- [ ] `tools.status` di Minime menunjukkan `marketdata`/`xerj`/`openviking` siap
      (xerj & openviking dipakai via CLI oleh agen sesuai skill masing-masing)
- [ ] FRED API key gratis: https://fred.stlouisfed.org/docs/api/api_key.html →
      simpan di Keychain akun `fred` (atau env `MINIME_FRED_KEY`)
- [ ] `pip install yfinance` (untuk op `quote` tool `marketdata`)
