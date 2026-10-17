# status.md — Minime

> Log kronologis proyek. Entri baru di-APPEND di bawah dengan header tanggal.
> JANGAN edit entri lama (pelajaran 14 Okt 2026: pakai anchor akhir-entri yang unik).

---

## 2026-10-16 — Kickoff & arsitektur (brainstorm selesai, menunggu review Patria)

### Latar
- 16 Okt ~16:15 WIB: Patria perintahkan **reset total PENDOPO** — semua file di-trash, 7 cron dihapus, dashboard artifact + goal dihapus permanen. Kredensial `custom.gemini` di Secure Vault TIDAK dihapus (bisa dipakai ulang).
- 16 Okt ~19:05 WIB: diputuskan bangun **agen house** dengan dashboard realtime mengikuti diagram alur Patria.
- 16 Okt ~19:05–19:24 WIB: serangkaian keputusan (di bawah) → proyek dinamai **Minime**.

### Keputusan final (tidak diubah tanpa Patria)
1. **Target akhir**: aplikasi desktop seperti `nightly-labs/openbot` — local-first workspace untuk AI teammates persisten. **Bangun sendiri terinspirasi arsitekturnya (clean-room), BUKAN fork** — OpenBot berlisensi PolyForm Noncommercial.
2. **Nama proyek**: Minime.
3. **Platform**: desktop app langsung — **Tauri 2.x** (Rust backend + webview native). Target v1: dmg macOS untuk MacBook Patria.
4. **Alur kerja** (diagram Patria): Mimi (orkestrator) → Riset → Strategi Bisnis → 6 Playbook (Branding, Marketing, Selling, Operational, Finance, Code Builder) → Creative → 4 Playbook (Storytelling, Copywriting, Content, SMO) → Content Production (Image, Carousel, Reels) → Tools (Canva, Affinity, OpenCreator, VoiceStudio) → QC → Approval (Patria, manusia, wajib) → Deliverable Output. Cabang: Code Builder → UI/UX → QC; SMO → Page Optimization/Content Pilar/Planning/Distribution/Engagement → QC; feedback loop Deliverable → distribusi.
5. **LLM**: Gemini API (orkestrasi/riset/konten) + DeepSeek API (strategi/code). Kunci milik Patria, simpan di **macOS Keychain** — bukan di git, bukan plaintext.
6. **Ikon aplikasi**: robot biru dari upload Patria → `assets/app-icon.jpg` (220×220 JPEG).
7. **Avatar agen**: 15 ikon 3D (flat vector ditolak Patria 16 Okt) → `assets/avatars/`.
8. **Aturan keras**: kredensial tidak di git; TIDAK ADA auto-posting ke Meta (distribusi = draft sampai Patria approve manual); deliverable ke Drive folder MIMI; data sensitif tidak diupload; UI ikut `ui-rules-for-real-users.md` + lolos anti-pattern Impeccable; review gate tiap fase.
9. **Bahasa**: Indonesia untuk UI & dokumen user-facing; Inggris untuk kode & komentar.

### Artefak desain (16 Okt, tahap brainstorm)
- `AGENTS.md` — dibuat via `PROMPT-AGENTS.md` oleh subagent; **sudah diserahkan ke Patria untuk review** (status: menunggu vonis).
- `docs/status.md` (file ini), `docs/rule.md`, `docs/arsitektur.md` — ditulis subagent brainstorm 16 Okt ~19:25 WIB.
- File referensi Impeccable (`~/workspace/kantor-ai-agent/referensi/impeccable-skill.md`) sempat ikut ke-trash saat reset — **sudah dipulihkan** dari upstream github.com/pbakaus/impeccable (Apache 2.0).

### Status saat ini
- [x] Analisis kebutuhan (model/agen/tools/koneksi) — 16 Okt
- [x] Keputusan scope: bangun sendiri, nama Minime, desktop app Tauri
- [x] Prompt generator AGENTS.md (`PROMPT-AGENTS.md`)
- [x] AGENTS.md v1 — menunggu review Patria
- [x] Aset: app-icon + 15 avatar agen 3D
- [x] Brainstorm arsitektur: `docs/arsitektur.md` + rekomendasi fase — **menunggu review Patria**
- [ ] TODO.md / PLAN.md — dibuat SETELAH arsitektur + fase disetujui Patria
- [ ] Fase 1 build — belum mulai

### Berikutnya (butuh vonis Patria)
1. Review `AGENTS.md` — setuju / revisi.
2. Review `docs/arsitektur.md` + usulan 5 fase (dengan refinement) — setuju / revisi.
3. Setelah keduanya disetujui → tulis TODO.md + PLAN.md → mulai Fase 1.

---

## 2026-10-16 — Rewrite arsitektur v2: daemon + UI (menunggu review Patria)

### Keputusan final baru (tidak diubah tanpa Patria)
1. **Prinsip agen senior** (vonis Patria ~19:33 WIB): sedikit agen setara senior manager > banyak agen junior. Roster: 5 agen senior (Mimi/Orkestrator, Riset, Strategi Bisnis, Creative Director, QC) + Approval = Patria (manusia). Playbook menjadi skills milik agen senior. Kontrak senior: playbook/SOP + toolbox + proof of work + quality bar + eskalasi bila ragu.
2. **Agen tetap bekerja walau aplikasi ditutup** (permintaan Patria ~19:33 WIB) → arsitektur DUA KOMPONEN: `minime-daemon` (background service Rust: pemilik tunggal SQLite, eksekutor task, LLM, tools, scheduler, notifikasi native macOS) + aplikasi Tauri (dashboard UI sebagai client tipis via Unix socket). Daemon via launchd LaunchAgent (RunAtLoad + KeepAlive): start saat login, hidup walau UI di-quit. Quit app ≠ stop agents. Hanya daemon yang menulis DB (single-writer).
3. Perintah Patria ~19:34 WIB: **"buat ulang arsitekturnya"** → `docs/arsitektur.md` ditulis ulang DARI NOL (v1 satu-proses batal).

### Perubahan dokumen
- `docs/arsitektur.md` — rewrite penuh: diagram dua komponen, detail daemon (lifecycle launchd, crash recovery), UI client, protokol IPC Unix socket, skema 7 tabel, kontrak senior, pipeline YAML, alur QC→Approval + notifikasi native, packaging dmg + setup LaunchAgent, 5 fase disesuaikan.
- `AGENTS.md` — §b tech stack dua komponen, DoD +2 item (daemon tetap jalan setelah quit; start saat login; notifikasi approval saat UI tutup), §c catatan eksekusi di daemon, §d referensi rule #16, §f struktur repo (`daemon/` + `src-tauri/` thin client), §g 5 fase versi daemon.
- `docs/rule.md` — aturan #16 (prinsip agen senior), #17 (quit app ≠ stop agents), #18 (single-writer DB).

### Status saat ini
- Menunggu review Patria: `AGENTS.md` + `docs/arsitektur.md` (v2).
- Setelah disetujui → TODO.md + PLAN.md → Fase 1.

## 2026-10-16 — Rewrite arsitektur v3: komersialisasi (menunggu review Patria)

