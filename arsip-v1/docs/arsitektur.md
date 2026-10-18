# Arsitektur Minime (v3 — komersil: akun + lisensi + provider + distribusi)

> Dokumen DESAIN. **Ditulis ulang dari nol pada 16 Okt 2026 (v3)** — arsitektur v2 (daemon + UI, 16 Okt 2026) BATAL dan tidak berlaku lagi.
> Alasan rewrite: keputusan final Patria — Minime menjadi **aplikasi komersil** yang di-download & di-install seperti `nightly-labs/openbot`. Butuh: sistem akun (login OTP email), lisensi & billing (Stripe), sistem provider model (gratis + berbayar, default Muse), dan release pipeline dengan code signing.
> Fondasi v2 TETAP dan tidak diubah: desktop app Tauri 2.x local-first, dua komponen (daemon + UI client), 5 agen senior + Approval, task-based execution, pipeline YAML, SQLite single-writer, Keychain-only secrets, no-auto-post Meta, clean-room (dilarang salin kode OpenBot — PolyForm Noncommercial).
> Mekanisme login OpenBot dipelajari dari repo publiknya (OTP 8 karakter via email, hash-only di D1, API di Cloudflare Workers, SMTP port 465, rate limit, Stripe billing) — yang diadopsi hanya **polanya**, implementasi ditulis sendiri.
> Status: menunggu review Patria bersama `AGENTS.md`. Belum ada kode yang ditulis dari dokumen ini.

---

## 1. Gambaran besar — 4 komponen

```
┌──────────────────────────────────────────────────────────────────┐
│  minime-daemon  (Rust headless — background service, launchd)    │
│  Orkestrator · Executor (FIFO/agen) · Pipeline YAML · Scheduler  │
│  LLM via PROVIDER ABSTRACTION (gratis + berbayar, default Muse)  │
│  Tool runners · Metering (token & biaya/agen) · Notifikasi macOS │
│  SQLite (WAL, single-writer) · Keychain (API keys user)           │
│  IPC server: Unix socket ──► ~/Library/Application Support/      │
│                               com.minime.app/minime.sock          │
│  Lisensi: cek ke Account API saat start (cache + offline grace)  │
└──────────────┬───────────────────────────────────────────────────┘
               │  launchd: com.minime.daemon (RunAtLoad + KeepAlive)
┌──────────────┴───────────────────────────────────────────────────┐
│  APLIKASI TAURI (dashboard UI — client tipis)                     │
│  Views: Agen · Antrean · Berjalan · Persetujuan · Deliverable ·   │
│         Penggunaan (biaya) · Pengaturan → Providers · Akun        │
│  Layar login OTP (email → kode) · Status lisensi/trial            │
│  Realtime: subscribe daemon → Tauri events → frontend            │
└──────────────┬───────────────────────────────────────────────────┘
               │ HTTPS (di luar mesin)
┌──────────────┴───────────────────────────────────────────────────┐
│  ACCOUNT API (backend terpusat — repo & deploy sendiri)           │
│  Rekomendasi pola OpenBot: Cloudflare Workers + D1 (+ R2 avatar)  │
│  OTP email 8 karakter (hash-only) · session token · lisensi       │
│  Stripe webhooks → entitlements · SMTP (465/TLS) / webhook email  │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  RELEASE PIPELINE (GitHub Actions → GitHub Releases)              │
│  dmg (macOS) · exe/msi (Windows) · AppImage (Linux)               │
│  Code signing wajib: Apple Dev $99/thn + notarization;            │
│  sertifikat Windows (SmartScreen); auto-updater Tauri             │
└──────────────────────────────────────────────────────────────────┘
```

