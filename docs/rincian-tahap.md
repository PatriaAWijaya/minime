# Rincian Tahap — Minime v2

> Breakdown detail per fase, dari awal sampai akhir.
> Disusun 18 Okt 2026. Gate tiap tahap = review + vonis Patria.

---

## Tahap 1 — UI/UX klik-tayang ✅ SELESAI

**Tujuan:** aplikasi bisa dibuka, diklik, dan dinilai tampilannya —
tanpa fungsi AI sungguhan.

### 1.1 Scaffold proyek
- Struktur `app/`: frontend Svelte 5 + Vite, backend Tauri 2 (Rust)
- Konfigurasi: `tauri.conf.json` (id `id.minime.app`, window 1280×800),
  `capabilities/default.json`, `build.rs`
- Ikon app di-generate dari `assets/app-icon.jpg`

### 1.2 Layout 4 kolom (pola OpenBot, kode sendiri)
- Sidebar workflow: logo + tombol + Tugas Baru + navigasi
- Daftar agen: 6 agen + badge inisial + status (kuning/hijau/merah)
- Main: chat + panel delivery
- Monitor: daftar tugas berjalan + progress bar

### 1.3 i18n & tema
- 3 bahasa: Indonesia (default), Inggris, Mandarin — switcher di Pengaturan
- Tema dark minimalis: satu accent (emerald), font system, body 15px,
  tombol min 44px, label di atas field

### 1.4 Prinsip compact-but-powerful
- Satu primary action per section; label sederhana ("Jenis AI");
  pengaturan lanjutan di balik `<details>`; tombol peralatan disabled
  sampai Tahap 3

### Deliverable
- Kode di `app/` (commit `a8ffd11`), `npm run build` sukses, nol warning
- **Gate TERBUKA:** vonis Patria atas tampilan

---

## Tahap 2 — Fitur + skills agen

**Tujuan:** 6 agen benar-benar bekerja — chat dijawab AI sungguhan
dengan model pilihan pengguna.

### 2.1 Model router (Rust)
- Pendaftaran model: Gemini 3.8 (default Patria, key di keychain),
  model lokal gratis (Ollama/LM Studio), model custom pengguna
- Routing per-agen: tiap agen punya model default yang bisa diganti
- Fallback berlapis: utama → lokal gratis → custom
- Metering: catat token & estimasi biaya per agen; budget cap opsional
  (peringatan saat mendekati batas)

### 2.2 Agent runtime (Rust + IPC)
- Definisi agen: nama, peran, system prompt, model, skills, tools allowlist
- Mimi (CIO): terima permintaan → pecah jadi tugas → delegasikan →
  gabungkan hasil → cek quality bar → minta approval
- Chat real: streaming respons, render markdown, riwayat per agen
- Status agen live: bekerja / siap / butuh perhatian
- Monitor tugas: FIFO per agen, pause / resume / batal

### 2.3 Skills + quality bar
- Adaptasi playbook v1 dari `skills/` di repo menjadi skill terstruktur
- Skill dimuat on-demand sesuai agen & tugas
- Quality bar per agen didefinisikan di playbook-nya (kerangka umum
  sudah ada di `docs/framework.md` bagian 6)

### 2.4 Persistensi SQLite
- Tabel: riwayat chat, tugas, penggunaan model, konfigurasi,
  definisi agen custom
- Semua lokal di mesin pengguna

### 2.5 Agen custom (UI)
- Form "Agen Baru": nama, peran, model, skills, tools, budget
- Agen baru langsung muncul di daftar & bisa didelegasikan Mimi

### Deliverable
- Chat berfungsi dengan AI sungguhan; Mimi bisa mendelegasikan ke 5 agen;
  usage tercatat; agen custom bisa dibuat tanpa coding
- **Gate:** review + vonis Patria (uji 1 alur: minta → delegasi → hasil)

---

## Tahap 3 — Tools & distribusi

**Tujuan:** agen bisa memakai peralatan luar — selalu lewat approval
pengguna.

### 3.1 Tools gateway (Rust)
- Kerangka konektor: daftar peralatan, status koneksi, allowlist per agen
- Kredensial via OS keychain; tidak ada secret di kode/config

### 3.2 Meta — draft-only
- Baca profil & insights; **buat draft konten, tidak pernah auto-post**
- Setiap distribusi = draft + tombol approval Patria

### 3.3 Postiz / Viralin — draft-only
- Penjadwalan konten sebagai draft; kirim hanya setelah approval
- Channel yang tersambung tampil di UI Peralatan

### 3.4 Pireel
- Koneksi cloud MCP + OAuth; dipakai agen Kreatif untuk editing/rakit video
- Biaya kredit tercatat di Penggunaan

### 3.5 Approval gate + notifikasi
- Setiap aksi eksternal memicu kartu approval (setuju / revisi / tolak)
- Notifikasi native saat tugas selesai / butuh perhatian (status merah)

### 3.6 Upload Drive folder MIMI
- Deliverable dokumen bisa diunggah ke folder MIMI di Google Drive
  (setelah pengguna menghubungkan akun)

### Deliverable
- 1 pipeline end-to-end: permintaan → agen → tools → approval → hasil
- **Gate:** pipeline E2E jalan + vonis Patria

---

## Tahap 4 — Rilis macOS + Windows

**Tujuan:** installer yang bisa dipasang pengguna pemula tanpa bantuan.

### 4.1 Build
- macOS: `.dmg` (Tauri bundler, target aarch64 + x86_64 bila memungkinkan)
- Windows: `.exe` (NSIS/WiX via Tauri)
- Verifikasi kompilasi Rust penuh di MacBook (VM tidak bisa: tanpa
  header dev GTK/WebKit)

### 4.2 Installer UX
- Instalasi sedikit klik; bahasa mengikuti pilihan (ID default)
- Panduan pertama-buka: hubungkan model (Gemini key / lokal gratis)
  dalam ≤ 3 langkah

### 4.3 Audit
- Security audit: secret tidak bocor, IPC aman, keychain benar,
  tidak ada auto-post tersembunyi
- Impeccable audit: lolos anti-pattern UI

### 4.4 Uji instalasi bersih
- Pasang di mesin baru (Mac & Windows): buka → hubungkan model →
  buat tugas → hasil keluar

### 4.5 Scoring kinerja (catatan Patria 19 Okt sore)
Setelah kompilasi berhasil, aplikasi dinilai kinerjanya sebelum rilis:
- **Internal (benchmark suite):** startup time (`hyperfine`), RAM/CPU idle
  & aktif, ukuran bundle, latensi IPC, waktu respons chat (Ollama lokal)
- **Otomatis:** Lighthouse CI sebagai gate tiap rilis
- **Pihak ketiga kompeten:** satu kali audit independen oleh konsultan
  performance sebelum rilis publik (hasil jadi vonis tambahan selain
  vonis Patria)
- Target selaras prinsip #7: ringan untuk pemula, bertenaga untuk Patria

### Deliverable
- File `Minime-v2.dmg` + `Minime-v2.exe` + catatan rilis
- **Gate:** instalasi bersih lolos + vonis rilis Patria

---

## Ringkasan dependensi

```
Tahap 1 (UI) ──► Tahap 2 (runtime+model) ──► Tahap 3 (tools) ──► Tahap 4 (rilis)
   ✅ selesai        ▲ gate Patria              ▲ gate Patria       ▲ gate Patria
```

Tahap 2 butuh vonis Tahap 1. Tahap 3 butuh runtime Tahap 2.
Tahap 4 butuh semua fungsi selesai + audit.
