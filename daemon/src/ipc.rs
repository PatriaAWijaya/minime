//! Unix-socket IPC server: JSON-lines protocol (arsitektur §6).
//! Only the daemon writes the DB; the UI talks exclusively through here.

use crate::db::Db;
use crate::drive_oauth;
use crate::events::Event;
use crate::executor::enqueue_task;
use crate::models::{Agent, Approval, Deliverable, Run, Task};
use crate::pipeline;
use crate::runner;
use crate::secrets;
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
    pub data_dir: PathBuf,
    pub start: std::time::Instant,
    pub version: String,
    pub providers: crate::executor::Providers,
    pub browser_store: crate::tools::BrowserStore,
}

pub async fn serve(app: App, sock: &Path) -> Result<()> {
    if sock.exists() {
        std::fs::remove_file(sock).ok();
    }
    let listener = UnixListener::bind(sock).with_context(|| format!("bind {}", sock.display()))?;
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

async fn write_msg(wr: &mut tokio::net::unix::OwnedWriteHalf, v: &Value) -> Result<()> {
    let mut s = serde_json::to_string(v)?;
    s.push('\n');
    wr.write_all(s.as_bytes()).await?;
    Ok(())
}

fn p_str(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

async fn dispatch(app: &App, method: &str, params: Value) -> Result<Value> {
    let db = &app.db;
    match method {
        "daemon.status" => {
            let guard = db.lock().await;
            let agents: i64 = guard.query_row("SELECT COUNT(*) FROM agents", [], |r| r.get(0))?;
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
                sql.push_str(&format!(" AND agent_id = '{}'", a.replace('\'', "''")));
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
        "usage.summary" => {
            // Ringkasan penggunaan per agen: budget harian vs belanja hari ini + total.
            let guard = db.lock().await;
            let mut stmt = guard.prepare(
                "SELECT a.id, a.name, a.daily_budget_usd,
                        COALESCE(SUM(CASE WHEN date(r.created_at) = date('now') THEN r.cost_usd_est ELSE 0 END), 0.0) AS spent_today,
                        COALESCE(SUM(r.cost_usd_est), 0.0) AS spent_total,
                        COALESCE(SUM(r.input_tokens + r.output_tokens), 0) AS tokens_total
                 FROM agents a
                 LEFT JOIN tasks t ON t.agent_id = a.id
                 LEFT JOIN runs r ON r.id = t.run_id
                 GROUP BY a.id ORDER BY a.id",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(json!({
                        "agent_id": r.get::<_, String>(0)?,
                        "agent_name": r.get::<_, String>(1)?,
                        "daily_budget_usd": r.get::<_, f64>(2)?,
                        "spent_today_usd": r.get::<_, f64>(3)?,
                        "spent_total_usd": r.get::<_, f64>(4)?,
                        "tokens_total": r.get::<_, i64>(5)?,
                    }))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(rows))
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
        "approvals.decide" => {
            let approval_id =
                p_str(&params, "approval_id").context("params.approval_id required")?;
            let decision = p_str(&params, "decision").context("params.decision required")?;
            let note = p_str(&params, "note").unwrap_or_default();
            if decision != "approve" && decision != "reject" {
                anyhow::bail!("decision harus 'approve' atau 'reject'");
            }
            decide_approval(app, &approval_id, &decision, &note).await
        }

        "providers.list" => {
            // Daftar provider terdaftar + modelnya (tanpa key — tidak pernah dibaca balik).
            let mut out = Vec::new();
            for (id, p) in app.providers.iter() {
                out.push(json!({
                    "id": id,
                    "kind": format!("{:?}", p.kind()),
                    "models": p.list_models(),
                }));
            }
            Ok(json!(out))
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
            let run_id =
                runner::run_pipeline(db.clone(), app.tx.clone(), app.repo.clone(), &name, input)
                    .await?;
            Ok(json!({"run_id": run_id}))
        }

        "secrets.set" => {
            // Write-only: UI menyimpan secret ke Keychain; tidak ada jalur baca.
            let account = p_str(&params, "account").context("params.account required")?;
            let value = p_str(&params, "value").context("params.value required")?;
            if account.trim().is_empty() || value.trim().is_empty() {
                anyhow::bail!("account dan value tidak boleh kosong");
            }
            secrets::set_api_key(&account, &value)
                .map_err(|e| anyhow::anyhow!("gagal menyimpan secret: {e}"))?;
            Ok(json!({"stored": account}))
        }
        "secrets.has" => {
            // Hanya boolean — nilai secret tidak pernah dikembalikan.
            let account = p_str(&params, "account").context("params.account required")?;
            Ok(json!({"account": account, "has": secrets::api_key(&account).is_some()}))
        }

        "drive.auth_url" => drive_oauth::start_flow(app.tx.clone()).await,
        "drive.status" => Ok(drive_oauth::status()),
        "drive.disconnect" => {
            drive_oauth::disconnect()?;
            Ok(json!({"disconnected": true}))
        }

        // ---- Fase 4: tools ----
        "tools.status" => {
            let list = crate::tools::readiness(&app.data_dir, &app.repo).await;
            Ok(json!(list))
        }
        "tools.run" => {
            // Ad-hoc tool run (sinkron, timeout internal per tool).
            // Untuk pekerjaan berat (render video), pakai task pipeline sebagai gantinya.
            let tool = p_str(&params, "tool").context("params.tool required")?;
            let op = p_str(&params, "op").context("params.op required")?;
            let tool_params = params.get("params").cloned().unwrap_or(json!({}));
            let ctx = crate::tools::ToolCtx {
                data_dir: app.data_dir.clone(),
                repo_dir: app.repo.clone(),
                run_id: None,
                agent_id: p_str(&params, "agent_id").unwrap_or_else(|| "mimi".to_string()),
            };
            crate::tools::run_tool(db, &app.browser_store, &tool, &op, &tool_params, &ctx).await
        }

        // ---- Fase 4: built-in browser (dikontrol daemon) ----
        "browser.open" => {
            let ctx = browser_ctx(app, &params);
            crate::tools::run_tool(
                db,
                &app.browser_store,
                crate::tools::TOOL_BROWSER,
                "open",
                &params,
                &ctx,
            )
            .await
        }
        "browser.navigate" => {
            let ctx = browser_ctx(app, &params);
            crate::tools::run_tool(
                db,
                &app.browser_store,
                crate::tools::TOOL_BROWSER,
                "navigate",
                &params,
                &ctx,
            )
            .await
        }
        "browser.screenshot" => {
            let ctx = browser_ctx(app, &params);
            crate::tools::run_tool(
                db,
                &app.browser_store,
                crate::tools::TOOL_BROWSER,
                "screenshot",
                &params,
                &ctx,
            )
            .await
        }
        "browser.dom" => {
            let ctx = browser_ctx(app, &params);
            crate::tools::run_tool(
                db,
                &app.browser_store,
                crate::tools::TOOL_BROWSER,
                "dom",
                &params,
                &ctx,
            )
            .await
        }
        "browser.list" => {
            let ctx = browser_ctx(app, &params);
            crate::tools::run_tool(
                db,
                &app.browser_store,
                crate::tools::TOOL_BROWSER,
                "list",
                &params,
                &ctx,
            )
            .await
        }
        "browser.close" => {
            let ctx = browser_ctx(app, &params);
            crate::tools::run_tool(
                db,
                &app.browser_store,
                crate::tools::TOOL_BROWSER,
                "close",
                &params,
                &ctx,
            )
            .await
        }

        _ => anyhow::bail!("unknown method '{method}'"),
    }
}

/// Build a ToolCtx for browser IPC calls (agent_id opsional, default mimi).
fn browser_ctx(app: &App, params: &Value) -> crate::tools::ToolCtx {
    crate::tools::ToolCtx {
        data_dir: app.data_dir.clone(),
        repo_dir: app.repo.clone(),
        run_id: p_str(params, "run_id"),
        agent_id: p_str(params, "agent_id").unwrap_or_else(|| "mimi".to_string()),
    }
}

/// approvals.decide — human gate Patria.
/// approve → run selesai + draft deliverable dibuat.
/// reject → revisi (maks 3x, counter di runs.context); jika sudah 3x → run butuh perhatian.
async fn decide_approval(
    app: &App,
    approval_id: &str,
    decision: &str,
    note: &str,
) -> Result<Value> {
    let db = &app.db;
    let (run_id, _stage, status): (Option<String>, String, String) = {
        let guard = db.lock().await;
        guard
            .query_row(
                "SELECT run_id, stage, status FROM approvals WHERE id = ?1",
                [approval_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .with_context(|| format!("approval '{approval_id}' tidak ditemukan"))?
    };
    if status != "pending" {
        anyhow::bail!("approval sudah diputuskan ({status})");
    }
    let run_id = run_id.context("approval tidak terikat ke run")?;

    if decision == "approve" {
        approve_run(app, approval_id, &run_id, note).await?;
        return Ok(json!({"approval_id": approval_id, "decision": "approve"}));
    }

    // reject → hitung revisi
    let revs = runner::revision_count(db, &run_id).await;
    if revs >= 3 {
        // Maks 3 revisi tercapai → butuh perhatian Patria, run ditandai gagal.
        {
            let guard = db.lock().await;
            guard.execute(
                "UPDATE approvals SET status='rejected', decided_at=datetime('now'), note=?1 WHERE id=?2",
                params![note, approval_id],
            )?;
            guard.execute(
                "UPDATE runs SET status='failed', updated_at=datetime('now') WHERE id=?1",
                [&run_id],
            )?;
        }
        app.tx
            .send(Event::RunUpdated {
                run_id: run_id.clone(),
                status: "failed".to_string(),
                current_stage: String::new(),
            })
            .ok();
        crate::notify::notify(
            "Minime — Butuh perhatian",
            &format!("3x revisi ditolak untuk run {run_id}. Keputusan diserahkan ke Patria."),
        );
        return Ok(json!({"approval_id": approval_id, "decision": "reject", "final": true}));
    }

    // Cari stage produsen: stage sebelum approval di pipeline.
    let pipeline_name: String = {
        let guard = db.lock().await;
        guard.query_row("SELECT pipeline FROM runs WHERE id = ?1", [&run_id], |r| {
            r.get(0)
        })?
    };
    let p = pipeline::load_pipeline(&app.repo, &pipeline_name)?;
    let idx = p
        .stages
        .iter()
        .position(|s| {
            crate::pipeline::Transition::parse(&s.on_done)
                .map(|t| matches!(t, crate::pipeline::Transition::Approval))
                .unwrap_or(false)
        })
        .context("pipeline tidak punya stage approval")?;
    if idx == 0 {
        anyhow::bail!("tidak ada stage produsen sebelum approval");
    }
    let producer_stage = &p.stages[idx - 1];

    {
        let guard = db.lock().await;
        guard.execute(
            "UPDATE approvals SET status='rejected', decided_at=datetime('now'), note=?1 WHERE id=?2",
            params![note, approval_id],
        )?;
    }
    runner::resume_pipeline(
        db.clone(),
        app.tx.clone(),
        app.repo.clone(),
        &run_id,
        &producer_stage.id,
        note,
    )
    .await?;
    Ok(json!({
        "approval_id": approval_id,
        "decision": "reject",
        "revision": revs + 1,
        "resumed_from": producer_stage.id,
    }))
}

/// approve: tandai approval + run selesai, tulis draft deliverable.
async fn approve_run(app: &App, approval_id: &str, run_id: &str, note: &str) -> Result<()> {
    let db = &app.db;
    let (payload_summary, qc_score): (String, Option<i64>) = {
        let guard = db.lock().await;
        guard.query_row(
            "SELECT payload_summary, qc_score FROM approvals WHERE id = ?1",
            [approval_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?
    };

    // Tulis draft deliverable ke data_dir/deliverables/<run_id>.md
    let dir = app.data_dir.join("deliverables");
    std::fs::create_dir_all(&dir)?;
    let file_path = dir.join(format!("{run_id}.md"));
    let draft = format!(
        "# Draft Deliverable — run {run_id}\n\nDisetujui Patria pada {now}.\n\nSkor QC: {score}\n\nCatatan: {note}\n\n---\n\n{payload}\n",
        run_id = run_id,
        now = chrono::Local::now().format("%Y-%m-%d %H:%M"),
        score = qc_score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string()),
        note = if note.is_empty() { "-" } else { note },
        payload = payload_summary,
    );
    std::fs::write(&file_path, &draft)?;

    let deliverable_id = uuid::Uuid::new_v4().to_string();
    {
        let guard = db.lock().await;
        guard.execute(
            "UPDATE approvals SET status='approved', decided_at=datetime('now'), note=?1 WHERE id=?2",
            params![note, approval_id],
        )?;
        guard.execute(
            "INSERT INTO deliverables (id, run_id, kind, file_path) VALUES (?1,?2,'draft',?3)",
            params![
                deliverable_id,
                run_id,
                file_path.to_string_lossy().to_string()
            ],
        )?;
        guard.execute(
            "UPDATE runs SET status='done', updated_at=datetime('now') WHERE id=?1",
            [run_id],
        )?;
    }
    app.tx
        .send(Event::RunUpdated {
            run_id: run_id.to_string(),
            status: "done".to_string(),
            current_stage: String::new(),
        })
        .ok();

    // Fase 5: upload best-effort ke Drive folder MIMI. Kegagalan TIDAK
    // menggagalkan approval — draft lokal tetap tersimpan.
    let filename = format!("{run_id}.md");
    match crate::drive_upload::upload_text("MIMI", &filename, &draft).await {
        Ok(file_id) => eprintln!("[approve] deliverable ter-upload ke Drive (id={file_id})"),
        Err(e) => eprintln!("[approve] upload Drive dilewati: {e:#}"),
    }
    Ok(())
}