**Keputusan desain kunci** (tambahan v3, dengan alasan):
- **Akun terpusat, eksekusi tetap lokal.** Satu-satunya yang keluar mesin: login OTP, cek lisensi, dan panggilan LLM ke provider pilihan user. Semua data kerja (DB, workspace, artefak) tetap di mesin user — local-first tidak dikorbankan demi komersialisasi.
- **Lisensi toleran offline.** Daemon meng-cache status lisensi; tanpa internet, lisensi terakhir berlaku dengan grace period 30 hari. Aplikasi local-first tidak boleh mati hanya karena WiFi putus — ini batasan jujur yang dicatat di sini.
- **Provider abstraction sejak Fase 2.** Trait didefinisikan sebelum LLM client pertama ditulis, agar menambah provider = implementasi trait, bukan refactor.
- **v1 = pribadi (ad-hoc signing cukup); rilis publik = signing penuh.** Dua standar eksplisit agar Fase 5 tidak tersangkut urusan sertifikat.

---

## 2. minime-daemon (background service)

### 2.1 Tanggung jawab
1. Pemilik tunggal database SQLite (WAL) — satu-satunya proses read-write.
2. Executor task: FIFO per agen (satu berjalan per agen), pause/resume/cancel/retry.
3. Pipeline runner: `pipelines/*.yaml`, handoff antar-agen, fan-out/fan-in.
4. **LLM via provider abstraction** (§2.2–2.4) — bukan client hardcode per vendor.
5. Tool runners: sidecar/CLI/MCP/HTTP; setiap pemanggilan tercatat (proof of work).
6. Scheduler internal (cron-like, tanpa cron sistem).
7. Notifier: notifikasi native macOS (approval, task gagal) — berfungsi walau UI tertutup.
8. **License gate**: cek lisensi ke Account API saat start + refresh berkala; cache + offline grace 30 hari.

### 2.2 Provider abstraction (trait)

```rust
trait ModelProvider {
    fn id(&self) -> &str;              // "muse" | "anthropic" | "openai"
                                       // | "gemini" | "deepseek" | "opencode"
                                       // | "ollama" | "lmstudio"
    fn kind(&self) -> ProviderKind;    // Free | PaidApiKey
    fn list_models(&self) -> Vec<ModelInfo>;   // id, label, context_window, usd_per_1m_in/out
    async fn complete_stream(&self, req: ChatRequest) -> Result<Stream<ChatChunk>>;
    fn estimate_tokens(&self, text: &str) -> usize;
}
```

- **Gratis**: (a) **OpenCode free models** — katalog disamakan dengan OpenBot (diputuskan Patria 16 Okt 2026): Muse Spark 1.3 Free, GLM-5.3-Flash Free, DeepSeek V4.1 Flash Free, Nemotron 3.5 Lightning Free, Nemotron 3.5 Super VL Free, Nemotron 3 Ultra Free, MiMo-V2.6-Flash Free, LongCat 2.5 Preview Free, Big Pickle Free, Cat Jam Free — tanpa akun/sign-in (via OpenCode CLI yang di-download & di-pin aplikasi, pola yang sama dipakai OpenBot); (b) deteksi server lokal **Ollama** (`localhost:11434`) dan **LM Studio** — benar-benar gratis & privat. Parameter effort per task mengikuti OpenCode: Low / Medium / High / Extra high.
- **Berbayar**: user menempel API key sendiri di Settings → Providers → disimpan ke **macOS Keychain** (per provider, write-only dari UI). **Default berbayar = Muse** (Meta Model API, `ai.meta.com`; ±$1,25/1M input, ±$4,25/1M output). Provider lain: Anthropic, OpenAI, Gemini, DeepSeek.
- **CATATAN implementasi**: verifikasi ketersediaan region Indonesia untuk Meta Model API saat Fase 2; bila belum tersedia, default sementara = Gemini, dan Muse tetap terdaftar sebagai provider (tinggal aktif saat region dibuka).

### 2.3 Routing per-agen + fallback chain

- Setiap agen punya konfigurasi: `provider_id` + `model` + `fallback_chain: [(provider, model), ...]`.
- Contoh default: QC → model murah-cepat (fallback: provider lokal); Strategi Bisnis → DeepSeek atau Muse (fallback: Gemini); Riset → Gemini long-context.
- Bila provider gagal (error/rate-limit/timeout): otomatis pindah ke fallback berikutnya, tercatat di `runs` (transparan di dashboard).
- Tabel harga per model disimpan di konfigurasi daemon (diperbarui berkala) — dipakai untuk metering; selalu dilabeli **estimasi**.

