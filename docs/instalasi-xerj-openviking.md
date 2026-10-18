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

> **Pelajaran instalasi nyata 18 Okt 2026 (MacBook, RAM 8GB):**
> - `pip3 install openviking` langsung OK (tanpa venv — macOS tidak kena PEP 668)
> - Binary CLI bernama **`openviking`**, bukan `ov` → buat symlink dulu:
>   `ln -sf /Library/Frameworks/Python.framework/Versions/3.13/bin/openviking ~/.local/bin/ov`
> - Config final HEMAT-RAM (wajib di 8GB — setup Ollama penuh bikin MacBook berat sampai restart):
>   init → **Start over** → **Step-by-step** → Embedding **Lightweight CPU**
>   (llama-cpp-python + `bge-small-zh-v1.5-f16`, ~24MB) → VLM **Custom (OpenAI-compatible)**
>   (`https://generativelanguage.googleapis.com/v1beta/openai/` + key Gemini Keychain +
>   `gemini-2.0-flash`) → Query planner `ov_intent_analysis_sft v7_q8` (~0.8B)
>   → server 127.0.0.1:1933 dev mode
> - Ganti dimensi embedding (mis. 1024→512) → server gagal start
>   (`EmbeddingRebuildRequiredError`); solusi: `rm -rf ~/.openviking/data` (bila kosong)

```sh
pip3 install openviking
ln -sf /Library/Frameworks/Python.framework/Versions/3.13/bin/openviking ~/.local/bin/ov
ov language en
openviking-server init     # interaktif — ikuti wizard (lihat pelajaran di atas)
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
