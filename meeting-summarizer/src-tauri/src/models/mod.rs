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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub transcription_id: String,
    pub summary_text: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub model_used: String,
    pub processing_time_ms: Option<u64>,
    pub status: SummaryStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SummaryStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

impl Summary {
    pub fn new(transcription_id: String, model_used: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            transcription_id,
            summary_text: String::new(),
            key_points: Vec::new(),
            action_items: Vec::new(),
            model_used,
            processing_time_ms: None,
            status: SummaryStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_content(mut self, summary_text: String, key_points: Vec<String>, action_items: Vec<String>) -> Self {
        self.summary_text = summary_text;
        self.key_points = key_points;
        self.action_items = action_items;
        self.status = SummaryStatus::Completed;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.status = SummaryStatus::Failed(error);
        self.updated_at = Utc::now();
        self
    }

    pub fn set_processing(mut self) -> Self {
        self.status = SummaryStatus::Processing;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_processing_time(mut self, time_ms: u64) -> Self {
        self.processing_time_ms = Some(time_ms);
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub base_url: String,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    Ollama,
    OpenAI,
    GPT4All,
    LMStudio,
    Custom,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::Ollama,
            base_url: "http://localhost:11434".to_string(),
            model_name: "llama3.2:3b".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 120,
        }
    }
}