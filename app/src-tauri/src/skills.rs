//! Skills engine (Tahap 2): muat playbook per agen on-demand.
//!
//! Skill = direktori di `skills/` berisi `SKILL.md` (frontmatter YAML +
//! markdown): prosedur kerja + quality bar + contoh output, sesuai
//! arsitektur §4.3. Isi skill disuntikkan ke system prompt agen.
//!
//! Lokasi direktori skills (urutan pencarian):
//! 1. env `MINIME_SKILLS_DIR` (dev),
//! 2. `<dir-exe>/skills`,
//! 3. `<dir-exe>/../Resources/skills` (bundle macOS .app),
//! 4. `./skills` (cwd, fallback dev).
//!
//! Bila tidak ditemukan, agen tetap jalan dengan system prompt dasar.

use std::path::PathBuf;

/// Skill yang dimuat untuk tiap agen (keputusan awal; user bisa menambah).
fn skills_for(agent_id: &str) -> &'static [&'static str] {
    match agent_id {
        "mimi" => &["skill-warehouse"],
        "kreatif" => &["content", "copywriting", "storytelling", "branding-playbook"],
        "riset" => &["riset-web", "openviking"],
        "finance" => &["finance-playbook"],
        "strategis" => &[
            "marketing-playbook",
            "selling-playbook",
            "operational-playbook",
            "smo-playbook",
        ],
        "code" => &["code-builder-playbook"],
        _ => &[],
    }
}

/// Batas per skill agar konteks tidak meledak (karakter).
const MAX_CHARS_PER_SKILL: usize = 6000;
/// Batas total semua skill per request (karakter).
const MAX_TOTAL_CHARS: usize = 18000;

fn skills_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("MINIME_SKILLS_DIR") {
        let p = PathBuf::from(p);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("skills");
            if p.is_dir() {
                return Some(p);
            }
            let p = dir.join("../Resources/skills");
            if p.is_dir() {
                return Some(p);
            }
        }
    }
    let p = PathBuf::from("./skills");
    if p.is_dir() {
        return Some(p);
    }
    None
}

fn load_skill(dir: &PathBuf, name: &str) -> Option<String> {
    let path = dir.join(name).join("SKILL.md");
    let raw = std::fs::read_to_string(&path).ok()?;
    // Pisahkan frontmatter (--- ... ---) untuk deskripsi singkat.
    let (desc, body) = split_frontmatter(&raw);
    let mut text = String::new();
    if !desc.is_empty() {
        text.push_str(&format!("Ringkasan: {}\n\n", desc.trim()));
    }
    let body: String = body.chars().take(MAX_CHARS_PER_SKILL).collect();
    text.push_str(body.trim());
    if text.trim().is_empty() {
        return None;
    }
    Some(text)
}

fn split_frontmatter(raw: &str) -> (String, &str) {
    let mut desc = String::new();
    let mut body = raw;
    if let Some(rest) = raw.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            body = &rest[end + 4..];
            for line in fm.lines() {
                if let Some(v) = line.strip_prefix("description:") {
                    desc = v.trim().to_string();
                    break;
                }
            }
        }
    }
    (desc, body)
}

/// Konteks skill untuk agen, siap ditempel ke system prompt.
/// String kosong bila tidak ada skill yang bisa dimuat.
pub fn skill_context(agent_id: &str) -> String {
    let names = skills_for(agent_id);
    if names.is_empty() {
        return String::new();
    }
    let dir = match skills_dir() {
        Some(d) => d,
        None => return String::new(),
    };
    let mut out = String::new();
    let mut total = 0usize;
    for name in names {
        if total >= MAX_TOTAL_CHARS {
            break;
        }
        if let Some(text) = load_skill(&dir, name) {
            let chunk = format!("## Skill: {name}\n{text}\n\n");
            total += chunk.len();
            out.push_str(&chunk);
        }
    }
    if out.is_empty() {
        return out;
    }
    format!("# Playbook skill\n\n{out}")
}
