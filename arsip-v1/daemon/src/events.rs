//! Realtime events broadcast by the daemon (→ IPC subscribers → Tauri events).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum Event {
    #[serde(rename = "minime://agent-status")]
    AgentStatus { agent_id: String, status: String },
    #[serde(rename = "minime://task-updated")]
    TaskUpdated {
        task_id: String,
        agent_id: String,
        status: String,
        progress: i64,
    },
    #[serde(rename = "minime://run-updated")]
    RunUpdated {
        run_id: String,
        status: String,
        current_stage: String,
    },
    #[serde(rename = "minime://approval-requested")]
    ApprovalRequested { approval_id: String, run_id: String },
    #[serde(rename = "minime://drive-auth")]
    DriveAuth { status: String },
}