### 2.4 Metering + budget cap

- Setiap LLM call mencatat ke `runs`: `provider`, `model`, `input_tokens`, `output_tokens`, `cost_usd_est`.
- **Budget cap**: batas biaya per agen per hari (default diusulkan, bisa diubah di Pengaturan). Cap tercapai → task di-pause + notifikasi "budget habis" + butuh persetujuan Patria untuk lanjut. Ini pagar anti kaget tagihan.
- View **Penggunaan** di dashboard: ringkasan biaya per agen/periode (dari kolom `runs`, tanpa tabel tambahan — skema lokal tetap 7 tabel).

### 2.5 Lifecycle launchd & crash recovery
- Plist `~/Library/LaunchAgents/com.minime.daemon.plist` (RunAtLoad + KeepAlive); stdout/stderr → `~/Library/Logs/com.minime.daemon/`; install/unload via installer/first-run UI; tombol "Hentikan daemon" eksplisit di Pengaturan.
- Crash recovery jujur: task `running` → `failed` (`error='daemon restart'`) saat start; `runs` berstatus `waiting-approval` TETAP (state manusia).

---

## 3. Aplikasi Tauri (dashboard UI — client tipis)

- Backend Rust = **proxy IPC** ke daemon via Unix socket; tidak menyentuh DB, tidak mengeksekusi task. Realtime: satu koneksi `subscribe` → Tauri events (`minime://agent-status`, `minime://task-updated`, `minime://run-updated`, `minime://approval-requested`).
- **Baru di v3**:
  - **Layar login**: email → kode OTP 8 karakter (menghubungi Account API langsung via HTTPS; session token disimpan ke Keychain via IPC `secrets.set`).
  - **Pengaturan → Providers**: tambah API key (write-only), test koneksi, enable/disable provider, set model default per agen, budget cap per agen.
  - **Penggunaan**: dashboard biaya (token & estimasi USD per agen/periode).
  - **Akun**: status lisensi (trial/pro/lifetime), sisa trial, tombol kelola langganan (Stripe Customer Portal).
- Views lain tetap (Agen, Antrean, Berjalan, Persetujuan, Deliverable, Pengaturan) — Bahasa Indonesia, ikut UI rules + Impeccable. Mode dashboard: Operate (presisi & restraint, bukan ekspresi).
- **Menu konteks per-agen** (diputuskan Patria 16 Okt 2026, meniru OpenBot): klik kanan / menu pada agen membuka: "Sembunyikan komputer" (toggle panel browser/komputer agen), "Pengaturan agen", section **Tasks** (daftar task FIFO agen: queued/running/paused/done/failed/cancelled + pause/resume/cancel per task), dan **"+ New task"** (buat task manual ke antrean agen).
- **Pengaturan per-agen**: provider + model, effort (Low/Medium/High/Extra high), skills aktif, tools allowlist, budget cap, avatar.

---

## 4. Account API (backend terpusat)

### 4.1 Stack & repo
- **Rekomendasi pola OpenBot**: Cloudflare Workers + **D1** (data akun/auth) + R2 (avatar, opsional). Alasan: serverless, murah di awal (free tier cukup), deploy via Wrangler.
- **Repo terpisah** dari aplikasi desktop (`minime-account-api`), deploy sendiri. Secret via `wrangler secrets` / GitHub Environments — dilarang di repo.
- Endpoint (v1): `POST /v1/auth/request-code`, `POST /v1/auth/verify`, `POST /v1/auth/refresh`, `POST /v1/auth/revoke`, `GET /v1/license`, `GET /v1/me`, `POST /v1/stripe/webhook`.

### 4.2 Skema D1

