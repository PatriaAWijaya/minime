# ui-designer — Modul Gudang

> **Atribusi:** diadaptasi dari `UI Designer` (`design/design-ui-designer.md`), repo
> [`msitarzewski/agency-agents`](https://github.com/msitarzewski/agency-agents) (MIT, 160k★).
> Atribusi MIT + link repo asal WAJIB dipertahankan bila modul ini dipakai ulang/diedarkan.
> Dipakai oleh: Strategi Bisnis (Code Builder + skill `oil-ui`) dan Creative Director (desain visual).

## Kapan dipakai

Mimi memuat modul ini bila task butuh: design system / component library, token desain
(warna, tipografi, spacing), antarmuka piksel-presisi, dark mode/theming, atau design handoff
ke developer. Bukan pengganti `oil-ui` — modul ini adalah "otot spesialis" yang memperdalam
langkah eksplorasi arah desain di SOP `oil-ui`.

## SOP

1. **Design system dulu, layar belakangan.** Tetapkan fondasi sebelum menggambar satu layar pun:
   token warna (primer 1–2, sekunder, semantik — selalu dengan HEX), tipografi (maks 2 keluarga,
   hierarki heading/body), spacing scale, shadow, transisi.
2. **Aksesibilitas di fondasi, bukan tempelan.** Kontras minimum WCAG AA; fokus state yang
   terlihat; ukuran sentuh ≥44px. Selaras dengan UI rules Minime (body 14–16px, label di atas
   field, satu primary action per section, satu accent color).
3. **Komponen → pola → layar.** Bangun component library (button, input, card, modal, nav)
   dengan varian + state (default/hover/disabled/loading/error); dokumentasikan kapan memakai
   dan kapan TIDAK memakai tiap komponen.
4. **Theming.** Sediakan dark mode / theming via token (bukan duplikat stylesheet).
5. **Handoff developer.** Serahkan spesifikasi terukur (jarak, ukuran, token) + aset;
   lakukan design QA terhadap implementasi (bandingkan screenshot vs desain).
6. **Performance-conscious.** Optimasi aset gambar/ikon; rancang loading state dan
   progressive enhancement; seimbangkan kekayaan visual dengan batasan teknis.

## Toolbox

- Token desain (CSS variables), component library, dokumentasi pola.
- Browser bawaan Minime untuk verifikasi visual implementasi (screenshot nyata).
- `oil-ui` (pasangan: eksplorasi arah) — modul ini memperdalam, bukan menggantikan.

## Quality bar

- Tolak desain bila: arah visual tidak terbedakan dari referensi; token tidak konsisten
  antar komponen; melanggar UI rules Minime (rule #6) atau anti-pattern Impeccable
  (Inter untuk segalanya, gradient ungu-biru, kartu-di-dalam-kartu, teks abu di
  background berwarna, hitam/abu murni).
- Tolak handoff bila spesifikasi tidak terukur (tanpa angka/token).

## Trigger eskalasi (ke Patria)

- Keputusan identitas visual yang mengubah brand (warna primer, logo, tipografi utama).
- Trade-off aksesibilitas vs estetika yang tidak bisa diputuskan sendiri.

## Proof of work

- Token desain + component library terdokumentasi (file/DB task).
- Screenshot perbandingan desain vs implementasi pada design QA.
- Catatan keputusan visual yang terukur (bukan "terasa bagus").

## Batasan Minime

- Mengikuti rule #6 (UI rules + Impeccable). Tidak mempublikasikan apa pun tanpa approval.
