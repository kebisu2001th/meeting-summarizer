use crate::errors::AppResult;
use crate::models::{Recording, Summary, SummaryStatus};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> AppResult<Self> {
        let conn = Connection::open(db_path)?;
        
        // 同期的にテーブル初期化
        conn.execute(
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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_created_at 
             ON recordings(created_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_filename 
             ON recordings(filename)",
            [],
        )?;

        // Summaries table for LLM-generated summaries
        conn.execute(
            "CREATE TABLE IF NOT EXISTS summaries (
                id TEXT PRIMARY KEY,
                transcription_id TEXT NOT NULL,
                summary_text TEXT NOT NULL,
                key_points TEXT, -- JSON array as string
                action_items TEXT, -- JSON array as string
                model_used TEXT NOT NULL,
                processing_time_ms INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_summaries_transcription_id 
             ON summaries(transcription_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_summaries_status 
             ON summaries(status)",
            [],
        )?;

        let db = Self { 
            conn: Arc::new(Mutex::new(conn)) 
        };
        
        Ok(db)
    }

    pub fn in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        
        // 同期的にテーブル初期化
        conn.execute(
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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_created_at 
             ON recordings(created_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_filename 
             ON recordings(filename)",
            [],
        )?;

        // Summaries table for LLM-generated summaries
        conn.execute(
            "CREATE TABLE IF NOT EXISTS summaries (
                id TEXT PRIMARY KEY,
                transcription_id TEXT NOT NULL,
                summary_text TEXT NOT NULL,
                key_points TEXT, -- JSON array as string
                action_items TEXT, -- JSON array as string
                model_used TEXT NOT NULL,
                processing_time_ms INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_summaries_transcription_id 
             ON summaries(transcription_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_summaries_status 
             ON summaries(status)",
            [],
        )?;

        let db = Self { 
            conn: Arc::new(Mutex::new(conn)) 
        };
        
        Ok(db)
    }


    pub async fn create_recording(&self, recording: &Recording) -> AppResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
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

    pub async fn get_recording(&self, id: &str) -> AppResult<Option<Recording>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, filename, file_path, duration, file_size, created_at, updated_at 
             FROM recordings WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], Self::row_to_recording)?;
        
        match rows.next() {
            Some(recording) => Ok(Some(recording?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_recordings(&self) -> AppResult<Vec<Recording>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, filename, file_path, duration, file_size, created_at, updated_at 
             FROM recordings ORDER BY created_at DESC"
        )?;

        let recordings = stmt.query_map([], Self::row_to_recording)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(recordings)
    }

    pub async fn update_recording(&self, recording: &Recording) -> AppResult<()> {
        let updated_at = Utc::now().to_rfc3339();
        let conn = self.conn.lock().await;
        
        conn.execute(
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

    pub async fn delete_recording(&self, id: &str) -> AppResult<bool> {
        let conn = self.conn.lock().await;
        let rows_affected = conn.execute(
            "DELETE FROM recordings WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    pub async fn get_recordings_count(&self) -> AppResult<i64> {
        let conn = self.conn.lock().await;
        let count: i64 = conn.query_row(
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

    // Summary CRUD operations
    pub async fn create_summary(&self, summary: &Summary) -> AppResult<()> {
        let conn = self.conn.lock().await;
        let status_str = match &summary.status {
            SummaryStatus::Pending => "pending",
            SummaryStatus::Processing => "processing", 
            SummaryStatus::Completed => "completed",
            SummaryStatus::Failed(err) => &format!("failed:{}", err),
        };

        let key_points_json = serde_json::to_string(&summary.key_points).unwrap_or_else(|_| "[]".to_string());
        let action_items_json = serde_json::to_string(&summary.action_items).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO summaries (id, transcription_id, summary_text, key_points, action_items, model_used, processing_time_ms, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                summary.id,
                summary.transcription_id,
                summary.summary_text,
                key_points_json,
                action_items_json,
                summary.model_used,
                summary.processing_time_ms,
                status_str,
                summary.created_at.to_rfc3339(),
                summary.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_summary(&self, id: &str) -> AppResult<Option<Summary>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, transcription_id, summary_text, key_points, action_items, model_used, processing_time_ms, status, created_at, updated_at 
             FROM summaries WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], Self::row_to_summary)?;
        
        match rows.next() {
            Some(summary) => Ok(Some(summary?)),
            None => Ok(None),
        }
    }

    pub async fn get_summaries_by_transcription(&self, transcription_id: &str) -> AppResult<Vec<Summary>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, transcription_id, summary_text, key_points, action_items, model_used, processing_time_ms, status, created_at, updated_at 
             FROM summaries WHERE transcription_id = ?1 ORDER BY created_at DESC"
        )?;

        let summaries = stmt.query_map(params![transcription_id], Self::row_to_summary)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    pub async fn update_summary(&self, summary: &Summary) -> AppResult<()> {
        let updated_at = Utc::now().to_rfc3339();
        let status_str = match &summary.status {
            SummaryStatus::Pending => "pending",
            SummaryStatus::Processing => "processing", 
            SummaryStatus::Completed => "completed",
            SummaryStatus::Failed(err) => &format!("failed:{}", err),
        };
        
        let key_points_json = serde_json::to_string(&summary.key_points).unwrap_or_else(|_| "[]".to_string());
        let action_items_json = serde_json::to_string(&summary.action_items).unwrap_or_else(|_| "[]".to_string());
        
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE summaries 
             SET summary_text = ?2, key_points = ?3, action_items = ?4, model_used = ?5, processing_time_ms = ?6, status = ?7, updated_at = ?8
             WHERE id = ?1",
            params![
                summary.id,
                summary.summary_text,
                key_points_json,
                action_items_json,
                summary.model_used,
                summary.processing_time_ms,
                status_str,
                updated_at,
            ],
        )?;
        Ok(())
    }

    pub async fn delete_summary(&self, id: &str) -> AppResult<bool> {
        let conn = self.conn.lock().await;
        let rows_affected = conn.execute(
            "DELETE FROM summaries WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    fn row_to_summary(row: &Row) -> rusqlite::Result<Summary> {
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: String = row.get("updated_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let status_str: String = row.get("status")?;
        let status = if status_str.starts_with("failed:") {
            SummaryStatus::Failed(status_str[7..].to_string())
        } else {
            match status_str.as_str() {
                "pending" => SummaryStatus::Pending,
                "processing" => SummaryStatus::Processing,
                "completed" => SummaryStatus::Completed,
                _ => SummaryStatus::Failed("Unknown status".to_string()),
            }
        };

        let key_points_json: String = row.get("key_points").unwrap_or_else(|_| "[]".to_string());
        let key_points: Vec<String> = serde_json::from_str(&key_points_json).unwrap_or_else(|_| Vec::new());

        let action_items_json: String = row.get("action_items").unwrap_or_else(|_| "[]".to_string());
        let action_items: Vec<String> = serde_json::from_str(&action_items_json).unwrap_or_else(|_| Vec::new());

        Ok(Summary {
            id: row.get("id")?,
            transcription_id: row.get("transcription_id")?,
            summary_text: row.get("summary_text")?,
            key_points,
            action_items,
            model_used: row.get("model_used")?,
            processing_time_ms: row.get("processing_time_ms")?,
            status,
            created_at,
            updated_at,
        })
    }
}