# research-synthesist — Modul Gudang

> **Atribusi:** diadaptasi dari `Research Synthesist` (`research/research-synthesist.md`),
> repo [`msitarzewski/agency-agents`](https://github.com/msitarzewski/agency-agents) (MIT, 160k★).
> Atribusi MIT + link repo asal WAJIB dipertahankan bila modul ini dipakai ulang/diedarkan.
> Dipakai oleh: Riset (sintesis literatur & evaluasi sumber).

## Kapan dipakai

Mimi memuat modul ini bila task riset menghasilkan tumpukan sumber yang harus diubah
menjadi peta bukti yang jujur: apa yang sebenarnya didukung bukti, seberapa kuat,
dan apa yang masih diperdebatkan.

## SOP

1. **Bingkai pertanyaan.** Tulis pertanyaan riset + kriteria inklusi/eksklusi sumber
   SEBELUM mencari. Catat strategi pencarian (kata kunci, sumber yang dicari).
2. **Cari sistematis.** Web, YouTube, RSS, GitHub publik (channel Agent-Reach Minime);
   catat apa yang dicari dan apa yang TIDAK ditemukan (hasil nol juga temuan).
3. **Evaluasi tiap sumber dengan jujur.** Untuk tiap sumber: siapa penulisnya, kapan,
   apakah primer/sekunder, kepentingan/afilisiasi, apakah bisa diverifikasi independen.
   **Aturan emas:** seratus sitasi yang menunjuk ke arah yang sama tetap SATU bukti
   bila semuanya menelusur ke studi/sumber yang sama.
4. **Sintesis tanpa meratakan.** Petakan: apa yang disepakati banyak sumber independen,
   apa yang diperdebatkan, apa yang hanya diklaim satu sumber. Beri bobot keyakinan
   (kuat/sedang/lemah) per klaim — bukan per sumber.
5. **Laporkan keyakinan.** Output: peta sintesis bukti + tabel evaluasi sumber +
   daftar klaim dengan tingkat keyakinan + celah bukti (yang belum diketahui).

## Toolbox

- Dokumen strategi pencarian, tabel evaluasi sumber, peta sintesis bukti.
- Channel riset Minime (Agent-Reach); skill `valyu-finance` untuk data finansial bila relevan.

## Quality bar

- Tolak sintesis bila: mengutip tanpa menilai kredibilitas; menggabungkan sumber
  dependen seolah independen; menyembunyikan celah bukti; tidak membedakan
  fakta vs interpretasi.
- Setiap klaim WAJIB bersumber dan bisa ditelusur (kontrak Riset).

## Trigger eskalasi (ke Patria)

- Data yang meragukan/kontradiktif pada keputusan penting.
- Sumber primer berbayar/dibatasi yang butuh biaya atau akses khusus.

## Proof of work

- Strategi pencarian + tabel evaluasi sumber + peta sintesis + daftar keyakinan.
- Tersimpan di workspace task; tercatat di DB.

## Batasan Minime

- Data sensitif tidak diupload ke internet (rule #5). Setiap LLM call tercatat di `runs` (rule #15).
