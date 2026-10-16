//! Database row models (English code; user-facing text lives in agents/pipelines).

use rusqlite::{Result as SqlResult, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    pub provider_id: String,
    pub fallback_chain: String,
    pub system_prompt: String,
    pub avatar_path: String,
    pub color: String,
    pub status: String,
    pub tools_allowlist: String,
    pub skills: String,
    pub workspace_path: String,
    pub paused: bool,
    pub daily_budget_usd: f64,
    pub effort: String,
}

impl Agent {
    pub fn from_row(r: &Row) -> SqlResult<Self> {
        Ok(Self {
            id: r.get("id")?,
            name: r.get("name")?,
            role: r.get("role")?,
            model: r.get("model")?,
            provider_id: r.get("provider_id")?,
            fallback_chain: r.get("fallback_chain")?,
            system_prompt: r.get("system_prompt")?,
            avatar_path: r.get("avatar_path")?,
            color: r.get("color")?,
            status: r.get("status")?,
            tools_allowlist: r.get("tools_allowlist")?,
            skills: r.get("skills")?,
            workspace_path: r.get("workspace_path")?,
            paused: r.get::<_, i64>("paused")? != 0,
            daily_budget_usd: r.get("daily_budget_usd")?,
            effort: r.get("effort")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub run_id: Option<String>,
    pub title: String,
    pub input: String,
    pub status: String,
    pub position: i64,
    pub progress: i64,
    pub result_summary: String,
    pub error: String,
    pub retry_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl Task {
    pub fn from_row(r: &Row) -> SqlResult<Self> {
        Ok(Self {
            id: r.get("id")?,
            agent_id: r.get("agent_id")?,
            run_id: r.get("run_id")?,
            title: r.get("title")?,
            input: r.get("input")?,
            status: r.get("status")?,
            position: r.get("position")?,
            progress: r.get("progress")?,
            result_summary: r.get("result_summary")?,
            error: r.get("error")?,
            retry_count: r.get("retry_count")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffMessage {
    pub id: String,
    pub run_id: Option<String>,
    pub from_agent: String,
    pub to_agent: String,
    pub kind: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub pipeline: String,
    pub status: String,
    pub current_stage: String,
    pub context: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd_est: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl Run {
    pub fn from_row(r: &Row) -> SqlResult<Self> {
        Ok(Self {
            id: r.get("id")?,
            pipeline: r.get("pipeline")?,
            status: r.get("status")?,
            current_stage: r.get("current_stage")?,
            context: r.get("context")?,
            provider: r.get("provider")?,
            model: r.get("model")?,
            input_tokens: r.get("input_tokens")?,
            output_tokens: r.get("output_tokens")?,
            cost_usd_est: r.get("cost_usd_est")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub run_id: Option<String>,
    pub stage: String,
    pub payload_summary: String,
    pub artifact_refs: String,
    pub qc_score: Option<i64>,
    pub status: String,
    pub decided_at: Option<String>,
    pub note: String,
    pub created_at: String,
}

impl Approval {
    pub fn from_row(r: &Row) -> SqlResult<Self> {
        Ok(Self {
            id: r.get("id")?,
            run_id: r.get("run_id")?,
            stage: r.get("stage")?,
            payload_summary: r.get("payload_summary")?,
            artifact_refs: r.get("artifact_refs")?,
            qc_score: r.get("qc_score")?,
            status: r.get("status")?,
            decided_at: r.get("decided_at")?,
            note: r.get("note")?,
            created_at: r.get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    pub id: String,
    pub run_id: Option<String>,
    pub kind: String,
    pub file_path: String,
    pub drive_file_id: String,
    pub drive_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub name: String,
    pub version: String,
    pub path: String,
    pub owner_agent: String,
    pub enabled: bool,
    pub description: String,
}
