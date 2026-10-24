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
    pub cost_usd: f64,
}

/// Tarif per 1 juta token (input, output) dalam USD.
/// Model image/transcribe dihitung per pemanggilan (lihat cost_for_call).
pub fn model_pricing(model_id: &str) -> Option<(f64, f64)> {
    match model_id {
        // Meta (api.meta.ai)
        id if id.contains("muse-spark-1.3") => Some((1.25, 4.25)),
        id if id.contains("muse-wonder-1.0") => Some((4.0, 20.0)),
        // Muse
        id if id.contains("claude-opus") => Some((15.0, 75.0)),
        id if id.contains("claude-sonnet") => Some((3.0, 15.0)),
        id if id.contains("claude-haiku") => Some((0.8, 4.0)),
        // Gemini
        id if id.contains("gemini-3") => Some((2.0, 8.0)),
        id if id.contains("gemini-2.5-pro") => Some((1.25, 10.0)),
        id if id.contains("gemini-2.5-flash") => Some((0.3, 2.5)),
        id if id.contains("gemini-2.0-flash") => Some((0.1, 0.4)),
        // DeepSeek
        id if id.contains("deepseek-chat") => Some((0.27, 1.1)),
        id if id.contains("deepseek-reasoner") => Some((0.55, 2.19)),
        _ => None,
    }
}

/// Biaya per pemanggilan untuk model non-token (image, transcribe).
pub fn model_call_pricing(model_id: &str) -> Option<f64> {
    match model_id {
        id if id.contains("muse-image-1.0") => Some(0.01),
        id if id.contains("muse-voice-transcribe-1.0") => Some(0.003), // $0,18/jam ≈ $0,003/menit
        id if id.contains("sam-3.1") => Some(0.005),
        _ => None,
    }
}

/// Hitung estimasi biaya USD dari UsageRow.
pub fn usage_cost(row: &UsageRow) -> f64 {
    if let Some((inp, out)) = model_pricing(&row.model_id) {
        (row.input_tokens as f64 / 1_000_000.0) * inp
            + (row.output_tokens as f64 / 1_000_000.0) * out
    } else if let Some(per_call) = model_call_pricing(&row.model_id) {
        row.calls as f64 * per_call
    } else {
        0.0
    }
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

    /// Buka database in-memory untuk testing.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        Self::open_memory()
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
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                agent_id TEXT NOT NULL,
                prompt_hash TEXT NOT NULL,
                prompt_len INTEGER NOT NULL,
                complexity_score INTEGER NOT NULL,
                model_id TEXT NOT NULL,
                qc_score INTEGER
            );",
            [],
        )
        .map_err(|e| format!("migrasi db gagal: {e}"))?;

        // Tabel #7: tracking parse failures untuk dead letter
        conn.execute(
            "CREATE TABLE IF NOT EXISTS parse_attempts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                success INTEGER NOT NULL
            );",
            [],
        )
        .map_err(|e| format!("migrasi parse_attempts gagal: {e}"))?;

        // Migrasi #5: tambah duration_ms ke tabel lama (ignore jika sudah ada)
        {
            let conn2 = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
            let _ = conn2.execute("ALTER TABLE model_decisions ADD COLUMN duration_ms INTEGER", []);
        }
        Ok(())
    }

    /// Ambil riwayat keputusan model (untuk tab Orkestrasi di Diagnostics).
    /// Return: vec of (created_at, agent_id, prompt_len, complexity_score, model_id, qc_score)
    pub fn get_model_decisions(
        &self,
        limit: i64,
    ) -> Result<Vec<(String, String, i64, i64, String, Option<i64>)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock gagal: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT created_at, agent_id, prompt_len, complexity_score, model_id, qc_score                  FROM model_decisions ORDER BY id DESC LIMIT ?1",
            )
            .map_err(|e| format!("prepare gagal: {e}"))?;
        let rows = stmt
            .query_map([limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            })
            .map_err(|e| format!("query gagal: {e}"))?;

        let mut out = vec![];
        for r in rows {
            out.push(r.map_err(|e| format!("row gagal: {e}"))?);
        }
        Ok(out)
    }

    /// Catat hasil parse attempt (saran #7).
    pub fn log_parse_attempt(&self, success: bool) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        conn.execute(
            "INSERT INTO parse_attempts (success) VALUES (?1)",
            rusqlite::params![if success { 1 } else { 0 }],
        )
        .map_err(|e| format!("log parse gagal: {e}"))?;
        Ok(())
    }

    /// Hitung failure rate parse dalam 24 jam terakhir (saran #7).
    /// Return: (total, failed, rate_percent)
    pub fn get_parse_failure_rate(&self) -> Result<(i64, i64, f64), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        let (total, failed): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END), 0)
                 FROM parse_attempts
                 WHERE created_at > datetime('now', '-24 hours')",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| format!("query gagal: {e}"))?;
        let rate = if total > 0 {
            (failed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        Ok((total, failed, rate))
    }

    /// Catat keputusan pemilihan model (untuk audit & learning).
    pub fn log_model_decision(
        &self,
        agent_id: &str,
        prompt: &str,
        complexity_score: i32,
        model_id: &str,
        duration_ms: Option<i64>,
    ) -> Result<(), String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        prompt.hash(&mut h);
        let prompt_hash = format!("{:x}", h.finish());

        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        conn.execute(
            "INSERT INTO model_decisions (agent_id, prompt_hash, prompt_len, complexity_score, model_id, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                agent_id,
                prompt_hash,
                prompt.chars().count() as i64,
                complexity_score as i64,
                model_id,
                duration_ms,
            ],
        )
        .map_err(|e| format!("log model decision gagal: {e}"))?;
        Ok(())
    }

    /// Update skor QC untuk keputusan terakhir agen ini (untuk korelasi).
    #[allow(dead_code)]
    pub fn update_decision_qc(&self, agent_id: &str, model_id: &str, qc_score: i32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("db lock: {e}"))?;
        conn.execute(
            "UPDATE model_decisions SET qc_score = ?1
             WHERE agent_id = ?2 AND model_id = ?3 AND qc_score IS NULL
             ORDER BY id DESC LIMIT 1",
            rusqlite::params![qc_score as i64, agent_id, model_id],
        )
        .map_err(|e| format!("update qc gagal: {e}"))?;
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
                let r = UsageRow {
                    agent_id: row.get(0)?,
                    provider_id: row.get(1)?,
                    model_id: row.get(2)?,
                    calls: row.get(3)?,
                    input_tokens: row.get(4)?,
                    output_tokens: row.get(5)?,
                    cost_usd: 0.0,
                };
                let cost = usage_cost(&r);
                Ok(UsageRow { cost_usd: cost, ..r })
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
