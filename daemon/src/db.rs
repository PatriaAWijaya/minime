//! SQLite access: single-writer connection, WAL mode, versioned migrations.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared single-writer handle. Only the daemon opens the DB file.
pub type Db = Arc<Mutex<Connection>>;

/// Resolve the daemon data directory.
///
/// Precedence: $MINIME_DATA_DIR → macOS Application Support → XDG/Linux default.
pub fn data_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("MINIME_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").context("HOME is not set")?;
        Ok(PathBuf::from(home).join("Library/Application Support/com.minime.app"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{home}/.local/share")
        });
        Ok(PathBuf::from(base).join("minime"))
    }
}

/// Resolve the repo directory (agents/, skills/, pipelines/, daemon/migrations/).
/// $MINIME_REPO_DIR wins; otherwise the current working directory.
pub fn repo_dir() -> PathBuf {
    std::env::var("MINIME_REPO_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub async fn open(db_path: &Path) -> Result<Db> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let conn =
        Connection::open(db_path).with_context(|| format!("open db {}", db_path.display()))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// Apply pending SQL migrations from daemon/migrations/*.sql (ordered by name).
pub async fn run_migrations(db: &Db, repo: &Path) -> Result<Vec<i64>> {
    let mig_dir = repo.join("daemon/migrations");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&mig_dir)
        .with_context(|| format!("read migrations dir {}", mig_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "sql").unwrap_or(false))
        .collect();
    files.sort();

    let guard = db.lock().await;
    guard.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )?;
    let applied: Vec<i64> = guard
        .prepare("SELECT version FROM schema_migrations")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut done = Vec::new();
    for path in files {
        let stem = path
            .file_stem()
            .context("migration without stem")?
            .to_string_lossy()
            .to_string();
        let version: i64 = stem
            .split('_')
            .next()
            .context("bad migration name")?
            .parse()
            .context("migration version not numeric")?;
        if applied.contains(&version) {
            continue;
        }
        let sql =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        guard.execute_batch(&sql)?;
        guard.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [version],
        )?;
        done.push(version);
    }
    Ok(done)
}

/// Honest crash recovery: tasks/runs stuck in `running` become `failed`.
pub async fn crash_recovery(db: &Db) -> Result<(usize, usize)> {
    let guard = db.lock().await;
    let tasks = guard.execute(
        "UPDATE tasks SET status='failed', error='daemon restart', updated_at=datetime('now') WHERE status='running'",
        [],
    )?;
    let runs = guard.execute(
        "UPDATE runs SET status='failed', updated_at=datetime('now') WHERE status='running'",
        [],
    )?;
    Ok((tasks, runs))
}
