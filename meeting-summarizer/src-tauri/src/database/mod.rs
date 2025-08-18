use crate::errors::AppResult;
use crate::models::{Recording, Transcription, TranscriptionStatus, RecordingQuery, RecordingStats, CategoryStats, SortBy, SortOrder};
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
                title TEXT,
                description TEXT,
                category TEXT,
                tags TEXT, -- JSON array as string
                duration INTEGER,
                file_size INTEGER,
                sample_rate INTEGER,
                channels INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                recording_id TEXT NOT NULL,
                text TEXT NOT NULL,
                language TEXT NOT NULL,
                confidence REAL,
                processing_time_ms INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (recording_id) REFERENCES recordings (id) ON DELETE CASCADE
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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_category 
             ON recordings(category)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_recording_id 
             ON transcriptions(recording_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_status 
             ON transcriptions(status)",
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
                title TEXT,
                description TEXT,
                category TEXT,
                tags TEXT, -- JSON array as string
                duration INTEGER,
                file_size INTEGER,
                sample_rate INTEGER,
                channels INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                recording_id TEXT NOT NULL,
                text TEXT NOT NULL,
                language TEXT NOT NULL,
                confidence REAL,
                processing_time_ms INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (recording_id) REFERENCES recordings (id) ON DELETE CASCADE
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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_category 
             ON recordings(category)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_recording_id 
             ON transcriptions(recording_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_status 
             ON transcriptions(status)",
            [],
        )?;

        let db = Self { 
            conn: Arc::new(Mutex::new(conn)) 
        };
        
        Ok(db)
    }


    pub async fn create_recording(&self, recording: &Recording) -> AppResult<()> {
        let conn = self.conn.lock().await;
        let tags_json = serde_json::to_string(&recording.tags).unwrap_or_else(|_| "[]".to_string());
        
        conn.execute(
            "INSERT INTO recordings (id, filename, file_path, title, description, category, tags, duration, file_size, sample_rate, channels, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                recording.id,
                recording.filename,
                recording.file_path,
                recording.title,
                recording.description,
                recording.category,
                tags_json,
                recording.duration,
                recording.file_size,
                recording.sample_rate,
                recording.channels,
                recording.created_at.to_rfc3339(),
                recording.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_recording(&self, id: &str) -> AppResult<Option<Recording>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, filename, file_path, title, description, category, tags, duration, file_size, sample_rate, channels, created_at, updated_at 
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
            "SELECT id, filename, file_path, title, description, category, tags, duration, file_size, sample_rate, channels, created_at, updated_at 
             FROM recordings ORDER BY created_at DESC"
        )?;

        let recordings = stmt.query_map([], Self::row_to_recording)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(recordings)
    }

    pub async fn update_recording(&self, recording: &Recording) -> AppResult<()> {
        let updated_at = Utc::now().to_rfc3339();
        let tags_json = serde_json::to_string(&recording.tags).unwrap_or_else(|_| "[]".to_string());
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE recordings 
             SET filename = ?2, file_path = ?3, title = ?4, description = ?5, category = ?6, tags = ?7, 
                 duration = ?8, file_size = ?9, sample_rate = ?10, channels = ?11, updated_at = ?12
             WHERE id = ?1",
            params![
                recording.id,
                recording.filename,
                recording.file_path,
                recording.title,
                recording.description,
                recording.category,
                tags_json,
                recording.duration,
                recording.file_size,
                recording.sample_rate,
                recording.channels,
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

        let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_else(|_| Vec::new());

        Ok(Recording {
            id: row.get("id")?,
            filename: row.get("filename")?,
            file_path: row.get("file_path")?,
            title: row.get("title")?,
            description: row.get("description")?,
            category: row.get("category")?,
            tags,
            duration: row.get("duration")?,
            file_size: row.get("file_size")?,
            sample_rate: row.get("sample_rate")?,
            channels: row.get("channels")?,
            created_at,
            updated_at,
        })
    }

    // Transcription CRUD operations
    pub async fn create_transcription(&self, transcription: &Transcription) -> AppResult<()> {
        let conn = self.conn.lock().await;
        let status_str = match &transcription.status {
            TranscriptionStatus::Pending => "pending",
            TranscriptionStatus::Processing => "processing", 
            TranscriptionStatus::Completed => "completed",
            TranscriptionStatus::Failed(err) => &format!("failed:{}", err),
        };

        conn.execute(
            "INSERT INTO transcriptions (id, recording_id, text, language, confidence, processing_time_ms, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                transcription.id,
                transcription.recording_id,
                transcription.text,
                transcription.language,
                transcription.confidence,
                transcription.processing_time_ms,
                status_str,
                transcription.created_at.to_rfc3339(),
                transcription.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_transcription(&self, id: &str) -> AppResult<Option<Transcription>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, recording_id, text, language, confidence, processing_time_ms, status, created_at, updated_at 
             FROM transcriptions WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], Self::row_to_transcription)?;
        
        match rows.next() {
            Some(transcription) => Ok(Some(transcription?)),
            None => Ok(None),
        }
    }

    pub async fn get_transcriptions_by_recording(&self, recording_id: &str) -> AppResult<Vec<Transcription>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, recording_id, text, language, confidence, processing_time_ms, status, created_at, updated_at 
             FROM transcriptions WHERE recording_id = ?1 ORDER BY created_at DESC"
        )?;

        let transcriptions = stmt.query_map(params![recording_id], Self::row_to_transcription)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transcriptions)
    }

    pub async fn update_transcription(&self, transcription: &Transcription) -> AppResult<()> {
        let updated_at = Utc::now().to_rfc3339();
        let status_str = match &transcription.status {
            TranscriptionStatus::Pending => "pending",
            TranscriptionStatus::Processing => "processing", 
            TranscriptionStatus::Completed => "completed",
            TranscriptionStatus::Failed(err) => &format!("failed:{}", err),
        };
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE transcriptions 
             SET text = ?2, language = ?3, confidence = ?4, processing_time_ms = ?5, status = ?6, updated_at = ?7
             WHERE id = ?1",
            params![
                transcription.id,
                transcription.text,
                transcription.language,
                transcription.confidence,
                transcription.processing_time_ms,
                status_str,
                updated_at,
            ],
        )?;
        Ok(())
    }

    pub async fn delete_transcription(&self, id: &str) -> AppResult<bool> {
        let conn = self.conn.lock().await;
        let rows_affected = conn.execute(
            "DELETE FROM transcriptions WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    fn row_to_transcription(row: &Row) -> rusqlite::Result<Transcription> {
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
            TranscriptionStatus::Failed(status_str[7..].to_string())
        } else {
            match status_str.as_str() {
                "pending" => TranscriptionStatus::Pending,
                "processing" => TranscriptionStatus::Processing,
                "completed" => TranscriptionStatus::Completed,
                _ => TranscriptionStatus::Failed("Unknown status".to_string()),
            }
        };

        Ok(Transcription {
            id: row.get("id")?,
            recording_id: row.get("recording_id")?,
            text: row.get("text")?,
            language: row.get("language")?,
            confidence: row.get("confidence")?,
            processing_time_ms: row.get("processing_time_ms")?,
            status,
            created_at,
            updated_at,
        })
    }

    // Search and filtering functions
    pub async fn search_recordings(&self, query: &RecordingQuery) -> AppResult<Vec<Recording>> {
        let conn = self.conn.lock().await;
        
        let mut sql = String::from(
            "SELECT id, filename, file_path, title, description, category, tags, duration, file_size, sample_rate, channels, created_at, updated_at 
             FROM recordings WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_index = 1;

        // Search text filter (filename, title, description)
        if let Some(search_text) = &query.search_text {
            sql.push_str(&format!(" AND (filename LIKE ?{} OR title LIKE ?{} OR description LIKE ?{})", 
                                param_index, param_index + 1, param_index + 2));
            let search_pattern = format!("%{}%", search_text);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
            param_index += 3;
        }

        // Category filter
        if let Some(category) = &query.category {
            sql.push_str(&format!(" AND category = ?{}", param_index));
            params.push(Box::new(category.clone()));
            param_index += 1;
        }

        // Tags filter
        for tag in &query.tags {
            sql.push_str(&format!(" AND tags LIKE ?{}", param_index));
            params.push(Box::new(format!("%\"{}\"", tag)));
            param_index += 1;
        }

        // Date range filter
        if let Some(date_from) = &query.date_from {
            sql.push_str(&format!(" AND created_at >= ?{}", param_index));
            params.push(Box::new(date_from.to_rfc3339()));
            param_index += 1;
        }

        if let Some(date_to) = &query.date_to {
            sql.push_str(&format!(" AND created_at <= ?{}", param_index));
            params.push(Box::new(date_to.to_rfc3339()));
            param_index += 1;
        }

        // Duration range filter
        if let Some(min_duration) = query.min_duration {
            sql.push_str(&format!(" AND duration >= ?{}", param_index));
            params.push(Box::new(min_duration));
            param_index += 1;
        }

        if let Some(max_duration) = query.max_duration {
            sql.push_str(&format!(" AND duration <= ?{}", param_index));
            params.push(Box::new(max_duration));
            param_index += 1;
        }

        // Sort by
        let sort_column = match query.sort_by {
            SortBy::CreatedAt => "created_at",
            SortBy::UpdatedAt => "updated_at", 
            SortBy::Filename => "filename",
            SortBy::Duration => "duration",
            SortBy::FileSize => "file_size",
        };

        let sort_direction = match query.sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        sql.push_str(&format!(" ORDER BY {} {}", sort_column, sort_direction));

        // Limit and offset
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = query.offset {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let recordings = stmt.query_map(&param_refs[..], Self::row_to_recording)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(recordings)
    }

    pub async fn get_recording_stats(&self) -> AppResult<RecordingStats> {
        let conn = self.conn.lock().await;
        
        // Total counts and sizes
        let (total_count, total_duration, total_size): (i64, i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(duration), 0), COALESCE(SUM(file_size), 0) FROM recordings",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        )?;

        // Recent count (last 7 days)
        let seven_days_ago = Utc::now() - chrono::Duration::days(7);
        let recent_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recordings WHERE created_at >= ?1",
            params![seven_days_ago.to_rfc3339()],
            |row| row.get(0)
        )?;

        // Category stats
        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*), COALESCE(SUM(duration), 0) 
             FROM recordings 
             WHERE category IS NOT NULL 
             GROUP BY category 
             ORDER BY COUNT(*) DESC"
        )?;

        let categories = stmt.query_map([], |row| {
            Ok(CategoryStats {
                name: row.get(0)?,
                count: row.get(1)?,
                total_duration: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(RecordingStats {
            total_count,
            total_duration,
            total_size,
            categories,
            recent_count,
        })
    }

    pub async fn get_all_categories(&self) -> AppResult<Vec<String>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT category FROM recordings WHERE category IS NOT NULL ORDER BY category"
        )?;

        let categories = stmt.query_map([], |row| {
            let category: String = row.get(0)?;
            Ok(category)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    pub async fn get_all_tags(&self) -> AppResult<Vec<String>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT tags FROM recordings WHERE tags IS NOT NULL AND tags != '[]'")?;

        let mut all_tags = std::collections::HashSet::new();
        let rows = stmt.query_map([], |row| {
            let tags_json: String = row.get(0)?;
            Ok(tags_json)
        })?;

        for row in rows {
            let tags_json = row?;
            if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tags_json) {
                for tag in tags {
                    all_tags.insert(tag);
                }
            }
        }

        let mut tags: Vec<String> = all_tags.into_iter().collect();
        tags.sort();
        Ok(tags)
    }
}