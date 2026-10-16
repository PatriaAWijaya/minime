//! FIFO task executor: one running task per agent, pause/resume/cancel.
//! Fase 2: real LLM execution via the provider registry with per-agent
//! routing + fallback chain, budget-cap guard, and metering into `runs`.

use crate::db::Db;
use crate::events::Event;
use crate::models::Agent;
use crate::provider::{ChatRequest, ModelProvider, ProviderError};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Shared provider registry: provider id → implementation.
pub type Providers = Arc<HashMap<String, Arc<dyn ModelProvider>>>;

pub struct Executor {
    db: Db,
    tx: broadcast::Sender<Event>,
    providers: Providers,
}

impl Executor {
    pub fn new(db: Db, tx: broadcast::Sender<Event>, providers: Providers) -> Self {
        Self { db, tx, providers }
    }

    /// Spawn the background scheduling loop.
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.tick().await {
                    eprintln!("[executor] tick error: {e:#}");
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
    }

    async fn tick(&self) -> Result<()> {
        let agents: Vec<String> = {
            let guard = self.db.lock().await;
            let mut stmt = guard.prepare("SELECT id FROM agents WHERE paused = 0 ORDER BY id")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            let mut out = Vec::new();
            for id in rows {
                out.push(id?);
            }
            out
        };
        for agent_id in agents {
            let running: i64 = {
                let guard = self.db.lock().await;
                guard.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE agent_id = ?1 AND status = 'running'",
                    [&agent_id],
                    |r| r.get(0),
                )?
            };
            if running > 0 {
                continue;
            }
            let next: Option<(String, String)> = {
                let guard = self.db.lock().await;
                guard
                    .query_row(
                        "SELECT id, title FROM tasks WHERE agent_id = ?1 AND status = 'queued'
                         ORDER BY position ASC, created_at ASC LIMIT 1",
                        [&agent_id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .ok()
            };
            if let Some((task_id, title)) = next {
                self.claim_and_run(&agent_id, &task_id, &title).await?;
            }
        }
        Ok(())
    }

    /// CAS-claim a queued task, then run it on a worker task.
    async fn claim_and_run(&self, agent_id: &str, task_id: &str, title: &str) -> Result<()> {
        let changed = {
            let guard = self.db.lock().await;
            guard.execute(
                "UPDATE tasks SET status='running', progress=0, updated_at=datetime('now')
                 WHERE id = ?1 AND status = 'queued'",
                [task_id],
            )?
        };
        if changed == 0 {
            return Ok(()); // someone else claimed it
        }
        set_agent_status(&self.db, agent_id, "working").await?;
        self.tx
            .send(Event::AgentStatus {
                agent_id: agent_id.to_string(),
                status: "working".to_string(),
            })
            .ok();
        self.tx
            .send(Event::TaskUpdated {
                task_id: task_id.to_string(),
                agent_id: agent_id.to_string(),
                status: "running".to_string(),
                progress: 0,
            })
            .ok();

        let db = self.db.clone();
        let tx = self.tx.clone();
        let providers = self.providers.clone();
        let task_id = task_id.to_string();
        let agent_id = agent_id.to_string();
        let title = title.to_string();
        tokio::spawn(async move {
            run_llm(&db, &tx, &providers, &task_id, &agent_id, &title).await;
        });
        Ok(())
    }
}

/// Real LLM execution for one task:
/// budget guard → build prompt → walk provider chain → metering → done/failed.
async fn run_llm(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    providers: &Providers,
    task_id: &str,
    agent_id: &str,
    title: &str,
) {
    // 1. Load agent definition.
    let agent: Agent = {
        let guard = db.lock().await;
        match guard.query_row(
            "SELECT * FROM agents WHERE id = ?1",
            [agent_id],
            Agent::from_row,
        ) {
            Ok(a) => a,
            Err(e) => {
                fail_task(
                    db,
                    tx,
                    task_id,
                    agent_id,
                    &format!("agen tidak ditemukan: {e}"),
                )
                .await;
                return;
            }
        }
    };

    // 2. Budget-cap guard (rule: cap tercapai → pause + butuh persetujuan Patria).
    if agent.daily_budget_usd > 0.0 {
        let spent = daily_spend(db, agent_id).await.unwrap_or(0.0);
        if spent >= agent.daily_budget_usd {
            eprintln!(
                "[executor] budget harian agen '{agent_id}' tercapai (${spent:.2} >= ${:.2}) — task di-pause",
                agent.daily_budget_usd
            );
            let guard = db.lock().await;
            guard
                .execute(
                    "UPDATE tasks SET status='paused', error='budget harian tercapai — butuh persetujuan Patria', updated_at=datetime('now') WHERE id=?1",
                    [task_id],
                )
                .ok();
            drop(guard);
            tx.send(Event::TaskUpdated {
                task_id: task_id.to_string(),
                agent_id: agent_id.to_string(),
                status: "paused".to_string(),
                progress: 0,
            })
            .ok();
            relax_agent(db, tx, agent_id).await;
            return;
        }
    }

    // 3. Assemble prompt: system prompt + skill summaries + task input + run context.
    let task_input: serde_json::Value = {
        let guard = db.lock().await;
        let raw: String = guard
            .query_row("SELECT input FROM tasks WHERE id = ?1", [task_id], |r| {
                r.get(0)
            })
            .unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
    };
    let run_id: Option<String> = {
        let guard = db.lock().await;
        guard
            .query_row("SELECT run_id FROM tasks WHERE id = ?1", [task_id], |r| {
                r.get::<_, Option<String>>(0)
            })
            .unwrap_or(None)
    };
    let system_prompt = build_system_prompt(db, &agent, &task_input).await;
    let user_text = build_user_text(db, run_id.as_deref(), &task_input, title).await;

    let req = ChatRequest {
        model: agent.model.clone(),
        system_prompt,
        messages: serde_json::json!([{"role": "user", "content": user_text}]),
        effort: agent.effort.clone(),
        max_tokens: Some(2048),
        workdir: if agent.workspace_path.is_empty() {
            None
        } else {
            Some(agent.workspace_path.clone())
        },
    };

    // 4. Walk the provider chain: (primary) + fallback_chain.
    let mut chain: Vec<(String, String)> = vec![(agent.provider_id.clone(), agent.model.clone())];
    if let Ok(fb) = serde_json::from_str::<Vec<serde_json::Value>>(&agent.fallback_chain) {
        for f in fb {
            let p = f.get("provider").and_then(|v| v.as_str()).unwrap_or("");
            let m = f.get("model").and_then(|v| v.as_str()).unwrap_or("");
            if !p.is_empty() {
                chain.push((p.to_string(), m.to_string()));
            }
        }
    }

    let mut last_err = "tidak ada provider terdaftar".to_string();
    for (pid, mid) in &chain {
        let Some(provider) = providers.get(pid.as_str()) else {
            eprintln!("[executor] provider '{pid}' tidak terdaftar — lewati");
            last_err = format!("provider '{pid}' tidak terdaftar");
            continue;
        };
        let mut attempt = req.clone();
        attempt.model = mid.clone();
        eprintln!("[executor] task {task_id}: mencoba {pid}/{mid}");
        match complete_blocking(provider.as_ref(), &attempt, task_id, agent_id, tx).await {
            Ok((text, in_tok, out_tok)) => {
                record_metering(
                    db,
                    tx,
                    run_id.as_deref(),
                    pid,
                    mid,
                    providers,
                    in_tok,
                    out_tok,
                )
                .await;
                finish_task_done(db, tx, task_id, agent_id, &text, pid, mid).await;
                return;
            }
            Err(e) => {
                eprintln!("[executor] task {task_id}: {pid}/{mid} gagal: {e} — coba fallback");
                last_err = format!("{pid}: {e}");
            }
        }
    }
    fail_task(
        db,
        tx,
        task_id,
        agent_id,
        &format!("semua provider gagal. Terakhir: {last_err}"),
    )
    .await;
}

/// Drain a provider's stream into full text + token counts.
/// Emits progress events as chunks arrive; checks cancellation between chunks.
async fn complete_blocking(
    provider: &dyn ModelProvider,
    req: &ChatRequest,
    task_id: &str,
    agent_id: &str,
    tx: &broadcast::Sender<Event>,
) -> Result<(String, u64, u64), ProviderError> {
    let mut rx = provider.complete_stream(req.clone());
    let mut text = String::new();
    let (mut in_tok, mut out_tok) = (0u64, 0u64);
    let mut progress = 10i64;
    loop {
        match rx.recv().await {
            Some(Ok(chunk)) => {
                if !chunk.delta_text.is_empty() {
                    text.push_str(&chunk.delta_text);
                }
                if chunk.done {
                    in_tok = chunk.input_tokens;
                    out_tok = chunk.output_tokens;
                    break;
                }
                // Light progress heartbeat (capped) — real progress unknown.
                if progress < 90 {
                    progress += 7;
                    let _ = tx.send(Event::TaskUpdated {
                        task_id: task_id.to_string(),
                        agent_id: agent_id.to_string(),
                        status: "running".to_string(),
                        progress,
                    });
                }
            }
            Some(Err(e)) => return Err(e),
            None => break, // channel closed without done chunk — accept what we have
        }
    }
    if text.trim().is_empty() {
        return Err(ProviderError::Failed(
            "provider mengembalikan teks kosong".to_string(),
        ));
    }
    Ok((text, in_tok, out_tok))
}

async fn build_system_prompt(db: &Db, agent: &Agent, task_input: &serde_json::Value) -> String {
    let mut sp = agent.system_prompt.clone();
    // Append skill summaries (cached descriptions from the skills table).
    if let Ok(names) = serde_json::from_str::<Vec<String>>(&agent.skills) {
        if !names.is_empty() {
            let guard = db.lock().await;
            let mut parts = Vec::new();
            for n in &names {
                let desc: String = guard
                    .query_row("SELECT description FROM skills WHERE name = ?1", [n], |r| {
                        r.get(0)
                    })
                    .unwrap_or_default();
                if !desc.trim().is_empty() {
                    parts.push(format!("- {n}: {desc}"));
                }
            }
            drop(guard);
            if !parts.is_empty() {
                sp.push_str("\n\nSKILLS YANG KAMU KUASAI:\n");
                sp.push_str(&parts.join("\n"));
            }
        }
    }
    // Fase 3: inject full playbook bodies for stage-level skills (task input
    // "skills": [...]). These are the SOPs the agent MUST follow for this task.
    if let Some(arr) = task_input.get("skills").and_then(|v| v.as_array()) {
        let guard = db.lock().await;
        let mut bodies = Vec::new();
        for s in arr {
            let Some(name) = s.as_str() else { continue };
            let path: String = guard
                .query_row("SELECT path FROM skills WHERE name = ?1", [name], |r| {
                    r.get(0)
                })
                .unwrap_or_default();
            if path.is_empty() {
                continue;
            }
            let full = std::fs::read_to_string(std::path::Path::new(&path).join("SKILL.md"))
                .unwrap_or_default();
            let body = strip_frontmatter(&full);
            let body = body.trim();
            if body.is_empty() {
                continue;
            }
            // Cap per skill agar konteks prompt tidak meledak.
            let capped: String = body.chars().take(6000).collect();
            bodies.push(format!("=== PLAYBOOK: {name} ===\n{capped}"));
        }
        drop(guard);
        if !bodies.is_empty() {
            sp.push_str(
                "\n\nPLAYBOOK AKTIF UNTUK TUGAS INI (ikuti SOP-nya; quality bar = standar lulus; \
                 bila ragu, nyatakan eskalasi di output):\n",
            );
            sp.push_str(&bodies.join("\n\n"));
        }
    }
    sp
}

/// Strip YAML frontmatter (--- ... ---) from a SKILL.md file.
fn strip_frontmatter(text: &str) -> &str {
    let mut lines = text.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return text;
    }
    let mut offset = 0usize;
    for line in lines {
        offset += line.len() + 1; // +1 for '\n'
        if line.trim() == "---" {
            break;
        }
    }
    // offset relatif terhadap sisa setelah baris pertama
    let first_len = text.lines().next().map(|l| l.len() + 1).unwrap_or(0);
    &text[first_len + offset..]
}