### Keputusan final baru (tidak diubah tanpa Patria)
1. **Target komersil** (~19:40 WIB): Minime menjadi aplikasi komersil yang di-download & di-install seperti `nightly-labs/openbot`. Mimi mempelajari mekanisme login + verifikasi email OpenBot dari repo publiknya (OTP 8 karakter via email, hash-only di D1, API di Cloudflare Workers, SMTP port 465/TLS, rate limit, Stripe billing) — **pola diadopsi, implementasi ditulis sendiri** (bukan fork).
2. **Distribusi**: installer per OS dari GitHub Releases — dmg (macOS), exe/msi (Windows), AppImage (Linux). **Code signing wajib** untuk rilis publik: Apple Developer $99/thn + notarization (tanpa ini Gatekeeper memblokir), sertifikat Windows (SmartScreen). **Auto-updater** bawaan Tauri (artifact ter-sign). v1 fokus macOS arm64; v1 pribadi boleh ad-hoc signing.
3. **Sistem akun**: login **OTP 8 karakter via email** — di DB hanya hash + expiry (tidak pernah plaintext); rate limit per email/IP/kirim ulang; kirim via SMTP (465/TLS) + fallback webhook POST `{email, code, expiresAt}`; mode dev boleh expose kode di response, **dilarang di production**; cron pembersih kode/session kadaluarsa; session token per perangkat. **Account API terpusat** (rekomendasi: Cloudflare Workers + D1, repo & deploy sendiri).
4. **Lisensi & billing**: akun ↔ lisensi (`trial`/`pro`/`lifetime`); app cek lisensi tiap start (cache + offline grace 30 hari — local-first tidak boleh mati karena WiFi putus); trial expired → fitur berbayar dikunci (data tetap bisa dibaca). **Stripe**: plan bulanan/tahunan, Customer Portal untuk upgrade/cancel mandiri, webhooks (renewal/gagal bayar/cancel).
5. **Sistem provider model** (~19:45 WIB): abstraction layer di daemon (trait: `list_models`, `complete_stream`, estimasi token). **Gratis**: deteksi server lokal Ollama/LM Studio (gratis & privat) + tier freemium. **Berbayar**: user tempel API key sendiri → Keychain. **Default berbayar = Muse** (~19:46 WIB; Meta Model API `ai.meta.com`, ±$1,25/1M input, ±$4,25/1M output, $20 kredit gratis akun baru) — CATATAN: verifikasi region Indonesia saat implementasi; fallback Gemini/DeepSeek. Provider lain: Anthropic, OpenAI, Gemini, DeepSeek. **Routing per-agen + fallback chain** + **metering** (token & estimasi biaya, kolom di tabel `runs`) + **budget cap** per agen/hari. UI: Settings → Providers, view Penggunaan.

### Perubahan dokumen
- `docs/arsitektur.md` — **rewrite penuh v3** (v2 batal): overview 4 komponen (daemon, UI client, account API, release pipeline); detail provider system (trait, routing, fallback, metering, budget cap); skema D1 account API (accounts, email_challenges [hash-only], sessions, licenses, stripe_customers); alur OTP end-to-end; alur lisensi & Stripe webhook; packaging + signing + auto-update; **6 fase** (Fase 6 komersialisasi, boleh paralel setelah Fase 2); keputusan bisnis terbuka (model harga, lama trial, fitur terkunci, kuota LLM trial, region Indonesia Meta API).
- `AGENTS.md` — §b (+provider system, account API, release pipeline), DoD +3 item (dmg ter-sign; login OTP; lisensi enforced), §g 6 fase (+Fase 6).
- `docs/rule.md` — aturan #19 (API key user hanya di Keychain), #20 (OTP hash-only + rate limit), #21 (signing wajib rilis publik), #22 (secret backend via secret manager).

### Status saat ini
- Menunggu review Patria: `AGENTS.md` + `docs/arsitektur.md` (v3).
- Setelah disetujui → TODO.md + PLAN.md → Fase 1.

## 2026-10-16 — Fase 1 selesai dibangun (menunggu review gate Patria)

### Perintah
Patria 16 Okt ~19:56 WIB: **"Build arsitektur"** — mulai Fase 1. Scope tambahan: framework agen
(`agents/<id>/`) + workflow (`pipelines/`) konkret.

### Yang dibangun (subagent, 16 Okt)
- **Dokumen**: `docs/TODO.md` + `docs/PLAN.md` ditulis dari arsitektur v3 (6 fase).
- **Daemon Rust** (`daemon/`, crate `minime-daemon`): SQLite WAL 7 tabel + migrasi berversi
  (`001_init.sql`); crash recovery jujur (running → failed); IPC Unix socket JSON-lines
  (`daemon.status`, `agents.*`, `tasks.*`, `runs.*`, `approvals.list`, `deliverables.list`,
  `pipelines.list`, `pipelines.run`, `subscribe` event); FIFO executor per agen (Fase 1: eksekusi
  dummy simulasi progres ~4 detik); provider trait (interface saja); template plist launchd
  `com.minime.daemon` (RunAtLoad + KeepAlive).
- **Framework agen**: `agents/{mimi,riset,strategi-bisnis,creative-director,qc}/` — tiap agen punya
  `agent.yaml` (model + fallback chain, skills, tools allowlist, effort, budget cap) +
  `system-prompt.md` (Bahasa Indonesia: peran senior, SOP, quality bar, eskalasi, larangan auto-post).
  Agent loader membaca folder ini saat start → seed tabel `agents` (tambah agen = tambah folder).
- **Workflow**: `pipelines/minime-pipeline.yaml` (Mimi → Riset → Strategi Bisnis → Creative Director
  → QC → Approval → Deliverable) + `smo-subpipeline.yaml` + `codebuilder-subpipeline.yaml`.
  Pipeline runner: parse & validasi YAML (DAG, agen & skill terdaftar, sub-pipeline asiklik),
  eksekusi FIFO per stage, handoff via `messages`, fan-out/fan-in, terminal approval → record `approvals`.
- **Skills**: 18 stub `SKILL.md` → seed tabel `skills` (bukan hardcode).
- **UI skeleton** (`src/`, Svelte 5 — dikunci): dashboard 6 views (Agen, Antrean, Berjalan,
  Persetujuan, Deliverable, Pengaturan) Bahasa Indonesia; kartu 5 agen + status
  (kuning=aktif, hijau=idle, merah=butuh perhatian); menu konteks per-agen
  (Tasks, + New task, Pengaturan agen, Sembunyikan komputer — stub); UI rules diterapkan;
  adapter `daemon.ts` (Tauri invoke → fallback mock mode demo).
- **`src-tauri/`** thin client: proxy IPC Unix socket → Tauri commands (scaffold; full build di macOS).
- **Git**: repo diinisialisasi di `~/workspace/minime/`; commit `94f0e33` "feat: fase 1 fondasi Minime".
  Remote privat: TUNGGU konfirmasi Patria (dilarang buat tanpa izin).

### Bukti terverifikasi (Linux, 16 Okt)
- `cargo build` sukses (hanya warning dead-code wajar).
- Daemon start: migrasi OK, 5 agen + 18 skills ter-load, 3 pipeline tervalidasi.
- IPC: `tasks.enqueue` → progres 0→100 → done; `agents.list` benar (avatar, warna, skills).
- **Pipeline end-to-end**: `pipelines.run minime-pipeline` → 5 stage tereksekusi FIFO →
  9 handoff messages tercatat → run `waiting-approval` → 1 approval pending di stage qc →
  53 event realtime diterima subscriber (task-updated, agent-status, run-updated, approval-requested).
- `vite build` sukses (peringatan Svelte diperbaiki).
- **Bukti quit ≠ stop**: daemon berjalan sebagai proses OS mandiri (PID independen), tetap responsif
  setelah semua proses lain selesai (uptime terukur). LaunchAgent macOS: template plist siap;
  verifikasi di MacBook Patria saat gate.

### Blocker / batasan jujur
- Full `tauri build` (dmg) dan LaunchAgent **hanya bisa diuji di MacBook Patria** — butuh macOS.
- Eksekusi task Fase 1 = dummy (tanpa LLM); LLM provider di Fase 2.
- `approvals.decide`, `secrets.set`, `providers.*`, `usage.summary`, `license.*` belum diimplementasikan (sesuai fase).
- Branch pipeline (`when`) divalidasi tapi belum dievaluasi kondisinya (Fase 2).

### Untuk review gate Fase 1 (butuh Patria)
1. Cek dashboard skeleton (mode demo via `vite dev`, atau build Tauri di MacBook).
2. Vonis: lanjut Fase 2, atau ada revisi.
3. Konfirmasi: boleh buat remote GitHub privat untuk repo ini?

## 16 Okt 2026 — Fase 2 SELESAI: Orkestrasi inti + provider

