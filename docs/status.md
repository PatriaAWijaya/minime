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
