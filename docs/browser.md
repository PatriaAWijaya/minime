# Browser Bawaan — Operasi & Batasan (Fase 4)

> Chromium headless dikontrol **daemon** (bukan UI) — tetap bekerja walau
> aplikasi ditutup (rule #17). UI hanya me-mirror sesi via IPC.

## Operasi (IPC `browser.*`)

| method | params | hasil |
|---|---|---|
| `browser.open` | `agent_id`, `url?` | `{session: {id, agent_id, created_at, ...}}` |
| `browser.navigate` | `session_id`, `url` | `{url, title}` |
| `browser.screenshot` | `session_id`, `url?`, `size?` | `{path}` (PNG di `data_dir/browser/<id>/shots/`) |
| `browser.dom` | `session_id`, `url` | `{text}` (teks halaman, maks 6000 char) |
| `browser.list` | — | `{sessions: [...]}` |
| `browser.close` | `session_id` | `{closed}` |

Juga tersedia sebagai task tool: `tool: {id: "browser", op: "screenshot", params: {...}}`
(diperiksa terhadap `tools_allowlist` agen — default: agen `riset`).

## Isolasi sesi

- Satu profil Chromium (`user-data-dir`) per sesi → cookie/storage tidak bocor
  antar agen.
- Binary: `MINIME_CHROMIUM_BIN` → cache Chrome Headless Shell milik HyperFrames
  → `chromium`/`google-chrome` di PATH.

## Proof of work

Setiap aksi (`open`/`navigate`/`screenshot`/`dom`/`close`) dicatat di tabel
`messages` dengan `kind='browser-action'`: session_id, action, URL, timestamp
(+ path screenshot / ringkasan). Tidak ada aksi browser yang tak tercatat.

## Batasan keras

- **Tidak ada auto-login** ke akun pribadi Patria: modul ini tidak mengekspos
  pengisian kredensial dalam bentuk apa pun. Login hanya bila Patria
  mengizinkan eksplisit per tugas (dan dilakukan manual/di luar modul ini).
- Muat halaman eksternal membutuhkan network yang bisa dijangkau Chromium
  (di VM build ada limitasi proxy — verifikasi penuh di MacBook Patria).
- Sesi bersifat in-memory: hilang saat daemon restart (jujur — tidak
  pura-pura persisten). Profil (cookie) tetap ada di disk.
