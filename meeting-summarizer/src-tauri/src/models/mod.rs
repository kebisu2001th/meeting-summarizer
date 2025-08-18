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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: String,
    pub recording_id: String,
    pub text: String,
    pub language: String,
    pub confidence: Option<f32>,
    pub processing_time_ms: Option<u64>,
    pub status: TranscriptionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

impl Transcription {
    pub fn new(recording_id: String, text: String, language: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            recording_id,
            text,
            language,
            confidence: None,
            processing_time_ms: None,
            status: TranscriptionStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn new_empty(recording_id: String, language: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            recording_id,
            text: String::new(),
            language,
            confidence: None,
            processing_time_ms: None,
            status: TranscriptionStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_text(mut self, text: String, confidence: Option<f32>) -> Self {
        self.text = text;
        self.confidence = confidence;
        self.status = TranscriptionStatus::Completed;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.status = TranscriptionStatus::Failed(error);
        self.updated_at = Utc::now();
        self
    }

    pub fn set_processing(mut self) -> Self {
        self.status = TranscriptionStatus::Processing;
        self.updated_at = Utc::now();
        self
    }

    pub fn set_processing_time(mut self, time_ms: u64) -> Self {
        self.processing_time_ms = Some(time_ms);
        self.updated_at = Utc::now();
        self
    }
    
    pub fn with_confidence(mut self, confidence: Option<f32>) -> Self {
        self.confidence = confidence;
        self.updated_at = Utc::now();
        self
    }
    
    pub fn with_processing_time(mut self, time_ms: Option<u64>) -> Self {
        self.processing_time_ms = time_ms;
        self.updated_at = Utc::now();
        self
    }
    
    pub fn with_status(mut self, status: TranscriptionStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }
}