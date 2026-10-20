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

## Tahap 3 — Tools & distribusi
- [ ] Meta (draft-only), Postiz/Viralin (draft-only), Pireel
- [ ] Approval gate + notifikasi native
- [ ] Upload deliverable → folder MIMI (Google Drive)
- [ ] Gate: 1 pipeline end-to-end + vonis Patria
- Rencana rebuild rinci: `docs/RENCANA-REBUILD-TAHAP-3.md` (disiapkan 20 Okt 12:31) — sumber kode lama: commit b34e4f2, 4ddc817, 186d9ae, 28674c5

## Tahap 4 — Rilis macOS + Windows
- [ ] Build dmg + exe; installer mudah & intuitif
- [ ] Security audit + Impeccable audit
- [ ] Gate: instalasi bersih + vonis rilis Patria

## MeHarness — Adopsi Harness (20 Okt 2026)
> Framework lengkap: `docs/meharness.md`. Proses DEV, bukan fitur runtime.
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