| Tabel | Kolom kunci | Guna |
|---|---|---|
| `accounts` | `id` (PK), `email` (UNIQUE), `name`, `avatar_url`, `status`, `created_at` | identitas user |
| `email_challenges` | `id` (PK), `account_id` (FK), `code_hash`, `expires_at`, `attempts`, `consumed_at`, `created_at` | **hanya hash kode, tidak pernah plaintext** |
| `sessions` | `id` (PK), `token_hash` (UNIQUE), `account_id` (FK), `device_label`, `expires_at`, `revoked_at`, `created_at` | sesi per perangkat |
| `licenses` | `id` (PK), `account_id` (FK UNIQUE), `plan` ('trial'/'pro'/'lifetime'), `status` ('active'/'past_due'/'cancelled'/'expired'), `trial_ends_at`, `current_period_end`, `stripe_customer_id`, `stripe_subscription_id`, `updated_at` | entitlements |
| `stripe_customers` | `account_id` (PK), `stripe_customer_id` (UNIQUE), `created_at` | mapping Stripe |

### 4.3 Alur OTP end-to-end
1. UI: user memasukkan email → `POST /v1/auth/request-code {email}`.
2. API: normalisasi email → **rate limit** (default: per email maks 5/jam; per IP maks 15/15 menit; cooldown kirim ulang 60 detik) → upsert `accounts` → generate kode 8 karakter (alfabet tanpa karakter ambigu) → **hash (argon2id)** → INSERT `email_challenges` (`expires_at` = +10 menit) → kirim kode via **SMTP (port 465, TLS)**; bila SMTP gagal → **fallback webhook** `POST {email, code, expiresAt}` (dengan Bearer secret bila dikonfigurasi) → respons `200 {ok:true}` **tanpa kode** di production.
3. User memasukkan kode → `POST /v1/auth/verify {email, code}` → ambil challenge terbaru yang belum `consumed` & belum expired → `attempts`+1 (maks 5, selebihnya hangus) → bandingkan hash **constant-time** → cocok → `consumed_at=now` → buat session (token 32-byte random; yang disimpan hanya hash-nya; expiry 90 hari) → respons `200 {session_token}` — token plaintext **hanya muncul sekali di sini**.
4. UI menyimpan `session_token` ke macOS Keychain (via IPC `secrets.set`).
5. Panggilan berikutnya: `Authorization: Bearer <session_token>`.

### 4.4 Session & license check
- Daemon membaca session token dari Keychain → `GET /v1/license` → cache hasilnya secara lokal (termasuk `cached_at`).
- **Offline grace 30 hari**: tanpa internet, lisensi cache terakhir tetap berlaku. Setelah 30 hari tanpa refresh → fitur berbayar dikunci sampai online kembali. Batasan ini jujur dicantumkan di UI.
- Akun Patria (pemilik): `plan='lifetime'` — v1 pribadi tidak terkunci apa pun.

### 4.5 Rate limit, keamanan, housekeeping
- Rate limit dihitung per email, per IP, per challenge, per kirim ulang (D1 counters + Cloudflare rate limiting).
- **Mode dev** (`AUTH_EXPOSE_DEV_CODE=true`): `request-code` boleh mengembalikan kode di respons untuk testing — **DILARANG keras di production** (fail-closed: bila env production, flag ini diabaikan).
- **Cron pembersih** (Workers Cron Trigger tiap 5 menit): hapus challenges expired/consumed >24 jam, sessions expired/revoked >30 hari, rate-limit records expired. Midnight UTC: agregat log.
- Kode OTP satu upaya per challenge; challenge hangus setelah 5 percobaan salah atau 10 menit.

### 4.6 Secrets backend
- `EMAIL_SMTP_PASSWORD`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, signing keys — hanya via `wrangler secret put` / GitHub Environments. Tidak di repo, tidak di argumen proses.

---

## 5. Lisensi & billing (Stripe)

