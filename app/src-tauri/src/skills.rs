//! Skills engine (Tahap 2): muat playbook per agen on-demand.
//!
//! Skill = direktori berisi `SKILL.md` (frontmatter YAML + markdown):
//! prosedur kerja + quality bar + contoh output, sesuai arsitektur §4.3.
//! Isi skill disuntikkan ke system prompt agen.
//!
//! Dua sumber skill:
//! 1. **Bundled** — direktori `skills/` bawaan aplikasi (read-only):
//!    env `MINIME_SKILLS_DIR` → `<exe>/skills` →
//!    `<exe>/../Resources/skills` (bundle macOS .app) → `./skills`.
//! 2. **User** — `<app-data>/skills`, dibuat lewat Marketplace
//!    (skill buatan pengguna). User menang bila id bertabrakan.
//!
//! Penugasan skill per agen: default mapping di bawah, dapat diganti
//! user lewat Marketplace — override tersimpan di SQLite
//! (`agent_skill_overrides`) dan menang atas default.
//!
//! Bila tidak ditemukan, agen tetap jalan dengan system prompt dasar.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Skill bawaan untuk tiap agen (keputusan awal; user bisa mengubah
/// lewat Marketplace — override di SQLite menang atas daftar ini).
fn skills_for(agent_id: &str) -> &'static [&'static str] {
    match agent_id {
        "mimi" => &["skill-warehouse", "swarm", "qc-rubrik"],
        "kreatif" => &[
            "content",
            "copywriting",
            "storytelling",
            "branding-playbook",
            "unslop",
        ],
        "riset" => &["riset-web", "openviking"],
        "finance" => &["finance-playbook"],
        "strategis" => &[
            "marketing-playbook",
            "selling-playbook",
            "operational-playbook",
            "smo-playbook",
        ],
        "code" => &["code-builder-playbook", "reverse-engineer-anything", "tdd"],
        _ => &[],
    }
}

/// Batas per skill agar konteks tidak meledak (karakter).
const MAX_CHARS_PER_SKILL: usize = 6000;
/// Batas total semua skill per request (karakter).
const MAX_TOTAL_CHARS: usize = 18000;

/* ---------- Skill directories ---------- */

static USER_SKILLS_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Register the writable user skills directory (`<app-data>/skills`).
/// Called once from `main` before the app starts.
pub fn init_user_skills_dir(dir: PathBuf) {
    let _ = std::fs::create_dir_all(&dir);
    let _ = USER_SKILLS_DIR.set(dir);
}

fn user_skills_dir() -> Option<PathBuf> {
    USER_SKILLS_DIR.get().cloned()
}

