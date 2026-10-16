---
name: qc-rubrik
description: Rubrik penilaian QC — skor 0-100 (kebenaran 30, kesesuaian brief 25, quality bar 25, kesiapan produksi 20).
owner_agent: qc
version: 0.1.0
example-prompt: Nilai output ini dengan rubrik QC.
---

# qc-rubrik

Skor ≥ 75 → LOLOS ke Approval. Skor < 75 → TOLAK dengan alasan spesifik + instruksi perbaikan.
Maks 3x revisi → status attention + eskalasi ke Patria.
Status: stub Fase 1 (rubrik penuh di Fase 5).
