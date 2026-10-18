//! Agent + skill loader: reads agents/<id>/ and skills/*/ at startup and
//! seeds the database. Adding an agent = adding a folder; no code change.

use anyhow::{Context, Result};
use rusqlite::params;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

use crate::db::Db;

#[derive(Debug, Deserialize)]
struct ModelRefSimple {
    provider: String,
    model_id: String,
}

#[derive(Debug, Deserialize)]
struct ModelRef {
    provider: String,
    model_id: String,
    #[serde(default)]
    fallback_chain: Vec<ModelRefSimple>,
}

#[derive(Debug, Deserialize)]
struct AgentDef {
    id: String,
    name: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    description: String,
    model: Option<ModelRef>,
    #[serde(default)]
    effort: String,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    tools_allowlist: Vec<String>,
    #[serde(default)]
    daily_budget_usd: f64,
    #[serde(default)]
    avatar: String,
    #[serde(default)]
    color: String,
    #[serde(default)]
    paused: bool,
}

#[derive(Debug, Deserialize)]
struct SkillFront {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    owner_agent: String,
    #[serde(default)]
    version: String,
}

/// Load agents/<id>/agent.yaml + system-prompt.md into the agents table.
/// Definition columns are refreshed; runtime columns (status, paused) are preserved.
pub async fn load_agents(db: &Db, repo: &Path, data_dir: &Path) -> Result<Vec<String>> {
    let agents_dir = repo.join("agents");
    let mut loaded = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&agents_dir)
        .with_context(|| format!("read {}", agents_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let dir = entry.path();
        let yaml_text = std::fs::read_to_string(dir.join("agent.yaml"))
            .with_context(|| format!("read {}/agent.yaml", dir.display()))?;
        let def: AgentDef = serde_yaml::from_str(&yaml_text)
            .with_context(|| format!("parse {}/agent.yaml", dir.display()))?;
        let prompt_path = dir.join("system-prompt.md");
        let system_prompt = std::fs::read_to_string(&prompt_path).unwrap_or_default();

        let model = def.model.as_ref();
        let provider_id = model.map(|m| m.provider.as_str()).unwrap_or("");
        let model_id = model.map(|m| m.model_id.as_str()).unwrap_or("");
        let fallback_chain = model
            .map(|m| {
                serde_json::to_string(
                    &m.fallback_chain
                        .iter()
                        .map(|f| serde_json::json!({"provider": f.provider, "model": f.model_id}))
                        .collect::<Vec<_>>(),
                )
                .unwrap_or_else(|_| "[]".to_string())
            })
            .unwrap_or_else(|| "[]".to_string());
        let effort = if def.effort.is_empty() {
            "medium".to_string()
        } else {
            def.effort.clone()
        };
        let workspace_path = data_dir.join("workspaces").join(&def.id);
        std::fs::create_dir_all(&workspace_path).ok();

        let guard = db.lock().await;
        guard.execute(
            "INSERT INTO agents (id, name, role, model, provider_id, fallback_chain, system_prompt,
                                 avatar_path, color, status, tools_allowlist, skills, workspace_path,
                                 paused, daily_budget_usd, effort, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'idle',?10,?11,?12,?13,?14,?15,datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                 name=excluded.name, role=excluded.role, model=excluded.model,
                 provider_id=excluded.provider_id, fallback_chain=excluded.fallback_chain,
                 system_prompt=excluded.system_prompt, avatar_path=excluded.avatar_path,
                 color=excluded.color, tools_allowlist=excluded.tools_allowlist,
                 skills=excluded.skills, workspace_path=excluded.workspace_path,
                 daily_budget_usd=excluded.daily_budget_usd, effort=excluded.effort,
                 updated_at=datetime('now')",
            params![
                def.id,
                def.name,
                def.role,
                model_id,
                provider_id,
                fallback_chain,
                system_prompt,
                def.avatar,
                if def.color.is_empty() {
                    "#888888".to_string()
                } else {
                    def.color
                },
                serde_json::to_string(&def.tools_allowlist).unwrap_or_else(|_| "[]".to_string()),
                serde_json::to_string(&def.skills).unwrap_or_else(|_| "[]".to_string()),
                workspace_path.to_string_lossy().to_string(),
                def.paused as i64,
                def.daily_budget_usd,
                effort,
            ],
        )?;
        drop(guard);
        loaded.push(def.id);
    }
    Ok(loaded)
}

/// Scan skills/*/SKILL.md frontmatter into the skills table.
pub async fn load_skills(db: &Db, repo: &Path) -> Result<Vec<String>> {
    let skills_dir = repo.join("skills");
    let mut loaded = Vec::new();
    let Ok(entries) = std::fs::read_dir(&skills_dir) else {
        return Ok(loaded);
    };
    let mut entries: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let dir = entry.path();
        let text = match std::fs::read_to_string(dir.join("SKILL.md")) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let front = match parse_frontmatter(&text) {
            Some(f) => f,
            None => continue,
        };
        let version = if front.version.is_empty() {
            "0.1.0".to_string()
        } else {
            front.version
        };
        let guard = db.lock().await;
        guard.execute(
            "INSERT INTO skills (name, version, path, owner_agent, enabled, description)
             VALUES (?1,?2,?3,?4,1,?5)
             ON CONFLICT(name) DO UPDATE SET
                 version=excluded.version, path=excluded.path,
                 owner_agent=excluded.owner_agent, description=excluded.description",
            params![
                front.name,
                version,
                dir.to_string_lossy().to_string(),
                front.owner_agent,
                front.description,
            ],
        )?;
        drop(guard);
        loaded.push(front.name);
    }
    Ok(loaded)
}

fn parse_frontmatter(text: &str) -> Option<SkillFront> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut yaml = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    serde_yaml::from_str(&yaml).ok()
}

/// Convenience: agent ids currently in DB (for pipeline validation).
pub async fn agent_ids(db: &Db) -> Result<HashSet<String>> {
    let guard = db.lock().await;
    let mut stmt = guard.prepare("SELECT id FROM agents")?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(ids)
}

/// Convenience: skill names currently in DB.
pub async fn skill_names(db: &Db) -> Result<HashSet<String>> {
    let guard = db.lock().await;
    let mut stmt = guard.prepare("SELECT name FROM skills")?;
    let names = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(names)
}