async fn build_user_text(
    db: &Db,
    run_id: Option<&str>,
    task_input: &serde_json::Value,
    title: &str,
) -> String {
    let mut out = format!("TUGAS: {title}\n\n");
    out.push_str(&format!(
        "INPUT:\n{}\n\n",
        serde_json::to_string_pretty(task_input).unwrap_or_default()
    ));
    // Real handoff: previous stages' outputs from runs.context.
    if let Some(rid) = run_id {
        let guard = db.lock().await;
        let ctx_raw: String = guard
            .query_row("SELECT context FROM runs WHERE id = ?1", [rid], |r| {
                r.get(0)
            })
            .unwrap_or_else(|_| "{}".to_string());
        let recent: Vec<String> = guard
            .query_row(
                "SELECT GROUP_CONCAT(content, '\n---\n') FROM (SELECT content FROM messages WHERE run_id = ?1 ORDER BY created_at DESC LIMIT 6)",
                [rid],
                |r| r.get::<_, Option<String>>(0),
            )
            .unwrap_or(None)
            .map(|s| vec![s])
            .unwrap_or_default();
        drop(guard);
        if let Ok(ctx) = serde_json::from_str::<serde_json::Value>(&ctx_raw) {
            if let Some(stages) = ctx.get("stages").and_then(|s| s.as_object()) {
                if !stages.is_empty() {
                    out.push_str("OUTPUT TAHAP SEBELUMNYA (gunakan sebagai konteks):\n");
                    for (sid, sdata) in stages {
                        let summary = sdata.get("output").and_then(|o| o.as_str()).unwrap_or("");
                        if !summary.trim().is_empty() {
                            let clipped: String = summary.chars().take(3000).collect();
                            out.push_str(&format!("\n[{sid}]\n{clipped}\n"));
                        }
                    }
                    out.push('\n');
                }
            }
        }
        for msgs in recent {
            if !msgs.trim().is_empty() {
                let clipped: String = msgs.chars().take(2000).collect();
                out.push_str(&format!("RIWAYAT HANDOFF:\n{clipped}\n"));
            }
        }
    }
    out.push_str("\nJawablah dalam Bahasa Indonesia sesuai peran dan quality bar-mu.");
    out
}

