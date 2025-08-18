use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub filename: String,
    pub file_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub duration: Option<i64>, // seconds
    pub file_size: Option<i64>, // bytes
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
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
            title: None,
            description: None,
            category: None,
            tags: Vec::new(),
            duration: None,
            file_size: None,
            sample_rate: None,
            channels: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self.updated_at = Utc::now();
        self
    }

    pub fn add_tag(mut self, tag: String) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
        self
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

    pub fn with_audio_info(mut self, sample_rate: i32, channels: i32) -> Self {
        self.sample_rate = Some(sample_rate);
        self.channels = Some(channels);
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingQuery {
    pub search_text: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub min_duration: Option<i64>,
    pub max_duration: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    CreatedAt,
    UpdatedAt,
    Filename,
    Duration,
    FileSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for RecordingQuery {
    fn default() -> Self {
        Self {
            search_text: None,
            category: None,
            tags: Vec::new(),
            date_from: None,
            date_to: None,
            min_duration: None,
            max_duration: None,
            limit: Some(50),
            offset: Some(0),
            sort_by: SortBy::CreatedAt,
            sort_order: SortOrder::Desc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStats {
    pub total_count: i64,
    pub total_duration: i64,
    pub total_size: i64,
    pub categories: Vec<CategoryStats>,
    pub recent_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub name: String,
    pub count: i64,
    pub total_duration: i64,
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