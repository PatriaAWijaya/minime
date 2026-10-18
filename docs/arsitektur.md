# Arsitektur Aplikasi — Minime v2

> Disusun 18 Okt 2026 dari keterangan & verdict Patria.
> UI/UX meniru OpenBot sebagai referensi pola tampilan (repo referensi:
> `~/workspace/referensi/openbot-base/`, TIDAK menyalin kode — clean-room).
> Stack final (verdict Patria 21:18): **Tauri 2 + Svelte 5** (ramuan sendiri).

---

## 1. Visi

Minime v2 adalah **aplikasi pembuat agen tambahan** — intinya membantu
pengguna membuat agen-agen baru yang membuat model Muse miliknya lebih
efektif dan efisien. Model AI adalah pilihan pengguna (Patria: **Gemini 3.8**
berbayar + model gratis lokal).

Fokus utama: **kebutuhan kreatif untuk bisnis yang berpijak pada data
akurat**. Terbuka untuk keperluan lain — kerangkanya umum, isinya bisa
diisi ulang oleh pengguna.

## 2. Prinsip desain (catatan Patria)

1. **Compact tapi powerful.** Sedikit klik, tanpa jargon, tapi mampu
   menangani pekerjaan berat di balik layar.
2. **Minim coding, intuitif untuk pemula.** Pengaturan lanjutan
   disembunyikan di balik default yang cerdas; label berbahasa sederhana.
3. **Model-agnostic.** Default Patria = Gemini 3.8; model lokal gratis
   sebagai fallback; pengguna lain bebas mendaftarkan modelnya sendiri.
   API key disimpan di OS keychain — tidak pernah di git, tidak lewat chat.
4. **Data-grounded creative.** Output kreatif harus berpijak pada data /
   sumber terverifikasi, bukan karangan.
5. **Local-first.** Data pengguna di mesin sendiri (SQLite lokal).
6. **Human-in-the-loop.** Approval pengguna wajib sebelum aksi eksternal
   apa pun. **Tidak ada auto-posting** (aturan berdiri sejak 28 Sep 2026).

## 3. Stack teknologi

| Lapisan | Pilihan | Alasan |
|---|---|---|
| Desktop shell | Tauri 2 | Ringan (~10MB), hemat RAM, backend Rust menyatu — terbukti di v1 |
| UI | Svelte 5 + Vite | Kode ringkas, reaktivitas bawaan, build cepat |
| Backend | Rust (dalam Tauri) | Aman, cepat; command via Tauri IPC |
| Persistensi | SQLite (rusqlite) | Lokal, tanpa server tambahan |
| i18n | Implementasi sendiri | 3 bahasa: Indonesia (default), Inggris, Mandarin |
| Target rilis | macOS (dmg), Windows (exe) | Installer mudah & intuitif |

**Tidak ada daemon terpisah di awal v2** — disederhanakan dari v1;
backend Rust menyatu di proses Tauri.

## 4. Lapisan sistem

```
┌──────────────────────────────────────────────────────────────┐
│ UI — Svelte 5                                                 │
│ ┌─────────┐ ┌──────────┐ ┌────────────────┐ ┌───────────────┐  │
│ │Sidebar  │ │Daftar    │ │ Main: chat +   │ │Monitor tugas  │  │
│ │workflow │ │6 agen    │ │ delivery       │ │berjalan       │  │
│ └─────────┘ └──────────┘ └────────────────┘ └───────────────┘  │
│ Pengaturan · Penggunaan · Peralatan                            │
└──────────────────────────┬───────────────────────────────────┘
                           │ Tauri IPC (commands)
┌──────────────────────────▼───────────────────────────────────┐
│ Backend — Rust                                                │
│ ┌──────────┐ ┌────────────┐ ┌────────┐ ┌──────────────────┐   │
│ │Agent     │ │Model router│ │Skills  │ │Tools gateway     │   │
│ │runtime   │ │Gemini 3.8  │ │engine  │ │meta/postiz/pireel│   │
│ │(Mimi CIO │ │default +   │ │        │ │+ approval gate   │   │
│ │+ 5 agen) │ │lokal gratis│ │        │ │                  │   │
│ └──────────┘ └────────────┘ └────────┘ └──────────────────┘   │
│ SQLite lokal (riwayat chat, tugas, usage, konfigurasi)         │
└──────────────────────────────────────────────────────────────┘
```

