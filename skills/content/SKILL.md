---
name: content
description: Playbook content — content pilar, kalender editorial, repurpose, standar produksi konten.
owner_agent: creative-director
version: 1.0.0
example-prompt: Susun content pilar dan kalender 2 minggu.
---

# content

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
> Sumber: "Playbook Guidelines — Marketing" Bab 5.4 (content marketing) & Bab 11
> (strategi channel); pola kualitas dari 4 konten acuan IG.

## Kapan dipakai

Perencanaan dan produksi konten: content pilar, kalender editorial, repurpose,
standar per format (reels, carousel, stories, long-form).

## SOP

### A. Content pilar (3–5 pilar, tetapkan dulu)
1. Turunkan dari WHY brand dan pain point audiens (lihat `storytelling`, `marketing-playbook`).
   Contoh pilar: edukasi, bukti/sosial proof, behind-the-scenes, opini, offer.
2. Setiap pilar punya tujuan funnel yang jelas (awareness / trust / conversion).
3. Rasio format (Instagram): 50% Reels / 30% carousel / 20% Stories + Live.
   Prinsip umum: discovery (short video) → kedalaman (carousel/long-form) → kedekatan (stories/live).

### B. Kalender editorial
1. Berbasis keyword dan pain point audiens, bukan "yang penting posting".
2. Satu konten besar → repurpose ke banyak format
   (artikel/long-form → carousel → short video → email/WA).
3. Operasional minimum: IG 3–4 feed/minggu + Stories hampir harian; TikTok 1–3 video/hari
   bila dipakai; YouTube 1 long-form/minggu cukup (produksi batch).
   Konsistensi 3 bulan > sprint 2 minggu. TikTok: standar uji 90 hari. YouTube: ±50 video
   agar algoritma memahami channel.
4. Setiap item kalender: pilar, format, hook (ditulis dulu!), CTA, tujuan funnel.

### C. Standar per format
- **Reels/short video:** mesin discovery (70–90% penonton non-followers). Hook 0–3 detik brutal,
  retention >50%, SATU pesan per video, caption pendek + CTA komen. Ucapkan keyword niche
  di 5 detik pertama (SEO sosial).
- **Carousel:** mesin kedalaman (save & share tertinggi). Slide 1 = hook; tengah = daging
  (1 gagasan/slide); akhir = CTA.
- **Stories:** mesin kedekatan (poll, Q&A, quiz, countdown, soft selling).
- **Long-form (YouTube/artikel):** mesin otoritas. Packaging = 80% kesuksesan
  (judul hasil spesifik + penasaran; thumbnail maks 3 elemen). Struktur: hook 30 detik →
  konteks → daging (3–5 poin) → ringkasan → satu CTA.

### D. Distribusi peran funnel
TikTok/IG mendatangkan → YouTube meyakinkan (perpustakaan otoritas) → WhatsApp menutup.
Satu arsitektur pesan untuk semua channel.

### E. Routing produksi video (keputusan arsitektur Patria 17 Okt)

**Veo dan OpenCreator dipasang PARALEL — tidak ada yang dicopot.** Pilih per kebutuhan:

| Kebutuhan | Pakai | Catatan |
|---|---|---|
| Footage generatif sinematik, b-roll AI, visual eye-catching dari prompt teks | **Veo** — tool `veo3` op `render` (Gemini API → HyperFrames lokal; fal.ai DIHAPUS 17 Okt) | Dipakai pipeline `konten-video-otomatis`. Model default `veo-3.1-fast-generate-preview` (1080p/8 dtk/16:9). Perkiraan biaya: ±$0,96/klip (harga publik Google, bukan janji). Key: Keychain akun `gemini` |
| Naskah → video + voiceover Bahasa Indonesia; eksplainer, konten bernarasi, faceless video | **OpenCreator** — skill `open-creator` (sidecar) | Live test menunggu kunci Patria |
| Tanpa kunci / offline / cepat | **HyperFrames** (HTML→MP4) atau **manimgx** (animasi edukasi) | Fallback lokal otomatis bila backend AI tidak tersedia |

Aturan: render AI (Veo/OpenCreator) WAJIB lewat QC sebelum distribusi; distribusi tetap draft-only + approval Patria (rule #3).

## Toolbox

- Template content pilar: nama pilar → tujuan funnel → contoh topik (10 per pilar)
- Template kalender: tanggal → pilar → format → hook → CTA → status
- Alur repurpose: 1 konten besar → carousel → short video → email/WA → stories
- Checklist pra-publish: hook tertulis? 1 pesan? CTA sesuai tujuan? klaim terverifikasi?
  caption + alt text + keyword? (lihat `copywriting`)

## Quality bar

- Setiap konten punya pilar, hook tertulis, dan SATU tujuan funnel.
- Hook kuat; satu pesan per konten; CTA sesuai tujuan.
- Repurpose: tidak ada konten besar yang hanya dipakai sekali.
- Konsistensi jadwal > intensitas sesaat.
- Bukan vanity metrics: likes/views tanpa follow/klik/inbound = gagal membangun.
- Larangan: SARA & politik; klaim tak terverifikasi; cross-posting mentah antar platform.

## Trigger eskalasi — WAJIB ke Patria bila

- Topik sensitif atau berpotensi kontroversi.
- Klaim/dampak yang belum terverifikasi.
- Perubahan pilar atau arah editorial (keputusan strategis).
- Kolaborasi dengan pihak eksternal (kreator, brand lain).

## Proof of work

Setiap output WAJIB mencantumkan: pilar + tujuan funnel per konten, hook tertulis,
format & CTA, rencana repurpose, dan checklist pra-publish (tercentang).
