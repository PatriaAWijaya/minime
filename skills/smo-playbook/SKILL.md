---
name: smo-playbook
description: "Playbook SMO — optimasi profil/halaman, strategi per channel, engagement. ATURAN KERAS: distribusi HANYA draft, dilarang auto-posting."
owner_agent: creative-director
version: 1.0.0
example-prompt: Rancang rencana distribusi konten 1 minggu (draft saja).
---

# smo-playbook

> Kontrak agen senior. SOP WAJIB diikuti; toolbox adalah alat yang boleh dipakai;
> quality bar adalah standar kelulusan; bila ragu → eskalasi ke Patria (Approval).
>
> ⛔ **ATURAN KERAS (rule #3): TIDAK ADA auto-posting ke Meta (IG/FB/Threads) atau platform
> apa pun. Output distribusi SELALU berupa draft untuk approval manual Patria.**
> Skill ini merancang & menyiapkan; yang menekan tombol publish adalah manusia.
>
> Sumber: "Playbook Guidelines — Marketing" Bab 11 (strategi 6 channel).

## Kapan dipakai

Optimasi halaman/profil, rencana distribusi konten (draft), dan engagement.
Bukan untuk mempublish.

## SOP

### A. Optimasi profil/halaman (fondasi — kerjakan dulu)
- **Instagram:** name field = keyword (satu-satunya bagian terindeks pencarian);
  bio = siapa + untuk siapa + hasil + CTA (maks 150 karakter); highlights sebagai landing page
  (Tentang, Testimoni, Layanan, FAQ); 3 pin terbaik.
- **TikTok:** name field keyword; bio 80 karakter (siapa + hasil + CTA); satu link.
- **YouTube:** packaging = 80% kesuksesan — judul (hasil spesifik + penasaran) + thumbnail
  (maks 3 elemen, teks besar, wajah ekspresif); playlist bertema sebagai kurikulum channel.
- **LinkedIn:** headline 220 karakter = proposisi nilai + keyword (bukan jabatan);
  banner = proposisi + CTA; About orang pertama (cerita, angka & milestone, CTA di akhir);
  Featured = portofolio; Experience dengan hasil terukur.
- **X/Threads:** bio 160 karakter (siapa + sudut pandang + kredibilitas); pin thread terbaik
  sebagai portofolio. Threads: tanpa budaya hashtag (pakai topic tag).

### B. Strategi per channel (untuk rencana distribusi draft)
- **Instagram:** sinyal terkuat = shares/sends → saves → komentar bermakna → watch time → likes.
  Reels = discovery; carousel = kedalaman; Stories = kedekatan; Live = kepercayaan.
  Balas komentar & DM <24 jam. Growth: collab post, remix reels viral niche, boost HANYA
  untuk konten organik yang terbukti menang. Kesalahan: cross-posting mentah dari TikTok,
  bio puitis tanpa penjelasan.
- **TikTok:** interest graph — setiap video diuji ke batch kecil baru didorong.
  Hook 0–3 detik brutal (target 70%+ bertahan 3 detik). Tren = bahan bakar, bukan tujuan.
  Stitch/Duet untuk menumpang percakapan; reply with video untuk loop engagement.
  Jangan hapus video sepi (bisa meledak belakangan). Kesalahan: terlalu korporat,
  berhenti sebelum 90 hari.
- **YouTube:** dua mesin — Search (intent kuat, konversi tinggi) & Browse/Suggested
  (CTR + retensi menentukan 80% distribusi). Long-form = otoritas; Shorts = discovery
  (arahkan ke long-form); 1 long-form → 3–5 Shorts + carousel IG. Retention editing:
  pola visual berubah tiap 20–40 detik. Ukur: CTR (sehat 4–10%), retention graph.
- **LinkedIn:** profil personal menjangkau 10–50x company page. Link eksternal di badan
  postingan dipenalti (taruh di komentar pertama). Text post + foto = reach tertinggi
  (cerita personal 2–3 baris → insight profesional → pertanyaan penutup). 2–3 post/minggu
  cukup. Jangan jualan di DM pertama; engage di konten target sebelum DM.
- **X/Threads:** yang menang = pikiran terpukul paling tepat, bukan produksi termewah.
  Post teks pendek (satu ide tajam); thread = bedah mini; quote post = menumpang percakapan.
  Reply strategy = lever pertumbuhan tercepat (balas akun besar niche dengan respons cerdas).
  Ritme: X 3–5/hari; Threads 1–3/hari.
- **WhatsApp:** tanpa algoritma; open rate 90%+. BUKAN untuk mencari orang baru —
  untuk menutup & merawat yang sudah hangat. WAJIB opt-in; broadcast tanpa izin = spam.
  Broadcast 1–2x/minggu maks (list umum); tiap pesan lulus uji "apakah penerima merasa ini
  untuk dia?" Automation: sapaan otomatis, quick replies, drip sequence
  (H+0 sambutan, H+2 edukasi, H+5 testimoni, H+7 penawaran). Database WA tersegmentasi =
  aset termahal (tidak tergantung algoritma siapa pun).

### C. Engagement
- Balas komentar & DM <24 jam (IG); balas komentar dengan video (TikTok).
- Social listening: dari monolog ke dialog — respons komentar, libatkan audiens.
- Jangan pernah: membeli followers/database nomor; broadcast tiap hari seperti feed;
  pesan panjang tanpa CTA.

### D. Output selalu DRAFT
Setiap rencana distribusi menghasilkan: jadwal draft (tanggal, channel, format, copy, aset),
bukan penjadwalan otomatis. Status tiap item: `draft` → menunggu approval Patria →
`approved` (diterbitkan manual oleh manusia).

### E. Distribusi video AI via Blotato (pipeline konten-video-otomatis)
- **9 platform** (sesuai diagram Patria): Instagram, YouTube, TikTok, Facebook,
  Threads, Twitter/X, LinkedIn, Bluesky, Pinterest.
- **Alur:** tool `blotato` op `draft` — upload video ke Blotato (bila API key ada)
  + simpan bundle draft lokal per platform (`drafts/blotato-draft-<platform>-<ts>.json`).
  Op ini TIDAK PERNAH memanggil publish.
- **IG/FB/Threads = manual Patria.** Fungsi publish untuk ketiga platform ini
  DITOLAK di kode tool (rule #3) — draft diserahkan, Patria yang menekan tombol
  di aplikasi masing-masing/Blotato dashboard.
- **6 platform non-Meta:** publish HANYA via op `blotato.publish` dengan
  `approval_id` yang merujuk approval berstatus `approved` di inbox. Tanpa itu
  → ditolak. Tidak ada publish otomatis dari pipeline.
- **API key Blotato:** Keychain akun `blotato` / env `MINIME_BLOTATO_KEY`
  (minta ke Patria; tidak pernah di-commit/diminta via chat).
- **Konvensi caption:** stage skrip pipeline menulis baris `CAPTION: ...`;
  stage distribusi mengambilnya otomatis bila params caption kosong.
- **Tanpa key:** draft lokal tetap dibuat (video_path + caption + hashtag) —
  distribusi tidak gagal diam-diam, hanya menunggu key untuk upload.

## Toolbox

- Checklist optimasi profil per channel (di atas)
- Template rencana distribusi: tanggal → channel → format → copy → aset → status (draft)
- Metrik per format: Reels (non-follower reach + shares), carousel (saves + shares + profile visits),
  Stories (replies + link clicks), YouTube (CTR + retention), LinkedIn (profile views + inbound),
  TikTok (watch time + follows/video), WA (read rate + reply rate + opt-out rate)
- Aturan ukur: follows/inbound/klik > likes/views
- Bundle draft Blotato per platform: `video_path` valid + `caption` + `hashtags`
  + `status: draft` + catatan publish yang benar (Meta = manual Patria)

## Quality bar

- Profil/halaman lolos checklist optimasi sebelum distribusi berjalan.
- Setiap item distribusi: status draft, copy + aset siap, CTA sesuai tujuan.
- Konsistensi jadwal > intensitas sesaat.
- Engagement <24 jam; tidak ada broadcast spam; tidak ada database beli.
- Nol auto-posting — pelanggaran = gagal total skill ini.

## Trigger eskalasi — WAJIB ke Patria bila

- Setiap keputusan publish (tombol publish = manusia).
- Isu sensitif, krisis, atau respons publik.
- Boost/iklan berbayar (budget = keputusan manusia).
- Perubahan bio/profil yang menyentuh positioning.
- Tanda-tanda risiko blokir (opt-out WA naik, reach anjlok tiba-tiba).

## Proof of work

Setiap output WAJIB mencantumkan: checklist optimasi profil (status), tabel rencana
distribusi (semua berstatus `draft`), metrik target per channel, dan pernyataan
"Tidak ada item yang dijadwalkan otomatis — menunggu approval manual."
