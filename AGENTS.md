# AGENTS.md — Minime

> Dibuat dari `PROMPT-AGENTS.md` pada 16 Okt 2026. Status: menunggu review Patria.
> Setelah di-review → brainstorm → `docs/status.md` + `docs/rule.md` → fase build (review tiap fase).

---

## a. Ringkasan proyek & definisi selesai

### Ringkasan

**Minime** adalah aplikasi desktop **local-first** untuk AI teammates yang persisten — terinspirasi pola arsitektur `nightly-labs/openbot`, **BUKAN fork**. OpenBot berlisensi PolyForm Noncommercial: dilarang menyalin kodenya; yang diadopsi hanya pola arsitektur. Implementasi clean-room.

- Satu pengguna: Patria A. Wijaya (Indonesia).
- Bahasa Indonesia untuk seluruh UI dan dokumen user-facing.
- Pola yang diadopsi dari OpenBot: per-agent workspace + conversation, antrean task FIFO (pause/resume/cancel, crash-safe), agent-to-agent messaging untuk handoff, skills library (playbook sebagai skill), human-in-the-loop approval, data di mesin sendiri.

### Alur kerja (final, dari diagram Patria)

Mimi (orkestrator) → Riset → Strategi Bisnis → 6 Playbook (Branding, Marketing, Selling, Operational, Finance, Code Builder) → [Branding/Marketing/Selling/Operational] → Creative → 4 Playbook (Storytelling, Copywriting, Content, SMO) → Content Production (Image, Carousel, Reels) → Tools (Canva, Affinity, OpenCreator, VoiceStudio) → QC → Approval (Patria) → Deliverable Output.

Cabang: Code Builder → UI/UX → QC. SMO → (Page Optimization, Content Pilar, Content Planning, Content Distribution, Engagement) → QC. Deliverable Output memberi feedback loop ke tahap distribusi/engagement.

### Definition of Done v1

1. Aplikasi desktop ter-install di MacBook Patria (dmg), berjalan local-first tanpa server eksternal.
2. **Daemon tetap berjalan setelah aplikasi di-quit** (terbukti: quit app → status daemon "berjalan"); daemon start otomatis saat login via LaunchAgent.
3. Dashboard realtime menampilkan roster 5 agen senior + status (kuning = aktif/kerja, hijau = idle, merah = butuh perhatian).
4. Satu pipeline end-to-end berjalan nyata: Riset → Strategi Bisnis → Creative Director → QC → Approval → Deliverable (draft).
5. Gate QC + Approval Patria berfungsi: tidak ada aksi eksternal tanpa approval; **notifikasi native macOS untuk approval masuk walau UI tertutup**.
6. Deliverable tersimpan ke Google Drive folder MIMI.
7. Tidak ada auto-posting ke Meta; distribusi hanya berupa draft.
8. UI lolos UI rules + anti-pattern Impeccable.
9. Kredensial tersimpan di OS keychain; tidak ada secret di repo.
10. Instalasi dari dmg ter-sign (v1 pribadi: ad-hoc signing; rilis publik: Apple notarization).
11. Login OTP email berfungsi end-to-end (kode tidak pernah tersimpan plaintext di DB).
12. Lisensi trial/berbayar enforced (trial expired → fitur berbayar terkunci; data tetap bisa dibaca).

---

## b. Tech stack (arsitektur dua komponen — keputusan final 16 Okt 2026)

Keputusan final Patria: **agen harus tetap bekerja meskipun aplikasi ditutup** → aplikasi dipecah dua:

