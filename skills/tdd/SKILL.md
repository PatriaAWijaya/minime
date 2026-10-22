---
name: tdd
description: Perbaiki bug dengan test yang gagal dulu baru perbaiki — regression test terfokus untuk setiap bug fix.
owner_agent: code
version: 1.0.0
example-prompt: Fix bug toggle tema ini dengan TDD: bikin failing test dulu.
---

# tdd

Skill disiplin bug-fix milik agen Code Builder. Diadopsi dari `backnotprop/pstack` (skill `tdd`, MIT, karya poteto). Referensi penuh: `~/workspace/pstack-check/skills/tdd/`.

## Kapan dipakai

Ketika user minta TDD/failing test/regression test secara eksplisit, atau ketika bug punya target test lokal yang jelas dan murah. JANGAN dipaksakan bila jalur test tidak jelas, mahal, atau integration-heavy — pakai verifikasi terdekat yang masuk akal.

## Workflow

1. **Pahami bug**: perilaku diinginkan vs aktual, jalur terdampak, reproduksi terkecil.
2. **Pilih cek tersempit**: utamakan test yang sudah dipakai di codepath itu. Tidak ada jalur praktis → jangan buat dari nol demi workflow.
3. **Tulis test yang gagal dulu**: test terkecil yang seharusnya menangkap bug; mengkodekan perilaku yang diinginkan, bukan implementasi saat ini.
4. **Jalankan SEBELUM fix**: pastikan gagal karena alasan yang dimaksud.
5. **Fix**: perubahan produksi terkecil yang memenuhi perilaku diinginkan.
6. **Jalankan ulang**: pastikan lolos.

## Jika failing test tidak praktis

Pakai cek regresi terdekat: skrip terarah, reproduksi manual, otomasi browser, snapshot, asersi log. Lebih baik tanpa test baru daripada test buruk (kebanyakan mengetes mock, mengkodekan detail implementasi, bergantung timing, butuh infra mahal untuk fix kecil).

## Guardrails

- Jangan ubah test agar cocok dengan implementasi yang salah.
- Jangan lemahkan asersi kecuali perilaku yang diharapkan memang berubah.
- Jaga fokus pada bug; hindari churn fixture yang luas.
- Bug flaky → buat deterministik, dokumentasikan sinyalnya.

## Output

Laporkan bukti: test yang gagal-sebelum + failure-nya; test yang lolos-sesudah + validasi terdekat. Jika bukti gagal-sebelum tak bisa ditunjukkan, nyatakan alasannya.
