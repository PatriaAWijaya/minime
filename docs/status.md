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
