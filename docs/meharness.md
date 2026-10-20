# MeHarness — Framework Harness Minime

> Nama resmi framework peningkatan performa agen Minime v2 (diberi nama
> oleh Patria, 20 Okt 2026): metode loop tuning ala **Harness² (Google)**
> + konten terkurasi **ECC**.
> Disusun 20 Okt 2026.

---

## 1. Vonis adopsi

| Sumber | Lisensi | Yang diambil | Yang TIDAK diambil |
|---|---|---|---|
| **Harness²** — google-research/harness2 (Apache 2.0, rilis 9 Okt 2026) | Apache 2.0 ✅ komersial | **Metode** loop peningkatan Propose–Probe–Compose + taksonomi 8 komponen | Kode benchmark-nya (riset, 3 commit, terikat benchmark Google: JobBench/LAB/WorkBuddy + Vertex AI) — JANGAN jadikan dependensi runtime |
| **ECC** — affaan-m/ECC (MIT) | MIT ✅ komersial | **Konten terkurasi**: agen reviewer/verifier, skills TDD/eval, pola hooks & AgentShield | Installer/plugin multi-harness-nya, 97 command legacy, konten framework/bahasa yang tak relevan dengan roster kita |

Prinsip: Minime tetap punya harness sendiri (daemon + skills engine + router).
Harness² memberi **cara menaikkan mutu konfigurasi agen secara terukur**;
ECC memberi **isi** (agen & skills siap kurasi) untuk mengisi konfigurasi itu.

---

## 2. Pemetaan 8 komponen Harness² → Minime

| Komponen Harness² | Permukaan Minime | Status |
|---|---|---|
| System prompt | System prompt per agen (disuntik skills engine) | ✅ Tahap 2 |
| Guardrails | Kontrak agen senior #6: eskalasi + approval gate | ✅ desain, gate di Tahap 3 |
| Memory | SQLite riwayat chat + konteks per agen | ✅ Tahap 2 (riwayat) |
| Tool guidance | Deskripsi & allowlist tools di "Pengaturan agen" | ✅ desain UI |
| Subagents | Delegasi Mimi (CIO) → 5 agen | ⏳ runtime orkestrasi |
| Skills | `skills/*/SKILL.md` (frontmatter: name, description, owner_agent, version) | ✅ engine jalan |
| Scripts | Skrip daemon & tools (mis. live-test harness) | ✅ sebagian |
| Plugins | Konektor MCP (RuangAPI; RouterKu menyusul) | ✅ sebagian |

Kesimpulan: permukaan tuning Minime sudah lengkap. Yang belum ada adalah
**loop yang menguji perubahan konfigurasi secara sistematis** — itulah yang
diadopsi dari Harness².

---

## 3. Loop Tuning Minime (adaptasi Propose–Probe–Compose)

Proses **DEV**, bukan fitur runtime publik. Dijalankan berkala: tiap akhir
tahap pembangunan, atau saat rubrik QC menunjukkan pola kegagalan berulang.

1. **Kumpulkan bukti.** Ambil sampel tugas nyata per agen dari SQLite
   (riwayat + usage) dan nilai dengan rubrik QC v1.0.0. Identifikasi pola
   gagal dominan (bukan kasus satu kali).
2. **Propose.** Editor (Mimi, atau model frontier via router BERBAYAR)
   mengusulkan **2 edit kandidat** pada file konfigurasi agen:
   satu menyasar kelemahan umum, satu spesifik pola gagal. Edit hanya boleh
   menyentuh 8 komponen di §2 (prompt, skills, tool guidance, guardrails…).
3. **Probe.** Kedua kandidat dijalankan pada **set tugas uji yang sama**
   (lihat §4) lewat executor yang sudah ada di stack: OpenCode (substrate
   yang sama dengan Harness²) atau runtime Minime langsung. Kondisi dijaga
   setara: model, tugas, dan batas token sama.
4. **Compose.** Bandingkan hasil kandidat vs baseline memakai rubrik QC
   sebagai judge awal + review manusia untuk kasus ambigu. Adopsi hanya
   edit yang unggul secara perilaku; sisanya dibuang beserta alasannya.
