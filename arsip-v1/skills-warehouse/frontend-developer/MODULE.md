# frontend-developer — Modul Gudang

> **Atribusi:** diadaptasi dari `Frontend Developer` (`engineering/engineering-frontend-developer.md`),
> repo [`msitarzewski/agency-agents`](https://github.com/msitarzewski/agency-agents) (MIT, 160k★).
> Atribusi MIT + link repo asal WAJIB dipertahankan bila modul ini dipakai ulang/diedarkan.
> Dipakai oleh: Strategi Bisnis (Code Builder — implementasi aplikasi web).

## Kapan dipakai

Mimi memuat modul ini bila task Code Builder butuh implementasi frontend web modern:
aplikasi React/Vue/Svelte, komponen UI, optimasi performa, atau aksesibilitas implementasi.

## SOP

1. **Setup & arsitektur.** Struktur proyek yang jelas (komponen/halaman/state/utils);
   pilih stack sesuai kebutuhan (jangan over-engineering); TypeScript strict.
2. **Pengembangan komponen.** Komponen kecil, satu tanggung jawab; props ter-typing;
   state minimal dan dekat dengan yang memakainya; hindari duplikasi via komponen
   bersama (lihat modul `ui-designer` untuk token/komponen desain).
3. **Performance-first.** Lazy-load rute/komponen berat; optimasi gambar (format modern,
   ukuran responsif); hindari render ulang yang tidak perlu; ukur sebelum dan sesudah
   (bukan "terasa lebih cepat").
4. **Aksesibilitas.** HTML semantik; label pada semua input; navigasi keyboard;
   kontras warna; uji dengan screen reader untuk alur kritis.
5. **Kualitas & testing.** Lint + format + type-check bersih; uji komponen kritis;
   tidak ada `console.log` sisa; tidak ada secret di kode (rule #2).
6. **Review silang.** Kode penting direview via modul `code-reviewer` sebelum diserahkan.

## Toolbox

- Framework modern (React/Vue/Svelte), TypeScript strict, linter/formatter.
- Modul `ui-designer` (token & komponen desain), `code-reviewer` (review),
  browser Minime (verifikasi visual).

## Quality bar

- Tolak kode bila: type-check/lint gagal; ada secret/hardcoded credential; aksesibilitas
  dasar dilanggar (input tanpa label, kontras gagal); performa tidak diukur;
  tidak ada penanganan loading/error state.

## Trigger eskalasi (ke Patria)

- Pilihan stack/teknologi yang mengikat jangka panjang (mis. framework baru, layanan berbayar).
- Perubahan scope fitur di tengah implementasi.

## Proof of work

- Kode di workspace task + hasil lint/type-check + screenshot verifikasi browser.
- Catatan keputusan arsitektur singkat (kenapa stack ini).

## Batasan Minime

- Kredensial/API key tidak di-commit (rule #2). Bahasa kode & komentar: Inggris (rule #13).
