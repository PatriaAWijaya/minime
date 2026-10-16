# VoiceStudio — Panduan Instalasi (untuk MacBook Patria)

> Status Fase 4: instalasi masih pending. Selama belum terinstal, daemon memakai
> **fallback `edge-tts`** secara otomatis (hasil dicatat jujur sebagai fallback).

## Yang dibutuhkan

1. Service VoiceStudio berjalan lokal (REST) dengan endpoint TTS.
2. API key (bila service membutuhkannya).

## Langkah instalasi

1. Instal/pasang VoiceStudio di MacBook mengikuti dokumentasi resminya.
2. Pastikan service mendengar di `http://127.0.0.1:8765` (atau catat URL-nya).
3. Uji endpoint kesehatan:
   ```bash
   curl -s http://127.0.0.1:8765/health
   ```
4. Bila URL bukan default, set env untuk daemon:
   ```bash
   # di ~/.zshrc atau plist LaunchAgent (EnvironmentVariables)
   export MINIME_VOICESTUDIO_URL="http://127.0.0.1:8765"
   ```
5. Bila butuh API key, simpan via IPC (write-only, masuk Keychain):
   ```json
   {"method": "secrets.set", "params": {"account": "voice-studio", "value": "KUNCI-DI-SINI"}}
   ```
   (Jangan pernah menaruh kunci di chat, file, atau git.)
6. Verifikasi readiness:
   ```json
   {"method": "tools.status", "params": {}}
   ```
   → `voice-studio` harus `ready: true`.
7. Uji TTS Bahasa Indonesia:
   ```json
   {"method": "tools.run", "params": {
     "tool": "voice-studio", "op": "tts", "agent_id": "creative-director",
     "params": {"text": "Halo, ini uji suara Bahasa Indonesia.", "lang": "id"}
   }}
   ```
   → mengembalikan `artifact_path` (MP3). Dengarkan hasilnya.

## Kontrak REST yang dipakai daemon

- `GET /health` → 200 bila hidup (untuk readiness).
- `POST /v1/tts` + `Authorization: Bearer <key>` + JSON `{"text": "...", "lang": "id", "voice": "..."}`
  → bytes audio (MP3/wav).

Bila VoiceStudio Patria memakai kontrak berbeda (mis. MCP), sesuaikan
`daemon/src/tools/voicestudio.rs` dan catat di sini.

## Fallback edge-tts

```bash
pip install edge-tts
```

Daemon otomatis memakai `edge-tts --voice id-ID-ArdiNeural` bila VoiceStudio
tidak siap. Kualitas cukup untuk draft; untuk siar komersil, tetap prioritaskan
instalasi VoiceStudio.
