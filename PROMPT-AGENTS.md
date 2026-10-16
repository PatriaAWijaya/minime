# PROMPT — Generate AGENTS.md Proyek "MINIME"

> Prompt ini dijalankan SATU KALI oleh build agent sebelum build apapun dimulai.
> Output: file `AGENTS.md` di root proyek `~/workspace/minime/`.
> Setelah AGENTS.md selesai dan di-review Patria, barulah lanjut ke brainstorm → status.md/rule.md → fase build.
> Berlaku protokol build Patria (framework @aabiizaa + @ficky.dev): jangan bercabang antar fase, review tiap fase.

## 1. Visi proyek

**Minime**: local-first workspace untuk AI teammates yang persisten — terinspirasi arsitektur `nightly-labs/openbot`, BUKAN fork. OpenBot berlisensi PolyForm Noncommercial: **dilarang menyalin kodenya**; yang diadopsi hanya pola arsitektur. Implementasi clean-room.

Target pengguna: Patria, satu user, Indonesia. Bahasa Indonesia untuk semua dokumen dan UI user-facing.

Pola arsitektur yang diadopsi dari OpenBot:
- Setiap agen: workspace sendiri, conversation sendiri, antrean task FIFO (pause/resume/cancel, crash-safe).
- Agent-to-agent messaging untuk handoff antar tahap.
- Skills library: playbook sebagai skill yang bisa dipasang ke agen.
- Human-in-the-loop: tidak ada aksi eksternal (posting, kirim, publish) tanpa approval Patria.
- Local-first: data di mesin sendiri; kredensial tidak pernah di-commit ke git.

## 2. Alur kerja (final, dari diagram Patria)

Mimi (orkestrator) → Riset → Strategi Bisnis → 6 Playbook (Branding, Marketing, Selling, Operational, Finance, Code Builder) → [Branding/Marketing/Selling/Operational] → Creative → 4 Playbook (Storytelling, Copywriting, Content, SMO) → Content Production (Image, Carousel, Reels) → Tools (Canva, Affinity, OpenCreator, VoiceStudio) → QC → Approval (Patria) → Deliverable Output.

Cabang: Code Builder → UI/UX → QC. SMO → (Page Optimization, Content Pilar, Content Planning, Content Distribution, Engagement) → QC. Deliverable Output memberi feedback loop ke tahap distribusi/engagement.

## 3. Roster agen & model AI

- **Orkestrator (Mimi)**: router task, antrean kerja, status realtime di dashboard. Model: Gemini (`custom.gemini`, Secure Vault).
- **Riset**: riset web/YouTube/RSS (channel Agent-Reach). Model: Gemini (long-context).
- **Strategi Bisnis + 6 playbook**: penalaran dalam. Model: DeepSeek (`custom.legacy-deepseek`). Knowledge base: 4 dokumen Playbook Bisnis di Google Drive — Branding (`14bAxNF_vCJjyd9ZzUt0Z85iVlZGzDU4BmEHxFjhp7mY`), Marketing (`16Jctf4wt11InFccmN1kgSFKEFo9zZAVz127Ite2JClo`), Selling (`1hMOVWhdKQGFHlIw2BSJXEQR0Q4R3CFF6C1zTQD8mGrA`), Operasional (`1exok4QG8YHfOyxrN3JIj3fzzLgCD6IrzHpwhMcR9vCM`); dokumen Finance & Code Builder menyusul.
- **Creative**: konsolidasi brief dari 4 playbook. Model: Gemini.
- **Storytelling / Copywriting / Content / SMO**: drafting. Model: Gemini. Quality bar: struktur naskah Golden Circle + 13-item quality bar dari postingan referensi (@thechrisdo, @marketingharry).
- **Content Production** (Image, Carousel, Reels): eksekusi via Tools.
- **QC**: judge. Model: Gemini-as-judge (atau `jev` bila kunci OpenRouter tersedia).
- **Approval**: manusia (Patria) — wajib, tidak bisa diwakilkan.

## 4. Tools & koneksi

- **Tools**: Canva (CLI), Affinity (bridge MacBook), OpenCreator (render video 9:16: subtitle→TTS→render; clone ada di `~/workspace/referensi/open-creator/`), VoiceStudio (default voiceover), HyperFrames (alternatif render lokal), Google Drive (deliverable → folder MIMI).
- **Koneksi**: `custom.gemini` & `custom.legacy-deepseek` (Secure Vault), IG/Threads/WA/Messenger, Meta Business Manager.

## 5. Aturan keras (tidak bisa ditawar)

1. Kredensial/API key tidak pernah di-commit ke git (pakai Secure Vault / env lokal).
2. **TIDAK ADA auto-posting ke Meta** (IG/FB/Threads) — distribusi hanya berupa draft sampai Patria approve manual.
3. Semua deliverable user-facing disimpan ke Google Drive folder MIMI (subfolder per task).
4. Data sensitif tidak diupload ke internet.
5. UI: ikuti `~/workspace/build-rules/ui-rules-for-real-users.md` (readability dulu: body 14–16px, judul section 32px bold, label di atas field, button min 44px, satu primary action per section, satu accent color, hierarki via tipografi/spacing) dan harus lolos anti-pattern Impeccable (`~/workspace/kantor-ai-agent/referensi/impeccable-skill.md`).
6. Setiap fase di-review Patria sebelum lanjut — jangan bercabang, hemat token.

## 6. Tugasmu sekarang

1. Baca dulu SEMUA file wajib sebelum menulis: diagram alur (minta ke Mimi bila belum ada), `~/workspace/build-rules/ui-rules-for-real-users.md`, `~/workspace/kantor-ai-agent/referensi/impeccable-skill.md`, dan ringkasan evaluasi OpenBot dari Mimi.
2. Tulis `AGENTS.md` di root proyek `~/workspace/minime/` dengan struktur:
   a. Ringkasan proyek & definisi selesai (definition of done).
   b. Tech stack — platform: **desktop app** (keputusan Patria 16 Okt 2026). Rekomendasi: **Tauri** (ringan, binary kecil, backend Rust, webview native — cocok untuk local-first; UI dashboard tetap web tech sehingga iterasi cepat). Electron sebagai alternatif bila ada alasan teknis kuat — justifikasi tertulis bila memilihnya. Arsitektur: backend lokal + frontend webview, pola terinspirasi OpenBot tapi implementasi clean-room (tanpa menyalin kode OpenBot).
   c. Roster agen + model AI + tanggung jawab tiap agen.
   d. Skills/playbooks + sumber knowledge (termasuk ID dokumen Drive).
   e. Aturan keras (§5 di atas).
   f. Konvensi repo & koding.
   g. Usulan fase build (maksimal 5 fase) dengan review gate tiap fase.
   h. Konvensi status.md / rule.md / TODO.md / PLAN.md.
3. Setelah AGENTS.md selesai: **BERHENTI**. Jangan lanjut ke brainstorm/build sebelum Patria me-review.