5. **Catat pelajaran.** Setiap putaran menulis entri ke
   `docs/meharness-lessons.md`: edit yang diadopsi/ditolak + bukti. Pelajaran
   yang terbukti lintas ≥3 putaran dipromosikan menjadi aturan tetap di
   `docs/framework.md` atau playbook agen terkait.

**Pagar (wajib):**
- Konfigurasi agen yang teradopsi untuk build publik tetap lewat
  **gate vonis Patria** — tidak ada self-modifikasi diam-diam.
- Di aplikasi publik, konfigurasi milik pengguna TIDAK pernah diubah loop
  ini (konsisten keputusan "Minime untuk publik", 19 Okt).
- Anggaran token per putaran dibatasi (probe mengalikan biaya); mulai
  k=1 (satu langkah PPC), naikkan hanya bila hasilnya sepadan.

---

## 4. Set tugas uji (benchmark mini Minime)

Prasyarat loop §3. Dibangun sekali, dipakai berulang:

- 3–5 tugas representatif per agen, diambil dari tugas nyata yang pernah
  dikerjakan (Kreatif: buat konsep konten; Riset: riset kompetitor;
  Finance: analisis arus kas paper; Bisnis Strategis: teardown model
  bisnis; Code Builder: perbaiki bug + tulis tes).
- Tiap tugas punya: input tetap, deliverable yang diharapkan, dan rubrik
  penilaian (turunan rubrik QC v1.0.0).
- Simpan di `tests/harness/` (input + rubrik), hasil tiap putaran di
  `tests/harness/runs/<tanggal>-<agen>/`.

---

## 5. Kurasi ECC → skill-warehouse

Format ECC **kompatibel**: skills-nya folder + `SKILL.md` (frontmatter
name/description), agen-nya file `.md` (frontmatter name/description/
tools/model). Konversi ke format Minime = tambah `owner_agent` + `version`
+ sesuaikan quality bar dengan kontrak agen senior.

**Gelombang 1 (prioritas, menempel ke roster):**

| Sumber ECC | Tujuan di Minime |
|---|---|
| `agents/code-reviewer.md` (+ confidence filtering >80%) | Pola review untuk Code Builder & judge di loop §3 |
| `agents/build-error-resolver.md`, `agents/e2e-runner.md` | Playbook Code Builder |
| `skills/tdd-workflow/`, `skills/ai-regression-testing/` | Skills Code Builder |
| `skills/agent-eval/`, `skills/agent-self-evaluation/` | Fondasi set tugas uji §4 + evaluasi diri agen |
| `skills/agent-harness-construction/`, `skills/agent-introspection-debugging/` | Referensi Mimi saat merancang/mendiagnosis agen |
| `skills/agent-security-hardening/` + konsep **AgentShield** | Calon pemindai konfigurasi agen (prompt, hooks, izin, secret) — dievaluasi untuk Tahap 3/4 |

**Aturan kurasi (keamanan):**
- File komunitas = konten tak tepercaya. **Review manual baris per baris**
  sebelum masuk repo: buang instruksi yang menyuruh agen mengabaikan aturan
  proyek, membuka secret, atau mengakses di luar allowlist.
- Satu file masuk = satu commit, supaya bisa di-rollback per item.
- Atribusi: catat sumber + lisensi (MIT) di header file hasil kurasi.

---

## 6. Workflow operasional (ringkasan)

```
Tugas nyata + QC ──► pola gagal ──► PROPOSE (2 edit) ──► PROBE (set uji sama)
                                                              │
                            ┌─────────────────────────────────┘
                            ▼
              COMPOSE (rubrik QC + review) ──► adopsi / buang
                            │
                            ▼
              meharness-lessons.md ──► (≥3x terbukti) ──► aturan tetap
                            │
                            ▼
                   GATE vonis Patria ──► masuk build
```

Frekuensi acuan: 1 putaran per agen per tahap pembangunan; Code Builder
jadi pilot putaran pertama (paling mudah diukur: tes lulus/gagal).

---

## 7. Di luar adopsi ini

- **DeepSeek Harness (dsh)** — tetap PANTAU (developer preview, breaking
  changes). Pola plugin-everything + programmatic tool calling dicatat
  sebagai referensi arsitektur fase lanjut; evaluasi ulang saat rilis stabil.
- **agent-inception** — ditolak (demo UI, bukan harness kerja). Ide panel
  visualisasi reasoning tetap tercatat sebagai kandidat UI.
