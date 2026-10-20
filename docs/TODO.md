# TODO — Minime v2

> Diperbarui 18 Okt 2026 malam. Gate tiap tahap = review + vonis Patria.

## Tahap 1 — UI/UX ✅ (gate LOLOS — vonis Patria 19 Okt ~10:12)
- [x] Verdict stack: Tauri 2 + Svelte 5
- [x] Scaffold proyek fresh (`app/`)
- [x] Layout 4 kolom ala OpenBot (sidebar | agen | chat+delivery | monitor)
- [x] 6 agen seed + badge avatar + status (kuning/hijau/merah)
- [x] 3 bahasa (ID default, EN, ZH)
- [x] Navigasi: Pengaturan, Penggunaan, Peralatan
- [x] Prinsip compact-but-powerful, pemula-friendly
- [x] Dokumen: arsitektur, framework agen, workflow
- [x] Topbar mobile ≤860px: 2 baris rapi
- [x] Badge Tools: tetap warna brand (vonis Patria)
- [ ] Avatar animasi: dibuat ulang oleh Patria sendiri (tidak blocking)

## Tahap 2 — Model router nyata ✅ (gate LOLOS — vonis Patria 20 Okt 11:59)
- [x] Fondasi Rust: trait provider + registry + error routing (adaptasi pola v1)
- [x] Secrets: OS Keychain (`keyring` + fallback `security` CLI di macOS)
- [x] Ollama: deteksi `localhost:11434`, list models, chat (default GRATIS)
- [x] OpenCode: CLI free tier via shell-out (fallback GRATIS)
- [x] HTTP OpenAI-compatible: Muse (`api.meta.ai`), Gemini, DeepSeek, RouterKu (base URL konfigurabel; RouterKu blocked: gateway unknown + saldo 0)
- [x] Router per-agen: Gratis Ollama→OpenCode; Berbayar Muse→Gemini→DeepSeek; fallback otomatis
- [x] Tauri commands: `provider_probe`, `agent_chat`, `provider_connect/disconnect`, `routerku_set_base_url`
- [x] Frontend: Penyedia Model → status koneksi nyata + form API key (Keychain); chat agen → router nyata (fallback mock di browser)
- [x] Verifikasi `cargo check` di MacBook — LOLOS 19 Okt ~10:55 (nol error, nol warning)
- [x] Wiring SQLite: riwayat chat + usage metering
- [x] System prompt per agen + skills engine (playbook disuntik otomatis)
- [x] Frontend: riwayat dari SQLite; tab Cost pakai data usage nyata
- [x] Verifikasi `cargo check` putaran 2 di MacBook — LOLOS 19 Okt ~11:05 (nol error, nol warning)
- [ ] Runtime agen: chat → Mimi orkestrasi → eksekusi → output (delegasi multi-agen)
- [x] Gate: review + vonis Patria (LOLOS 20 Okt 11:59)

## Tahap 3 — Tools & distribusi (REBUILD — rencana: `docs/RENCANA-REBUILD-TAHAP-3.md`)
> Metode: pulihkan dari git history → adaptasi ke kode saat ini → uji satu per satu.
> MeHarness berlaku (framework §7). Tiap sub-fase ditutup cek konsistensi + performa.

### 3.1 Tools Gateway (Rust) — sumber: `b34e4f2`
- [ ] Pulihkan `app/src-tauri/src/tools.rs` (registry 12 tool + ToolInfo)
- [ ] Daftarkan 6 command di main.rs: `list_tools`, `get_tool_status`, `set_tool_connected`, `get_agent_tools`, `set_agent_tools`, `agent_can_use_tool`
- [ ] `cargo check` nol error/nol warning
- [ ] Hubungkan ToolsView ke status koneksi nyata (gantikan status lokal)
- [ ] Uji: status tool tampil di UI; toggle koneksi bekerja
- [ ] Cek konsistensi: IPC baru cocok frontend↔backend

