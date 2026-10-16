//! FIFO task executor: one running task per agent, pause/resume/cancel.
//! Fase 1 executes tasks as a *simulated* run (progress events, no LLM) so the
//! full mechanics — queue, IPC, events, pipeline runner — are verifiable now.

use crate::db::Db;
use crate::events::Event;
use anyhow::Result;
use std::time::Duration;
use tokio::sync::broadcast;
use uuid::Uuid;

pub struct Executor {
    db: Db,
    tx: broadcast::Sender<Event>,
}

impl Executor {
    pub fn new(db: Db, tx: broadcast::Sender<Event>) -> Self {
        Self { db, tx }
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
        self.tx.send(Event::AgentStatus {
            agent_id: agent_id.to_string(),
            status: "working".to_string(),
        }).ok();
        self.tx.send(Event::TaskUpdated {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            status: "running".to_string(),
            progress: 0,
        }).ok();

        let db = self.db.clone();
        let tx = self.tx.clone();
        let task_id = task_id.to_string();
        let agent_id = agent_id.to_string();
        let title = title.to_string();
        tokio::spawn(async move {
            run_dummy(&db, &tx, &task_id, &agent_id, &title).await;
        });
        Ok(())
    }
}

/// Fase 1 dummy execution: simulated progress over ~4s, then done.
/// Checks cancellation between steps. Real LLM execution lands in Fase 2.
async fn run_dummy(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    task_id: &str,
    agent_id: &str,
    title: &str,
) {
    for progress in [15_i64, 35, 60, 85, 100] {
        tokio::time::sleep(Duration::from_millis(800)).await;
        let status: String = {
            let guard = db.lock().await;
            guard
                .query_row("SELECT status FROM tasks WHERE id = ?1", [task_id], |r| {
                    r.get(0)
                })
                .unwrap_or_else(|_| "failed".to_string())
        };
        if status == "cancelled" {
            tx.send(Event::TaskUpdated {
                task_id: task_id.to_string(),
                agent_id: agent_id.to_string(),
                status: "cancelled".to_string(),
                progress,
            })
            .ok();
            relax_agent(db, tx, agent_id).await;
            return;
        }
        {
            let guard = db.lock().await;
            guard
                .execute(
                    "UPDATE tasks SET progress = ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![progress, task_id],
                )
                .ok();
        }
        tx.send(Event::TaskUpdated {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            status: "running".to_string(),
            progress,
        })
        .ok();
    }
    let summary = format!(
        "Fase 1 dummy execution selesai (tanpa LLM). Task: \"{title}\" — siap disambung ke provider di Fase 2."
    );
    {
        let guard = db.lock().await;
        guard
            .execute(
                "UPDATE tasks SET status='done', progress=100, result_summary=?1, updated_at=datetime('now') WHERE id=?2",
                rusqlite::params![summary, task_id],
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
