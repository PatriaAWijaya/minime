# Riset: Alternatif Blotato untuk Distribusi Konten Minime

> Tanggal: 17 Okt 2026 · Perintah Patria ("lakukan") · Status: **riset murni, tidak ada kode diubah**
>
> Kebutuhan Minime: distribusi **draft** ke 9 platform (Instagram, YouTube, TikTok,
> Facebook, Threads, Twitter/X, LinkedIn, Bluesky, Pinterest), dipakai agen via API,
> pola **draft-only + approval gate Patria** (rule #3 — tidak ada auto-posting).

## Ringkasan eksekutif

| | postiz-app (kandidat utama) | cogsend (kandidat kedua) |
|---|---|---|
| Repo | `gitroomhq/postiz-app` | `cogsend/cogsend` |
| Stars | 37.293 | 230 |
| Lisensi | AGPL-3.0 ⚠️ | MIT ✅ |
| Platform | **9/9** ✅ (semua yang dibutuhkan + ekstra) | **5/9** ❌ (tanpa IG, YouTube, TikTok, FB, Pinterest) |
| API untuk agen | REST publik `/public/v1` + Node SDK + n8n node | REST `/api` + MCP server `/api/mcp` |
| Cocok draft-only | ✅ Ya — `type: 'draft'`, ubah status draft → schedule | ✅ Ya — `create_draft` → `schedule_draft` / `publish_draft` |
| Self-host | Berat: app + Postgres + Redis + Temporal + Elasticsearch | Ringan: Cloudflare Workers + D1 + R2 (free tier cukup) |
| Verifikasi | Kode dibaca penuh; **uji hidup GAGAL di VM** (lihat §4) | Kode dibaca; uji hidup tidak dicoba (platform tidak cukup) |

**Rekomendasi: `postiz-app` sebagai pengganti Blotato untuk 6 platform non-Meta** (atau
semua 9 bila Patria mau), dijalankan sebagai **layanan eksternal via API** — bukan
kode yang di-vendor. Koneksi Meta langsung (IG/FB/Threads, sedang di-wire) tetap
jalan sebagai jalur first-party. `cogsend` tidak cukup (hanya 5 platform) — gugur
sebagai pengganti penuh.

---

## 1. postiz-app — evaluasi mendalam

### 1.1 Platform (terverifikasi dari kode, bukan klaim README)

Provider di `libraries/nestjs-libraries/src/integrations/social/`:

| Platform Minime | Provider | Status |
|---|---|---|
| Instagram | `instagram.provider.ts` (+ `instagram.standalone`) | ✅ |
| YouTube | `youtube` | ✅ |
| TikTok | `tiktok.provider.ts` (+ `tiktok.business`) | ✅ |
| Facebook | `facebook` | ✅ |
| Threads | `threads` | ✅ |
| Twitter/X | `x` | ✅ |
| LinkedIn | `linkedin` (+ `linkedin.page`) | ✅ |
| Bluesky | `bluesky.provider.ts` | ✅ |
| Pinterest | `pinterest` | ✅ |

Ekstra: Dribbble, Reddit, Slack, Discord, Mastodon, Telegram, Tumblr, Medium, WordPress, dsb.

### 1.2 API surface (relevan untuk agen Minime)

Base: `POST/GET /public/v1/...` (`apps/backend/src/public-api/routes/v1/public.integrations.controller.ts`).
Auth: header `Authorization: <api-key>` (atau token OAuth `pos_...`) — sederhana, cocok
untuk pola Keychain Minime.

Endpoint kunci untuk pola draft-only + approval:

| Endpoint | Fungsi | Catatan |
|---|---|---|
| `POST /public/v1/upload` / `/upload-from-url` | Upload media (termasuk video) | — |
| `POST /public/v1/posts` | Buat postingan | Body `type: 'draft'` → **validasi longgar** (cukup konten tidak kosong); tanpa `draft` → validasi ketat |
| `GET /public/v1/posts` | Daftar postingan | Untuk monitoring |
| `DELETE /public/v1/posts/:id` | Hapus draft | — |
| (internal) `change.post.status` | Ubah status `draft` ↔ `schedule` | Inilah titik **approval gate**: draft dibuat agen → Patria setujui → status jadi `schedule` → worker mempublish |

Alur yang pas untuk Minime: daemon membuat **draft** via API → muncul di approval
inbox Minime (atau dashboard Postiz) → Patria tekan Setujui → status diubah ke
`schedule` → Postiz mempublish sesuai jadwal. Tidak ada publish otomatis sebelum
approval — rule #3 terjaga.

Tambahan: NodeJS SDK (`@postiz/node`), n8n custom node, integrasi Make.com,
dan klaim "agentic" (siap dipakai Claude/OpenClaw/ChatGPT via API).

### 1.3 Self-host (docker-compose.yaml)

Stack: `postiz-app` + `postgres:17` + `redis:7.2` + `temporalio/auto-setup` +
`elasticsearch:7.17` (+ postgres/redis kedua untuk Temporal, admin tools, UI).
Ini stack **berat** — butuh beberapa GB RAM dan GB image. Cocok untuk VPS kecil
atau MacBook Patria, bukan VM hemat ini.

**Penting — tidak menghilangkan kerja OAuth:** `docker-compose.yaml` tetap meminta
`X_API_KEY`, `FACEBOOK_APP_ID`, `THREADS_APP_ID`, `YOUTUBE_CLIENT_ID`,
`TIKTOK_CLIENT_ID`, `PINTEREST_CLIENT_ID`, `LINKEDIN_CLIENT_ID`, dsb. Artinya
Patria tetap harus membuat **developer app per platform** (termasuk Meta App) —
yang dihemat adalah biaya langganan Blotato, bukan kerja konfigurasi. Inilah
yang dibayar ke Blotato sebagai SaaS: mereka yang mengurus abstraksi ini.

### 1.4 Hasil uji hidup di VM — GAGAL (jujur)

- Docker CLI tersedia (`docker compose v2.40.3`), tetapi **daemon tidak bisa start**:
  `failed to start daemon ... iptables ... missing kernel module` — VM ini tidak
  punya modul kernel nftables/iptables yang dibutuhkan Docker.
- Kalaupun Docker jalan, stack penuh (app + 2× postgres + redis + temporal +
  elasticsearch) melebihi resource VM (2 CPU, ~1 GB RAM bebas, 7 GB disk).
- Backend dijalankan via Node langsung juga tidak realistis (monorepo pnpm +
  butuh Postgres + Redis + Temporal).
- **Kesimpulan:** verifikasi API hidup (1 endpoint draft terpanggil) **belum bisa**
  dilakukan di VM ini. Yang terverifikasi adalah **kontrak API dari kode sumber**
  (§1.2). Uji hidup menunggu: VPS/MacBook Patria dengan Docker yang berfungsi.

### 1.5 Lisensi

`LICENSE` = **GNU AGPL-3.0** (37k★, update 17 Okt 2026 — sangat aktif).

## 2. cogsend — pemeriksaan ringan

- Repo: `cogsend/cogsend`, 230★, **MIT** ✅ (aman untuk target komersil).
- Platform: **hanya 5** — Mastodon, Bluesky, LinkedIn, Threads, X.
  **Tidak ada**: Instagram, YouTube, TikTok, Facebook, Pinterest (terverifikasi:
  tidak ada provider untuk kelimanya di `src/lib/server/providers/`).
- API: `Authorization: Bearer <key>` (atau `X-API-Key`), endpoint `/api/*`;
  workflow draft: `create_draft` → `schedule_draft` / `publish_draft` /
  `cancel_delivery`; ada **MCP server** (`/api/mcp`) untuk agen.
- Self-host: Cloudflare Workers + D1 + R2 — ringan, free tier cukup, deploy
  satu klik. Ada opsi Zernio (pihak ketiga) untuk konek X/Threads/LinkedIn/
  Bluesky tanpa developer app sendiri.
- **Vonis: gugur sebagai pengganti Blotato penuh** — 4 dari 9 platform Minime
  tidak didukung. MIT-nya bersih, tapi fungsinya tidak cukup.

## 3. Tabel perbandingan final

| Kriteria | Blotato (status quo) | postiz-app | cogsend |
|---|---|---|---|
| 9 platform Minime | ✅ | ✅ 9/9 | ❌ 5/9 |
| Draft via API | ✅ | ✅ (`type: 'draft'`) | ✅ (`create_draft`) |
| Approval-gate | ✅ (draft + gate Minime) | ✅ (draft → schedule) | ✅ (schedule/cancel) |
| Video/reels | ✅ | ✅ (upload + provider reels) | ⚠️ (terbatas platform) |
| Biaya | Langganan SaaS | Gratis (self-host) | Gratis (CF free tier) |
| Lisensi | Proprietary | AGPL-3.0 ⚠️ | MIT ✅ |
| Beban self-host | Tidak ada (SaaS) | Berat (7+ container) | Ringan (serverless) |
| Kerja OAuth Patria | Tidak ada | Tetap ada (app per platform) | Sebagian (Zernio opsional) |
| Uji hidup di VM | n/a | ❌ Gagal (Docker tak bisa start) | Tidak dicoba |

## 4. Analisis lisensi AGPL-3.0 untuk Minime komersil

AGPL-3.0 **tidak melarang** pemakaian oleh produk komersil. Yang diatur adalah
**distribusi/modifikasi kode**, bukan pemakaian via jaringan:

- **BOLEH:** menjalankan postiz-app stock (tanpa modifikasi) sebagai **layanan
  eksternal terpisah** (container sendiri), diakses Minime **hanya via REST API**.
  Ini pola yang sama dengan `nautilus_trader` (proses eksternal, bukan link).
  Kewajiban copyleft **tidak menular** ke kode Minime karena tidak ada
  penggabungan/distribusi kode.
- **DILARANG:** mem-vendor/menyalin kode postiz-app ke repo Minime; memodifikasi
  postiz-app lalu mendistribusikannya tanpa menyediakan source modifikasi
  (AGPL §13: bila versi modifikasi dijalankan sebagai layanan jaringan, source
  harus ditawarkan ke pengguna jaringan).
- **Batasan operasional:** gunakan image resmi `ghcr.io/gitroomhq/postiz-app`
  apa adanya; konfigurasi via env saja; bila suatu hari perlu patch, publikasikan
  patch-nya (fork publik + catat di `docs/lisensi-pihak-ketiga.md`).
- Alternatif tanpa urus server: postiz.com versi cloud (berbayar) — bukan
  self-host, bukan masalah lisensi sama sekali.

## 5. Rekomendasi akhir

1. **Pakai `postiz-app` sebagai pengganti Blotato**, dengan dua catatan:
   - Dijalankan sebagai **layanan eksternal** (Docker di VPS/MacBook Patria),
     Minime hanya memanggil `/public/v1` via API key (Keychain).
   - Koneksi **Meta langsung** (IG/FB/Threads) yang sedang di-wire tetap
     dipertahankan sebagai jalur first-party — postiz menjadi pengganti
     Blotato terutama untuk 6 platform non-Meta (atau semua 9 bila Patria
     ingin satu pintu).
2. **Jangan pakai `cogsend`** untuk kebutuhan ini — platform tidak cukup.
3. **Jangan hapus Blotato dulu** — pertahankan sampai postiz terbukti hidup
   dan 1 draft nyata terkirim (uji hidup).
4. Keputusan yang tetap di tangan Patria: (a) setuju pakai postiz-app?
   (b) di-host di mana (VPS/MacBook)? (c) siapa yang membuat developer app
   per platform (kerja OAuth tetap ada)?

## 6. Langkah integrasi (bila Patria setuju)

1. Siapkan host Docker (VPS kecil / MacBook): `docker compose up` dari
   `docker-compose.yaml` resmi; isi env per-platform (Meta App dkk).
2. Buat API key Postiz (dashboard → API access); simpan di Keychain Minime.
3. Wire tool daemon `postiz` (pola `blotato.rs`): op `draft` (wajib),
   `schedule` (hanya dengan approval_id approved), `accounts` read-only;
   `publish` langsung **ditolak di kode** (rule #3).
4. Update `pipelines/konten-video-otomatis.yaml` stage distribusi:
   Meta via tool `meta`, 6 non-Meta via tool `postiz`.
5. Uji: 1 draft nyata per platform → approval → schedule → verifikasi terbit.
6. Setelah stabil: nonaktifkan Blotato; catat di `docs/status.md` + commit.
