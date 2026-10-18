# rule.md — Minime

> Aturan proyek bernomor. Satu baris satu aturan. TIDAK dihapus/diubah tanpa persetujuan Patria.

1. Dilarang menyalin kode OpenBot (lisensi PolyForm Noncommercial) — pola arsitektur boleh diadopsi, implementasi harus clean-room.
2. Kredensial/API key HANYA di macOS Keychain — dilarang di git, di plaintext, di log, dan di file dokumen.
3. TIDAK ADA auto-posting ke Meta (IG/FB/Threads) — distribusi hanya berupa draft sampai Patria menyetujui manual.
4. Semua deliverable user-facing disimpan ke Google Drive folder MIMI (subfolder per task).
5. Data sensitif tidak diupload ke internet.
6. UI mengikuti `~/workspace/build-rules/ui-rules-for-real-users.md` dan harus lolos anti-pattern Impeccable (tanpa: font Inter untuk segalanya, gradient ungu-biru, kartu-di-dalam-kartu, teks abu di background berwarna, hitam/abu murni).
7. Setiap fase di-review Patria sebelum lanjut — dilarang memulai fase berikut tanpa persetujuan eksplisit.
8. `docs/status.md` bersifat append-only — dilarang mengedit entri lama.
9. `docs/rule.md` tidak boleh diubah/dihapus tanpa persetujuan Patria.
10. Format commit: `<tipe>: <ringkasan singkat>` (tipe: feat/fix/docs/refactor/chore); `git status` harus bersih dari secret sebelum commit.
11. Rust: `cargo fmt` + `clippy` bersih; dilarang `unwrap()` di path produksi (gunakan `Result`). Frontend: TypeScript strict + formatter/linter.
12. Aksi eksternal baru di luar daftar tools yang disetujui wajib konfirmasi Patria dulu.
13. Bahasa Indonesia untuk UI dan dokumen user-facing; bahasa Inggris untuk kode dan komentar.
14. Dilarang bercabang antar fase (hemat token/konteks) — satu fase selesai dan gate lolos, baru lanjut.
15. Setiap LLM call yang memproses data user harus tercatat di tabel `runs` (observability).
16. Prinsip agen senior: sedikit agen setara senior manager LEBIH BAIK daripada banyak agen junior — setiap agen wajib punya playbook/SOP (skills), toolbox jelas (allowlist), proof of work (tercatat di DB), quality bar output, dan kewajiban eskalasi ke Patria bila ragu. Menambah kemampuan = menambah skill, BUKAN menambah agen.
17. Menutup/men-quit aplikasi TIDAK menghentikan agen — daemon tetap berjalan via launchd (RunAtLoad + KeepAlive). Mematikan agen hanya lewat perintah eksplisit ("Hentikan daemon").
18. Hanya daemon yang menulis database (single-writer) — UI dilarang menyentuh file DB langsung; semua komunikasi via IPC lokal (Unix socket).
19. Semua API key milik user (provider model: Muse, Anthropic, OpenAI, Gemini, DeepSeek, dll.) HANYA di macOS Keychain — dilarang di git, plaintext, log, dan dokumen.
20. Kode OTP tidak pernah disimpan / di-log dalam bentuk plaintext di mana pun — hanya hash + masa berlaku; rate limit per email/IP/kirim ulang; mode dev boleh expose kode di response API, production DILARANG keras.
21. Setiap rilis publik (dmg/exe/AppImage) WAJIB code-signing (+ notarization untuk macOS); v1 pemakaian pribadi boleh ad-hoc signing.
22. Secret backend (SMTP, Stripe, wrangler) hanya via secret manager (wrangler secrets / GitHub Environments) — dilarang di repo dan di argumen proses.
