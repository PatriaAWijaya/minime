//! Pipeline runner: validate → create run → execute stages FIFO → handoff via
//! messages → terminal Approval/End. Fan-out via `parallel` + join.
//! Fase 1: branches (`when`) are validated but not evaluated (logged + skipped);
//! condition evaluation lands in Fase 2.

use crate::db::Db;
use crate::events::Event;
use crate::executor::enqueue_task;
use crate::pipeline::{self, Transition};
use anyhow::{Context, Result};
use rusqlite::params;
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Validate + create the run, then execute in the background. Returns run_id immediately.
pub async fn run_pipeline(
    db: Db,
    tx: broadcast::Sender<Event>,
    repo: PathBuf,
    name: &str,
    input: serde_json::Value,
) -> Result<String> {
    let agents = crate::agent_loader::agent_ids(&db).await?;
    let skills = crate::agent_loader::skill_names(&db).await?;
    let p = pipeline::load_pipeline(&repo, name)?;
    pipeline::validate_pipeline(&p, &repo, &agents, &skills)
        .with_context(|| format!("cannot run invalid pipeline '{name}'"))?;

    let run_id = Uuid::new_v4().to_string();
    {
        let guard = db.lock().await;
        guard.execute(
            "INSERT INTO runs (id, pipeline, status, current_stage, context) VALUES (?1,?2,'running','',?3)",
            params![
                run_id,
                name,
                serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_string())
            ],
        )?;
    }
    tx.send(Event::RunUpdated {
        run_id: run_id.clone(),
        status: "running".to_string(),
        current_stage: "".to_string(),
    })
    .ok();

    let db2 = db.clone();
    let tx2 = tx.clone();
    let name2 = name.to_string();
    let run_id2 = run_id.clone();
    tokio::spawn(async move {
        match execute_pipeline(&db2, &tx2, &repo, &p, &run_id2).await {
            Ok(()) => {}
            Err(e) => {
                eprintln!("[runner] pipeline '{name2}' run {run_id2} failed: {e:#}");
                let guard = db2.lock().await;
                guard
                    .execute(
                        "UPDATE runs SET status='failed', updated_at=datetime('now') WHERE id=?1",
                        [&run_id2],
                    )
                    .ok();
                tx2.send(Event::RunUpdated {
                    run_id: run_id2.clone(),
                    status: "failed".to_string(),
                    current_stage: "".to_string(),
                })
                .ok();
            }
        }
    });
    Ok(run_id)
}