### 5.1 Model lisensi
- `trial` → `pro` (bulanan/tahunan) → `lifetime` (khusus pemilik). Satu akun = satu lisensi (`licenses.account_id` UNIQUE).
- Trial: akses penuh fitur selama masa trial (default usulan 14 hari — keputusan bisnis Patria). Trial habis → fitur berbayar dikunci; data & riwayat tetap bisa dibaca (tidak disandera).
- Definisi "fitur berbayar vs gratis" = keputusan bisnis Patria (terbuka, §13).

### 5.2 Alur Stripe webhook
- `checkout.session.completed` → buat/upgrade `licenses` → `plan='pro'`, `status='active'`.
- `customer.subscription.updated` → perbarui `current_period_end` / status.
- `customer.subscription.deleted` → `status='cancelled'` (tetap aktif sampai akhir periode berjalan).
- `invoice.payment_failed` → `status='past_due'` + notifikasi ke user; gagal berulang → kunci fitur (grace 7 hari).
- **Customer Portal** Stripe untuk upgrade/cancel mandiri (ditautkan dari layar Akun di aplikasi).

### 5.3 Enforcement di daemon
- Saat start + refresh berkala (24 jam): `GET /v1/license` → simpan cache → event `minime://license-updated` ke UI.
- Bila `status` tidak aktif & bukan grace → daemon menolak enqueue task baru (task lama tetap bisa dilihat); UI menampilkan banner + tombol "Perpanjang lisensi".

---

## 6. Protokol IPC (Unix socket daemon↔UI)

Lokasi: `~/Library/Application Support/com.minime.app/minime.sock` (dir 700). JSON per baris; `subscribe` untuk event. **Hanya daemon yang menulis DB.**

| Method | Guna |
|---|---|
| `daemon.status` | versi, uptime, antrean, status launchd |
| `agents.list` / `agents.pause` / `agents.resume` | roster + kontrol |
| `tasks.enqueue` / `tasks.list` / `tasks.cancel` / `tasks.retry` | antrean |
| `runs.get` / `runs.list` | detail & riwayat (+ kolom provider/model/token/biaya) |
| `approvals.list` / `approvals.decide` | inbox + keputusan Patria |
| `deliverables.list` | output + tautan Drive |
| `secrets.set` | tulis secret ke Keychain (**write-only**) |
| `providers.list` / `providers.set_key` / `providers.test` / `providers.set_default` | kelola provider model (key write-only) |
| `usage.summary` | agregat token & estimasi biaya per agen/periode |
| `license.status` / `license.refresh` | status lisensi (dari cache daemon) |
| `daemon.shutdown` | hentikan daemon (aksi eksplisit di UI) |

---

## 7. Skema SQLite lokal (7 tabel — tetap)

Lokasi: `~/Library/Application Support/com.minime.app/minime.db` (WAL, `synchronous=NORMAL`). **Hanya daemon yang membuka file ini.**

| Tabel | Kolom kunci | Guna |
|---|---|---|
| `agents` | `id` (slug, PK), `name`, `role`, `model`, `provider_id`, `fallback_chain` (JSON), `system_prompt`, `avatar_path`, `color`, `status`, `tools_allowlist` (JSON), `skills` (JSON), `workspace_path`, `paused`, `daily_budget_usd` | 5 agen senior + routing provider + budget cap |
| `tasks` | `id` (PK), `agent_id` (FK), `run_id` (FK), `title`, `input` (JSON), `status` ('queued'/'running'/'paused'/'done'/'failed'/'cancelled'), `position`, `progress`, `result_summary`, `error`, `retry_count`, timestamps | FIFO per agen |
| `messages` | `id` (PK), `run_id` (FK), `from_agent`, `to_agent`, `kind`, `content`, `created_at` | handoff + riwayat |
| `runs` | `id` (PK), `pipeline`, `status`, `current_stage`, `context` (JSON), `provider`, `model`, `input_tokens`, `output_tokens`, `cost_usd_est`, timestamps | eksekusi pipeline + metering |
| `approvals` | `id` (PK), `run_id` (FK), `stage`, `payload_summary`, `artifact_refs` (JSON), `qc_score`, `status`, `decided_at`, `note` | human gate Patria |
| `deliverables` | `id` (PK), `run_id` (FK), `kind`, `file_path`, `drive_file_id`, `drive_url`, `created_at` | output + tautan Drive |
| `skills` | `name` (PK), `version`, `path`, `owner_agent`, `enabled` | registry skills + pemilik |

