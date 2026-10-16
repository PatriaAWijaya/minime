---
name: canva
description: Tool skill — Canva CLI (MCP) untuk produksi desain dan gambar; di-wire daemon Fase 4.
owner_agent: creative-director
version: 1.0.0
example-prompt: Buatkan desain carousel 5 slide via Canva.
---

# canva

> Tool skill (bukan playbook). Dieksekusi oleh `minime-daemon` via CLI `canva`
> (protokol: `/opt/hatch/connectors/canva/CONNECTOR.md`).
> Auth = OAuth milik user di mesinnya (dihubungkan via `canva authorize-url`);
> daemon tidak pernah memegang token Canva.

## Kapan dipakai

Bila tugas butuh: desain layout editabel (carousel, poster, presentasi),
generate gambar AI, atau export desain ke file.

## Operasi (via `tools.run` / task `tool:`)

| op | params | hasil |
|---|---|---|
| `status` | — | status koneksi + daftar tools MCP |
| `design.create` | `brief` (wajib), `format`, `width`, `height` | job async → `{design_id, url}` (polling jujur: interval server dihormati) |
| `image.generate` | `prompt` (wajib) | `{media_id, url}` |
| `design.export` | `design_id`, `format` (png/jpg/pdf) | file lokal → `artifact_path` |

## Readiness

`tools.status` → `canva`: siap bila CLI ada dan `canva status` = connected.
Tidak siap → **fallback**: render lokal via HyperFrames.

## Quality bar

- Setiap pembuatan desain baru butuh brief eksplisit dari task (tidak mengarang teks).
- Jangan membuat desain kedua untuk "memperbaiki" tanpa izin — laporkan mismatch + sertakan link Canva.
- Export selalu diunduh ke artefak lokal (proof of work).

## Trigger eskalasi — WAJIB ke Patria bila

- Butuh Canva Pro/Enterprise atau AI credits yang tidak tersedia.
- Tool MCP yang dibutuhkan tidak ada di daftar `list-tools`.

## Proof of work

Setiap pemanggilan tercatat di `messages` (kind=`tool-run`): tool, op, ringkasan, durasi.