### 3.2 Approval Gate + Notifikasi Native — sumber: `4ddc817`
- [ ] Pulihkan permission `notification` di capabilities
- [ ] Aksi tool sensitif → kartu di Kanban kolom "Butuh Approval"
- [ ] Keputusan user (Setuju/Revisi/Tolak) tercatat + notifikasi native terkirim
- [ ] Uji: 1 kartu approval dari chat → notifikasi muncul → keputusan tercatat

### 3.3 Meta & Postiz — Draft-Only — sumber: `186d9ae`
- [ ] Pulihkan `createSocialDraft()` — hasil agen = DRAFT, tidak pernah auto-post
- [ ] Postiz: penjadwalan sebagai draft
- [ ] Direct post HANYA atas perintah eksplisit user (aturan revisi 19 Okt)
- [ ] Uji: 1 konten agen → draft di Library → TIDAK terposting

### 3.4 Pireel — Editing Video — sumber: `186d9ae`
- [ ] Pulihkan framework koneksi cloud MCP + OAuth (akun Free, 20 kredit/hari)
- [ ] Pipeline: generate → Pireel edit → QC → approval; hemat kredit
- [ ] Uji: OAuth tersambung + status tampil (tanpa memotong kredit)

### 3.5 Google Drive — Folder MIMI — sumber: `28674c5`
- [ ] Pulihkan framework upload deliverable → Drive folder MIMI
- [ ] OAuth kredensial user sendiri (aplikasi publik, tidak dibundel)
- [ ] Uji: 1 file ter-upload ke MIMI + tercatat di Library

### 3.6 Integrasi Runtime
- [ ] Agen memanggil tool sesuai allowlist saat bekerja (dari chat)
- [ ] Hasil tool → Library; progres → Kanban
- [ ] Sambungkan tab Brief: todo hasil inisiasi = tugas tool nyata
- [ ] Uji: alur penuh brief → delegasi → tool → hasil di Library

### Gate Tahap 3
- [ ] Pipeline E2E teruji (brief → delegasi → tool → approval → draft/upload → Library/Kanban)
- [ ] Draft-only terbukti (tidak ada auto-post)
- [ ] Cek konsistensi + performa akhir fase lolos
- [ ] Vonis Patria

## Tahap 4 — Rilis macOS + Windows
- [ ] Build dmg + exe; installer mudah & intuitif
- [ ] Security audit + Impeccable audit
- [ ] Gate: instalasi bersih + vonis rilis Patria

## MeHarness — Adopsi Harness (20 Okt 2026)
> Framework lengkap: `docs/meharness.md`. Proses DEV, bukan fitur runtime.
> Status framework: resmi menjadi bagian framework agen — `docs/framework.md` §7 (20 Okt 12:38). Berlaku juga untuk rebuild Tahap 3.
- [x] Set tugas uji (benchmark mini) di `tests/harness/` — pilot Code Builder (3 tugas + verifikasi mesin); agen lain menyusul
- [x] `docs/meharness-lessons.md` — library pelajaran loop tuning (putaran #1 tercatat)
- [x] Putaran tuning pilot: Code Builder (Propose–Probe–Compose, k=1, executor OpenCode) — kandidat-umum diadopsi ke playbook
- [ ] Kurasi ECC gelombang 1 → format Minime (code-reviewer, build-error-resolver, e2e-runner, tdd-workflow, ai-regression-testing, agent-eval, agent-self-evaluation)
- [ ] Evaluasi konsep AgentShield (pindai konfigurasi agen) untuk Tahap 3/4

## Catatan
- Repo `PatriaAWijaya/minime-v2` (kosong): hapus manual via GitHub settings
  (token tidak punya scope delete).
- Avatar karakter v1 ditolak Patria — arah avatar baru menunggu keputusannya.
- Daemon v1 di MacBook: di-load ulang atas perintah Patria 19 Okt ~10:00 (`launchctl load`, PID 51270, berjalan)
  (belum terkonfirmasi).
