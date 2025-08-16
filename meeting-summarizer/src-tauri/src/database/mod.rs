use crate::errors::{AppError, AppResult};
use crate::models::Recording;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> AppResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    pub fn in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> AppResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS recordings (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                duration INTEGER,
                file_size INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // インデックス作成
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_created_at 
             ON recordings(created_at DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_filename 
             ON recordings(filename)",
            [],
        )?;

        Ok(())
    }

    pub fn create_recording(&self, recording: &Recording) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO recordings (id, filename, file_path, duration, file_size, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                recording.id,
                recording.filename,
                recording.file_path,
                recording.duration,
                recording.file_size,
                recording.created_at.to_rfc3339(),
                recording.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_recording(&self, id: &str) -> AppResult<Option<Recording>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, file_path, duration, file_size, created_at, updated_at 
             FROM recordings WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], Self::row_to_recording)?;
        
        match rows.next() {
            Some(recording) => Ok(Some(recording?)),
            None => Ok(None),
        }
    }

    pub fn get_all_recordings(&self) -> AppResult<Vec<Recording>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, file_path, duration, file_size, created_at, updated_at 
             FROM recordings ORDER BY created_at DESC"
        )?;

        let recordings = stmt.query_map([], Self::row_to_recording)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(recordings)
    }

    pub fn update_recording(&self, recording: &Recording) -> AppResult<()> {
        let updated_at = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE recordings 
             SET filename = ?2, file_path = ?3, duration = ?4, file_size = ?5, updated_at = ?6
             WHERE id = ?1",
            params![
                recording.id,
                recording.filename,
                recording.file_path,
                recording.duration,
                recording.file_size,
                updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete_recording(&self, id: &str) -> AppResult<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM recordings WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn get_recordings_count(&self) -> AppResult<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM recordings",
            [],
            |row| row.get(0)
        )?;
        Ok(count)
    }

    fn row_to_recording(row: &Row) -> rusqlite::Result<Recording> {
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: String = row.get("updated_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        Ok(Recording {
            id: row.get("id")?,
            filename: row.get("filename")?,
            file_path: row.get("file_path")?,
            duration: row.get("duration")?,
            file_size: row.get("file_size")?,
            created_at,
            updated_at,
        })
    }
}