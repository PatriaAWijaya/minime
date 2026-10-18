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
        match execute_pipeline(&db2, &tx2, &repo, &p, &run_id2, None).await {
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

/// Resume a run from a specific stage (used after a rejection → revision).
/// The producing stage re-runs, then the pipeline continues normally
/// (including a fresh QC + new approval record).
pub async fn resume_pipeline(
    db: Db,
    tx: broadcast::Sender<Event>,
    repo: PathBuf,
    run_id: &str,
    from_stage: &str,
    revision_note: &str,
) -> Result<()> {
    // Catat revisi di context.
    {
        let guard = db.lock().await;
        let ctx_raw: String = guard
            .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
                r.get(0)
            })
            .unwrap_or_else(|_| "{}".to_string());
        let mut ctx: serde_json::Value =
            serde_json::from_str(&ctx_raw).unwrap_or(serde_json::json!({}));
        let revs = ctx.get("revisions").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
        ctx["revisions"] = serde_json::json!(revs);
        ctx["revision_note"] = serde_json::json!(revision_note);
        guard.execute(
            "UPDATE runs SET context=?1, status='running', current_stage=?2, updated_at=datetime('now') WHERE id=?3",
            params![ctx.to_string(), from_stage, run_id],
        )?;
    }
    tx.send(Event::RunUpdated {
        run_id: run_id.to_string(),
        status: "running".to_string(),
        current_stage: from_stage.to_string(),
    })
    .ok();

    let agents = crate::agent_loader::agent_ids(&db).await?;
    let skills = crate::agent_loader::skill_names(&db).await?;
    // Pipeline name from the run row.
    let pipeline_name: String = {
        let guard = db.lock().await;
        guard.query_row("SELECT pipeline FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })?
    };
    let p = pipeline::load_pipeline(&repo, &pipeline_name)?;
    pipeline::validate_pipeline(&p, &repo, &agents, &skills)?;

    let db2 = db.clone();
    let tx2 = tx.clone();
    let repo2 = repo.clone();
    let run_id2 = run_id.to_string();
    let from2 = from_stage.to_string();
    tokio::spawn(async move {
        if let Err(e) =
            execute_pipeline(&db2, &tx2, &repo2, &p, &run_id2, Some(from2.as_str())).await
        {
            eprintln!("[runner] resume run {run_id2} failed: {e:#}");
            let guard = db2.lock().await;
            guard
                .execute(
                    "UPDATE runs SET status='failed', updated_at=datetime('now') WHERE id=?1",
                    [&run_id2],
                )
                .ok();
        }
    });
    Ok(())
}

/// Revision count for a run (from runs.context).
pub async fn revision_count(db: &Db, run_id: &str) -> i64 {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str::<serde_json::Value>(&ctx_raw)
        .ok()
        .and_then(|v| v.get("revisions").and_then(|r| r.as_i64()))
        .unwrap_or(0)
}

async fn execute_pipeline(
    db: &Db,
    tx: &broadcast::Sender<Event>,
    repo: &Path,
    p: &pipeline::PipelineRaw,
    run_id: &str,
    start_from: Option<&str>,
) -> Result<()> {
    let mut prev_agent = "pipeline".to_string();
    let mut skipping = start_from.is_some();
    for stage in &p.stages {
        if skipping {
            if Some(stage.id.as_str()) == start_from {
                skipping = false;
            } else {
                continue;
            }
        }
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
                &format!(
                    "stage '{}' selesai ({} cabang paralel)",
                    stage.id,
                    stage.parallel.len()
                ),
            )
            .await?;
        } else {
            let title = if stage.title.is_empty() {
                stage.id.clone()
            } else {
                stage.title.clone()
            };
            let mut task_input = serde_json::json!({"stage": stage.id, "skills": stage.skills, "notes": stage.notes, "pipeline": p.name});
            // Fase 4: tool-driven stage — teruskan spesifikasi tool ke task.
            if let Some(tool) = &stage.tool {
                task_input["tool"] = tool.clone();
            }
            // Saat resume setelah revisi: sertakan catatan Patria agar agen memperbaikinya.
            if start_from.is_some() {
                if let Some(note) = revision_note_of(db, run_id).await {
                    task_input["revision_note"] = serde_json::json!(note);
                    // Hapus agar tidak terbawa ke stage berikutnya.
                    clear_revision_note(db, run_id).await;
                }
            }
            let task_id = enqueue_task(db, &stage.agent, Some(run_id), &title, &task_input).await?;
            wait_task(db, &task_id).await?;
            // Handoff NYATA: baca output stage dari runs.context.
            let stage_output = stage_output(db, run_id, &stage.id).await;
            let handoff_text = if stage_output.trim().is_empty() {
                format!("stage '{}' selesai → handoff", stage.id)
            } else {
                let clipped: String = stage_output.chars().take(1200).collect();
                format!("stage '{}' selesai → handoff:\n{clipped}", stage.id)
            };
            insert_message(
                db,
                Some(run_id),
                &prev_agent,
                &stage.agent,
                "handoff",
                &handoff_text,
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
                // Ambil output QC dari context untuk skor + ringkasan nyata.
                let qc_output = stage_output(db, run_id, &stage.id).await;
                let (mut score, mut reason) = parse_qc_score(&qc_output);
                // Bila stage approval bukan stage QC (mis. distribusi setelah QC),
                // cari skor di output stage-stage sebelumnya.
                if score.is_none() {
                    if let Some((s, r)) = qc_score_from_any_stage(db, run_id).await {
                        score = Some(s);
                        reason = r;
                    }
                }
                let summary = if qc_output.trim().is_empty() {
                    format!("Pipeline '{}' menunggu persetujuan Patria.", p.name)
                } else {
                    let clipped: String = qc_output.chars().take(800).collect();
                    match score {
                        Some(s) => format!("Skor QC: {s}/100. {reason}\n\n{clipped}"),
                        None => clipped,
                    }
                };
                let approval_id = Uuid::new_v4().to_string();
                // Kumpulkan artefak semua stage (mis. MP4 dari stage tool-driven)
                // → artifact_refs, supaya approval & deliverable merujuk file nyata.
                let artifact_refs: Vec<String> = {
                    let ctx_raw: String = {
                        let guard = db.lock().await;
                        guard
                            .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
                                r.get(0)
                            })
                            .unwrap_or_else(|_| "{}".to_string())
                    };
                    serde_json::from_str::<serde_json::Value>(&ctx_raw)
                        .ok()
                        .and_then(|v| v.get("stages").cloned())
                        .and_then(|s| s.as_object().cloned())
                        .map(|stages| {
                            let mut out = Vec::new();
                            for (_id, sv) in stages {
                                if let Some(arr) = sv.get("artifacts").and_then(|a| a.as_array()) {
                                    for a in arr {
                                        if let Some(p) = a.as_str() {
                                            out.push(p.to_string());
                                        }
                                    }
                                }
                            }
                            out
                        })
                        .unwrap_or_default()
                };
                let artifact_refs_json =
                    serde_json::to_string(&artifact_refs).unwrap_or_else(|_| "[]".to_string());
                {
                    let guard = db.lock().await;
                    guard.execute(
                        "INSERT INTO approvals (id, run_id, stage, payload_summary, qc_score, artifact_refs, status)
                         VALUES (?1,?2,?3,?4,?5,?6,'pending')",
                        params![approval_id, run_id, stage.id, summary, score, artifact_refs_json],
                    )?;
                    guard.execute(
                        "UPDATE runs SET status='waiting-approval', updated_at=datetime('now') WHERE id=?1",
                        [run_id],
                    )?;
                }
                // Notifikasi native — berfungsi walau UI tertutup (rule: human gate).
                crate::notify::approval_requested(&stage.id, &summary);
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

