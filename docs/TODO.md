# TODO — Minime v2

> Diperbarui 18 Okt 2026 malam. Gate tiap tahap = review + vonis Patria.

## Tahap 1 — UI/UX ✅ (scaffold selesai, gate vonis terbuka)
- [x] Verdict stack: Tauri 2 + Svelte 5
- [x] Scaffold proyek fresh (`app/`)
- [x] Layout 4 kolom ala OpenBot (sidebar | agen | chat+delivery | monitor)
- [x] 6 agen seed + badge avatar + status (kuning/hijau/merah)
- [x] 3 bahasa (ID default, EN, ZH)
- [x] Navigasi: Pengaturan, Penggunaan, Peralatan
- [x] Prinsip compact-but-powerful, pemula-friendly
- [x] Dokumen: arsitektur, framework agen, workflow
- [ ] Gate: vonis Patria atas tampilan Tahap 1

## Tahap 2 — Fitur + skills agen
- [ ] Runtime agen: chat → Mimi orkestrasi → eksekusi → output
- [ ] Skills + quality bar per agen (adaptasi playbook v1 di `skills/`)
- [ ] Model router: Gemini 3.8 default + lokal gratis + custom;
      routing per-agen + fallback + metering + budget cap
- [ ] Wiring SQLite (riwayat, tugas, usage)
- [ ] Gate: review + vonis Patria

## Tahap 3 — Tools & distribusi
- [ ] Meta (draft-only), Postiz/Viralin (draft-only), Pireel
- [ ] Approval gate + notifikasi native
- [ ] Upload deliverable → folder MIMI (Google Drive)
- [ ] Gate: 1 pipeline end-to-end + vonis Patria

## Tahap 4 — Rilis macOS + Windows
- [ ] Build dmg + exe; installer mudah & intuitif
- [ ] Security audit + Impeccable audit
- [ ] Gate: instalasi bersih + vonis rilis Patria

## Catatan
- Repo `PatriaAWijaya/minime-v2` (kosong): hapus manual via GitHub settings
  (token tidak punya scope delete).
- Avatar karakter v1 ditolak Patria — arah avatar baru menunggu keputusannya.
- Daemon v1 di MacBook: Patria sudah diinstruksikan `launchctl unload`
  (belum terkonfirmasi).
