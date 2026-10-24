# Panduan Orkestrasi Mimi

> Fungsi `orchestrate_task` — Mimi sebagai orkestrator multi-agen.
> Tanpa UI (keputusan Patria 24 Okt 2026). Dipanggil via Tauri command.

## Kapan Pakai Orkestrasi vs Chat Biasa

| Situasi | Pakai |
|---------|-------|
| Satu pertanyaan, satu agen | Chat biasa |
| Tugas butuh 2+ agen (riset → tulis → review) | **Orkestrasi** |
| Butuh QC otomatis dengan revisi | **Orkestrasi** |
| Eksperimen cepat, tidak butuh struktur | Chat biasa |

## Cara Kerja

```
[0] Pre-flight → cek kredensial Postiz (jika butuh distribusi)
[1] Parse → Mimi ubah request jadi JSON subtugas
[2] Dispatch → subtugas jalan PARALEL per agen
[3] Agregat → Mimi gabung jadi laporan kohesif
[4] QC → nilai dengan rubrik (ambang 75)
[5] Revisi → jika QC gagal, ulangi maks 3x (hanya agen yang perlu)
```

## Contoh Prompt yang Baik

**Baik:**
> "Riset 3 kompetitor kopi di Surabaya, buatkan perbandingan harga dan strategi marketing mereka, lalu tulis ringkasan eksekutif"

**Kurang baik:**
> "Bikinin sesuatu tentang kopi"

Orkestrasi butuh request yang jelas agar bisa dipecah jadi subtugas.

## Ekspektasi Biaya & Waktu

| Komponen | Estimasi |
|----------|----------|
| Parse (1x Spark) | ~$0.001 |
| Per subtugas (Spark) | ~$0.002 |
| Agregat + QC (2x Spark) | ~$0.004 |
| **Total tipikal (3 subtugas)** | **~$0.01** |
| Revisi (jika QC gagal) | +$0.005 per putaran |

**Waktu:** 2–5 menit untuk 3 subtugas (tergantung latensi LLM).
Jika ada revisi, tambah 1–2 menit per putaran.

**Batas:** Estimasi > $5 ditolak otomatis. Peringatan jika > $1.

## Model Selection Otomatis

- Skor kompleksitas < 60 → **Spark** (murah, cepat)
- Skor kompleksitas ≥ 60 → **Wonder** (powerful, 3–5x lebih mahal)

Faktor: panjang prompt, kata kunci (analisis, strategi, banding), jenis agen.

## Troubleshooting

| Gejala | Penyebab | Solusi |
|--------|----------|--------|
| Error "Postiz belum dikonfigurasi" | Minta posting tapi kredensial kosong | Isi di Pengaturan → Tools → Apps |
| QC selalu gagal | Request terlalu ambigu | Perjelas request, pecah manual |
| Timeout | Subtugas terlalu kompleks | Sederhanakan, atau pecah jadi 2 request |
| Biaya tinggi | Banyak subtugas + Wonder | Kurangi scope, atau pakai chat biasa |

## Melihat Riwayat

Buka **Pengaturan → Diagnostik → Riwayat Orkestrasi**.
Menampilkan: waktu, agen, panjang prompt, skor kompleksitas, model, skor QC.

Jika **parse failure rate > 20%** dalam 24 jam, muncul peringatan —
tandanya prompt template perlu diperbaiki (lapor ke developer).