### 4.1 Agent runtime
- Setiap agen = definisi (nama, peran, system prompt, model pilihan,
  skills, tools allowlist, budget) + state (status: bekerja/siap/perhatian).
- **Mimi (CIO, model Muse)** adalah orkestrator: menerima permintaan
  pengguna, memecah jadi tugas, mendelegasikan ke agen yang tepat,
  menggabungkan hasil, dan meminta approval pengguna.

### 4.2 Model router (keputusan Patria 18 Okt 22:40)
- **Gratis: Ollama = default**, OpenCode = fallback (pola UI Providers
  meniru layar Providers OpenBot: daftar radio, status koneksi,
  penyedia custom, deteksi lokal otomatis).
- **Berbayar: Muse = default**, fallback model populer lain
  (Gemini 3.8, DeepSeek).
- **RouterKu (routerku.com, keputusan Patria 19 Okt): gateway AI Indonesia
  kompatibel OpenAI** — satu API key untuk Claude, GPT, Gemini, Grok,
  DeepSeek, Kimi, GLM (16+ model); bayar Rupiah via QRIS mulai Rp20rb,
  tanpa langganan, saldo tak hangus. Dipakai sebagai **gerbang model
  berbayar** di router (cukup ganti base URL + API key).
- Routing per-agen: tiap agen punya model default yang bisa diganti.
- Fallback berlapis: utama → cadangan → custom pengguna.
- Metering: catat token & estimasi biaya per agen; budget cap opsional
  (peringatan saat mendekati batas).

### 4.3 Skills engine
- Skill = playbook terstruktur (diadaptasi dari `skills/` v1 di repo):
  prosedur kerja + quality bar + contoh output.
- Skill dimuat on-demand per agen; pengguna bisa menambah skill sendiri.

### 4.4 Tools gateway
- Koneksi ke aplikasi/peralatan luar: **Meta** (selalu draft-only),
  **Postiz/Viralin** (selalu draft-only), **Pireel** (editing video),
  dan peralatan lain yang ditambahkan nanti.
- Semua aksi eksternal melewati **approval gate** — tidak ada yang jalan
  otomatis tanpa vonis pengguna.

## 5. Roster agen (seed)

| Agen | Peran |
|---|---|
| **Mimi** (CIO) | Orkestrator tim; model Muse |
| **Kreatif** | Merencanakan, membuat, mendistribusikan materi kreatif via channel yang tersedia |
| **Riset** | Riset untuk pengambilan keputusan bisnis |
| **Finance** | Analisa finansial mikro & makro, saran finansial, bantu kebutuhan akuntansi |
| **Bisnis Strategis** | Perhitungan bisnis, brand, strategi marketing, sales & closing |
| **Code Builder** | Membantu membuat aplikasi tambahan bila diperlukan |

- Setiap agen setara **senior manager**: bisa mengakses peralatan/aplikasi
  lain sesuai allowlist-nya.
- **Pengguna bisa menambah agen sendiri** (nama, peran, model, skills,
  tools) — inilah inti "aplikasi pembuat agen".
- Avatar: badge inisial placeholder (avatar karakter v1 ditolak Patria;
  arah baru menunggu keputusannya).

## 6. Bahasa

Indonesia (default), Inggris, Mandarin — ganti di Pengaturan, tersimpan
di perangkat.

## 7. Keamanan & aturan berdiri

- Kredensial hanya di OS keychain (`com.minime.app`), tidak di git.
- Tidak ada auto-posting ke Meta (IG/FB/Threads) — distribusi selalu
  draft + approval.
- Tidak ada live-trading otomatis.
- Deliverable dokumen → folder MIMI di Google Drive (bila pengguna
  menghubungkan).
- UI lolos aturan dasar (body 14–16px, tombol min 44px, satu accent color)
  dan anti-pattern Impeccable.
