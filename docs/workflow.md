# Workflow Tahapan — Minime v2

> Urutan (keputusan Patria 18 Okt 2026): UI/UX dulu → fitur+skills →
> tools → rilis macOS+Windows. Gate tiap tahap: review + vonis Patria.

---

## Tahap 1 — UI/UX (klik-tayang)

- [ ] Layout meniru OpenBot: sidebar workflow | daftar agen | main (chat+delivery) | monitor tugas
- [ ] 6 agen (Mimi + 5) tampil dengan avatar & status
- [ ] Bahasa: Indonesia (default), Inggris, Mandarin
- [ ] Navigasi: Pengaturan, Penggunaan, Peralatan
- **Gate:** Patria lihat & vonis

## Tahap 2 — Fitur + skills agen

- [ ] Runtime 6 agen: chat → eksekusi → output
- [ ] Skills + quality bar per agen (adaptasi playbook v1 di `skills/`)
- [ ] Model router: Gemini default + lokal gratis + custom; routing + fallback + metering
- **Gate:** review Patria

## Tahap 3 — Tools & distribusi

- [ ] Wiring: meta (draft-only), postiz/Viralin (draft-only), pireel
- [ ] Approval gate; notifikasi native; upload Drive folder MIMI
- **Gate:** 1 pipeline end-to-end

## Tahap 4 — Rilis macOS + Windows

- [ ] Build dmg + exe; installer mudah & intuitif
- [ ] Security audit + Impeccable audit
- **Gate:** instalasi bersih + vonis rilis
