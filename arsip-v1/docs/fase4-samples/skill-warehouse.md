# Sampel uji skill: skill-warehouse

> Uji wiring (17 Okt 2026) — mock provider `MINIME_MOCK_LLM=1`.
> Task dijalankan agen `mimi` (orkestrator) dengan playbook stage-level `["skill-warehouse"]`.
> Bukti wiring: `PLAYBOOK AKTIF: skill-warehouse`.
> Status wiring: OK — TERBUKTI.
> Isi lengkap skill: `skills/skill-warehouse/SKILL.md` (gudang on-demand dari
> `msitarzewski/agency-agents`, MIT, 160k★).

## Tugas
Uji wiring skill-warehouse — bila butuh review kode, muat modul code-reviewer dari gudang.

## Output (mock — deterministik, untuk verifikasi pipeline)

**JUJUR: ini BUKAN output beneran skill.** Ini bukti wiring mock —
playbook `skill-warehouse` ter-inject ke prompt agen dan di-echo oleh mock provider.
Kualitas output nyata skill hanya bisa dinilai dengan LLM beneran.

```
PLAYBOOK AKTIF: skill-warehouse
(Ini adalah output deterministik dari mock provider untuk verifikasi pipeline.)
```

## Catatan uji

- Playbook full-body hanya ter-inject bila task input memuat `"skills": [...]`
  (stage-level); daftar skill agen hanya muncul sebagai ringkasan "SKILLS YANG KAMU KUASAI".
- Skill terdaftar sebagai skill ke-27 di DB (`owner_agent: mimi` — skill pertama milik Mimi).
- `agents/mimi/agent.yaml`: `skills: [skill-warehouse]` ditambahkan permanen;
  perubahan `provider: mock` saat uji sudah di-revert (kembali ke gemini).
- Gudang: `skills-warehouse/INDEX.md` (267 spesialis, 18 kategori) + 8 modul teradaptasi
  (`ui-designer`, `ux-researcher`, `frontend-developer`, `code-reviewer`,
  `reality-checker`, `research-synthesist`, `carousel-growth-engine`,
  `short-video-editing-coach`) + `CONTRIBUTING.md`.
- Prinsip dijaga: gudang BUKAN agen baru — roster tetap 5 agen + Approval (rule #16).
