use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Recording error: {message}")]
    Recording { message: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;