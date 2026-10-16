//! Unix-socket IPC server: JSON-lines protocol (arsitektur §6).
//! Only the daemon writes the DB; the UI talks exclusively through here.

use crate::db::Db;
use crate::events::Event;
use crate::executor::enqueue_task;
use crate::models::{Agent, Approval, Deliverable, Run, Task};
use crate::pipeline;
use crate::runner;
use anyhow::{Context, Result};
use rusqlite::params;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct App {
    pub db: Db,
    pub tx: broadcast::Sender<Event>,
    pub repo: PathBuf,
    pub start: std::time::Instant,
    pub version: String,
}

pub async fn serve(app: App, sock: &Path) -> Result<()> {
    if sock.exists() {
        std::fs::remove_file(sock).ok();
    }
    let listener = UnixListener::bind(sock)
        .with_context(|| format!("bind {}", sock.display()))?;
    loop {
        let (stream, _) = listener.accept().await?;
        let app = app.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(app, stream).await {
                eprintln!("[ipc] connection error: {e:#}");
            }
        });
    }
}

async fn handle_conn(app: App, stream: UnixStream) -> Result<()> {
    let (rd, mut wr) = stream.into_split();
    let mut reader = BufReader::new(rd);
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let req: Value = match serde_json::from_str(line.trim()) {
            Ok(v) => v,
            Err(e) => {
                write_msg(
                    &mut wr,
                    &json!({"id": null, "ok": false, "error": format!("bad json: {e}")}),
                )
                .await?;
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        if method == "subscribe" {
            write_msg(
                &mut wr,
                &json!({"id": id, "ok": true, "result": {"subscribed": true}}),
            )
            .await?;
            let mut rx = app.tx.subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        let msg = serde_json::to_string(&ev)? + "\n";
                        if wr.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            break;
        }

        let params = req.get("params").cloned().unwrap_or(Value::Null);
        let out = match dispatch(&app, &method, params).await {
            Ok(result) => json!({"id": id, "ok": true, "result": result}),
            Err(e) => json!({"id": id, "ok": false, "error": format!("{e:#}")}),
        };
        write_msg(&mut wr, &out).await?;
    }
    Ok(())
}

async fn write_msg(
    wr: &mut tokio::net::unix::OwnedWriteHalf,
    v: &Value,
) -> Result<()> {
    let mut s = serde_json::to_string(v)?;
    s.push('\n');
    wr.write_all(s.as_bytes()).await?;
    Ok(())
}

fn p_str(params: &Value, key: &str) -> Option<String> {
    params.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

async fn dispatch(app: &App, method: &str, params: Value) -> Result<Value> {
    let db = &app.db;
    match method {
        "daemon.status" => {
            let guard = db.lock().await;
            let agents: i64 =
                guard.query_row("SELECT COUNT(*) FROM agents", [], |r| r.get(0))?;
            let queued: i64 = guard.query_row(
                "SELECT COUNT(*) FROM tasks WHERE status = 'queued'",
                [],
                |r| r.get(0),
            )?;
            let running: i64 = guard.query_row(
                "SELECT COUNT(*) FROM tasks WHERE status = 'running'",
                [],
                |r| r.get(0),
            )?;
            Ok(json!({
                "version": app.version,
                "uptime_secs": app.start.elapsed().as_secs(),
                "agents": agents,
                "queued_tasks": queued,
                "running_tasks": running,
            }))
        }

        "agents.list" => {
            let guard = db.lock().await;
            let mut stmt = guard.prepare("SELECT * FROM agents ORDER BY id")?;
            let agents = stmt
                .query_map([], Agent::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(agents))
        }
        "agents.pause" => {
            let id = p_str(&params, "agent_id").context("params.agent_id required")?;
            let guard = db.lock().await;
            guard.execute("UPDATE agents SET paused = 1 WHERE id = ?1", [&id])?;
            Ok(json!({"paused": id}))
        }
        "agents.resume" => {
            let id = p_str(&params, "agent_id").context("params.agent_id required")?;
            let guard = db.lock().await;
            guard.execute("UPDATE agents SET paused = 0 WHERE id = ?1", [&id])?;
            Ok(json!({"resumed": id}))
        }

        "tasks.enqueue" => {
            let agent_id = p_str(&params, "agent_id").context("params.agent_id required")?;
            let title = p_str(&params, "title").unwrap_or_else(|| "Task manual".to_string());
            let input = params.get("input").cloned().unwrap_or(json!({}));
            let run_id = p_str(&params, "run_id");
            let id = enqueue_task(db, &agent_id, run_id.as_deref(), &title, &input).await?;
            Ok(json!({"task_id": id}))
        }
        "tasks.list" => {
            let guard = db.lock().await;
            let mut sql = String::from("SELECT * FROM tasks WHERE 1=1");
            if let Some(a) = p_str(&params, "agent_id") {
                sql.push_str(&format!(
                    " AND agent_id = '{}'",
                    a.replace('\'', "''")
                ));
            }
            if let Some(s) = p_str(&params, "status") {
                sql.push_str(&format!(" AND status = '{}'", s.replace('\'', "''")));
            }
            sql.push_str(" ORDER BY created_at DESC LIMIT 200");
            let mut stmt = guard.prepare(&sql)?;
            let tasks = stmt
                .query_map([], Task::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(tasks))
        }
        "tasks.cancel" => {
            let id = p_str(&params, "task_id").context("params.task_id required")?;
            let guard = db.lock().await;
            guard.execute(
                "UPDATE tasks SET status='cancelled', updated_at=datetime('now')
                 WHERE id = ?1 AND status IN ('queued','running','paused')",
                [&id],
            )?;
            Ok(json!({"cancelled": id}))
        }
        "tasks.retry" => {
            let id = p_str(&params, "task_id").context("params.task_id required")?;
            let guard = db.lock().await;
            guard.execute(
                "UPDATE tasks SET status='queued', progress=0, error='', retry_count=retry_count+1,
                 updated_at=datetime('now') WHERE id = ?1 AND status IN ('failed','cancelled')",
                [&id],
            )?;
            Ok(json!({"retried": id}))
        }

        "runs.list" => {
            let limit: i64 = params
                .get("limit")
                .and_then(|v| v.as_i64())
                .unwrap_or(50)
                .clamp(1, 200);
            let guard = db.lock().await;
            let mut stmt = guard.prepare("SELECT * FROM runs ORDER BY created_at DESC LIMIT ?1")?;
            let runs = stmt
                .query_map([limit], Run::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(runs))
        }
        "runs.get" => {
            let id = p_str(&params, "run_id").context("params.run_id required")?;
            let guard = db.lock().await;
            let run = guard
                .query_row("SELECT * FROM runs WHERE id = ?1", [&id], Run::from_row)
                .context("run not found")?;
            Ok(json!(run))
        }

        "approvals.list" => {
            let guard = db.lock().await;
            let mut sql = String::from("SELECT * FROM approvals WHERE 1=1");
            if let Some(s) = p_str(&params, "status") {
                sql.push_str(&format!(" AND status = '{}'", s.replace('\'', "''")));
            }
            sql.push_str(" ORDER BY created_at DESC LIMIT 100");
            let mut stmt = guard.prepare(&sql)?;
            let items = stmt
                .query_map([], Approval::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(items))
        }

        "deliverables.list" => {
            let guard = db.lock().await;
            let mut stmt =
                guard.prepare("SELECT * FROM deliverables ORDER BY created_at DESC LIMIT 100")?;
            let items = stmt
                .query_map([], |r| {
                    Ok(Deliverable {
                        id: r.get("id")?,
                        run_id: r.get("run_id")?,
                        kind: r.get("kind")?,
                        file_path: r.get("file_path")?,
                        drive_file_id: r.get("drive_file_id")?,
                        drive_url: r.get("drive_url")?,
                        created_at: r.get("created_at")?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(items))
        }

        "pipelines.list" => {
            let list = pipeline::list_pipelines(&app.repo);
            Ok(json!(list
                .into_iter()
                .map(|(name, description)| json!({"name": name, "description": description}))
                .collect::<Vec<_>>()))
        }
        "pipelines.run" => {
            let name = p_str(&params, "pipeline").context("params.pipeline required")?;
            let input = params.get("input").cloned().unwrap_or(json!({}));
            let run_id = runner::run_pipeline(
                db.clone(),
                app.tx.clone(),
                app.repo.clone(),
                &name,
                input,
            )
            .await?;
            Ok(json!({"run_id": run_id}))
        }

        _ => anyhow::bail!("unknown method '{method}'"),
    }
}
