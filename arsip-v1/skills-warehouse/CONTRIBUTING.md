# Menambah Modul ke Gudang Skill

> Gudang = adaptasi spesialis dari `msitarzewski/agency-agents` (MIT) ke format Minime.
> Referensi mentah: `~/workspace/referensi/agency-agents/`.

## Langkah

1. Pilih spesialis dari `INDEX.md` (atau langsung dari referensi) yang relevan dengan kebutuhan Minime.
2. Baca file sumbernya; suling ke Bahasa Indonesia dengan struktur modul:
   - Header atribusi (WAIJIB): lisensi MIT + nama repo asal + link + nama file sumber.
   - **Kapan dipakai** — trigger tugas yang membutuhkan modul ini.
   - **SOP** — langkah kerja bernomor, dalam Bahasa Indonesia.
   - **Toolbox** — alat/teknik yang boleh dipakai.
   - **Quality bar** — standar kelulusan output; kapan menolak output.
   - **Trigger eskalasi** — kapan eskalasi ke Patria (Approval).
   - **Proof of work** — bukti kerja yang wajib dihasilkan.
   - **Batasan Minime** — larangan yang berlaku (mis. tidak ada auto-posting ke Meta,
     tidak ada order trading otomatis, distribusi hanya draft).
3. Simpan sebagai `skills-warehouse/<nama-modul>/MODULE.md` (nama folder: kebab-case, tanpa prefix kategori).
4. Tandai ✅ di `INDEX.md` pada baris spesialis tersebut.
5. Commit: `feat: modul gudang <nama-modul>`.

## Aturan

- Jangan menyalin mentah seluruh file sumber — suling dan adaptasi ke konteks Minime.
- Atribusi MIT + link repo asal WAJIB di setiap modul.
- Modul gudang TIDAK menambah agen baru — mereka adalah "otot pinjaman" untuk 5 agen senior.
- Bila modul menyangkut publikasi/distribusi: terapkan aturan draft-only (rule #3).
