# Rencana Eksekusi — 18 Okt 2026

> Rekap tugas keseluruhan untuk dieksekusi besok.
> Vonis Gate 4 (17 Okt malam): **LOLOS BERSYARAT** → Fase 5 jalan,
> backlog Gate 4 diperbaiki paralel.

## Urutan eksekusi yang disarankan

### 1. Sesi kunci API terpadu (MacBook Patria, via Keychain) — pembuka semua
- [ ] Gemini (prioritas #1 — membuka: Veo live test + OpenCreator)
- [ ] Muse
- [ ] DeepSeek
- [ ] Valyu (`valyu-finance` menunggu ini)
- [ ] postiz (`pos_...` — untuk live test tool `postiz`)

### 2. Instalasi tools baru (MacBook)
- [ ] xerj: `curl -fsSL https://xerj.org/get | sh` → start node → autoindex awal
- [ ] OpenViking: `uv tool install openviking` (atau `pip install openviking`) → start server

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
- ✅ Semua commit ter-push (remote HEAD `e065d75`, working tree bersih)
- ✅ Identitas visual: app icon bubble pink + 5 avatar agen (Mimi pink, Riset biru+kacamata+jas lab, Strategi Bisnis abu+tuksedo, Creative oranye+jeans, QC ungu+kemeja) + 4 varian ekspresi Minime
- ✅ Skill `xerj` (Riset) + `openviking` (Mimi) — kode selesai & terdaftar; instalasi binary menunggu MacBook
- ✅ Audit konsistensi docs (2 kontradiksi diperbaiki)
- ✅ Keputusan: OpenCreator pakai Gemini; fal.ai dihapus total; Blotato → postiz

## Catatan aturan yang tetap berlaku
- Tidak ada auto-posting Meta (draft-only) — aturan #3.
- Tidak ada order trading live otomatis — ditolak di kode.
- Kredensial via Keychain, tidak pernah di git / chat.
- Setiap fase di-review Patria sebelum lanjut (aturan #7).