Index: `tasks(agent_id, status, position)`, `messages(run_id, created_at)`, `approvals(status)`, `runs(status)`, `runs(created_at)`.
Migrasi: SQL berversi di `daemon/migrations/`, dijalankan daemon saat startup. Metering dihitung dari kolom `runs` (tanpa tabel tambahan).

---

## 8. Alur QC → Approval (human gate + notifikasi native)

1. Agen QC menilai output (rubrik skill `qc`): skor 0–100 + alasan.
2. Skor ≥ ambang → daemon buat `approvals` (`pending`) → **notifikasi native macOS langsung dari daemon** (berfungsi walau UI tertutup) → event ke UI bila terbuka.
3. Klik notifikasi → membuka aplikasi ke inbox Persetujuan (deep link `minime://approvals/<id>`).
4. Patria **Setujui** → deliverable difinalisasi → upload ke Drive MIMI → run `done`.
5. Patria **Tolak + catatan** → task kembali ke agen produsen (maks 3 revisi → status `attention` + notifikasi).
6. **Tidak ada aksi eksternal** tanpa approval; distribusi yang disetujui pun hanya menghasilkan **draft** (aturan keras no-auto-post).

---

## 9. Model eksekusi agen & kontrak senior

1. Orkestrator (di daemon) membuat task (`queued`) di antrean agen target.
2. Executor mengambil task terdepan per agen yang tidak `paused` (satu berjalan per agen).
3. Prompt dirakit: `system_prompt` + skill pemilik (diringkas & di-cache) + `input` + `runs.context` + N pesan terakhir (budget token).
4. Stage dengan `skills: [...]` = sub-langkah berurutan, satu LLM call per skill (proof of work, sub-progres di dashboard).
5. LLM dipanggil via **provider abstraction** sesuai `agents.provider_id`/`model`, mengikuti `fallback_chain` bila gagal.
6. Hasil → kontrak output stage (`summary`, `artifacts`, `handoff`, token & biaya) → simpan ke task + `runs` → emit event.
7. Handoff → `messages` + enqueue agen berikut.
8. Stage tool-driven: executor menjalankan tool command (bukan LLM call).

