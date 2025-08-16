use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub filename: String,
    pub file_path: String,
    pub duration: Option<i64>, // seconds
    pub file_size: Option<i64>, // bytes
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Recording {
    pub fn new(filename: String, file_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            filename,
            file_path,
            duration: None,
            file_size: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_duration(mut self, duration: i64) -> Self {
        self.duration = Some(duration);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_file_size(mut self, file_size: i64) -> Self {
        self.file_size = Some(file_size);
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub temp_file_path: String,
    pub is_active: bool,
}

impl RecordingSession {
    pub fn new(temp_file_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            start_time: Utc::now(),
            temp_file_path,
            is_active: true,
        }
    }

    pub fn stop(mut self) -> Self {
        self.is_active = false;
        self
    }
}