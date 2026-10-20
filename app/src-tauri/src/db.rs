//! SQLite lokal (Tahap 2): riwayat chat + metering usage.
//!
//! Lokasi: direktori data aplikasi Tauri
//! (`~/Library/Application Support/com.minime.app/` di macOS).
//! Semua data milik user di perangkatnya sendiri — tidak dikirim ke mana pun.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: i64,
    pub agent_id: String,
    pub role: String,
    pub text: String,
    pub served_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRow {
    pub agent_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Buka (atau buat) database di direktori data aplikasi.
    /// Gagal → fallback ke in-memory agar aplikasi tetap jalan
    /// (riwayat tidak persisten antar restart dalam kasus ini).
    pub fn open() -> Result<Self, String> {
        if let Some(db) = Self::open_file().transpose() {
            return db;
        }
        Self::open_memory()
    }

    fn open_file() -> Result<Option<Self>, String> {
        let dir = match app_data_dir() {
            Some(d) => d,
            None => return Ok(None),
        };
        if std::fs::create_dir_all(&dir).is_err() {
            return Ok(None);
        }
        let path = dir.join("minime.db");
        let conn =
            Connection::open(&path).map_err(|e| format!("gagal membuka database: {e}"))?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(Some(db))
    }

    fn open_memory() -> Result<Self, String> {
        let conn =
            Connection::open_in_memory().map_err(|e| format!("gagal membuka db memory: {e}"))?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                role TEXT NOT NULL,
                text TEXT NOT NULL,
                served_by TEXT,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_chat_agent ON chat_messages(agent_id, id);
            CREATE TABLE IF NOT EXISTS usage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_usage_agent ON usage_events(agent_id, created_at);
            CREATE TABLE IF NOT EXISTS agent_skill_overrides (
                agent_id TEXT PRIMARY KEY,
                skills TEXT NOT NULL DEFAULT '[]'
            );",
        )
        .map_err(|e| format!("migrasi db gagal: {e}"))?;
        Ok(())
    }

    pub fn save_message(
        &self,
        agent_id: &str,
        role: &str,
        text: &str,
        served_by: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        conn.execute(
            "INSERT INTO chat_messages (agent_id, role, text, served_by)
             VALUES (?1, ?2, ?3, ?4)",
            params![agent_id, role, text, served_by],
        )
        .map_err(|e| format!("simpan pesan gagal: {e}"))?;
        Ok(())
    }

    pub fn history(&self, agent_id: &str, limit: i64) -> Result<Vec<StoredMessage>, String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_id, role, text, served_by, created_at
                 FROM chat_messages WHERE agent_id = ?1
                 ORDER BY id DESC LIMIT ?2",
            )
            .map_err(|e| format!("riwayat gagal: {e}"))?;
        let rows = stmt
            .query_map(params![agent_id, limit], |row| {
                Ok(StoredMessage {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    role: row.get(2)?,
                    text: row.get(3)?,
                    served_by: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("riwayat gagal: {e}"))?;
        let mut out: Vec<StoredMessage> = rows
            .collect::<Result<_, _>>()
            .map_err(|e| format!("riwayat gagal: {e}"))?;
        out.reverse(); // kronologis menaik untuk UI
        Ok(out)
    }

    pub fn record_usage(
        &self,
        agent_id: &str,
        provider_id: &str,
        model_id: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        conn.execute(
            "INSERT INTO usage_events
             (agent_id, provider_id, model_id, input_tokens, output_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![agent_id, provider_id, model_id, input_tokens, output_tokens],
        )
        .map_err(|e| format!("catat usage gagal: {e}"))?;
        Ok(())
    }

    /// Ringkasan usage per agen+provider+model (untuk tab Cost).
    pub fn usage_summary(&self) -> Result<Vec<UsageRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT agent_id, provider_id, model_id,
                        COUNT(*), COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0)
                 FROM usage_events
                 GROUP BY agent_id, provider_id, model_id
                 ORDER BY agent_id, provider_id",
            )
            .map_err(|e| format!("ringkasan usage gagal: {e}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(UsageRow {
                    agent_id: row.get(0)?,
                    provider_id: row.get(1)?,
                    model_id: row.get(2)?,
                    calls: row.get(3)?,
                    input_tokens: row.get(4)?,
                    output_tokens: row.get(5)?,
                })
            })
            .map_err(|e| format!("ringkasan usage gagal: {e}"))?;
        rows
            .collect::<Result<_, _>>()
            .map_err(|e| format!("ringkasan usage gagal: {e}"))
    }

    /// User-assigned skill set for an agent (Marketplace > Skills).
    /// `None` = no override stored; the caller falls back to defaults.
    pub fn skill_override(&self, agent_id: &str) -> Result<Option<Vec<String>>, String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        let row: Option<String> = conn
            .query_row(
                "SELECT skills FROM agent_skill_overrides WHERE agent_id = ?1",
                params![agent_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| format!("baca override skill gagal: {e}"))?;
        match row {
            Some(json) => Ok(Some(
                serde_json::from_str(&json).unwrap_or_default(),
            )),
            None => Ok(None),
        }
    }

    /// Replace an agent's skill set (stored as a JSON array of skill ids).
    pub fn set_skill_override(
        &self,
        agent_id: &str,
        skills: &[String],
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        let json = serde_json::to_string(skills).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agent_skill_overrides (agent_id, skills) VALUES (?1, ?2)
             ON CONFLICT(agent_id) DO UPDATE SET skills = excluded.skills",
            params![agent_id, json],
        )
        .map_err(|e| format!("simpan override skill gagal: {e}"))?;
        Ok(())
    }

    /// Agent ids that have a stored skill override.
    pub fn skill_override_agent_ids(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        let mut stmt = conn
            .prepare("SELECT agent_id FROM agent_skill_overrides")
            .map_err(|e| format!("baca override skill gagal: {e}"))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("baca override skill gagal: {e}"))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| format!("baca override skill gagal: {e}"))?);
        }
        Ok(out)
    }
}

/// Direktori data aplikasi per OS (tanpa crate tambahan).
pub(crate) fn app_data_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    #[cfg(target_os = "macos")]
    {
        Some(home.join("Library/Application Support/com.minime.app"))
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").ok().map(PathBuf::from)?;
        Some(appdata.join("com.minime.app"))
    }
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        Some(base.join("minime"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = home;
        None
    }
}
