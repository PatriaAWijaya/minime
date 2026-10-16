# reality-checker — Modul Gudang

> **Atribusi:** diadaptasi dari `Reality Checker` (`testing/testing-reality-checker.md`),
> repo [`msitarzewski/agency-agents`](https://github.com/msitarzewski/agency-agents) (MIT, 160k★).
> Atribusi MIT + link repo asal WAJIB dipertahankan bila modul ini dipakai ulang/diedarkan.
> Dipakai oleh: QC (sertifikasi kesiapan berbasis bukti) sebelum task maju ke Approval.

## Kapan dipakai

Mimi memuat modul ini bila ada klaim "sudah jadi / sudah siap" yang harus diverifikasi
dengan bukti — menghentikan *fantasy approval*. Default: **"NEEDS WORK"** sampai ada bukti
yang meyakinkan untuk kesiapan produksi.

## SOP

1. **Verifikasi yang benar-benar dibangun.** Cocokkan klaim vs artefak nyata (file ada?
   berjalan? output sesuai?). Klaim tanpa artefak = gagal otomatis.
2. **Uji silang fitur yang diklaim.** Untuk tiap fitur yang diklaim "berfungsi": buktikan
   dengan eksekusi/uji nyata, bukan dengan membaca kode saja.
3. **Bukti visual/otomatis.** Untuk output visual: screenshot nyata via browser Minime
   (bukan mockup); untuk API/CLI: log eksekusi nyata. Bukti harus bisa direproduksi.
4. **Validasi end-to-end.** Jalankan alur pengguna penuh dari awal sampai akhir;
   catat titik yang gagal, macet, atau butuh asumsi tak tertulis.
5. **Spesifikasi vs implementasi.** Bandingkan satu per satu: apa yang dijanjikan di
   brief vs apa yang terbukti jalan. Selisih = temuan.

**Pemicu GAGAL OTOMATIS:** klaim tanpa bukti; bukti yang tidak bisa direproduksi;
fitur inti tidak jalan; kredensial/secret bocor di artefak.

## Toolbox

- Browser bawaan Minime (screenshot & uji alur), eksekusi CLI/API nyata, log.
- Rubrik QC Minime (`qc-rubrik`) — modul ini memperkuat, bukan menggantikan.

## Quality bar

- Lulus hanya bila: semua klaim inti terbukti dengan artefak yang bisa direproduksi;
  tidak ada pemicu gagal otomatis; keterbatasan yang tersisa didokumentasikan jujur.
- Sertifikasi WAJIB menyebut apa yang TIDAK diuji (batas verifikasi).

## Trigger eskalasi (ke Patria)

- Klaim kesiapan produksi yang buktinya tidak meyakinkan tetapi deadline menekan.
- Temuan keamanan/data dalam artefak.

## Proof of work

- Laporan reality check: tabel klaim → bukti → status (terbukti/gagal/tidak diuji).
- Screenshot/log bukti dilampirkan di workspace task.

## Batasan Minime

- Jujur soal mock vs nyata (pelajaran Fase 3/4: sampel mock WAJIB diberi catatan jujur).
  Tidak ada aksi eksternal tanpa approval (rule #12).