fn bundled_skills_dir() -> Option<PathBuf> {
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

/// Directory that holds skill `name`, user skills first.
fn find_skill_dir(name: &str) -> Option<PathBuf> {
    if let Some(dir) = user_skills_dir() {
        let p = dir.join(name);
        if p.join("SKILL.md").is_file() {
            return Some(p);
        }
    }
    let bundled = bundled_skills_dir()?;
    let p = bundled.join(name);
    if p.join("SKILL.md").is_file() {
        return Some(p);
    }
    None
}

/* ---------- Frontmatter ---------- */

/// Extract `name:` and `description:` from the YAML frontmatter block.
fn parse_frontmatter_meta(raw: &str) -> (Option<String>, Option<String>) {
    let mut name = None;
    let mut desc = None;
    if let Some(rest) = raw.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            for line in rest[..end].lines() {
                if let Some(v) = line.strip_prefix("name:") {
                    name = Some(v.trim().trim_matches('"').to_string());
                } else if let Some(v) = line.strip_prefix("description:") {
                    desc = Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    (name, desc)
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

fn load_skill(dir: &Path, name: &str) -> Option<String> {
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

fn load_skill_any(name: &str) -> Option<String> {
    let skill_dir = find_skill_dir(name)?;
    let parent = skill_dir.parent()?.to_path_buf();
    load_skill(&parent, name)
}

/* ---------- Assignment (defaults + user overrides) ---------- */

/// Default skill ids for an agent (empty for unknown agents).
pub fn default_skills(agent_id: &str) -> Vec<String> {
    skills_for(agent_id).iter().map(|s| s.to_string()).collect()
}

/// The agent's effective skill set: the stored override when present,
/// otherwise the built-in default mapping.
pub fn effective_skills(db: &crate::db::Db, agent_id: &str) -> Vec<String> {
    match db.skill_override(agent_id) {
        Ok(Some(list)) => list,
        _ => default_skills(agent_id),
    }
}

/// Konteks skill untuk agen, siap ditempel ke system prompt.
/// String kosong bila tidak ada skill yang bisa dimuat.
pub fn skill_context(db: &crate::db::Db, agent_id: &str) -> String {
    let names = effective_skills(db, agent_id);
    if names.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut total = 0usize;
    for name in &names {
        if total >= MAX_TOTAL_CHARS {
            break;
        }
        if let Some(text) = load_skill_any(name) {
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

/* ---------- Marketplace catalog ---------- */

/// One catalog entry for the Marketplace > Skills tab.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    /// "bundled" (ships with the app) or "user" (created in-app).
    pub source: String,
    /// Agents whose effective skill set includes this skill.
    pub agents: Vec<String>,
}

fn scan_skills_dir(dir: &PathBuf, source: &str, out: &mut Vec<(String, String, String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(id) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        let Ok(raw) = std::fs::read_to_string(path.join("SKILL.md")) else {
            continue;
        };
        let (name, desc) = parse_frontmatter_meta(&raw);
        out.push((
            id.clone(),
            name.unwrap_or(id),
            desc.unwrap_or_default(),
            source.to_string(),
        ));
    }
}

/// Full catalog (bundled + user, user wins on id clashes) with the
/// agents that currently have each skill, for the given agent ids
/// (plus any agent that has a stored override).
pub fn catalog_with_agents(db: &crate::db::Db, agent_ids: &[String]) -> Vec<CatalogEntry> {
    let mut raw: Vec<(String, String, String, String)> = Vec::new();
    if let Some(dir) = bundled_skills_dir() {
        scan_skills_dir(&dir, "bundled", &mut raw);
    }
    if let Some(dir) = user_skills_dir() {
        // User skills replace bundled ones with the same id.
        let mut user: Vec<(String, String, String, String)> = Vec::new();
        scan_skills_dir(&dir, "user", &mut user);
        let user_ids: Vec<String> = user.iter().map(|u| u.0.clone()).collect();
        raw.retain(|r| !user_ids.contains(&r.0));
        raw.extend(user);
    }

    let mut ids: Vec<String> = agent_ids.to_vec();
    if let Ok(extra) = db.skill_override_agent_ids() {
        for id in extra {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    let effective: Vec<(String, Vec<String>)> = ids
        .iter()
        .map(|id| (id.clone(), effective_skills(db, id)))
        .collect();

    let mut out: Vec<CatalogEntry> = raw
        .into_iter()
        .map(|(id, name, description, source)| {
            let agents = effective
                .iter()
                .filter(|(_, skills)| skills.contains(&id))
                .map(|(agent, _)| agent.clone())
                .collect();
            CatalogEntry {
                id,
                name,
                description,
                source,
                agents,
            }
        })
        .collect();
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

/* ---------- Creating user skills ---------- */

/// "Riset Pasar Mingguan!" -> "riset-pasar-mingguan"
fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Create a new user skill: writes `<user-skills>/<slug>/SKILL.md`
/// with a starter playbook the user/agent can flesh out. Fails when
/// the id already exists or no user skills directory is available.
pub fn create_skill(name: &str, description: &str) -> Result<CatalogEntry, String> {
    let slug = slugify(name);
    if slug.is_empty() {
        return Err("Nama skill tidak valid.".to_string());
    }
    if find_skill_dir(&slug).is_some() {
        return Err(format!("Skill '{slug}' sudah ada."));
    }
    let Some(base) = user_skills_dir() else {
        return Err("Direktori skill pengguna belum siap.".to_string());
    };
    let dir = base.join(&slug);
    std::fs::create_dir_all(&dir).map_err(|e| format!("gagal membuat skill: {e}"))?;
    let desc = if description.trim().is_empty() {
        format!("Playbook {name} — dibuat dari Marketplace Minime.")
    } else {
        description.trim().to_string()
    };
    let body = format!(
        "---\nname: {slug}\ndescription: {desc}\nversion: 0.1.0\n---\n\n# {name}\n\n> Kontrak agen senior. SOP WAJIB diikuti; quality bar adalah standar\n> kelulusan; bila ragu → eskalasi ke pengguna (Approval).\n\n## Kapan dipakai\n\n{desc}\n\n## SOP\n\n1. Pahami permintaan dan hasil akhir yang diminta.\n2. Kerjakan langkah demi langkah; catat sumber dan asumsi.\n3. Periksa hasil terhadap quality bar sebelum menyerahkan.\n\n## Quality bar\n\n- Hasil sesuai brief, rapi, dan bisa diverifikasi.\n- Tidak mengarang fakta; ketidakpastian dinyatakan terus terang.\n"
    );
    std::fs::write(dir.join("SKILL.md"), body)
        .map_err(|e| format!("gagal menulis skill: {e}"))?;
    Ok(CatalogEntry {
        id: slug,
        name: name.trim().to_string(),
        description: desc,
        source: "user".to_string(),
        agents: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_frontmatter_meta, slugify};

    #[test]
    fn frontmatter_meta_is_parsed() {
        let raw = "---\nname: content\ndescription: Playbook konten.\nversion: 1.0.0\n---\n\n# content\n";
        let (name, desc) = parse_frontmatter_meta(raw);
        assert_eq!(name.as_deref(), Some("content"));
        assert_eq!(desc.as_deref(), Some("Playbook konten."));
    }

    #[test]
    fn frontmatter_meta_missing_is_none() {
        let (name, desc) = parse_frontmatter_meta("# tanpa frontmatter\n");
        assert!(name.is_none());
        assert!(desc.is_none());
    }

    #[test]
    fn slugify_normalizes_names() {
        assert_eq!(slugify("Riset Pasar Mingguan!"), "riset-pasar-mingguan");
        assert_eq!(slugify("  QC  Rubrik  "), "qc-rubrik");
        assert_eq!(slugify("!!!"), "");
    }
}