### Yang dibangun
- **3 provider LLM nyata** (`daemon/src/providers_*.rs`): OpenCode free (via CLI v1.19.3 yang di-pin di `tools/bin/`, tanpa key), Gemini (REST native), Muse (Meta Model API, OpenAI-compatible). Kunci via `secrets.rs`: env `MINIME_GEMINI_KEY`/`MINIME_MUSE_KEY` → Keychain (`keyring`), tidak pernah dibaca balik.
- **Muse region check + fallback**: HTTP 403/451 atau pesan region → `ProviderError::RegionBlocked` → executor otomatis lanjut ke Gemini (sesuai arsitektur).
- **Routing per-agen + fallback chain**: `executor.rs` ditulis ulang — `run_llm` gantikan `run_dummy`. Tiap task jalan di model agennya, pindah ke fallback bila gagal (typed `ProviderError`: NoKey/RegionBlocked/Failed).
- **Handoff nyata**: output stage N → `runs.context.stages.<id>` → prompt stage N+1 (via `build_user_text`); pesan handoff di tabel `messages` berisi ringkasan output asli.
- **QC scoring**: `parse_qc_score` baca `SKOR: n` + `ALASAN:` dari output QC → `approvals.qc_score`; format diinstruksikan di `agents/qc/system-prompt.md`.
- **`approvals.decide`** (IPC): approve → run done + draft deliverable di `data_dir/deliverables/<run_id>.md`; reject → revisi (counter di `runs.context`, maks 3x) → resume pipeline dari stage produsen → QC baru → approval baru. 3x tolak → run `failed` + notifikasi "butuh perhatian".
- **Notifikasi native** (`notify.rs`): macOS via `osascript`, Linux via `notify-send`/stderr. Dipanggil saat approval diminta.
- **Metering**: token in/out + estimasi biaya USD per stage → kolom `runs`; budget-cap harian per agen (task di-pause bila tercapai).
- **UI**: `PersetujuanView.svelte` — tombol Setujui/Tolak aktif, badge skor QC (hijau ≥75/merah <75), field catatan wajib saat menolak; `approvalsDecide` di `daemon.ts` + command Tauri `approvals_decide`.
- **Mock provider** (env-gated `MINIME_MOCK_LLM=1`, bukan produksi): untuk verifikasi integrasi tanpa ketergantungan LLM eksternal.
- `agent.yaml` ke-5 agen: fallback chain `muse → gemini → opencode/muse-spark-1.3-contributor-free` (strategi-bisnis: primary muse).

### Bukti verifikasi (mock provider, seluruh stack nyata via IPC)
1. Pipeline `minime-pipeline` dijalankan via `pipelines.run` → 5 stage selesai berurutan (orkestrasi→riset→strategi→produksi→qc).
2. **Fallback terbukti**: panggilan mock pertama disengaja digagalkan → executor otomatis coba fallback → sukses (terlihat di log daemon).
3. Approval terbuat otomatis: `qc_score=80` ter-parse dari `SKOR: 80`, status `pending`.
4. `approvals.decide approve` → approval `approved`, run `done`, draft deliverable terbuat (`deliverables/<run_id>.md`).
5. `approvals.decide reject` → `revision: 1`, `resumed_from: produksi` → stage produksi + qc jalan ulang → approval baru `pending`.
6. `cargo build` ✅, `cargo clippy` ✅ (hanya warnings), `cargo fmt` ✅, `vite build` ✅.

### Blocker jujur (eksternal, bukan kode)
1. **OpenCode free tier tidak bisa dipakai saat ini**: backend `opencode.ai/zen/v1` konsisten menjawab `400 "Model is unavailable"` untuk SEMUA model free (big-pickle, muse-spark, glm, deepseek, nemotron) selama ±1 jam pengujian; CLI `opencode run` hang tanpa aktivitas network di environment ini (kemungkinan terkait proxy). Kode provider sudah benar (terverifikasi: CLI ter-spawn dengan argumen tepat, stdin null, timeout 10 menit bekerja). Perlu dicoba ulang di lain waktu / dari MacBook Patria.
2. **API key Gemini & Muse belum ada** (env kosong, keyring Linux tak tersedia) → live test kedua provider itu belum bisa; JANGAN minta key via chat (aturan).
3. Notifikasi `osascript` + `tauri build` (dmg) + LaunchAgent **hanya bisa diuji di MacBook Patria**.

### Untuk review gate Fase 2 (butuh Patria)
1. Vonis: lanjut Fase 3, atau ada revisi.
2. Putuskan: coba ulang OpenCode free dari MacBook Patria, atau sediakan API key (via Keychain, bukan chat) untuk live test Gemini/Muse.
3. Konfirmasi ulang: boleh buat remote GitHub privat untuk repo ini? (ditunda dari Fase 1)

## 16 Okt 2026 — Fase 3 SELESAI: Skills & knowledge

### Yang dibangun
- **10 skills playbook penuh** (`skills/*/SKILL.md` v1.0.0) — kontrak "agen senior" per skill:
  SOP/playbook, toolbox, quality bar, trigger eskalasi, proof of work.
  - Strategi Bisnis (6): branding-playbook, marketing-playbook, selling-playbook,
    operational-playbook, finance-playbook, code-builder-playbook.
  - Creative Director (4): storytelling, copywriting, content, smo-playbook.
- **Knowledge disuling dari 4 dokumen Drive** (dibaca read-only via connector, tidak diubah):
  `docs/fase3-knowledge/{branding,marketing,selling,operasional}.md` — inti framework, SOP,
  toolbox, quality bar, catatan eskalasi, dan "yang TIDAK ada di dokumen" (jujur).
- **Finance & Code Builder**: struktur skill lengkap + placeholder `[KNOWLEDGE PATRIA]` yang
  ditandai jelas — TIDAK dikarang isinya (menunggu dokumen final Patria).
- **Creative skills** (tanpa dokumen Drive): storytelling (Golden Circle — Sinek),
  copywriting (PAS + 7 pola kualitas dari 4 konten acuan IG), content (pilar + kalender +
  repurpose), smo-playbook (checklist 6 channel + ⛔ ATURAN KERAS: distribusi hanya draft,
  dilarang auto-posting). Catatan jujur: 13-item quality bar rinci hilang bersama reset
  PENDOPO — 7 pola intinya terdokumentasi; daftar penuh dimasukkan kembali bila acuan tersedia.
- **Provider DeepSeek** (`daemon/src/providers_deepseek.rs`): OpenAI-compatible
  (`api.deepseek.com/v1/chat/completions`), model deepseek-chat + deepseek-reasoner,
  key via Keychain/env `MINIME_DEEPSEEK_KEY`, tanpa key → NoKey → fallback.
  Masuk fallback chain strategi-bisnis (muse → gemini → deepseek → opencode).
- **OAuth Google Drive** (`daemon/src/drive_oauth.rs`): authorization code flow +
  loopback listener (127.0.0.1:port acak) + state anti-CSRF; scope `drive.readonly`;
  refresh token di Keychain (write-only, tidak pernah di-log/dikembalikan);
  auto-refresh access token. IPC baru: `drive.auth_url`, `drive.status`, `drive.disconnect`,
  `secrets.set` (write-only), `secrets.has` (boolean saja). Event baru `minime://drive-auth`.
  Panduan uji MacBook: `docs/drive-oauth.md` (buat OAuth client Desktop app → simpan
  client_id via secrets.set → buka URL → approve).
- **Executor: injeksi playbook stage-level** (`executor.rs::build_system_prompt`):
  bila task input menyebut `skills: [...]`, isi penuh SKILL.md (tanpa frontmatter,
  cap 6000 karakter/skill) di-inject ke system prompt di bawah header "PLAYBOOK AKTIF".
  Mock provider (test-only) meng-echo playbook aktif untuk bukti wiring.

### Bukti verifikasi (Linux, 16 Okt)
- `cargo build` ✅, `cargo fmt` ✅, `cargo clippy` ✅ (8 warnings dead-code pre-existing, tanpa error).
- 18/18 skills ter-load di daemon (sebelumnya 3 gagal karena `:` di frontmatter YAML — diperbaiki).
- **Uji wiring 10/10 TERBUKTI**: 1 task per skill via mock (`MINIME_MOCK_LLM=1`) dengan
  `input.skills=[<skill>]` → semua `done`, semua output mock memuat `PLAYBOOK AKTIF: <skill>`.
  Sampel tersimpan: `docs/fase3-samples/<skill>.md` (10 file).
- **DeepSeek wiring terbukti**: fallback chain dieksekusi muse→gemini→deepseek→opencode;
  deepseek tanpa key → NoKey → lanjut fallback (terlihat di log). Live call menunggu key.
- **OAuth URL terverifikasi via IPC**: `drive.auth_url` → URL accounts.google.com valid
  (client_id, scope drive.readonly, state 32-char, redirect 127.0.0.1/callback,
  access_type=offline); `drive.status` → `{"connected": false, "has_client_id": true}`;
  `drive.disconnect` idempotent ✅.
