---
name: instagram-cli
description: Tool skill — instagram-cli KHUSUS DRAFT (publikasi DITOLAK di kode); di-wire daemon Fase 4.
owner_agent: creative-director
version: 1.0.0
example-prompt: Siapkan draft postingan Instagram dari konten ini.
---

# instagram-cli

> Tool skill (bukan playbook). Dieksekusi oleh `minime-daemon` via CLI.
>
> **ATURAN KERAS (rule #3): HANYA UNTUK DRAFT — auto-posting ke Meta DILARANG.**
> Penolakan dilakukan di level kode daemon (`tools/instagram.rs`): op maupun
> argumen yang menyerupai publikasi (`post`, `publish`, `upload`, `share`,
> `send`, `story`, …) langsung ditolak dengan error. Bukan sekadar tidak
> diimplementasikan — **tidak ada jalur kode menuju publikasi**.

## Kapan dipakai

Bila tugas butuh menyiapkan paket konten Instagram (caption + media + hashtag)
untuk diposting **manual oleh Patria**.

## Operasi (via `tools.run` / task `tool:`)

| op | params | hasil |
|---|---|---|
| `draft` | `caption` (wajib), `media[]` (path file lokal, wajib ada), `hashtags[]`, `kind` | bundle JSON di `data_dir/drafts/` → `draft_path` |
| `account` | `args[]` (read-only, eksplisit) | output CLI (dibatasi 3000 char) |

## Readiness

`tools.status` → `instagram-cli`: siap bila CLI ditemukan.
Tidak ada fallback publikasi — draft diserahkan ke Patria.

## Quality bar

- Draft tidak valid bila file media tidak ada (ditolak saat pembuatan).
- Bundle selalu berlabel "MENUNGGU posting manual".

## Trigger eskalasi — WAJIB ke Patria bila

- Ada permintaan memposting otomatis (tolak + eskalasi — ini pelanggaran aturan keras).

## Proof of work

Setiap pemanggilan tercatat di `messages` (kind=`tool-run`).
