---
name: open-creator
description: Tool skill — OpenCreator sidecar untuk render video; di-wire daemon Fase 4 (kunci menunggu Patria).
owner_agent: creative-director
version: 1.0.0
example-prompt: Render reels 9:16 dari naskah ini.
---

# open-creator

> Tool skill (bukan playbook). Dieksekusi oleh `minime-daemon` via sidecar
> binary di samping binary daemon (kontrak: `docs/open-creator.md`).
> API key via Keychain (`secrets.set` account=`open-creator`) — **live test
> menunggu kunci dari Patria**.

## Kapan dipakai

Bila tugas butuh render video dengan pipeline subtitle → TTS → render yang
tidak ter-cover HyperFrames.

## Operasi (via `tools.run` / task `tool:`)

| op | params | hasil |
|---|---|---|
| `render` | sesuai kontrak sidecar | `{result, artifact_path}` |
| `voices` | — | daftar suara |
| `script` | `brief` | draf naskah |

## Readiness

`tools.status` → `open-creator`: siap bila sidecar ada DAN kunci tersedia.
Tidak siap → **fallback**: render video via HyperFrames (lokal, tanpa kunci).

## Quality bar

- Sidecar adalah satu-satunya antarmuka — jangan mengarang argumen di luar kontrak.

## Trigger eskalasi — WAJIB ke Patria bila

- Kunci API belum ada (minta Patria sediakan — bukan via chat).
- Keputusan terbuka arsitektur §16.9: kunci LLM OpenAI-compatible ATAU wiring Gemini.

## Proof of work

Setiap pemanggilan tercatat di `messages` (kind=`tool-run`).
