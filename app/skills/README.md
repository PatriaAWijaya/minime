# Skills

Direktori ini dibundel ke aplikasi via `resources` di `tauri.conf.json`.
Skill = subdirektori berisi `SKILL.md` (frontmatter YAML + instruksi),
dibaca oleh skills engine (`app/src-tauri/src/skills.rs`).

Lokasi pencarian saat runtime:
1. `./skills` (dev),
2. `<dir-exe>/skills`,
3. `<dir-exe>/../Resources/skills` (bundle macOS .app).
