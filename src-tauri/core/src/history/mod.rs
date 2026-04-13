use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

pub struct HistoryStore {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub profile_id: Uuid,
    pub profile_name: String,
    pub backend_id: String,
    pub duration_ms: u64,
    pub text: String,
}

impl HistoryStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let conn = Connection::open(path)
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                profile_id TEXT NOT NULL,
                profile_name TEXT NOT NULL,
                backend_id TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                text TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_history_ts ON history(timestamp DESC);
        "#).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn insert(&self, e: &HistoryEntry) -> Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO history (timestamp, profile_id, profile_name, backend_id, duration_ms, text) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![e.timestamp.to_rfc3339(), e.profile_id.to_string(), e.profile_name, e.backend_id, e.duration_ms as i64, e.text],
        ).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(c.last_insert_rowid())
    }

    pub fn list(&self, limit: u32) -> Result<Vec<HistoryEntry>> {
        let c = self.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, timestamp, profile_id, profile_name, backend_id, duration_ms, text \
                                  FROM history ORDER BY id DESC LIMIT ?1")
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let ts: String = row.get(1)?;
            let pid: String = row.get(2)?;
            Ok(HistoryEntry {
                id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&ts).unwrap().with_timezone(&Utc),
                profile_id: Uuid::parse_str(&pid).unwrap_or(Uuid::nil()),
                profile_name: row.get(3)?,
                backend_id: row.get(4)?,
                duration_ms: row.get::<_, i64>(5)? as u64,
                text: row.get(6)?,
            })
        }).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.lock().unwrap().execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    pub fn trim(&self, keep: u32) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT ?1)",
            params![keep as i64],
        ).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample(text: &str) -> HistoryEntry {
        HistoryEntry {
            id: 0, timestamp: Utc::now(), profile_id: Uuid::new_v4(),
            profile_name: "P".into(), backend_id: "remote-whisper".into(),
            duration_ms: 500, text: text.into(),
        }
    }

    #[test]
    fn insert_and_list() {
        let dir = tempdir().unwrap();
        let store = HistoryStore::open(&dir.path().join("h.db")).unwrap();
        store.insert(&sample("eins")).unwrap();
        store.insert(&sample("zwei")).unwrap();
        let list = store.list(10).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].text, "zwei");
    }

    #[test]
    fn trim_keeps_n_newest() {
        let dir = tempdir().unwrap();
        let store = HistoryStore::open(&dir.path().join("h.db")).unwrap();
        for i in 0..5 { store.insert(&sample(&format!("t{i}"))).unwrap(); }
        store.trim(2).unwrap();
        let list = store.list(10).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].text, "t4");
    }
}
