---
name: qc-rubrik
description: Rubrik penilaian QC penuh — skor 0-100 (kebenaran 30, kesesuaian brief 25, quality bar 25, kesiapan produksi 20).
owner_agent: qc
version: 1.0.0
example-prompt: Nilai output ini dengan rubrik QC.
---

# qc-rubrik — Rubrik Penilaian Penuh v1.0.0

QC adalah juri senior, bukan proofreader. Tugas: memastikan output LAYAK keluar dengan standar
senior — benar, sesuai brief, memenuhi quality bar, siap produksi.

## Ambang kelolosan

- **Skor ≥ 75 → LOLOS** ke Approval Patria (dengan catatan perbaikan bila ada).
- **Skor < 75 → TOLAK** dengan alasan spesifik + instruksi perbaikan konkret.
- **Maks 3x revisi** → status `attention` + eskalasi ke Patria.

## Dimensi 1 — Kebenaran & sumber (30 poin)

| Skor | Kriteria |
|------|----------|
| 27–30 | Semua klaim penting terverifikasi dengan sumber jelas; angka/fakta akurat |
| 20–26 | Klaim penting terverifikasi; ada 1–2 klaim minor tanpa sumber tapi tidak berisiko |
| 10–19 | Ada klaim penting tanpa verifikasi, atau sumber meragukan |
| 0–9 | Klaim kunci salah/tidak terverifikasi, atau berpotensi menyesatkan |

Aturan keras: klaim angka/dampak WAJIB terverifikasi atau eksplisit ditandai `[belum terverifikasi]`.
Konten dengan klaim tak terverifikasi DILARANG lolos.

## Dimensi 2 — Kesesuaian brief (25 poin)

| Skor | Kriteria |
|------|----------|
| 22–25 | Menjawab brief dengan tepat; tidak melenceng; tujuan + audiens jelas terpenuhi |
| 15–21 | Sesuai brief secara umum; ada bagian yang melebar tapi tidak fatal |
| 8–14 | Sebagian brief tidak terjawab, atau tone/audiens meleset |
| 0–7 | Tidak menjawab brief, atau menyelesaikan masalah yang salah |

## Dimensi 3 — Quality bar konten (25 poin)

Berdasarkan 7 pola inti quality bar (didokumentasikan pasca-reset PENDOPO 16 Okt 2026):

1. **Hook kuat** — 0–3 detik / baris pertama menarik; tanpa basa-basi (4 poin)
2. **Struktur PAS murni** — problem terasa, agitasi jujur & empatik, solusi konkret (4 poin)
3. **Satu gagasan inti** — satu core message per output; tidak campur aduk (4 poin)
4. **Keterbacaan HP** — kalimat pendek, paragraf 2–3 baris, enak dibaca di layar kecil (4 poin)
5. **Satu CTA jelas** — tepat satu ajakan sesuai tujuan (awareness/leads/konversi) (3 poin)
6. **Tone tepat** — direct/berani, casual ID-EN; bukan bahasa korporat kaku (3 poin)
7. **Golden Circle** — Why → How → What terasa dalam alur pesan (3 poin)

Larangan keras (otomatis TOLAK bila dilanggar): SARA & politik; guilt-trip/manipulasi rasa
bersalah; klaim tak terverifikasi; melanggar aturan no-auto-post (distribusi harus draft).

## Dimensi 4 — Kesiapan produksi (20 poin)

| Skor | Kriteria |
|------|----------|
| 18–20 | Format benar untuk medianya; bahasa natural; bebas typo fatal; siap publish |
| 12–17 | Perlu rapikan minor (format/typo kecil) tapi struktur sudah benar |
| 6–11 | Perlu perbaikan signifikan sebelum bisa diproduksi |
| 0–5 | Tidak siap produksi (format salah, bahasa kaku/aneh, banyak typo) |

## Pita skor (interpretasi)

- **90–100**: Excellent — lolos langsung, contoh standar.
- **75–89**: Lolos — dengan catatan perbaikan minor.
- **60–74**: Tolak — revisi spesifik, bisa diajukan ulang.
- **40–59**: Tolak — revisi besar, brief mungkin perlu diklarifikasi.
- **0–39**: Tolak — mulai ulang atau eskalasi ke Patria.

## Protokol revisi

1. Penolakan WAJIB menyebut: dimensi yang gagal + skor per dimensi + bukti konkret + instruksi perbaikan.
2. Dilarang menolak dengan alasan generik ("kurang bagus", "coba lagi").
3. Setiap revisi dicatat (revisi ke-1/2/3). Setelah revisi ke-3 masih < 75 → `attention` + eskalasi.
4. DILARANG mengubah isi output — QC menilai, bukan mengedit.

## Eskalasi ke Patria (wajib)

- Ambang kelolosan tidak jelas untuk kasus ini.
- Output bagus tapi berisiko (reputasi, sensitif, SARA/politik/figur publik).
- Produsen dan QC berbeda pendapat setelah 3 revisi.
- Keraguan etis (empati vs manipulasi, crisis response).

## Format output (WAJIB — dibaca mesin)

Akhiri penilaian dengan dua baris persis seperti ini:
```
SKOR: <angka 0-100>
ALASAN: <1-2 kalimat alasan utama>
```

Sertakan juga rincian: `D1: <skor>/30, D2: <skor>/25, D3: <skor>/25, D4: <skor>/20`.
