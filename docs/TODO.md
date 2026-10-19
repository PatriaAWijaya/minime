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

## Tahap 2 — Model router nyata (gate Tahap 1 lolos 19 Okt; berjalan)
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
- [ ] Gate: review + vonis Patria

## Tahap 3 — Tools & distribusi
- [ ] Meta (draft-only), Postiz/Viralin (draft-only), Pireel
- [ ] Approval gate + notifikasi native
- [ ] Upload deliverable → folder MIMI (Google Drive)
- [ ] Gate: 1 pipeline end-to-end + vonis Patria

## Tahap 4 — Rilis macOS + Windows
- [ ] Build dmg + exe; installer mudah & intuitif
- [ ] Security audit + Impeccable audit
- [ ] **Scoring kinerja pasca-kompilasi:** benchmark suite internal
      (startup, RAM/CPU, bundle, latensi) + Lighthouse CI + audit
      konsultan independen (pihak ketiga kompeten)
- [ ] Gate: instalasi bersih + skor kinerja lolos + vonis rilis Patria

## Catatan
- Repo `PatriaAWijaya/minime-v2` (kosong): hapus manual via GitHub settings
  (token tidak punya scope delete).
- Avatar karakter v1 ditolak Patria — arah avatar baru menunggu keputusannya.
- Daemon v1 di MacBook: di-load ulang atas perintah Patria 19 Okt ~10:00 (`launchctl load`, PID 51270, berjalan)
  (belum terkonfirmasi).