async fn execute_pipeline(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    repo: &Path,
    p: &pipeline::PipelineRaw,
    run_id: &str,
) -> Result<()> {
    let mut prev_agent = "pipeline".to_string();
    for stage in &p.stages {
        set_run_stage(db, tx, run_id, &stage.id).await?;

        if !stage.parallel.is_empty() {
            // Fan-out: enqueue every branch, then join all.
            let mut handles = Vec::new();
            for b in &stage.parallel {
                let branch_title = if b.title.is_empty() {
                    format!("{} (paralel)", stage.id)
                } else {
                    b.title.clone()
                };
                let task_id = enqueue_task(
                    db,
                    &b.agent,
                    Some(run_id),
                    &branch_title,
                    &serde_json::json!({"stage": stage.id, "skills": b.skills, "pipeline": p.name}),
                )
                .await?;
                let dbw = db.clone();
                handles.push(tokio::spawn(async move { wait_task(&dbw, &task_id).await }));
            }
            for h in handles {
                h.await??;
            }
            insert_message(
                db,
                Some(run_id),
                &prev_agent,
                "pipeline",
                "fanout-join",
                &format!("stage '{}' selesai ({} cabang paralel)", stage.id, stage.parallel.len()),
            )
            .await?;
        } else {
            let title = if stage.title.is_empty() {
                stage.id.clone()
            } else {
                stage.title.clone()
            };
            let task_id = enqueue_task(
                db,
                &stage.agent,
                Some(run_id),
                &title,
                &serde_json::json!({"stage": stage.id, "skills": stage.skills, "notes": stage.notes, "pipeline": p.name}),
            )
            .await?;
            wait_task(db, &task_id).await?;
            insert_message(
                db,
                Some(run_id),
                &prev_agent,
                &stage.agent,
                "handoff",
                &format!("stage '{}' selesai → handoff", stage.id),
            )
            .await?;
            prev_agent = stage.agent.clone();
        }

        for b in &stage.branches {
            eprintln!(
                "[runner] branch '{}' dilewati di Fase 1 (evaluasi kondisi di Fase 2)",
                b.when
            );
            let _ = repo; // sub-pipelines already validated; execution wiring lands in Fase 2
        }

        match Transition::parse(&stage.on_done)? {
            Transition::Handoff(next) => {
                insert_message(
                    db,
                    Some(run_id),
                    &prev_agent,
                    &next,
                    "handoff",
                    &format!("handoff: {} → {}", prev_agent, next),
                )
                .await?;
                prev_agent = next;
            }
            Transition::Approval => {
                let approval_id = Uuid::new_v4().to_string();
                {
                    let guard = db.lock().await;
                    guard.execute(
                        "INSERT INTO approvals (id, run_id, stage, payload_summary, status)
                         VALUES (?1,?2,?3,?4,'pending')",
                        params![
                            approval_id,
                            run_id,
                            stage.id,
                            format!("Pipeline '{}' menunggu persetujuan Patria (Fase 1: dummy).", p.name),
                        ],
                    )?;
                    guard.execute(
                        "UPDATE runs SET status='waiting-approval', updated_at=datetime('now') WHERE id=?1",
                        [run_id],
                    )?;
                }
                tx.send(Event::RunUpdated {
                    run_id: run_id.to_string(),
                    status: "waiting-approval".to_string(),
                    current_stage: stage.id.clone(),
                })
                .ok();
                tx.send(Event::ApprovalRequested {
                    approval_id,
                    run_id: run_id.to_string(),
                })
                .ok();
                return Ok(());
            }
            Transition::End => {
                let guard = db.lock().await;
                guard.execute(
                    "UPDATE runs SET status='done', updated_at=datetime('now') WHERE id=?1",
                    [run_id],
                )?;
                tx.send(Event::RunUpdated {
                    run_id: run_id.to_string(),
                    status: "done".to_string(),
                    current_stage: stage.id.clone(),
                })
                .ok();
                return Ok(());
            }
        }
    }
    // Fell off the end without a terminal transition: mark done honestly.
    let guard = db.lock().await;
    guard.execute(
        "UPDATE runs SET status='done', updated_at=datetime('now') WHERE id=?1",
        [run_id],
    )?;
    Ok(())
}

/// Poll a task until it reaches a terminal state.
async fn wait_task(db: &Db, task_id: &str) -> Result<()> {
    loop {
        let status: String = {
            let guard = db.lock().await;
            guard.query_row("SELECT status FROM tasks WHERE id = ?1", [task_id], |r| {
                r.get(0)
            })?
        };
        match status.as_str() {
            "done" => return Ok(()),
            "failed" | "cancelled" => {
                anyhow::bail!("task {task_id} ended with status '{status}'")
            }
            _ => tokio::time::sleep(std::time::Duration::from_millis(400)).await,
        }
    }
}

async fn set_run_stage(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    run_id: &str,
    stage: &str,
) -> Result<()> {
    let guard = db.lock().await;
    guard.execute(
        "UPDATE runs SET current_stage=?1, updated_at=datetime('now') WHERE id=?2",
        params![stage, run_id],
    )?;
    tx.send(Event::RunUpdated {
        run_id: run_id.to_string(),
        status: "running".to_string(),
        current_stage: stage.to_string(),
    })
    .ok();
    Ok(())
}

async fn insert_message(
    db: &Db,
    run_id: Option<&str>,
    from_agent: &str,
    to_agent: &str,
    kind: &str,
    content: &str,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let guard = db.lock().await;
    guard.execute(
        "INSERT INTO messages (id, run_id, from_agent, to_agent, kind, content)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![id, run_id, from_agent, to_agent, kind, content],
    )?;
    Ok(())
}
