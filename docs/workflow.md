# Workflow — Minime v2

> Dua workflow: (A) tahapan pembangunan, (B) alur kerja runtime.
> Disusun 18 Okt 2026 dari keterangan & verdict Patria.

---

## A. Workflow pembangunan

Urutan (keputusan Patria 18 Okt 2026): **UI/UX dulu → fitur + skills agen
→ tools → rilis macOS + Windows.** Gate tiap tahap = review + vonis Patria.

### Tahap 1 — UI/UX klik-tayang ✅ SELESAI (18 Okt malam)

- [x] Verdict stack: Tauri 2 + Svelte 5
- [x] Scaffold proyek fresh (`app/`)
- [x] Layout 4 kolom ala OpenBot: sidebar workflow | daftar agen |
      main (chat + delivery) | monitor tugas berjalan
- [x] 6 agen seed (Mimi CIO + 5) dengan badge avatar & status
      (kuning/hijau/merah)
- [x] 3 bahasa: Indonesia (default), Inggris, Mandarin
- [x] Navigasi: Pengaturan, Penggunaan, Peralatan
- [x] Prinsip compact-but-powerful & pemula-friendly diterapkan
- **Gate:** vonis Patria atas tampilan (TERBUKA)

### Tahap 2 — Fitur + skills agen

- [ ] Runtime agen sungguhan: chat → Mimi orkestrasi → eksekusi → output
      (saat ini chat masih mock lokal)
- [ ] Skills + quality bar per agen (adaptasi playbook v1 di `skills/`)
- [ ] Model router: Muse default + model lokal gratis + custom user;
      routing per-agen + fallback + metering + budget cap
- [ ] Persistensi SQLite di-wiring (riwayat, tugas, usage)
- **Gate:** review + vonis Patria

### Tahap 3 — Tools & distribusi

- [ ] Wiring tools: Meta (**draft-only**), Postiz/Viralin (**draft-only**),
      Pireel; tombol hubung peralatan diaktifkan
- [ ] Approval gate sebelum aksi eksternal; notifikasi native
- [ ] Upload deliverable ke folder MIMI di Google Drive
- **Gate:** 1 pipeline end-to-end + vonis Patria

### Tahap 4 — Rilis macOS + Windows

- [ ] Build dmg (macOS) + exe (Windows); installer mudah & intuitif
- [ ] Security audit + Impeccable audit
- **Gate:** instalasi bersih di mesin baru + vonis rilis Patria

## B. Workflow runtime (saat aplikasi dipakai)

Lihat `docs/framework.md` bagian 4 untuk diagram lengkap. Ringkasnya:

1. **Pengguna** menulis permintaan ke Mimi dalam bahasa sehari-hari.
2. **Mimi (CIO)** memecah jadi tugas, mendelegasikan ke agen yang tepat.
3. **Agen** bekerja dengan playbook + tools allowlist-nya; progres terlihat
   di monitor tugas.
4. **Deliverable + proof** diserahkan; Mimi memeriksa quality bar.
5. **Pengguna approve** (atau minta revisi). Aksi eksternal — terutama
   distribusi ke Meta — selalu **draft + approval**, tidak pernah otomatis.
6. Hasil tersimpan lokal (SQLite); dokumen penting bisa diunggah ke
   folder MIMI di Google Drive.

## C. Aturan kerja repo

- Commit format `<tipe>: <ringkasan>`; UI & dokumen Bahasa Indonesia,
  kode & komentar Inggris.
- Push GitHub via Git Data API (`push_rebased.py`); catat pasangan
  SHA lokal/remote di `~/AGENTS.md`.
- Kredensial tidak di git. Token GitHub tidak punya scope `workflow`
  dan `delete_repo` (catat bila menemui batas ini).
- `docs/status.md`: setiap kejadian penting di-append sebagai entri baru.