- **`minime-daemon`** — background service Rust (headless): pemilik tunggal SQLite (WAL), eksekutor task (FIFO per agen), pipeline runner YAML, LLM clients (Gemini + DeepSeek via reqwest), tool runners (CLI/sidecar/MCP/HTTP), scheduler internal, dan pengirim **notifikasi native macOS**. Dijalankan via **launchd LaunchAgent** (`com.minime.daemon`: RunAtLoad + KeepAlive) — start saat login, hidup terus walau UI di-quit. Binary dibundel di `Minime.app/Contents/MacOS/minime-daemon`.
- **Aplikasi Tauri 2.x** — dashboard UI sebagai **client tipis**: backend Rust hanya proxy IPC ke daemon via **Unix socket** (`~/Library/Application Support/com.minime.app/minime.sock`); tidak menyentuh DB langsung, tidak mengeksekusi task. Realtime: langganan event daemon → Tauri events → frontend. **Hanya daemon yang menulis DB (single-writer).**
- **Frontend:** TypeScript + web tech (rekomendasi Svelte 5; alternatif Solid — dikunci saat scaffold Fase 1).
- **Data:** SQLite lokal, 7 tabel: `agents`, `tasks`, `messages`, `runs`, `approvals`, `deliverables`, `skills` (detail: `docs/arsitektur.md` §5). Crash recovery jujur: task `running` → `failed` saat daemon restart.
- **LLM:** Gemini API (orkestrasi, riset, creative) dan DeepSeek API (strategi, code). Kunci API milik Patria, disimpan di **macOS Keychain** via crate `keyring` (diakses daemon) — BUKAN di Secure Vault Mimi, BUKAN di git, BUKAN plaintext. UI menulis secret hanya via IPC `secrets.set` (write-only).
- **Provider system:** abstraction layer di daemon (trait: `list_models`, `complete_stream`, estimasi token). **Gratis**: **OpenCode free models** — katalog disamakan dengan OpenBot (Muse Spark 1.3 Free, GLM-5.3-Flash Free, DeepSeek V4.1 Flash Free, Nemotron 3.5 Lightning Free, Nemotron 3.5 Super VL Free, Nemotron 3 Ultra Free, MiMo-V2.6-Flash Free, LongCat 2.5 Preview Free, Big Pickle Free, Cat Jam Free; tanpa akun via OpenCode CLI) + deteksi server lokal Ollama/LM Studio (gratis & privat); effort per task: Low/Medium/High/Extra high. **Berbayar**: API key milik user — **default = Muse** (Meta Model API, `ai.meta.com`; ±$1,25/1M input, ±$4,25/1M output), alternatif: Anthropic, OpenAI, Gemini, DeepSeek — disimpan di Keychain. **Routing per-agen + fallback chain** (mis. QC pakai model murah-cepat, Strategi Bisnis pakai DeepSeek/Muse) + **metering** (token & estimasi biaya per agen) + **budget cap** per agen/hari. UI: Settings → Providers (tambah key, test koneksi, set model per agen). CATATAN: verifikasi ketersediaan region Indonesia untuk Meta Model API saat implementasi.
- **Account API (backend terpusat, repo terpisah):** rekomendasi Cloudflare Workers + D1 — pola dipelajari dari OpenBot, implementasi sendiri. Login **OTP 8 karakter via email** (hanya hash + expiry di DB, rate limit per email/IP/kirim ulang, SMTP port 465/TLS + fallback webhook); session token; lisensi `trial`/`pro`/`lifetime`/`coupon` (kupon: Patria generate via endpoint admin-only, 1 kode = maks 5 user, redeem = Pro 1 tahun; kode disimpan hash-only + rate limit); **Stripe** (plan bulanan/tahunan, Customer Portal, webhooks renewal/gagal bayar/cancel). Daemon cek lisensi tiap start (cache + offline grace 30 hari).
- **Release pipeline:** GitHub Actions → GitHub Releases: dmg (macOS), exe/msi (Windows), AppImage (Linux). **Code signing wajib** untuk rilis publik: Apple Developer $99/thn + notarization (tanpa ini Gatekeeper memblokir), sertifikat Windows (SmartScreen). **Auto-updater** bawaan Tauri (artifact ter-sign). v1 fokus macOS arm64.
- **Tools lokal (dieksekusi daemon):** Canva CLI, Affinity MCP bridge (`localhost:6767`), OpenCreator (sidecar di samping binary daemon), VoiceStudio (REST/MCP — instalasi MacBook masih pending), HyperFrames CLI, `instagram-cli` (draft-only).
- **Packaging:** dmg macOS arm64 (target utama v1); installer/first-run memasang LaunchAgent; ikon dari `assets/app-icon.jpg` via `tauri icon`; ad-hoc signing cukup untuk v1 pribadi. Windows/Linux di luar scope v1.
- **Repo:** git; remote GitHub privat (pola market-heatmap). Kredensial tidak di git.
- **Workspace agen:** `~/Minime/workspaces/<agent-id>/` (conversation + file kerja tiap agen).