**Kontrak senior** (keputusan Patria 16 Okt 2026): setiap agen wajib punya (1) playbook/SOP sebagai skills, (2) toolbox jelas (`tools_allowlist`), (3) proof of work (tercatat di DB), (4) quality bar output, (5) kewajiban eskalasi ke Patria bila ragu. Menambah kemampuan = menambah skill, BUKAN menambah agen (rule #16).

**Pagar biaya**: `daily_budget_usd` per agen; cap tercapai → task di-pause + notifikasi + butuh persetujuan Patria untuk lanjut.

---

## 10. Pipeline sebagai data YAML

```yaml
# pipelines/content.yaml — v1 (jalur utama)
name: pipeline-konten
version: 1
stages:
  - id: riset
    agent: riset
    on_done: handoff(strategi-bisnis)
  - id: strategi
    agent: strategi-bisnis
    skills: [branding-playbook, marketing-playbook,
             selling-playbook, operational-playbook]
    on_done: handoff(creative-director)
  - id: produksi
    agent: creative-director
    skills: [storytelling, copywriting, content, smo-playbook]
    on_done: handoff(qc)
  - id: qc
    agent: qc
    on_done: approval   # → approvals (pending) + notifikasi native
```

- `parallel: [...]` didukung untuk fan-out antar-agen di masa depan; v1 mengeksekusi skills sekuensial di dalam stage agen senior.
- Cabang diagram (SMO subtree, Code Builder → UI/UX) = stage/pipeline tambahan di YAML yang sama; diaktifkan bertahap Fase 3–4 **tanpa ubah kode executor**.

---

## 11. Sistem skills

- `skills/<nama>/SKILL.md` + opsional `scripts/` `referensi/` `assets/`; frontmatter `example-prompt` (maks 1000 char).
- Satu agen pemilik per skill (`skills.owner_agent`); avatar playbook dipakai sebagai ikon skill di UI.
- Playbook bisnis = skills dengan knowledge dari 4 dokumen Drive (**disuling** saat Fase 3, bukan dibaca mentah — hemat token).
- Skill baru via pola `create_skill` ala OpenBot (Fase 3+).

---

## 12. Integrasi tools (dieksekusi daemon)

| Tool | Cara koneksi | Status / catatan |
|---|---|---|
| Canva | CLI (`canva`) via Command | CLI terverifikasi 2026-10-07 |
| Affinity | MCP bridge `localhost:6767` (HTTP) | Token dipegang Patria; tunnel berubah tiap restart → tombol reconnect di Pengaturan |
| OpenCreator | sidecar di samping binary daemon | Butuh **keputusan**: kunci LLM OpenAI-compatible ATAU wiring Gemini + uji 1 video |
| VoiceStudio | REST/MCP | **Belum diinstal** di MacBook; fallback: edge-tts |
| HyperFrames | CLI lokal | Alternatif render (ditunda 15 Okt) — fallback kedua |
| instagram-cli | CLI, **draft-only** | Posting dilarang; hanya siapkan draft |
| Google Drive | OAuth2 desktop → token di Keychain | Baca (Fase 3) & tulis deliverable (Fase 5) — OAuth cukup **sekali** |
| Riset web | Gemini long-context / Agent-Reach | Detail difinalkan Fase 2 |
| **Built-in browser** | Chromium headless (Playwright) dikontrol daemon | Diputuskan Patria 16 Okt 2026 — meniru panel Browser OpenBot: agen (Riset, SMO) bisa membuka, menginspeksi & mengontrol browser; sesi/cookie terisolasi; UI dashboard menampilkan panel browser yang mirror sesi agen agar Patria bisa mengawasi |

**Catatan built-in browser:** browser berjalan di sisi daemon (bukan webview UI) agar tetap bekerja walau aplikasi ditutup; UI hanya me-mirror tampilan sesi aktif via IPC. Aksi browser agen tercatat di `runs` (proof of work). Tidak untuk login otomatis ke akun pribadi Patria tanpa izin eksplisit per tugas.

---

## 13. Secrets & keamanan

- **Lokal** (macOS Keychain via crate `keyring`, diakses daemon): API key semua provider model (Muse, Anthropic, OpenAI, Gemini, DeepSeek), token OAuth Google, token Affinity, session token akun. Tidak di git/plaintext/log; UI tulis via IPC `secrets.set` (write-only).
- **Backend** (account API): SMTP password, Stripe keys, webhook secrets, signing keys — hanya via `wrangler secret put` / GitHub Environments. Dilarang di repo & argumen proses.
- Security audit Fase 5: skill `local_security_audit` + `cargo audit`.
- Daemon & app berjalan dengan hak user biasa; tool eksternal hanya dari `tools_allowlist` per agen.

---

## 14. Packaging, signing & auto-update

- **Target**: dmg (macOS arm64) → v1; exe/msi (Windows x64), AppImage (Linux) → menyusul di Fase 6.
- **v1 pribadi**: ad-hoc signing cukup (tidak didistribusikan).
- **Rilis publik (wajib)**: Apple Developer Program **$99/tahun** + **notarization** (`notarytool` + staple) — tanpa ini Gatekeeper memblokir aplikasi; sertifikat Windows untuk SmartScreen (tanpa ini muncul peringatan); Linux: SHA256 checksum.
- **Auto-updater** (Tauri updater): keypair signing (private key di GitHub Secrets, public key di `tauri.conf.json`); setiap rilis menghasilkan artifact `.sig`.
- **Release pipeline** (GitHub Actions): matrix build per OS → sign → notarize → buat GitHub Release per tag `v*` (draft → publish) dengan installer + `.sig` + checksum.
- Ikon dari `assets/app-icon.jpg` (via `tauri icon`); binary daemon di `Minime.app/Contents/MacOS/minime-daemon`; installer/first-run memasang LaunchAgent (checklist eksplisit).

---

## 15. Enam fase build (review gate tiap fase)

- **Fase 1 — Fondasi daemon + UI skeleton.** Daemon: skema SQLite 7 tabel, IPC Unix socket, FIFO executor, install/load LaunchAgent. **Provider trait didefinisikan di sini** (interface dulu, implementasi menyusul). UI: 6 views skeleton + IPC client + realtime, UI rules diterapkan. *Bukti kunci: quit aplikasi → daemon TETAP jalan.* *Gate: Patria cek dashboard + buktikan quit ≠ stop.*
- **Fase 2 — Orkestrasi inti + provider.** Pipeline YAML runner, handoff, implementasi provider Gemini + **Muse** (dengan verifikasi region Indonesia; fallback Gemini bila belum tersedia), routing per-agen + fallback chain dasar, 1 pipeline vertikal end-to-end + approval inbox minimal + notifikasi native. *Gate: 1 job nyata penuh + tap approve Patria.*
- **Fase 3 — Skills & knowledge.** 6 skills → Strategi Bisnis, 4 skills → Creative Director (knowledge disuling dari 4 dokumen Drive); DeepSeek; OAuth Google Drive sekali (baca). *Gate: output tiap skill di-review Patria.*
- **Fase 4 — Tools produksi.** Readiness check eksplisit; wiring tools + fallback; Content Production; SMO draft-only. *Gate: 1 reels/carousel jadi dari pipeline.*
- **Fase 5 — Hardening & rilis v1 pribadi.** QC judge penuh, metering + budget caps + view Penggunaan, Providers settings UI, upload Drive, dmg ad-hoc + setup LaunchAgent, security audit, audit Impeccable. *Gate: v1 rilis di MacBook Patria.*
- **Fase 6 — Komersialisasi (boleh jalan paralel setelah Fase 2).** Account API (repo Workers + D1 + deploy), OTP email + SMTP, Stripe (products, Customer Portal, webhooks), license enforcement + offline grace, signing publik (Apple notarization, cert Windows), release pipeline GitHub Actions, auto-updater. *Gate: instalasi dari dmg ter-sign + login OTP + lisensi trial/berbayar berfungsi.*

Gate tiap fase: demo nyata ke Patria, bukan laporan.

---

## 16. Keputusan terbuka (butuh Patria)

**Bisnis (tidak menghambat Fase 1–2):**
1. **Model harga**: sekali-bayar vs langganan (vs keduanya)?
2. **Lama trial**: usulan default 14 hari.
3. **Fitur yang dikunci lisensi**: apa yang gratis vs berbayar? (Usulan awal: pipeline inti butuh lisensi aktif; riwayat & data selalu bisa dibaca.)
4. **Kuota LLM trial**: apakah trial menyertakan kuota LLM bawaan, atau user wajib membawa API key sendiri sejak hari pertama?

**Teknis:**
5. Review `AGENTS.md` — setuju / revisi (masih menunggu).
6. Review dokumen arsitektur ini (v3) — setuju / revisi.
7. Frontend: Svelte 5 (rekomendasi) vs Solid — dikunci saat scaffold Fase 1.
8. Meta Model API: verifikasi ketersediaan **region Indonesia** (dibutuhkan Fase 2 untuk default Muse).
9. OpenCreator: kunci LLM OpenAI-compatible ATAU wiring Gemini + uji 1 video (Fase 4).
10. VoiceStudio: instalasi MacBook + uji suara Bahasa Indonesia (Fase 4; fallback edge-tts siap).
11. OAuth Google: persetujuan saat tiba Fase 3 (baca Drive) — scope jelas, token di Keychain.