/// Sum of today's estimated spend for an agent (via tasks → runs).
async fn daily_spend(db: &Db, agent_id: &str) -> Result<f64> {
    let guard = db.lock().await;
    let v: f64 = guard.query_row(
        "SELECT COALESCE(SUM(r.cost_usd_est), 0.0) FROM runs r
         JOIN tasks t ON t.run_id = r.id
         WHERE t.agent_id = ?1 AND date(r.created_at) = date('now')",
        [agent_id],
        |r| r.get(0),
    )?;
    Ok(v)
}

async fn record_metering(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    run_id: Option<&str>,
    provider_id: &str,
    model_id: &str,
    providers: &Providers,
    in_tok: u64,
    out_tok: u64,
) {
    let (pin, pout) = providers
        .get(provider_id)
        .map(|p| {
            p.list_models()
                .into_iter()
                .find(|m| m.id == model_id)
                .map(|m| (m.usd_per_1m_in, m.usd_per_1m_out))
                .unwrap_or((0.0, 0.0))
        })
        .unwrap_or((0.0, 0.0));
    let cost = (in_tok as f64) * pin / 1_000_000.0 + (out_tok as f64) * pout / 1_000_000.0;
    if let Some(rid) = run_id {
        let guard = db.lock().await;
        guard
            .execute(
                "UPDATE runs SET provider=?1, model=?2,
                 input_tokens=input_tokens+?3, output_tokens=output_tokens+?4,
                 cost_usd_est=cost_usd_est+?5, updated_at=datetime('now') WHERE id=?6",
                rusqlite::params![
                    provider_id,
                    model_id,
                    in_tok as i64,
                    out_tok as i64,
                    cost,
                    rid
                ],
            )
            .ok();
        drop(guard);
        tx.send(Event::RunUpdated {
            run_id: rid.to_string(),
            status: "running".to_string(),
            current_stage: String::new(),
        })
        .ok();
    }
    eprintln!("[metering] {provider_id}/{model_id}: in={in_tok} out={out_tok} cost_est=${cost:.4}");
}

