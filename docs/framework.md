# Framework Agen — Minime v2

> Cara kerja agen di Minime v2: kontrak agen senior, orkestrasi Mimi (CIO),
> dan cara pengguna membuat agen sendiri.
> Disusun 18 Okt 2026 dari keterangan Patria.

---

## 1. Konsep dasar

Minime v2 bukan sekumpulan agen tetap — melainkan **pabrik agen**.
6 agen seed disediakan sebagai contoh jadi; pengguna bisa menambah,
mengubah, dan mengatur agennya sendiri lewat UI (tanpa coding).

## 2. Kontrak agen senior

Setiap agen — bawaan maupun buatan pengguna — terikat kontrak yang sama:

1. **Peran & system prompt.** Siapa dia, untuk apa, batas wewenangnya.
2. **Playbook (skills).** Prosedur kerja langkah-demi-langkah yang bisa
   diulang. Skill disimpan sebagai dokumen terstruktur, dimuat on-demand.
3. **Tools allowlist.** Peralatan/aplikasi yang boleh disentuh agen
   (mis. Kreatif → Pireel; Riset → web). Di luar daftar = tidak bisa.
4. **Quality bar.** Standar mutu output yang harus dipenuhi sebelum
   diserahkan (diadaptasi dari playbook v1 di `skills/`).
5. **Bukti kerja (proof).** Setiap tugas menghasilkan deliverable yang
   bisa diperiksa — bukan klaim.
6. **Eskalasi.** Bila mentok, butuh keputusan, atau butuh aksi eksternal:
   naik ke Mimi → minta approval pengguna. Tidak menebak sendiri.

## 3. Mimi — CIO & orkestrator

- Mimi menerima permintaan pengguna dalam bahasa sehari-hari.
- Ia memecahnya menjadi tugas-tugas dan mendelegasikan ke agen yang tepat
  (satu agen bisa mengerjakan berantai: mis. Riset → Bisnis Strategis →
  Kreatif).
- Ia menggabungkan hasil, memeriksa quality bar, lalu menyerahkan ke
  pengguna untuk approval.
- Mimi memakai model Muse; orkestrasi tidak bisa diambil alih agen lain.

## 4. Alur tugas (runtime workflow)

```
Pengguna (chat)
   │  permintaan bahasa sehari-hari
   ▼
Mimi (CIO)
   │  pecah jadi tugas, pilih agen
   ▼
Agen senior (Kreatif / Riset / Finance / Bisnis Strategis / Code Builder)
   │  kerja pakai playbook + tools allowlist
   ▼
Deliverable + proof
   │  Mimi cek quality bar
   ▼
Approval pengguna ──► [setuju] ──► distribusi (draft-only utk Meta)
        │
        └──► [revisi] ──► kembali ke agen
```

- **Status agen** selalu terlihat: kuning = bekerja, hijau = siap,
  merah = butuh perhatian (aturan warna locked Patria).
- **Monitor tugas** menampilkan progres tiap tugas yang berjalan.
- Tugas bersifat FIFO per agen; bisa pause / resume / batal dari monitor.

## 5. Membuat agen sendiri

Pengguna menambah agen lewat UI "Agen Baru" — cukup isi:

1. Nama & peran (apa tugasnya, untuk siapa).
2. Model AI pilihan (Muse / model lokal gratis / model custom
   yang didaftarkan — API key milik pengguna, di keychain).
3. Skills/playbook (pilih dari gudang atau tulis sendiri).
4. Tools yang boleh diakses (centang dari daftar peralatan terhubung).
5. Budget pemakaian (opsional).

Tanpa coding. Agen baru langsung muncul di daftar agen dan bisa
didelegasikan tugas oleh Mimi.

## 6. Quality bar (kerangka)

Setiap agen punya quality bar sendiri yang didefinisikan di playbook-nya.
Kerangka umumnya:

- **Akurasi:** data & klaim terverifikasi, sumber dicantumkan.
- **Relevansi:** menjawab kebutuhan yang diminta, bukan yang lain.
- **Kejelasan:** bahasa sederhana, bisa dipahami pemula.
- **Kelengkapan:** semua bagian tugas terpenuhi sebelum diserahkan.
- **Kepatuhan:** tidak melanggar aturan berdiri (no auto-post, no live-trading).

Detail per agen ditetapkan saat Tahap 2 dan divonis Patria.

## 7. MeHarness — framework peningkatan performa agen

MeHarness adalah framework resmi Minime untuk **menaikkan mutu
konfigurasi agen secara terukur** (nama resmi dari Patria, 20 Okt 2026).
Dokumen lengkap: `docs/meharness.md`; library pelajaran:
`docs/meharness-lessons.md`.

- **Sumber metode:** loop Propose–Probe–Compose dari Harness² (Google,
  Apache 2.0) + konten terkurasi ECC (MIT). Kode benchmark Harness²
  TIDAK menjadi dependensi runtime — yang diadopsi metodenya.
- **Permukaan tuning:** 8 komponen yang boleh disunting loop —
  system prompt, guardrails, memory, tool guidance, subagents, skills,
  scripts, plugins. Semua permukaan ini sudah ada di Minime (§2–§6).
- **Cara kerja:** kumpulkan bukti dari tugas nyata + rubrik QC →
  usulkan 2 edit kandidat → uji pada set tugas yang sama → adopsi yang
  unggul → catat pelajaran. Pelajaran terbukti ≥3 putaran dipromosikan
  jadi aturan tetap.
- **Posisi di framework:** MeHarness adalah lapisan *peningkatan mutu*
  di atas kontrak agen senior (§2) — kontrak menetapkan standar, loop
  MeHarness membuktikan dan menaikkannya dari waktu ke waktu.
- **Pagar:** hasil loop untuk build publik tetap lewat **gate vonis
  Patria**; konfigurasi milik pengguna aplikasi publik tidak pernah
  diubah loop ini. Proses DEV, bukan fitur runtime.
- **Pilot:** putaran #1 pada Code Builder LULUS uji (20 Okt ~12:45).