---

## c. Roster agen + model AI + tanggung jawab

> **Keputusan Patria 16 Okt 2026: sedikit agen dengan skill setara senior manager, bukan banyak agen junior.** Roster dirampingkan dari 18 menjadi 5 agen senior (+ Approval manusia). Playbook-playbook di diagram TIDAK hilang — mereka menjadi **skills** yang dikuasai agen senior pemiliknya. Diagram alur kerja tidak berubah; yang berubah hanya siapa mengeksekusi tiap kotak.
>
> Kelima agen di bawah **dieksekusi oleh `minime-daemon`** (bukan oleh UI) — definisinya tersimpan di tabel `agents`, antreannya di `tasks`, dan UI hanya menampilkan status serta mengirim perintah via IPC.

| # | Agen | Model | Tanggung jawab | Skills / Toolbox | Kontrak senior |
|---|------|-------|----------------|------------------|----------------|
| 1 | Mimi (Orkestrator) | Gemini | Router task, antrean kerja, status realtime, delegasi antar agen | pipeline YAML, Tauri events | — |
| 2 | Riset | Gemini (long-context) | Senior researcher: riset web/YouTube/RSS, sintesis ber-sumber | channel Agent-Reach (web, YouTube, RSS, GitHub publik) | playbook riset; setiap klaim wajib bersumber; eskalasi bila data meragukan |
| 3 | Strategi Bisnis | DeepSeek | Senior strategist: strategi bisnis; 6 playbook sebagai skills | `branding-playbook`, `marketing-playbook`, `selling-playbook`, `operational-playbook`, `finance-playbook`, `code-builder-playbook` | tiap skill punya quality bar; output harus bisa dieksekusi Creative Director |
| 4 | Creative Director | Gemini | Senior: 4 playbook konten sebagai skills + produksi konten via tools | `storytelling`, `copywriting`, `content`, `smo-playbook`; tools: Canva CLI, Affinity bridge, OpenCreator, VoiceStudio, HyperFrames, instagram-cli (draft-only) | Golden Circle + 13-item quality bar; distribusi hanya berupa draft |
| 5 | QC | Gemini-as-judge (jev bila kunci OpenRouter tersedia) | Senior judge: skor 0–100 + alasan; wewenang tolak; maks 3 revisi | rubrik penilaian (`qc`) | penolakan wajib beralasan; eskalasi ke Patria bila ambang tidak jelas |
| 6 | Approval | manusia (Patria) | Wajib; tidak bisa diwakilkan | — | — |

**Kontrak senior** (berlaku untuk agen 1–5): setiap agen wajib punya (1) playbook/SOP sebagai skills, (2) toolbox yang jelas (allowlist), (3) proof of work (setiap hasil tercatat di DB), (4) quality bar output yang jelas, dan (5) kewajiban eskalasi ke Patria bila ragu. Inilah yang membuat agen "setara senior manager" — bukan jumlahnya.

---

## d. Skills / playbooks + sumber knowledge + pemilik

Skills = folder `skills/<nama>/SKILL.md` (+ skrip/referensi/aset opsional), **dimiliki dan dikuasai oleh satu agen senior**. Playbook adalah skills dengan knowledge base:

**Dimiliki Strategi Bisnis:**
- `branding-playbook` ← Drive `14bAxNF_vCJjyd9ZzUt0Z85iVlZGzDU4BmEHxFjhp7mY`
- `marketing-playbook` ← Drive `16Jctf4wt11InFccmN1kgSFKEFo9zZAVz127Ite2JClo`
- `selling-playbook` ← Drive `1hMOVWhdKQGFHlIw2BSJXEQR0Q4R3CFF6C1zTQD8mGrA`
- `operational-playbook` ← Drive `1exok4QG8YHfOyxrN3JIj3fzzLgCD6IrzHpwhMcR9vCM`
- `finance-playbook`, `code-builder-playbook` ← dokumen menyusul
- `oil-ui` ← skill UI untuk code builder (diadaptasi dari `oil-oil/oil-ui` MIT)
- `valyu-finance` ← riset finansial via Valyu Search API (pola `yorkeccak/finance` MIT)
- `trading-agents` ← analisis multi-agen LLM (`TauricResearch/TradingAgents` Apache-2.0)
- `qlib` ← platform quant AI (`microsoft/qlib` MIT)
- `nautilus` ← backtest engine (`nautechsystems/nautilus_trader` LGPL-3.0 — proses eksternal saja)
- `openbb` ← data pasar (`openbb` SDK MIT)
- **Aturan keras finance:** riset/analisis/backtest/paper-trading SAJA; DILARANG order trading live otomatis (di setiap SKILL.md + ditolak di kode `daemon/src/tools/quant.rs`)

**Dimiliki Creative Director:**
- `storytelling` / `copywriting` / `content` / `smo-playbook` ← quality bar: Golden Circle + 13-item quality bar (referensi @thechrisdo, @marketingharry)
- Tool skills: `canva`, `affinity`, `open-creator` (referensi: `~/workspace/referensi/open-creator/`), `voice-studio`, `hyperframes` (referensi: skill `local_hyperframes`), `instagram-cli` (draft-only)

**Dimiliki Riset:** channel riset via Agent-Reach (web, YouTube, RSS, GitHub publik).
**Dimiliki QC:** rubrik penilaian `qc` (skor 0–100 + alasan).

