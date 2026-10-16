//! minime-daemon — background service: owns SQLite, executes tasks FIFO,
//! runs pipelines, serves IPC. Survives UI quits (launchd KeepAlive on macOS).

mod agent_loader;
mod db;
mod events;
mod executor;
mod ipc;
mod models;
mod pipeline;
mod provider;
mod runner;

use anyhow::{Context, Result};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    let data_dir = db::data_dir()?;
    let repo_dir = db::repo_dir();
    eprintln!(
        "[minime-daemon] data_dir={} repo_dir={}",
        data_dir.display(),
        repo_dir.display()
    );

    // 1. Database + migrations.
    let db = db::open(&data_dir.join("minime.db")).await?;
    let applied = db::run_migrations(&db, &repo_dir).await?;
    if !applied.is_empty() {
        eprintln!("[minime-daemon] migrations applied: {applied:?}");
    }

    // 2. Honest crash recovery.
    let (tasks, runs) = db::crash_recovery(&db).await?;
    if tasks + runs > 0 {
        eprintln!("[minime-daemon] crash recovery: {tasks} tasks, {runs} runs marked failed");
    }

    // 3. Load agent + skill definitions (source of truth: agents/<id>/, skills/*/).
    let agents = agent_loader::load_agents(&db, &repo_dir, &data_dir).await?;
    eprintln!("[minime-daemon] agents loaded: {}", agents.join(", "));
    let skills = agent_loader::load_skills(&db, &repo_dir).await?;
    eprintln!("[minime-daemon] skills loaded: {}", skills.len());

    // 4. Validate bundled pipelines at startup — fail fast on bad YAML.
    let agent_ids = agent_loader::agent_ids(&db).await?;
    let skill_names = agent_loader::skill_names(&db).await?;
    for (name, _) in pipeline::list_pipelines(&repo_dir) {
        let p = pipeline::load_pipeline(&repo_dir, &name)?;
        pipeline::validate_pipeline(&p, &repo_dir, &agent_ids, &skill_names)
            .with_context(|| format!("invalid bundled pipeline '{name}'"))?;
    }
    eprintln!("[minime-daemon] pipelines validated");

    // 5. Executor + IPC.
    let (tx, _rx) = broadcast::channel::<events::Event>(128);
    let _exec_handle = executor::Executor::new(db.clone(), tx.clone()).start();

    let sock = data_dir.join("minime.sock");
    let app = ipc::App {
        db,
        tx,
        repo: repo_dir,
        start: std::time::Instant::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    eprintln!("[minime-daemon] listening on {}", sock.display());

    tokio::select! {
        r = ipc::serve(app, &sock) => {
            if let Err(e) = r {
                eprintln!("[minime-daemon] ipc error: {e:#}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("[minime-daemon] shutting down (socket removed)");
            std::fs::remove_file(&sock).ok();
        }
    }
    Ok(())
}
