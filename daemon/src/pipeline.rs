//! Pipeline YAML: parse, validate (DAG valid, agents/skills exist), list.
//! Execution lives in runner.rs.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct BranchRaw {
    pub when: String,
    pub sub_pipeline: String,
    pub then: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParallelBranch {
    pub agent: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StageRaw {
    pub id: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub on_done: String,
    #[serde(default)]
    pub branches: Vec<BranchRaw>,
    #[serde(default)]
    pub parallel: Vec<ParallelBranch>,
    #[serde(default)]
    pub join: String,
    /// Fase 4: tool-driven stage — task menjalankan tool, bukan LLM call.
    /// Contoh: `tool: {id: hyperframes, op: render-reels, params: {...}}`
    #[serde(default)]
    pub tool: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineRaw {
    pub name: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub optional: bool,
    pub stages: Vec<StageRaw>,
}

#[derive(Debug, Clone)]
pub enum Transition {
    Handoff(String),
    Approval,
    End,
}

impl Transition {
    pub fn parse(s: &str) -> Result<Self> {
        let t = s.trim();
        if t.eq_ignore_ascii_case("approval") {
            return Ok(Transition::Approval);
        }
        if t.eq_ignore_ascii_case("end") {
            return Ok(Transition::End);
        }
        if let Some(inner) = t.strip_prefix("handoff(").and_then(|x| x.strip_suffix(")")) {
            let agent = inner.trim();
            if agent.is_empty() {
                anyhow::bail!("empty handoff target");
            }
            return Ok(Transition::Handoff(agent.to_string()));
        }
        anyhow::bail!("unknown transition '{t}' (expected handoff(<agent>), approval, or end)")
    }
}

pub fn pipeline_path(repo: &Path, name: &str) -> std::path::PathBuf {
    repo.join("pipelines").join(format!("{name}.yaml"))
}

pub fn load_pipeline(repo: &Path, name: &str) -> Result<PipelineRaw> {
    let path = pipeline_path(repo, name);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_yaml::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

/// List available pipelines (name + description).
pub fn list_pipelines(repo: &Path) -> Vec<(String, String)> {
    let dir = repo.join("pipelines");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|x| x == "yaml").unwrap_or(false) {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(p) = serde_yaml::from_str::<PipelineRaw>(&text) {
                    out.push((p.name, p.description));
                }
            }
        }
    }
    out.sort();
    out
}

/// Validate a pipeline: unique stage ids, agents/skills exist, transitions sane,
/// sub-pipelines exist and are acyclic.
pub fn validate_pipeline(
    p: &PipelineRaw,
    repo: &Path,
    agents: &HashSet<String>,
    skills: &HashSet<String>,
) -> Result<()> {
    let mut stack = Vec::new();
    validate_pipeline_inner(p, repo, agents, skills, &mut stack)
}

fn validate_pipeline_inner(
    p: &PipelineRaw,
    repo: &Path,
    agents: &HashSet<String>,
    skills: &HashSet<String>,
    stack: &mut Vec<String>,
) -> Result<()> {
    if stack.contains(&p.name) {
        anyhow::bail!(
            "pipeline cycle detected: {} -> {}",
            stack.join(" -> "),
            p.name
        );
    }
    stack.push(p.name.clone());
    let mut ids = HashSet::new();
    for stage in &p.stages {
        if !ids.insert(stage.id.clone()) {
            anyhow::bail!("pipeline '{}': duplicate stage id '{}'", p.name, stage.id);
        }
        validate_stage(stage, &p.name, repo, agents, skills, stack)?;
    }
    stack.pop();
    Ok(())
}

fn validate_stage(
    s: &StageRaw,
    pipeline: &str,
    repo: &Path,
    agents: &HashSet<String>,
    skills: &HashSet<String>,
    stack: &mut Vec<String>,
) -> Result<()> {
    let has_agent = !s.agent.is_empty();
    let has_parallel = !s.parallel.is_empty();
    match (has_agent, has_parallel) {
        (true, false) => {
            if !agents.contains(&s.agent) {
                anyhow::bail!(
                    "pipeline '{pipeline}': stage '{}' uses unknown agent '{}'",
                    s.id,
                    s.agent
                );
            }
        }
        (false, true) => {
            for b in &s.parallel {
                if !agents.contains(&b.agent) {
                    anyhow::bail!(
                        "pipeline '{pipeline}': parallel branch uses unknown agent '{}'",
                        b.agent
                    );
                }
                for sk in &b.skills {
                    if !skills.contains(sk) {
                        anyhow::bail!("pipeline '{pipeline}': unknown skill '{sk}'");
                    }
                }
            }
            if !s.join.is_empty() && s.join != "all" {
                anyhow::bail!(
                    "pipeline '{pipeline}': stage '{}': only join: all is supported in Fase 1",
                    s.id
                );
            }
        }
        _ => anyhow::bail!(
            "pipeline '{pipeline}': stage '{}' must set exactly one of 'agent' or 'parallel'",
            s.id
        ),
    }
    for sk in &s.skills {
        if !skills.contains(sk) {
            anyhow::bail!(
                "pipeline '{pipeline}': stage '{}' uses unknown skill '{sk}'",
                s.id
            );
        }
    }
    // Fase 4: validate tool-driven stages (id must be a known tool).
    if let Some(t) = &s.tool {
        let tid = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if !crate::tools::KNOWN_TOOLS.contains(&tid) {
            anyhow::bail!(
                "pipeline '{pipeline}': stage '{}' uses unknown tool '{tid}'",
                s.id
            );
        }
        if t.get("op")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty()
        {
            anyhow::bail!(
                "pipeline '{pipeline}': stage '{}': tool.op wajib diisi",
                s.id
            );
        }
    }
    let t = Transition::parse(&s.on_done)
        .with_context(|| format!("pipeline '{pipeline}': stage '{}'", s.id))?;
    if let Transition::Handoff(target) = &t {
        if !agents.contains(target) {
            anyhow::bail!(
                "pipeline '{pipeline}': stage '{}' hands off to unknown agent '{target}'",
                s.id
            );
        }
    }
    for b in &s.branches {
        Transition::parse(&b.then)
            .with_context(|| format!("pipeline '{pipeline}': branch '{}'", b.when))?;
        let sub_name = pipeline_name_from_path(&b.sub_pipeline)?;
        let sub = load_pipeline(repo, sub_name)?;
        validate_pipeline_inner(&sub, repo, agents, skills, stack)?;
    }
    Ok(())
}

fn pipeline_name_from_path(p: &str) -> Result<&str> {
    let base = p.rsplit('/').next().unwrap_or(p);
    base.strip_suffix(".yaml")
        .with_context(|| format!("sub_pipeline '{p}' must end with .yaml"))
}