- **OpenCode free dicoba ulang 1x** (bonus E): masih hang — timeout 300 dtk, stdout kosong,
  hanya banner model di stderr. Backend free tier belum pulih di environment ini.
  Tidak dibakar waktu lebih lanjut (sesuai instruksi).

### Blocker jujur (eksternal, bukan kode)
1. OpenCode free tier masih tidak bisa dipakai (hang/timeout; sebelumnya 400 "Model is unavailable").
   → Coba dari MacBook Patria / lain waktu.
2. API key Gemini, Muse, DeepSeek belum ada → live test ketiga provider menunggu key
   (via Keychain di MacBook, bukan chat).
3. Flow browser OAuth penuh (approve di browser + callback) hanya bisa diuji di MacBook Patria.
4. Knowledge finance & code-builder menunggu dokumen final Patria (placeholder jelas, tanpa karangan).
5. 13-item quality bar rinci (4 konten acuan IG) hilang bersama reset PENDOPO —
   perlu dimasukkan kembali dari acuan bila Patria menghendaki.

### Untuk review gate Fase 3 (butuh Patria)
1. Review 10 SKILL.md + 4 knowledge + 10 sampel (`docs/fase3-samples/`) — vonis: lanjut Fase 4 / revisi.
2. Kirim dokumen final knowledge Finance & Code Builder (bila sudah ada).
3. Putuskan: sediakan API key (Gemini/Muse/DeepSeek) via Keychain untuk live test, atau lanjut dengan mock.
4. Konfirmasi ulang: boleh buat remote GitHub privat untuk repo ini? (ditunda dari Fase 1–2)

## 16 Okt 2026 ~22:47 WIB — RESOLUSI blocker OpenCode free tier ✅

Patria membuktikan langsung dari MacBook-nya:
`opencode run "Jawab singkat: apa ibu kota Indonesia?" --model opencode/muse-spark-1.3-contributor-free`
→ menjawab **"Nusantara."**

Kesimpulan: error `400 "Model is unavailable"` + hang yang dialami di VM ini (Fase 2 & retry Fase 3)
**murni masalah network/proxy di VM build, bukan backend OpenCode**. Provider gratis Minime
(OpenCode free, default Muse Spark 1.3 Free, tanpa API key) **aman di mesin target (MacBook Patria)**.
Tidak perlu retry panjang dari VM lagi; live test end-to-end via OpenCode free dijadwalkan di MacBook.

## 16 Okt 2026 ~23:00 WIB — Remote GitHub privat dibuat ✅

