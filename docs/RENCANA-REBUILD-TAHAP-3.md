# Rencana Rebuild Tahap 3 — Tools & Distribusi
> Disiapkan 20 Okt 2026 12:31 WIB atas perintah Patria. Tahap 3 pernah selesai 19 Okt, hilang saat rollback 19 Okt 22:26.
> Metode: **pulihkan kode dari git history → adaptasi ke kode saat ini → uji satu per satu sambil monitor**.
> Aturan: setiap akhir sub-fase WAJIB cek konsistensi + performa (tidak boleh ada error lolos). Gate akhir = pipeline E2E + vonis Patria.

---

## SUMBER KODE LAMA (git history — semua bisa dipulihkan)
| Commit | Isi |
|---|---|
| `b34e4f2` | Tools gateway: `app/src-tauri/src/tools.rs` (176 baris) — registry Rust + status koneksi |
| `4ddc817` | Notifikasi native untuk approval gate |
| `186d9ae` | Allowlist tools per agen + draft-only + framework Pireel |
| `28674c5` | Framework Google Drive + docs Tahap 3 |
| `4c0c1f6` | Orkestrasi Mimi via delegasi chat (pendukung runtime) |

Cara pulih: `git show <commit>:<path>` per file, lalu adaptasi manual — BUKAN cherry-pick buta, karena kode saat ini sudah berubah (tab Brief, agentEnabled, dark glass, menu bubble, tombol X).

---

## SUB-FASE & LANGKAH

### 3.1 Tools Gateway (Rust)
- Pulihkan `tools.rs`: registry 12 tool (ToolInfo: id, nama, kategori, connected, requires_auth, draft_only) + allowlist per agen
- Daftarkan commands ke main.rs: `list_tools`, `get_tool_status`, `set_tool_connected`, `get_agent_tools`, `set_agent_tools`, `agent_can_use_tool`
- Hubungkan ke ToolsView (tab Terpasang/Jelajahi yang sudah ada) — status koneksi nyata menggantikan status lokal
- **Uji**: cargo check nol error; status tool tampil di UI
- **Cek konsistensi**: IPC bertambah → verifikasi pasangan frontend↔backend

### 3.2 Approval Gate + Notifikasi Native
- Pulihkan dari `4ddc817`: permission `notification` di capabilities + pemanggilan notifikasi saat ada item Butuh Approval
- Alur: aksi tool yang sensitif → kartu di kolom Kanban "Butuh Approval" → user Setuju/Revisi/Tolak → notifikasi native terkirim
- **Uji**: buat 1 kartu approval dummy dari chat → notifikasi muncul → keputusan tercatat

### 3.3 Meta & Postiz — Draft-Only
- Pulihkan dari `186d9ae`: `createSocialDraft()` (frontend) — hasil kerja agen disimpan sebagai DRAFT, tidak pernah auto-post
- Postiz: penjadwalan sebagai draft
- Aturan keras (revisi 19 Okt): direct post HANYA atas perintah eksplisit user ("langsung post") — agen tidak boleh memutuskan sendiri
- **Uji**: 1 konten dibuat agen → muncul sebagai draft di Library → TIDAK terposting

### 3.4 Pireel — Lapisan Editing Video
- Pulihkan framework dari `186d9ae`: koneksi cloud MCP + OAuth (akun Free, 20 kredit/hari)
- Posisi di pipeline: generate (Veo/OpenCreator) → Pireel (edit/rakit) → QC → approval
- Hanya transkrip/analisis/generasi media yang memotong kredit — hemat
- **Uji**: koneksi OAuth tersambung + status tampil; tanpa memotong kredit

### 3.5 Google Drive — Deliverable ke Folder MIMI
- Pulihkan framework dari `28674c5`: upload hasil kerja agen ke Google Drive folder MIMI
- Kredensial OAuth user sendiri (aplikasi publik) — tidak dibundel
- **Uji**: 1 file hasil kerja ter-upload ke folder MIMI dan tercatat di Library

### 3.6 Integrasi Runtime (penghubung)
- Hubungkan gateway ke chat: agen memanggil tool sesuai allowlist-nya saat bekerja
- Hasil tool → Library; progres → Kanban (Antrian → Dikerjakan → Butuh Approval → Selesai)
- Sambungkan dengan tab Brief: todo hasil inisiasi project menjadi tugas tool nyata
- **Uji**: 1 alur penuh dari brief → delegasi → tool jalan → hasil di Library

---

## GATE TAHAP 3 (kriteria lolos)
1. Pipeline E2E: brief → Mimi delegasi → agen pakai tool → approval gate bekerja → hasil tersimpan sebagai draft / ter-upload ke MIMI → tampil di Library & Kanban
2. Tidak ada auto-post (draft-only terbukti)
3. Cek konsistensi + performa akhir fase lolos (IPC cocok, build bersih, CPU/memori sehat)
4. **Vonis Patria**

## RISIKO & CATATAN
- **MeHarness berlaku** (framework agen §7): permukaan tuning Tahap 3 (tool guidance/allowlist, guardrails approval gate, skills) mengikuti `docs/meharness.md`. Perubahan konfigurasi agen hasil loop MeHarness tetap lewat gate vonis Patria.
- Kode lama ditulis sebelum fitur 20 Okt (Brief, agentEnabled, glass) → adaptasi wajib, jangan timpa file utuh
- Rollback 19 Okt dipicu state corrupt MacBook, BUKAN kode Tahap 3 — tapi tetap rebuild bertahap + monitor CPU tiap sub-fase (pelajaran bug blinking)
- Meta/Postiz butuh kredensial user — untuk uji cukup sampai draft lokal bila akun belum terhubung
- RouterKu belum aktif (gateway + saldo) — tidak blocking Tahap 3

## PERKIRAAN URUTAN KERJA
3.1 → 3.2 → 3.3 → 3.4 → 3.5 → 3.6 → Gate E2E. Satu sub-fase selesai + teruji sebelum lanjut (tidak bercabang).
