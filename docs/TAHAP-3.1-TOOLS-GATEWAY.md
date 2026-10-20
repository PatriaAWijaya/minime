# Detail Tahap 3.1 — Tools Gateway (Rust)
> Dirinci 20 Okt 2026 12:39 WIB atas perintah Patria. Bagian dari rebuild Tahap 3 (`docs/RENCANA-REBUILD-TAHAP-3.md`).
> Sumber kode: commit `b34e4f2` (tools.rs 176 baris + wiring main.rs). Metode: pulihkan → adaptasi → uji.

---

## 1. Tujuan
Gateway pusat untuk semua peralatan eksternal agen: satu registry di backend Rust yang tahu (a) tool apa saja yang ada, (b) mana yang terhubung, (c) agen mana boleh memakai tool apa. Semua pemakaian tool oleh agen WAJIB lewat gerbang ini — tidak ada jalan pintas.

## 2. Arsitektur (dari kode lama)
- **File**: `app/src-tauri/src/tools.rs` — modul baru, didaftarkan `mod tools;` di main.rs
- **State**: `ToolRegistry` (Tauri managed state) berisi 2 `Mutex<HashMap>`:
  - `status`: tool_id → connected (bool)
  - `allowlists`: agent_id → daftar tool_id yang diizinkan
- **Kredensial**: TIDAK disimpan di gateway — hanya di OS Keychain via `secrets.rs`. Gateway hanya tahu status terhubung/tidak.
- **Tanpa dependensi baru**: hanya serde + std + tauri (sudah ada).

## 3. Data: 12 tool yang dikenal
| ID | Nama | Kategori | Perlu auth | Draft-only |
|---|---|---|---|---|
| meta | Meta | social | ya | **ya** |
| postiz | Postiz | social | ya | **ya** |
| pireel | Pireel | video | ya | tidak |
| drive | Google Drive | storage | ya | tidak |
| slack | Slack | chat | ya | tidak |
| discord | Discord | chat | ya | tidak |
| telegram | Telegram | chat | ya | tidak |
| github | GitHub | code | ya | tidak |
| notion | Notion | docs | ya | tidak |
| canva | Canva | design | ya | tidak |
| ruangapi | RuangAPI | api | ya | tidak |
| browser | Browser | web | tidak | tidak |

- `ToolInfo` (dikirim ke frontend): id, name, category, connected, requires_auth, draft_only
- Catatan adaptasi: daftar ini harus dicocokkan dengan daftar aplikasi di ToolsView saat ini (meta, postiz, pireel, drive, slack, discord, telegram, github, notion, canva, ruangapi — 11 app; `browser` adalah tool internal, tidak tampil sebagai aplikasi).

## 4. Allowlist default per agen (keputusan Patria 19 Okt)
| Agen | Tool yang diizinkan |
|---|---|
| mimi (CIO) | `*` (semua — orkestrator) |
| kreatif | pireel, canva, meta, postiz |
| riset | browser, ruangapi |
| finance | ruangapi |
| strategis | meta, postiz, ruangapi |
| code | github, ruangapi |

- Catatan adaptasi: hanya Mimi yang aktif secara default (aturan 20 Okt) — allowlist agen lain tetap disiapkan, berlaku saat agen diaktifkan.
- Agen custom (buatan user) mulai dari allowlist KOSONG; user mengatur lewat UI Izin.

## 5. Enam IPC command
| Command | Parameter | Hasil | Fungsi |
|---|---|---|---|
| `list_tools` | — | `Vec<ToolInfo>` | Daftar semua tool + status koneksi |
| `get_tool_status` | tool_id | bool | Cek satu tool terhubung? |
| `set_tool_connected` | tool_id, connected | Result | Tandai tool terhubung/putus (validasi: tool harus dikenal, error bila tidak) |
| `get_agent_tools` | agent_id | `Vec<String>` | Allowlist satu agen |
| `set_agent_tools` | agent_id, tool_ids | Result | Ubah allowlist (validasi tiap ID; `*` hanya wildcard sah) |
| `agent_can_use_tool` | agent_id, tool_id | bool | Gate sebelum eksekusi: bolehkah agen ini memakai tool ini? (`*` = semua boleh; agen tak dikenal = false) |

## 6. Langkah pengerjaan (berurutan)
1. Pulihkan `tools.rs` dari `b34e4f2` ke `app/src-tauri/src/tools.rs`
2. Adaptasi: samakan ID tool dengan ToolsView saat ini; pastikan tidak ada konflik nama dengan modul lain
3. Wiring main.rs: `mod tools;` + `.manage(tools::ToolRegistry::new())` + 6 command di `invoke_handler`
4. `cargo check` di MacBook — target nol error, nol warning
5. Frontend store.ts: fungsi pembungkus invoke untuk 6 command + store `toolInfos`
6. Hubungkan ToolsView: status "Terpasang/Jelajahi" + badge koneksi membaca dari backend (bukan state lokal saja); toggle pasang/copot memanggil `set_tool_connected`
7. Hubungkan menu Izin agen (AgentsView): checkbox allowlist membaca `get_agent_tools`, menyimpan via `set_agent_tools`
8. Build debug MacBook + jalankan

## 7. Uji (kriteria lolos sub-fase)
- [ ] `cargo check` bersih; build debug sukses
- [ ] ToolsView menampilkan 11 aplikasi + status dari backend
- [ ] Toggle copot/pasang satu aplikasi → status berubah & konsisten setelah restart app (catatan: status lama disimpan in-memory — bila hilang saat restart, itu perilaku warisan; keputusan persistensi dicatat untuk 3.6)
- [ ] Ubah allowlist satu agen di menu Izin → tersimpan; `agent_can_use_tool` menjawab benar (boleh) & salah (tidak boleh) sesuai daftar
- [ ] Tool tidak dikenal → error jelas ("Tool tidak dikenal: …"), bukan crash
- [ ] Cek konsistensi: IPC menjadi 14 command (8 lama + 6 baru), semua cocok frontend↔backend
- [ ] Cek performa: CPU/memori tetap sehat setelah rebuild

## 8. Di luar cakupan 3.1 (sub-fase lain)
- Eksekusi tool sungguhan (panggil Meta/Pireel/dll.) → 3.3–3.5
- Approval sebelum eksekusi sensitif → 3.2
- Pemanggilan tool dari chat oleh agen → 3.6
- Persistensi status/allowlist ke SQLite → dievaluasi di 3.6