- Patria menyetujui: repo privat **`PatriaAWijaya/minime`** dibuat via API (`https://github.com/PatriaAWijaya/minime`).
- 5 commit ter-push via Git Data API (pola bootstrap: 1 commit `chore: bootstrap repo` via Contents API karena API menolak tulis di repo kosong → 5 commit lokal di atasnya, README bootstrap ditimpa commit #1).
- Tree remote IDENTIK dengan tree lokal ✅. Pasangan SHA lokal → remote: `94f0e33→ac4a0ab`, `d97aa0f→8fec735`, `838090c→62a9db1`, `5d17900→857376b`, `0b08853→6928922`.
- `git remote add origin` sudah dipasang lokal. Push berikutnya via skrip API (`push_rebased.py`), bukan `git push` HTTPS.
- Perbaikan kecil: path `dynamic_credentials.py` di `git_push.py`/`push_rebased.py` diperbaiki (lokasi skill template pindah).

## 16 Okt 2026 ~23:35 WIB — FASE 4 SELESAI ✅ (menunggu review gate Patria)

**A. Readiness check eksplisit tiap tool + fallback.** `daemon/src/tools.rs` — registry 7 tool,
`tools.status` (IPC) melaporkan `ready/state/detail/fallback` tiap tool; readiness di-log saat
startup daemon. Hasil di VM: canva ✅ ready (39 tools MCP), hyperframes ✅ ready,
instagram-cli ✅ ready (DRAFT-ONLY), browser ✅ ready; affinity ❌ `unreachable` (bridge di MacBook),
open-creator ❌ `not_installed` (sidecar belum ada), voice-studio ❌ `unreachable` (instalasi pending).
Setiap tool punya fallback terdokumentasi di SKILL.md-nya; tidak ada yang gagal diam-diam.

**B. Wiring tools ke daemon.** Modul baru `daemon/src/tools/`: `canva.rs` (CLI: status, design.create
async-poll, image.generate, design.export; read-only terverifikasi), `affinity.rs` (MCP bridge
127.0.0.1:6767 JSON-RPC passthrough), `opencreator.rs` (sidecar contract `--op --params-json`;
slot `secrets.set`), `voicestudio.rs` (REST + fallback otomatis `edge-tts`), `hyperframes.rs`
(render + render-reels + verifikasi ffprobe; TMPDIR diarahkan ke data_dir karena /tmp VM kecil —
pola dari skill local_hyperframes), `instagram.rs` (draft + account read-only; op publikasi
DITOLAK di kode via BLOCKED_OPS — rule #3 ditegakkan di level kode, bukan sekadar tidak diimplementasikan),
`browser.rs` (Chromium headless via CLI flags, profil terisolasi per sesi, semua aksi → `messages`
kind=`browser-action`). Task pipeline mendukung `tool:` (stage tool-driven, allowlist diperiksa);
`pipelines/runner.rs` validasi tool id terhadap KNOWN_TOOLS.

**C. Content Production.** `pipelines/reels-pipeline.yaml`: riset → strategi-bisnis → produksi
(creative-director + `tool: hyperframes/render-reels`) → qc → approval.

**D. Built-in browser.** IPC `browser.open/navigate/screenshot/dom/list/close`; terverifikasi:
screenshot + ekstraksi DOM + proof-of-work tercatat di DB. Muat halaman eksternal live TERBATAS di VM
(proxy — sama seperti kasus OpenCode; verifikasi penuh di MacBook). Sesi in-memory (jujur, tidak
pura-pura persisten); tidak ada auto-login (aturan).

**E. Bukti gate — 1 reels JADI end-to-end** ✅ (run `69930306`, MINIME_MOCK_LLM=1 + render nyata):
riset → strategi → **render MP4 nyata** (HyperFrames, 1080x1920, 6.0 detik, 3 slide) → QC skor 80 →
approval (approve via IPC = verifikasi teknis; vonis akhir di gate Patria) → done.
Artefak: `docs/bukti-fase4/reels-minime.mp4` (+ 2 frame PNG). Perbaikan selama verifikasi:
(1) `runner.rs`: approval kini mengisi `artifact_refs` dari artefak stage;
(2) `executor.rs`: `finish_task_done` merge (bukan timpa) objek stage — sebelumnya artefak hilang tertimpa output;
(3) temp render dibersihkan dari direktori artefak.

**UI Fase 4:** command Tauri `tools_status` + `browser_list`, `daemon.ts` (+mock), view baru
**Peralatan** (kartu readiness 7 tool + sesi browser), entri nav; `vite build` ✅.

**Dokumen:** 6 SKILL.md tool di-update stub→wired; baru: `docs/voice-studio.md` (langkah instalasi
MacBook), `docs/open-creator.md` (kontrak sidecar), `docs/browser.md` (operasi & batasan).

**Catatan jujur / menunggu MacBook & Patria:**
- Canva `design.create`/`export` terkode tapi belum live-test (menyentuh akun Canva Patria).
- Affinity bridge, VoiceStudio, OpenCreator: butuh MacBook Patria (+ kunci API untuk 2 terakhir).
- Browser live page-load: verifikasi penuh di MacBook (limitasi proxy VM).
- Verifikasi pipeline memakai mock LLM (sementara `provider: mock` di agent.yaml saat uji,
  sudah di-revert via `git checkout -- agents/`); stage tool (render) 100% nyata.
- `cargo build` ✅, `cargo fmt` ✅, clippy: hanya warning kosmetik pre-existing.

## 17 Okt 2026 — Skill `oil-ui` untuk code builder ✅

- Perintah Patria: repo `oil-oil/oil-ui` (MIT) dijadikan skill UI untuk code builder (saat butuh bikin aplikasi web-based).
- Skill `skills/oil-ui/SKILL.md` milik **strategi-bisnis** (pasangan `code-builder-playbook`): adaptasi workflow oil-ui ke Bahasa Indonesia — (0) pahami produk (kategori, intent layar utama, 2-3 benchmark), (1) tone → keputusan visual konkret (typeface, whitespace, warna hex, selaras `docs/rule.md` #6), (2) eksplorasi 2-3 arah berbeda → side-by-side → Patria memilih, (3) bangun + refine terhadap screenshot nyata via **built-in browser Minime**, (4) review independen bila ada, (5) subtractive pass → serah terima. Atribusi MIT + link repo asal dicantumkan.
- Wiring mock TERBUKTI: `PLAYBOOK AKTIF: oil-ui` (task strategi-bisnis via `MINIME_MOCK_LLM=1`).
- Bug ditemukan & diperbaiki saat uji: frontmatter awal tidak valid (deskripsi tak dikutip mengandung `: `) → loader melewatkan skill; setelah dikutip, terdaftar sebagai skill ke-20 dan ter-inject benar.
- Sampel: `docs/fase4-samples/oil-ui.md` (jujur: bukti wiring mock, bukan output beneran).
- Tindak lanjut: referensi asli oil-ui di-vendor ke `skills/oil-ui/referensi-asli/` (+ `ATRIBUSI.md`); update-check bawaan dinonaktifkan (aturan #12).

## 17 Okt 2026 — Skill produksi `html-anime` + bumper opening Minime NYATA ✅

- Perintah Patria: repo `blixvip/html-anime` (MIT) dijadikan skill produksi Minime.
- Skill `skills/html-anime/SKILL.md` milik **creative-director**: mengorkestrasi 5 sub-skill asli (di-vendor utuh di `skills/html-anime/sub-skills/`); SOP 6 langkah: (1) brief scene, (2) `compileBrief` → shot sheet, (3) agen menggambar frame HTML mengikuti 5 sub-skill, (4) `hyperframes lint` + `validate`, (5) render via tool HyperFrames, (6) QC → MP4. `harness/compile.js` di-vendor ke `tools/html-anime/` (+ `ATRIBUSI.md`, atribusi MIT).
- Wiring mock TERBUKTI: `PLAYBOOK AKTIF: html-anime` (21 skills ter-load di daemon).
- **Acceptance test Patria — UJI NYATA end-to-end, LULUS**: bumper opening Minime (`minimalist-blue-robot-awakens`) — brief → compileBrief → shot sheet → film HTML (robot biru minimalis, 3 shot: establish/sakuga/button, color-dip cuts, tagline Indonesia) → `hyperframes lint` 0 error → `validate` lolos → **render MP4 nyata** `docs/bukti-fase4/bumper-minime.mp4` (h264, 1920×1080, 8.0 detik) + 3 frame PNG.
- Deviasi jujur: (a) Patria minta ±4 detik, tapi `compileBrief` hanya mendukung 8/12/20 detik → dipakai 8 detik (minimum workflow); (b) GSAP di-vendor lokal (`gsap.min.js`) karena CDN jsdelivr diblokir proxy VM; (c) TMPDIR render diarahkan ke disk utama karena /tmp hanya 512M (tmpfs).
- Sampel wiring: `docs/fase4-samples/html-anime.md` (jujur: bukti wiring mock).

## 17 Okt 2026 — Tool + skill `manimgx` (video pembelajaran) ✅

- Perintah Patria: repo `academa-labs/manimgx` (MIT) sebagai alternatif produksi video pembelajaran.
- Tool `manimgx` di-wire di daemon (`daemon/src/tools/manimgx.rs`, op `render`/`inspect`): readiness eksplisit — cek binary (`uv tool install --python 3.13 manimgx`), versi, ffmpeg, GPU (/dev/dri); kredensial tidak diperlukan. `tools.status` → `ready=true` (manimgx 0.1.1).
- Skill `skills/manimgx/SKILL.md` milik **creative-director**: SOP tulis scene Python (API Manim CE, gaya 3Blue1Brown) → `inspect` storyboard → render draft → final → QC → deliverable.
- **Uji NYATA, LULUS**: `examples/circle_area.py` → `docs/bukti-fase4/manimgx-sample.mp4` (h264, 1280×720@30fps, 17.9 detik, render 68 detik) + frame PNG. Catatan: VM tanpa GPU → software render (lavapipe); verifikasi penuh GPU di MacBook Patria.
- Wiring mock TERBUKTI: `PLAYBOOK AKTIF: manimgx`; sampel `docs/fase4-samples/manimgx.md`.

## 17 Okt 2026 — Tool + skill `valyu-finance` (riset finansial) ✅ wiring / ⏳ menunggu API key

- Perintah Patria: repo `yorkeccak/finance` (MIT) sebagai finance skill — data institutional-grade via Valyu Search API.
- Tool `valyu` di-wire di daemon (`daemon/src/tools/valyu.rs`, op `search` → `POST https://api.valyu.ai/v1/search` header `x-api-key`); readiness eksplisit.
- Skill `skills/valyu-finance/SKILL.md` milik **strategi-bisnis** (pasangan `finance-playbook`): SOP query presisi → sintesis bersumber → QC → deliverable; quality bar + trigger eskalasi; **aturan keras: riset/analisis saja, DILARANG order trading live otomatis**.
- Status jujur: readiness `not_configured` — **menunggu VALYU_API_KEY dari Patria** (gratis $10 di platform.valyu.ai, via Keychain aturan #2/#19). Uji live menyusul setelah key tersedia.
- Wiring mock TERBUKTI: `PLAYBOOK AKTIF: valyu-finance`; sampel `docs/fase4-samples/valyu-finance.md`.

## 17 Okt 2026 — Gudang skill `skill-warehouse` (agency-agents, on-demand) ✅

- Perintah Patria: repo `msitarzewski/agency-agents` (MIT, 160k★) diintegrasikan sebagai **gudang skill** — "dipakai hanya jika dibutuhkan".
- Prinsip dijaga (rule #16): gudang BUKAN agen baru — roster tetap 5 agen + Approval. Menambah kemampuan = menambah modul, bukan agen.
- `skills-warehouse/INDEX.md`: katalog 267 spesialis dalam 18 kategori (dari frontmatter repo asal); referensi mentah di `~/workspace/referensi/agency-agents/` (di luar repo minime).
- 8 modul teradaptasi ke format Minime (Bahasa Indonesia + atribusi MIT + batasan Minime):
  `ui-designer`, `ux-researcher`, `frontend-developer`, `code-reviewer`,
  `reality-checker`, `research-synthesist`, `carousel-growth-engine` (dipaksa draft-only —
  perilaku auto-publish aslinya DINONAKTIFKAN per rule #3), `short-video-editing-coach`.
  Cara tambah modul: `skills-warehouse/CONTRIBUTING.md` (tandai ✅ di INDEX).
- Skill `skills/skill-warehouse/SKILL.md` milik **mimi** (skill pertama milik orkestrator):
  protokol on-demand — trigger (tugas butuh keahlian di luar skill inti), cara memuat
  (inject SOP+quality bar ke konteks task, mode berakhir saat task selesai),
  LARANGAN auto-load/preemptive, kewajiban atribusi MIT.
- `agents/mimi/agent.yaml`: `skills: [skill-warehouse]` (perubahan `provider: mock` saat uji sudah di-revert).
- Wiring mock TERBUKTI: `PLAYBOOK AKTIF: skill-warehouse` (task mimi via `MINIME_MOCK_LLM=1`);
  sampel `docs/fase4-samples/skill-warehouse.md` (jujur: bukti wiring mock).
- Pelajaran uji: playbook full-body hanya ter-inject bila task input memuat `"skills": [...]`
  (stage-level); daftar skill agen hanya jadi ringkasan.

## 17 Okt 2026 ~06:00 WIB — Finance skills quant/trading ✅

Perintah Patria: 4 sumber quant/trading dijadikan finance skills bila bisa dikoneksikan
(+ `yorkeccak/finance` sebagai skill `valyu-finance` — dikerjakan agen paralel, commit `b57f73a`).

### Tabel lisensi & status koneksi

| Sumber | Lisensi | Koneksi | Status uji |
|---|---|---|---|
| TauricResearch/TradingAgents (111k★) | Apache-2.0 ✅ | pip/CLI, env `TRADINGAGENTS_*`, LLM lokal via Ollama/OpenAI-compatible | Import + CLI TERBUKTI di VM |
| microsoft/qlib (49k★) | MIT ✅ | `pip install pyqlib`, `qrun`/Python API | Import + workflow TERBUKTI (v0.9.7) di VM |
| nautechsystems/nautilus_trader (30k★) | LGPL-3.0 ⚠️ | **Proses eksternal saja** (subprocess), bukan link | `BacktestEngine` TERBUKTI di VM (v1.231.0) |
| OpenBB (SDK PyPI) | MIT ✅ | `pip install openbb`, `obb.equity.*` | Install dalam proses; pola SDK terdokumentasi |
| yorkeccak/finance → Valyu API | MIT (pola) ✅ | HTTP `api.valyu.ai`, key via Keychain | Tool ter-wire; live test menunggu key Patria |

Detail lisensi: `docs/lisensi-pihak-ketiga.md`.

### Yang dibangun
- `skills/finance-playbook/SKILL.md` — diisi penuh (sebelumnya placeholder):
  sisi A keuangan bisnis (P&L, unit economics, cash flow, 3 skenario) +
  sisi B toolbox quant (tabel routing 4 framework + Valyu).
  [KNOWLEDGE PATRIA] tetap placeholder — menunggu dokumen final, tanpa karangan.
- 4 skill baru (owner: strategi-bisnis): `trading-agents`, `qlib`, `nautilus`, `openbb`
  — format SKILL.md Minime + larangan keras di setiap file.
- Tool daemon `quant` (`daemon/src/tools/quant.rs`): ops `ta_analyze`,
  `qlib_backtest`, `nautilus_backtest`, `openbb_data` + readiness check per paket.
- **Penolakan live-trading di kode**: `is_live_op()` menolak op bernama
  live/order/execute/trade/buy/sell; skrip Python di-scan untuk pola
  `place_order`/`submit_order`/adapter live (`binance`, `ibapi`, dll).

### Aturan keras (di setiap SKILL.md + tool)
Riset, analisis, backtest, paper-trading (venue SIM) SAJA.
**DILARANG order trading live otomatis dalam bentuk apa pun.**
Order live HANYA dengan approval eksplisit Patria PER AKSI.

### Catatan jujur
- Commit `b57f73a` (agen paralel) sempat broken (tools.rs mereferensikan `mod quant`
  sebelum `quant.rs` ada) — diperbaiki oleh commit ini.
- Live test butuh: API key Valyu (Keychain), LLM provider untuk TradingAgents,
  dataset qlib (official disabled — pakai sumber komunitas).
- Verifikasi penuh GPU/venue live (tidak akan di-wire) di MacBook Patria.

---

## 17 Okt 2026 — Pipeline konten video otomatis (adaptasi diagram n8n Patria, DRAFT-ONLY)

Perintah Patria: integrasikan pipeline otomasi video AI (diagram n8n 3-step) ke alur kerja tim konten Minime.

**Yang dibangun:**
- `pipelines/konten-video-otomatis.yaml` — 4 stage: `skrip` (creative-director: konsep + prompt Veo3 + caption) → `render` (tool `veo3`) → `qc` (skor 0–100) → `distribusi` (tool `blotato` op `draft`) → **approval Patria** (run berhenti di `waiting-approval`; tidak ada stage setelah approval karena `approve_run` mengakhiri run).
- Tool `veo3` (`daemon/src/tools/veo3.rs`): fal.ai queue API (submit → poll ±10 mnt → unduh MP4 ke artifact dir). Key via Keychain akun `fal` / `MINIME_FAL_KEY`. Tanpa key → **fallback jujur** HyperFrames lokal (dicatat di log). Mock mode → hasil mock eksplisit.
- Tool `blotato` (`daemon/src/tools/blotato.rs`): op `draft` (upload media bila key ada + bundle draft lokal per platform, TIDAK PERNAH publish), op `publish` (Meta: DITOLAK di kode — rule #3; 6 platform lain: wajib `approval_id` berstatus `approved`), op `accounts` (read-only). Key via Keychain akun `blotato` / `MINIME_BLOTATO_KEY`.
- Konvensi antar-stage: `PROMPT_VEO3:` / `CAPTION:` di output stage skrip (tool stage tidak bisa baca output LLM langsung).
- `smo-playbook` ditambah SOP §E (distribusi video via Blotato, 9 platform, draft-only).
- Perbaikan kecil `runner.rs`: `qc_score_from_any_stage` — approval yang stage-nya bukan QC tetap mencatat skor QC dari stage sebelumnya.

**Penyesuaian wajib dari diagram asli:** diagram mempublish otomatis ke 9 platform — di Minime SEMUA platform default draft + approval gate; IG/FB/Threads tidak bisa dipublish dari kode dalam kondisi apa pun (konsisten dengan `instagram-cli` Fase 4).

**Uji wiring mock** (`MINIME_MOCK_LLM=1`, daemon terisolasi): 4 stage → `waiting-approval`, approval `pending` dengan `qc_score=80`, 9 bundle draft lokal terbuat. Guard terverifikasi: publish IG ditolak; publish YT tanpa approval ditolak; `tools.status` jujur `not_configured` untuk kedua tool. Sampel: `docs/fase4-samples/konten-video-otomatis.md`.

**Belum teruji (butuh Patria):** render Veo3 asli (key fal.ai), upload/publish Blotato (key Blotato), fallback HyperFrames render nyata.

---

## 17 Okt 2026 — Render video primer pindah ke Gemini API (Veo 3.1)

Perintah Patria ("ganti ke gemini"): tool `veo3` tidak lagi primer fal.ai.
Rantai backend baru: **Gemini API (primer) → fal.ai (fallback) → HyperFrames lokal**.

**Endpoint terverifikasi** dari docs resmi Google AI for Developers
(`ai.google.dev/gemini-api/docs/veo`):
- `POST {BASE}/models/{model}:predictLongRunning` (header `x-goog-api-key`,
  body `{"instances":[{"prompt"}],"parameters":{aspectRatio, durationSeconds,
  resolution, personGeneration}}`) → `{"name":"operations/..."}` →
  `GET {BASE}/{name}` → `.done` →
  `.response.generateVideoResponse.generatedSamples[0].video.uri` → unduh MP4.
- Model default: `veo-3.1-fast-generate-preview` (1080p/8 dtk/16:9).
  ID `-001` adalah Vertex AI — Gemini API memakai `-preview`.
- Key: Keychain akun `gemini` / env `MINIME_GEMINI_KEY` (pola sama dengan
  provider Gemini Fase 2 — tidak ada key baru).

**Perilaku:** params `backend` = auto (default) | gemini | fal | local.
auto: gemini (bila key ada) → fal.ai (bila key ada) → HyperFrames lokal.
Tiap percobaan dicatat di `attempts`; kegagalan total memuat seluruh attempts
(tidak ada kegagalan diam-diam). `est_cost_usd` di hasil render = PERKIRAAN
kasar dari harga publik Google.

**Perkiraan biaya** (harga publik Google, BUKAN janji/tagihan aktual):
klip 8 dtk 1080p tier Fast ≈ $0,12/dtk → **≈ $0,96/klip**.

**Keputusan arsitektur Patria 17 Okt:** Gemini Veo dan OpenCreator dipasang
**paralel**, pemakaian tergantung kebutuhan — keduanya tetap ter-wire, tidak
ada yang dicopot. Panduan routing ditambahkan sebagai §E di
`skills/content/SKILL.md` (Veo: footage generatif sinematik/b-roll AI;
OpenCreator: naskah → video + voiceover Indonesia, menunggu kunci;
HyperFrames/manimgx: fallback lokal tanpa kunci).

**Uji:** wiring mock LULUS (`backend_chain: [gemini, fal, hyperframes]`);
uji rantai nyata tanpa key LULUS (gemini skipped → fal skipped → hyperframes
dicoba; attempts jujur). Guard `blotato.publish` IG tetap DITOLAK (tidak
tersentuh). Bug panic `fallback_hyperframes` untuk prompt pendek ditemukan
& diperbaiki saat uji. `cargo build` + `clippy` + `cargo fmt` bersih.

**Belum teruji:** render Veo 3.1 asli (menunggu API key Gemini Patria via Keychain).

---

## 17 Okt 2026 — Koneksi Meta langsung untuk distribusi (hybrid, DRAFT-ONLY)

Perintah Patria ("buat koneksi ke meta"): gantikan Blotato untuk 3 platform Meta
dengan koneksi langsung ke API resmi Meta. Keputusan arsitektur: **hybrid** —
Meta langsung untuk Instagram/Facebook/Threads, Blotato tetap untuk 6 platform
non-Meta (YouTube, TikTok, Twitter/X, LinkedIn, Bluesky, Pinterest).

**Yang dibangun:**
- Tool daemon baru `meta` (`daemon/src/tools/meta.rs`), terdaftar di `tools.rs`
  (`TOOL_META`, KNOWN_TOOLS, readiness, dispatch):
  - op `draft` — bundle draft lokal per platform (`drafts/meta-draft-<platform>-<ts>.json`:
    caption + media + hashtag + panduan posting manual). Konvensi pipeline:
    `media`/`video_path` kosong → artefak MP4 stage render; `caption` kosong →
    baris CAPTION: stage skrip. TIDAK PERNAH memanggil endpoint publish.
  - op `accounts` — read-only: `GET /me/accounts` (FB Pages),
    `/{page-id}?fields=instagram_business_account` (IG Bisnis),
    `GET graph.threads.com/v1.0/me` (Threads). Tidak membuat/mengubah apa pun.
  - Guard keras: op `publish`/`post`/`upload`/`share`/`send`/`media_publish`/
    `threads_publish`/`feed`/`reel` DITOLAK di kode (rule #3).
  - Kredensial: Keychain akun `meta` / env `MINIME_META_TOKEN`.
- Endpoint terverifikasi dari dokumentasi resmi Meta (Okt 2026), didokumentasikan
  di header `meta.rs`:
  - IG Graph API: `POST /{ig-user-id}/media` (container `media_type=REELS`) →
    poll `status_code` → `POST /{ig-user-id}/media_publish`; permission
    `instagram_business_basic` + `instagram_business_content_publish`; hanya
    akun Bisnis/Kreator; limit 100 post/24 jam.
  - FB Pages API: `GET /{user-id}/accounts` → `POST /{page-id}/feed|photos|videos|video_reels`;
    permission `pages_show_list` + `pages_manage_posts`.
  - Threads API: `POST /{threads-user-id}/threads` → tunggu ±30 dtk →
    `POST /{threads-user-id}/threads_publish`; permission `threads_basic` +
    `threads_content_publish`; media harus di-host publik.
- Pipeline `konten-video-otomatis` v3: stage `distribusi` dipecah dua —
  `distribusi-meta` (tool `meta`: instagram/facebook/threads) →
  `distribusi-nometa` (tool `blotato`: 6 non-Meta) → approval Patria.
- `skills/smo-playbook/SKILL.md` §E ditulis ulang jadi SOP distribusi hybrid
  (E.1 Meta langsung, E.2 Blotato non-Meta).
- `docs/koneksi-meta.md` (baru): panduan langkah-untuk-Patria — buat Meta App di
  developers.facebook.com, permission yang diminta, App Review (tidak perlu
  selama pemakai = Patria sendiri sebagai tester), syarat akun (IG Bisnis/Kreator,
  Threads publik), cara simpan token ke Keychain.

**Uji wiring mock** (`MINIME_MOCK_LLM=1`, daemon terisolasi): 5/5 PASS —
`tools.status` memuat `meta` (`not_configured`, jujur); `draft` 3 platform →
3 bundle lokal; `publish` DITOLAK; `post` DITOLAK; `accounts` mock OK.
Sampel: `docs/fase4-samples/konten-meta-langsung.md`. `cargo build` + `clippy`
bersih (warning doc-style pre-existing, sama seperti tool lain).

**Belum teruji (butuh Patria):** token Meta (ikuti `docs/koneksi-meta.md`),
lalu live test `accounts` untuk verifikasi FB Page + IG Bisnis + Threads terbaca.

---

## 17 Okt 2026 — fal.ai DIHAPUS TOTAL; Canva Video tidak bisa jadi pengganti (terbukti)

Perintah Patria: hapus fal.ai, ganti dengan Canva Video sebagai fallback.

**Hasil verifikasi jujur (3x test live via `canva` CLI di VM):**
- `create-design` dengan brief video → 2x GAGAL ("The design could not be generated from this brief").
- `generate-design` → enum `design_type` TIDAK memuat tipe video apa pun (hanya: business_card, card, poster, presentation, youtube_thumbnail, dst — tidak ada video/reel).
- **Kesimpulan: Canva MCP API tidak bisa generate video dari prompt.** Canva tetap berguna untuk thumbnail/cover (image.generate) dan export desain video yang dibuat manual di Canva UI (export-design → mp4), tetapi BUKAN pengganti fal.ai untuk render video AI otomatis.

**Yang dilakukan:** backend fal.ai dihapus total dari `daemon/src/tools/veo3.rs` (fungsi `render_fal`, konstanta, key, dan rantai dihapus; `cargo build` bersih). Rantai akhir: **Gemini API (Veo 3.1) → HyperFrames lokal**. `params.backend`: auto|gemini|local. Pipeline naik ke v3; docs/TODO/status/sampel/content-skill diselaraskan.

---

## 17 Okt 2026 — Blotato DIGANTI postiz (perintah Patria)

Perintah Patria: "ganti blotato dengan postiz".

**Yang dilakukan:**
- Tool daemon `postiz` baru (`daemon/src/tools/postiz.rs`): op `draft` (DRAFT-ONLY, `POST /public/v1/posts` type `draft`), `schedule` (WAJIB `approval_id` approved — tanpa itu DITOLAK), `accounts` + `list` (read-only). Platform Meta (instagram/facebook/threads) DITOLAK di tool ini — pakai tool `meta`.
- `daemon/src/tools/blotato.rs` DIHAPUS total (kode lama pulih via git history). Registry `tools.rs`, allowlist `agents/creative-director/agent.yaml`, `skills/smo-playbook/SKILL.md` §E.2, pipeline `konten-video-otomatis` v4 (stage `distribusi-nometa` → tool `postiz`) diselaraskan.
- postiz-app (AGPL-3.0) = layanan EKSTERNAL di MacBook Patria (`http://localhost:4007`); daemon hanya panggil Public API via Bearer `pos_...` (Keychain akun `postiz` / env `MINIME_POSTIZ_KEY`). Endpoint terverifikasi dari kode postiz (`/tmp/eval-blotato-alt/postiz`): `GET /is-connected`, `GET /integrations`, `POST /posts`, `GET /posts`.
- Wiring mock TERBUKTI (lihat `docs/fase4-samples/konten-postiz.md`).

**Belum teruji (butuh MacBook Patria):** kirim draft nyata via API (butuh 1 channel terhubung + API key `pos_...` di Keychain).

---

## 17 Okt 2026 (sore) — Uji Instagram via postiz BERHASIL; token meta diparkir; OpenCreator → Gemini

**Uji Instagram end-to-end (terbimbing tahap demi tahap, BERHASIL ~17:12 WIB):**
- Meta App "Minime" dibuat (ID 4452783818294531, tipe Bisnis) + produk Facebook Login for Business
- 2 Valid OAuth Redirect URIs tersimpan (`/integrations/social/facebook`, `/integrations/social/instagram`)
- FACEBOOK_APP_ID/SECRET ditempel ke .env postiz + restart
- Channel IG "Branding, Marke..." tersambung via OAuth di dashboard postiz
- 1 DRAFT ("uji coba dari Minime - abaikan") tersimpan via Create Post — **tanpa publish**
- Rantai Meta App → OAuth → channel → draft TERBUKTI jalan

**Token tool `meta` (draft-only) — DIAGNOSA TUNTAS, DIPARKIR atas pilihan Patria ~18:55:**
- 5 permission di-generate via Graph API Explorer (pages_show_list, pages_read_engagement, instagram_basic, instagram_manage_insights, instagram_manage_contents)
- Hasil: `me/accounts` → kosong; query Page by ID → error 100/33, MESKI sudah "Setujui semua Halaman" via Edit pengaturan
- Penyebab: token user dari Facebook Login for Business bersifat business-scoped — tidak mengembalikan Page via `/me/accounts`
- Jalur benar: System User token (tidak expire). System user "Minime" (Admin, ID 61595283518683) SUDAH dibuat; sisa: tambah App ke bisnis → assign aset → "Buat token". Detail di `docs/koneksi-meta.md`
- Catatan: op `draft` tool meta TIDAK butuh token (file lokal); token hanya untuk op `accounts` (verifikasi)

**Keputusan Patria: OpenCreator pakai Gemini** (bukan kunci OpenAI-compatible terpisah)
- Diterapkan: `daemon/src/tools/opencreator.rs` gate ke Keychain akun `gemini`; `docs/open-creator.md` diperbarui
- Manual sekali di MacBook: OpenCreator → Settings → AI Services → Gemini

**Push 4 commit** (via push_rebased.py, tree identik): 14ffd9b→548b8cf (Veo 3.1), 1e86a0a→5495a30 (tool meta), 1bba756→c7cf70c (hapus fal.ai), 57ff9d7→01a90de (OpenCreator-Gemini). Remote HEAD: 01a90de.

---

## 17 Okt 2026 (malam) — VONIS GATE FASE 4: LOLOS BERSYARAT → lanjut Fase 5

Vonis Patria: Gate 4 **lolos bersyarat** — Fase 5 dimulai besok (18 Okt), sambil
memperbaiki backlog Gate 4 yang tersisa paralel dengan pekerjaan Fase 5.

**Backlog Gate 4 yang dibawa ke Fase 5:**
- Live test postiz via API nyata (butuh MacBook Patria: koneksi + API key `pos_...` di Keychain)
- Live test Meta `accounts` (butuh system user token — diparkir, fondasi sudah dibuat)
- Uji 1 video OpenCreator via Gemini (butuh kunci Gemini + instalasi OpenCreator di MacBook)
- Sisa live test provider (Gemini/Muse/DeepSeek) di MacBook

---

## 18 Okt 2026 (dini hari) — Identitas visual baru

Keputusan Patria: karakter bubble 3D jadi identitas visual Minime.
- App icon (`assets/app-icon.jpg`, dipakai `tauri icon`): robot biru → bubble pink mata oval (background hitam).
- Avatar Mimi: `minime-standard.jpg` (bubble pink, mata bulat) — commit `3094f71`.
- Avatar Riset: `riset-bubble.png` (bubble biru + kacamata + jas lab).
- Avatar QC: `qc-bubble.png` (bubble ungu + kemeja putih).
- Varian ekspresi Minime tersimpan: `minime-oval`, `minime-squint`, `minime-happy`, `minime-joy`.
- Avatar Strategi Bisnis: `strategi-bisnis-bubble.png` (bubble abu + tuksedo + dasi kupu).
- Avatar Creative Director: `creative-director-bubble.png` (bubble oranye + celana jeans + sabuk).

**Keluarga avatar bubble lengkap (18 Okt):** Mimi pink, Riset biru+kacamata+jas lab, Strategi Bisnis abu+tuksedo, Creative Director oranye+jeans, QC ungu+kemeja putih.

---

## 18 Okt 2026 (dini hari) — Skill baru: xerj (Riset) + openviking (Mimi)

Perintah Patria (2 repo):
- **`xerj`** (`skills/xerj/`, milik Riset, terdaftar di `agents/riset/agent.yaml`):
  local AI search engine dari `xerj-org/xerj` (**Apache-2.0** — aman).
  Satu binary Rust: `autoindex` folder (kode/docs/PDF) sekali → `search`/`def`
  jawab dengan passage + sitasi file:line; 2,7x lebih hemat output token vs
  grep (terukur). Pola utama: reference-coding (index repo acuan SEBELUM tulis kode).
  Referensi clone: `~/workspace/referensi/xerj/`.
- **`openviking`** (`skills/openviking/`, milik Mimi, terdaftar di
  `agents/mimi/agent.yaml`): context database dari `volcengine/OpenViking`
  (**AGPL-3.0** — ⚠️ aturan setara postiz: HANYA sebagai layanan eksternal
  via `pip`/`ov` CLI, DILARANG vendor kode ke repo). Satukan memory, knowledge
  RAG, skills dalam filesystem virtual `viking://` (ls/tree/read/find +
  ringkasan per direktori). Umum: memori lintas-agen + handoff via URI.
  Khusus: index pengetahuan per agen (Riset, QC, Strategi Bisnis).
  Referensi clone: `~/workspace/referensi/openviking/`.
- Instalasi keduanya (binary xerj + server openviking) menunggu MacBook Patria.

---

## 18 Okt 2026 (dini hari) — Tool `marketdata` + routing data finansial

Perintah Patria: koneksikan EDGAR, FRED, OpenBB, yFinance/stooq; Valyu jadi
fallback premium untuk retrieval proprietary yang dalam.
- **Tool daemon baru `marketdata`** (`daemon/src/tools/marketdata.rs`,
  terdaftar di registry + allowlist strategi-bisnis):
  - `quote` — harga via yfinance (gratis, tanpa key; butuh `pip install yfinance`)
  - `edgar_ticker` / `edgar_facts` / `edgar_filings` — SEC EDGAR tanpa key
    (User-Agent jujur sesuai aturan SEC; concept XBRL cth: Revenues, NetIncomeLoss)
  - `fred` — deret makro Fed (DGS10, FEDFUNDS, CPIAUCSL, UNRATE; butuh key
    gratis via Keychain akun `fred` / env `MINIME_FRED_KEY`)
  - Readiness check per komponen + fallback eksplisit; unit test 2 lolos,
    3 live-test EDGAR di-ignore (jalan di MacBook: `cargo test -- --ignored`)
- **Stooq DINONAKTIFKAN**: endpoint CSV kini di balik verifikasi JS browser —
  tidak bisa via HTTP langsung; digantikan yfinance.
- **OpenBB** sudah terkoneksi (`quant` op `openbb_data`, default provider yfinance).
- **Routing di `finance-playbook`**: GRATIS dulu (`marketdata` → `openbb`) →
  Valyu fallback premium hanya bila data tak ada di gratis / butuh proprietary
  dalam. `valyu-finance` direposisi sebagai fallback premium (aturan
  no-live-trading tetap).
- Dry run instalasi: **xerj v1.0.0-rc.96** (node → autoindex → retrieval OK) dan
  **openviking 0.5.0** (pip install → CLI OK) terverifikasi di VM;
  panduan MacBook di `docs/instalasi-xerj-openviking.md`.

---

## 18 Okt 2026 (dini hari) — Evaluasi ArtCraft: DITOLAK untuk saat ini

Keputusan Patria: abaikan `storytold/artcraft` untuk saat ini.
Alasan: bukan mesin text-to-video melainkan studio GUI untuk manusia
(butuh langganan/kredit sendiri, tanpa CLI/API untuk agen); fungsinya
tumpang tindih dengan Veo/OpenCreator/HyperFrames; tidak menyelesaikan
hambatan aktual (kunci Gemini, instalasi OpenCreator). Satu nilai yang diakui:
konsep scene-blocking untuk konsistensi karakter antar-shot (lubang nyata di
pipeline). Evaluasi ulang HANYA bila ArtCraft merilis mode headless/API yang
bisa disetir agen.

---

## 18 Okt 2026 (dini hari) — Pireel: disiapkan untuk uji coba

Keputusan Patria: siapkan `pireel/pireel` untuk diuji; bila lancar → permanen.
- **Skill `pireel`** (`skills/pireel/`, milik Creative Director, terdaftar di
  `agents/creative-director/agent.yaml`, STATUS: UJI COBA): AI video editor
  (alternatif CapCut) yang bisa disetir agen via MCP — transcript-aware cuts,
  captions, talking-head cleanup, multi-source assembly, export.
- **Lisensi AGPL-3.0** — aturan setara postiz/OpenViking: HANYA layanan
  eksternal (Studio via `pnpm dev` + MCP), DILARANG vendor kode ke repo.
- **Posisi bila lolos**: Veo/OpenCreator (generate) → Pireel (edit/rakit) → QC.
  Pireel = lapisan editing, BUKAN generator (tidak menggantikan Veo).
- **Rencana uji**: `docs/uji-pireel.md` — 4 tahap di MacBook Patria
  (install → koneksi MCP → alur penuh → billing check) + rubrik 7 kriteria.
  Vonis: 7/7 PERMANEN, 5–6 BERSYARAT, ≤4 DITOLAK.
- Billing: orkestrasi di subscription agen sendiri; tool `[CHARGES]` dilarang
  tanpa approval Patria.
- Referensi clone: `~/workspace/referensi/pireel/`.
