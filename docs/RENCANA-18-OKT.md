# Rencana Eksekusi — 18 Okt 2026 (HARI INI)

> Vonis Gate 4 (17 Okt malam): **LOLOS BERSYARAT** → Fase 5 jalan,
> backlog Gate 4 diperbaiki paralel.

## Urutan eksekusi yang disarankan

### 1. Sesi kunci API terpadu (MacBook Patria, via Keychain) — pembuka semua ✅ SELESAI
- [x] Gemini (prioritas #1 — membuka: Veo live test + OpenCreator)
- [x] Muse (key "Minime"; region Indonesia TERBUKTI bisa)
- [x] DeepSeek
- [x] Valyu (`valyu-finance` menunggu ini)
- [x] postiz (key hex hasil Rotate Key; bug base path/auth diperbaiki commit 357291c)
- [x] FRED (Keychain akun `fred`) + `pip install yfinance` v1.7.0

### 2. Instalasi tools baru (MacBook) ✅ SELESAI (18 Okt ~10:41)
- [x] xerj v1.0.0-rc.96: node jalan (127.0.0.1:8080/9200); autoindex ditunda
- [x] OpenViking 0.5.0: config HEMAT-RAM (embedding lokal 24MB + VLM gemini-2.0-flash + query planner 0.8B); server background, Connected (Healthy); tes memory OK
      (panduan diperbarui: `docs/instalasi-xerj-openviking.md`)
- [x] FRED API key gratis → Keychain akun `fred`
- [x] `pip install yfinance` v1.7.0

### 2b. Uji coba Pireel (MacBook) — ✅ SELESAI, vonis PERMANEN (18 Okt ~12:21)
- [x] Tahap 1: install + Studio jalan (`docs/uji-pireel.md`)
- [x] Tahap 2: koneksi agen via MCP (akun FREE cukup — kondisi STOP tidak terpicu)
- [x] Tahap 3: alur penuh (import → transcript cut → caption → export)
- [x] Tahap 4: billing check (2 kredit dari 20/hari, $0)
- [x] **Vonis: 7/7 PERMANEN** — skill dipromosikan permanen (`skills/pireel/SKILL.md` v1.0.0)

### 3. Backlog Gate 4 — live test (paralel Fase 5)
- [ ] Live test postiz via API nyata (butuh: postiz di `localhost:4007` + key)
- [ ] Live test provider Gemini / Muse / DeepSeek
- [ ] Uji 1 video OpenCreator via Gemini (butuh: instalasi OpenCreator + set provider Gemini di Settings → AI Services)
- [ ] Live test Meta `accounts` (butuh: system user token — **diparkir**, fondasi sudah dibuat)

### 4. Fase 5 — Hardening & rilis v1 pribadi
- [ ] QC judge penuh (rubrik skor 0–100)
- [ ] Metering + budget caps + view Penggunaan
- [ ] Providers settings UI
- [ ] Upload deliverable ke Drive folder MIMI
- [ ] Build dmg ad-hoc + setup LaunchAgent
- [ ] Tes MacBook: notifikasi `osascript`, buktikan quit app ≠ stop agents (daemon tetap jalan)
- [ ] Tes OAuth Google Drive (mode penuh)
- [ ] Security audit (skill `local_security_audit`)
- [ ] Audit Impeccable (UI)
- [ ] **Gate: v1 rilis di MacBook Patria** (vonis Patria)

### 5. Keputusan yang masih menunggu Patria
- [ ] Keputusan bisnis: harga, trial, fitur terkunci (mempengaruhi desain Account API Fase 6)
- [ ] Dokumen final Finance & Code Builder playbook
- [ ] Lanjut/tidak: system user token Meta (diparkir)

## Sudah selesai 17–18 Okt (tidak dikerjakan lagi)
- ✅ Vonis Gate 4: LOLOS BERSYARAT (tercatat di `docs/TODO.md` + `docs/status.md`)
- ✅ Push: remote HEAD `d1ff357` (18 Okt 10:42); tidak ada commit lokal tertunda
- ✅ Sesi kunci API 5/5 + FRED + yfinance (semua terverifikasi live)
- ✅ Instalasi xerj + OpenViking MacBook (config hemat-RAM)
- ✅ Identitas visual: app icon bubble pink + 5 avatar agen + 4 varian ekspresi Minime
- ✅ Skill `xerj` (Riset) + `openviking` (Mimi) — kode selesai & terdaftar
- ✅ Dry run instalasi xerj (v1.0.0-rc.96) + OpenViking (0.5.0) di VM
- ✅ Tool daemon `marketdata` (EDGAR/FRED/yfinance) + routing Valyu fallback premium
- ✅ Skill `pireel` (status UJI COBA) + `docs/uji-pireel.md` — siap diuji
- ✅ Audit konsistensi docs (2 kontradiksi diperbaiki)
- ✅ Keputusan: OpenCreator pakai Gemini; fal.ai dihapus total; Blotato → postiz;
      ArtCraft DITOLAK untuk saat ini; yorkeccak/finance clone diabaikan

## Catatan aturan yang tetap berlaku
- Tidak ada auto-posting Meta (draft-only) — aturan #3.
- Tidak ada order trading live otomatis — ditolak di kode.
- Kredensial via Keychain, tidak pernah di git / chat.
- Setiap fase di-review Patria sebelum lanjut (aturan #7).