async fn finish_task_done(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    task_id: &str,
    agent_id: &str,
    text: &str,
    provider_id: &str,
    model_id: &str,
) {
    // Simpan output ke task + ke runs.context.stages (handoff nyata antar stage).
    let (run_id, stage_id): (Option<String>, Option<String>) = {
        let guard = db.lock().await;
        let (run_id, input_raw): (Option<String>, String) = guard
            .query_row(
                "SELECT run_id, input FROM tasks WHERE id = ?1",
                [task_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((None, "{}".to_string()));
        let stage_id = serde_json::from_str::<serde_json::Value>(&input_raw)
            .ok()
            .and_then(|v| {
                v.get("stage")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
            });
        guard
            .execute(
                "UPDATE tasks SET status='done', progress=100, result_summary=?1, updated_at=datetime('now') WHERE id=?2",
                rusqlite::params![text, task_id],
            )
            .ok();
        (run_id, stage_id)
    };
    if let (Some(rid), Some(sid)) = (run_id, stage_id) {
        let guard = db.lock().await;
        let ctx_raw: String = guard
            .query_row("SELECT context FROM runs WHERE id = ?1", [&rid], |r| {
                r.get(0)
            })
            .unwrap_or_else(|_| "{}".to_string());
        let mut ctx: serde_json::Value =
            serde_json::from_str(&ctx_raw).unwrap_or(serde_json::json!({}));
        let clipped: String = text.chars().take(6000).collect();
        ctx["stages"][&sid] = serde_json::json!({
            "agent": agent_id,
            "provider": provider_id,
            "model": model_id,
            "output": clipped,
        });
        guard
            .execute(
                "UPDATE runs SET context=?1, updated_at=datetime('now') WHERE id=?2",
                rusqlite::params![ctx.to_string(), rid],
            )
            .ok();
    }
    tx.send(Event::TaskUpdated {
        task_id: task_id.to_string(),
        agent_id: agent_id.to_string(),
        status: "done".to_string(),
        progress: 100,
    })
    .ok();
    relax_agent(db, tx, agent_id).await;
}

async fn fail_task(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    task_id: &str,
    agent_id: &str,
    error: &str,
) {
    {
        let guard = db.lock().await;
        guard
            .execute(
                "UPDATE tasks SET status='failed', error=?1, updated_at=datetime('now') WHERE id=?2",
                rusqlite::params![error, task_id],
            )
            .ok();
    }
    eprintln!("[executor] task {task_id} failed: {error}");
    tx.send(Event::TaskUpdated {
        task_id: task_id.to_string(),
        agent_id: agent_id.to_string(),
        status: "failed".to_string(),
        progress: 0,
    })
    .ok();
    relax_agent(db, tx, agent_id).await;
}

/// Set agent back to idle when its queue is empty.
async fn relax_agent(db: &Db, tx: &broadcast::Sender<Event>, agent_id: &str) {
    let pending: i64 = {
        let guard = db.lock().await;
        guard
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE agent_id = ?1 AND status IN ('queued','running')",
                [agent_id],
                |r| r.get(0),
            )
            .unwrap_or(0)
    };
    if pending == 0 {
        set_agent_status(db, agent_id, "idle").await.ok();
        tx.send(Event::AgentStatus {
            agent_id: agent_id.to_string(),
            status: "idle".to_string(),
        })
        .ok();
    }
}

async fn set_agent_status(db: &Db, agent_id: &str, status: &str) -> Result<()> {
    let guard = db.lock().await;
    guard.execute(
        "UPDATE agents SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, agent_id],
    )?;
    Ok(())
}

/// Enqueue a task at the end of an agent's FIFO queue. Returns the task id.
pub async fn enqueue_task(
    db: &Db,
    agent_id: &str,
    run_id: Option<&str>,
    title: &str,
    input: &serde_json::Value,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let guard = db.lock().await;
    let position: i64 = guard.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM tasks WHERE agent_id = ?1 AND status IN ('queued','running','paused')",
        [agent_id],
        |r| r.get(0),
    )?;
    guard.execute(
        "INSERT INTO tasks (id, agent_id, run_id, title, input, status, position)
         VALUES (?1,?2,?3,?4,?5,'queued',?6)",
        rusqlite::params![
            id,
            agent_id,
            run_id,
            title,
            serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string()),
            position
        ],
    )?;
    Ok(id)
}
