use std::path::PathBuf;
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

    #[error("Invalid file path: {message}")]
    InvalidPath { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Transcription failed: {message}")]
    TranscriptionFailed { message: String },
    
    #[error("Whisper service error: {message}")]
    WhisperService { message: String },
    
    #[error("Whisper initialization failed: {message}")]
    WhisperInit { message: String },
    
    #[error("Whisper not initialized: {message}")]
    WhisperNotInitialized { message: String },
    
    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        // 本番環境では詳細なエラー情報を隠蔽
        match error {
            AppError::InvalidPath { .. } => "Invalid file path".to_string(),
            AppError::PermissionDenied { .. } => "Access denied".to_string(),
            _ => error.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

// セキュリティ関連のユーティリティ関数
pub fn validate_file_path(file_path: &str, allowed_dir: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(file_path);
    
    // 基本的な検証
    if file_path.is_empty() {
        return Err(AppError::InvalidPath {
            message: "File path cannot be empty".to_string(),
        });
    }
    
    if file_path.len() > 1000 {
        return Err(AppError::InvalidPath {
            message: "File path too long".to_string(),
        });
    }
    
    // パストラバーサル攻撃防止
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AppError::InvalidPath {
            message: "Path traversal detected".to_string(),
        });
    }
    
    // 許可されたディレクトリ内かチェック
    let allowed_canonical = match PathBuf::from(allowed_dir).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return Err(AppError::InvalidPath {
                message: "Invalid allowed directory".to_string(),
            });
        }
    };
    
    // ファイルパスを正規化
    let canonical = if path.is_absolute() {
        match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // ファイルが存在しない場合でも、親ディレクトリが正当かチェック
                let parent = path.parent().ok_or_else(|| AppError::InvalidPath {
                    message: "Invalid path structure".to_string(),
                })?;
                
                if parent.exists() {
                    let canonical_parent = parent.canonicalize().map_err(|_| AppError::InvalidPath {
                        message: "Invalid parent directory".to_string(),
                    })?;
                    canonical_parent.join(path.file_name().ok_or_else(|| AppError::InvalidPath {
                        message: "Invalid filename".to_string(),
                    })?)
                } else {
                    return Err(AppError::InvalidPath {
                        message: "Parent directory does not exist".to_string(),
                    });
                }
            }
        }
    } else {
        // 相対パスを許可されたディレクトリ内で解決
        allowed_canonical.join(&path)
    };
    
    if !canonical.starts_with(&allowed_canonical) {
        return Err(AppError::PermissionDenied {
            message: "Path outside allowed directory".to_string(),
        });
    }
    
    Ok(canonical)
}

// ファイル名の安全性をチェック
pub fn validate_filename(filename: &str) -> AppResult<()> {
    if filename.is_empty() {
        return Err(AppError::ValidationError {
            message: "Filename cannot be empty".to_string(),
        });
    }
    
    if filename.len() > 255 {
        return Err(AppError::ValidationError {
            message: "Filename too long".to_string(),
        });
    }
    
    // 危険な文字をチェック
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0', '/', '\\'];
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(AppError::ValidationError {
            message: "Filename contains invalid characters".to_string(),
        });
    }
    
    // Windowsの予約ファイル名をチェック
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5",
        "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5",
        "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    
    let name_upper = filename.to_uppercase();
    let name_without_ext = name_upper.split('.').next().unwrap_or("");
    if reserved_names.iter().any(|&reserved| name_without_ext == reserved) {
        return Err(AppError::ValidationError {
            message: "Filename is reserved".to_string(),
        });
    }
    
    Ok(())
}

// 音声ファイル形式の検証
pub fn validate_audio_format(file_path: &PathBuf) -> AppResult<()> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| AppError::ValidationError {
            message: "File has no extension".to_string(),
        })?;
    
    let allowed_extensions = ["wav", "mp3", "m4a", "flac", "ogg"];
    if !allowed_extensions.iter().any(|&ext| ext.eq_ignore_ascii_case(extension)) {
        return Err(AppError::ValidationError {
            message: format!("Unsupported audio format: {}", extension),
        });
    }
    
    Ok(())
}

// ファイルサイズ制限のチェック
pub fn validate_file_size(file_path: &PathBuf, max_size_mb: u64) -> AppResult<()> {
    if !file_path.exists() {
        return Err(AppError::FileNotFound {
            path: file_path.to_string_lossy().to_string(),
        });
    }
    
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();
    let max_size_bytes = max_size_mb * 1024 * 1024;
    
    if file_size > max_size_bytes {
        return Err(AppError::ValidationError {
            message: format!("File too large: {} MB (max: {} MB)", 
                file_size / (1024 * 1024), max_size_mb),
        });
    }
    
    Ok(())
}