/// Baca revision_note dari context (sekali pakai).
async fn revision_note_of(db: &Db, run_id: &str) -> Option<String> {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str::<serde_json::Value>(&ctx_raw)
        .ok()
        .and_then(|v| {
            v.get("revision_note")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
}

async fn clear_revision_note(db: &Db, run_id: &str) {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    let mut ctx: serde_json::Value =
        serde_json::from_str(&ctx_raw).unwrap_or(serde_json::json!({}));
    if let Some(obj) = ctx.as_object_mut() {
        obj.remove("revision_note");
        guard
            .execute(
                "UPDATE runs SET context=?1 WHERE id=?2",
                params![ctx.to_string(), run_id],
            )
            .ok();
    }
}

/// Baca output sebuah stage dari runs.context.stages.<stage>.output.
async fn stage_output(db: &Db, run_id: &str, stage_id: &str) -> String {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str::<serde_json::Value>(&ctx_raw)
        .ok()
        .and_then(|v| {
            v.get("stages")?
                .get(stage_id)?
                .get("output")?
                .as_str()
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Cari "SKOR: <0-100>" di output SEMUA stage (untuk approval yang stage-nya
/// bukan stage QC, mis. distribusi setelah QC di pipeline konten-video-otomatis).
async fn qc_score_from_any_stage(db: &Db, run_id: &str) -> Option<(i64, String)> {
    let guard = db.lock().await;
    let ctx_raw: String = guard
        .query_row("SELECT context FROM runs WHERE id = ?1", [run_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "{}".to_string());
    drop(guard);
    let stages = serde_json::from_str::<serde_json::Value>(&ctx_raw)
        .ok()?
        .get("stages")?
        .as_object()?
        .clone();
    for (_id, sv) in &stages {
        if let Some(out) = sv.get("output").and_then(|o| o.as_str()) {
            let (score, reason) = parse_qc_score(out);
            if let Some(s) = score {
                return Some((s, reason));
            }
        }
    }
    None
}

/// Parse "SKOR: <0-100>" dari output agen QC → (skor, alasan singkat).
/// Format ini diinstruksikan di system-prompt agen qc.
fn parse_qc_score(output: &str) -> (Option<i64>, String) {
    let mut score: Option<i64> = None;
    let mut reason = String::new();
    for line in output.lines() {
        let t = line.trim();
        let upper = t.to_uppercase();
        if upper.starts_with("SKOR") {
            // Format: "SKOR: 85" atau "SKOR 85/100"
            let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
            // Ambil angka pertama yang masuk akal (0-100)
            let mut num = String::new();
            for ch in digits.chars() {
                num.push(ch);
                if let Ok(n) = num.parse::<i64>() {
                    if n > 100 {
                        num.pop();
                        break;
                    }
                }
            }
            if let Ok(n) = num.parse::<i64>() {
                if (0..=100).contains(&n) {
                    score = Some(n);
                }
            }
        } else if upper.starts_with("ALASAN") {
            let r = t
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .trim_start_matches("ALASAN")
                .trim_start_matches(|c: char| c == ':' || c == '-' || c.is_whitespace());
            reason = r.chars().take(300).collect();
        }
    }
    (score, reason)
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
