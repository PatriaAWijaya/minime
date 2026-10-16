---
name: affinity
description: Tool skill — Affinity via MCP bridge (localhost:6767) untuk olah desain; di-wire daemon Fase 4.
owner_agent: creative-director
version: 1.0.0
example-prompt: Terapkan template Affinity ke desain ini.
---

# affinity

> Tool skill (bukan playbook). Dieksekusi oleh `minime-daemon` via HTTP
> JSON-RPC 2.0 ke MCP bridge di `127.0.0.1:6767`.
> Bridge berjalan di MacBook Patria; token dipegang Patria dan disimpan via
> `secrets.set` (account=`affinity`). Tunnel berubah tiap restart → cek
> `tools.status` sebelum memakai.

## Kapan dipakai

Bila tugas butuh olah desain di Affinity (template, manipulasi layer/dokumen)
yang tidak ter-cover Canva/HyperFrames.

## Operasi (via `tools.run` / task `tool:`)

| op | params | hasil |
|---|---|---|
| `call` | `method` (nama method MCP bridge), `params` | hasil JSON-RPC (atau error bridge) |

Nama method mengikuti permukaan MCP milik bridge itu sendiri.

## Readiness

`tools.status` → `affinity`: siap bila bridge terjangkau (probe HTTP) DAN token
tersedia. Di VM build: `unreachable` (jujur) — bridge hanya ada di MacBook Patria.
Tidak siap → **fallback**: olah desain via Canva / HyperFrames.

## Quality bar

- Jangan menebak nama method — pakai method yang terdokumentasi bridge.
- Operasi tulis (edit dokumen) butuh konteks tugas yang eksplisit.

## Trigger eskalasi — WAJIB ke Patria bila

- Bridge tidak terjangkau / token kedaluwarsa (minta Patria nyalakan ulang + reconnect).
- Operasi berisiko merusak dokumen (minta konfirmasi dulu).

## Proof of work

Setiap pemanggilan tercatat di `messages` (kind=`tool-run`).