Skill baru dibuat via agen (pola OpenBot `create_skill`): folder + `SKILL.md` + `example-prompt` di frontmatter. Menambah playbook baru = menambah skill, BUKAN menambah agen (lihat rule.md #16).

---

## e. Aturan keras (tidak bisa ditawar)

1. Jangan salin kode OpenBot (PolyForm Noncommercial) — pola arsitektur boleh, kode harus clean-room.
2. Kredensial/API key tidak pernah di-commit ke git (OS keychain / env lokal; `.env` di gitignore).
3. **TIDAK ADA auto-posting ke Meta** (IG/FB/Threads) — distribusi hanya berupa draft sampai Patria approve manual.
4. Semua deliverable user-facing disimpan ke Google Drive folder MIMI (subfolder per task).
5. Data sensitif tidak diupload ke internet.
6. UI mengikuti `~/workspace/build-rules/ui-rules-for-real-users.md` (readability dulu: body 14–16px, judul section 32px bold, label di atas field, button min 44px, satu primary action per section, satu accent color, hierarki via tipografi/spacing) dan harus lolos anti-pattern Impeccable — tanpa: font Inter untuk segalanya, gradient ungu-biru, kartu-di-dalam-kartu, teks abu di background berwarna, hitam/abu murni. Audit dengan `npx impeccable install`.
7. Setiap fase di-review Patria sebelum lanjut — jangan bercabang, hemat token.

---

## f. Konvensi repo & koding

- Struktur: `daemon/` (Rust headless: executor, IPC server, LLM clients, tool runners, scheduler, migrations), `src-tauri/` (thin IPC client + UI shell), `src/` (frontend TS), `agents/` (definisi agen: system prompt, model, tool allowlist), `skills/`, `pipelines/` (YAML), `docs/` (`status.md`, `rule.md`, `TODO.md`, `PLAN.md`), `.env.example` (tanpa nilai).
- Bahasa dokumen & UI: Indonesia. Bahasa kode & komentar: Inggris.
- Commit: `<tipe>: <ringkasan singkat>` — tipe: feat/fix/docs/refactor/chore; deskripsi boleh Bahasa Indonesia.
- Rust: `cargo fmt` + `clippy`; hindari `unwrap()` di path produksi (gunakan `Result`).
- Frontend: formatter + linter (biome/eslint); TypeScript strict.
- Secret hanya via keychain/env; pastikan `git status` bersih dari secret sebelum commit.
- Setiap fase: update `docs/status.md`.

---

## g. Enam fase build (disesuaikan arsitektur v3; review gate tiap fase)

- **Fase 1 — Fondasi daemon + UI skeleton.** Daemon: skema SQLite 7 tabel, IPC Unix socket, FIFO executor, install/load LaunchAgent. **Provider trait didefinisikan di sini** (interface dulu, implementasi menyusul). UI: 6 views skeleton + IPC client + realtime subscription, UI rules diterapkan. *Bukti kunci: install → daemon jalan via launchd → quit aplikasi → daemon TETAP jalan.* *Gate: Patria cek dashboard + buktikan quit ≠ stop.*
- **Fase 2 — Orkestrasi inti + provider.** Pipeline YAML runner, handoff, implementasi provider Gemini + **Muse** (verifikasi region Indonesia; fallback Gemini bila belum tersedia), routing per-agen + fallback chain dasar, 1 pipeline vertikal end-to-end (Riset → Strategi Bisnis → Creative Director → QC → Approval → Deliverable draft), approval inbox minimal + notifikasi native macOS. *Gate: 1 job nyata berjalan penuh + tap approve Patria.*
- **Fase 3 — Skills & knowledge.** 6 skills → Strategi Bisnis, 4 skills → Creative Director (knowledge disuling dari 4 dokumen Drive); DeepSeek; OAuth Google Drive sekali (baca). *Gate: output tiap skill di-review Patria.*
- **Fase 4 — Tools produksi.** Diawali readiness check eksplisit; wiring Canva/Affinity/OpenCreator/VoiceStudio/HyperFrames + fallback; Content Production; SMO draft-only. *Gate: 1 reels/carousel jadi dari pipeline.*
- **Fase 5 — Hardening & rilis v1 pribadi.** QC judge penuh, metering + budget caps + view Penggunaan, Providers settings UI, upload Drive, dmg ad-hoc + setup LaunchAgent, security audit (skill `local_security_audit`), audit Impeccable. *Gate: v1 rilis di MacBook Patria.*
- **Fase 6 — Komersialisasi (boleh jalan paralel setelah Fase 2).** Account API (repo Workers + D1 + deploy), OTP email + SMTP, Stripe (products, Customer Portal, webhooks), license enforcement + offline grace 30 hari, signing publik (Apple notarization, sertifikat Windows), release pipeline GitHub Actions, auto-updater. *Gate: instalasi dari dmg ter-sign + login OTP + lisensi trial/berbayar berfungsi.*

---

## h. Konvensi status.md / rule.md / TODO.md / PLAN.md

- `docs/status.md` — log kronologis proyek; entri baru di-**APPEND** di bawah dengan header tanggal. Jangan edit entri lama (pelajaran: jangan pakai baris pembuka entri lama sebagai anchor edit).
- `docs/rule.md` — aturan proyek bernomor, satu baris satu aturan; tidak dihapus/diubah tanpa persetujuan Patria.
- `docs/TODO.md` — checklist `- [ ]` per fase; centang saat selesai + catat tanggal.
- `docs/PLAN.md` — rencana fase + arsitektur ringkas; diperbarui setiap review gate.
