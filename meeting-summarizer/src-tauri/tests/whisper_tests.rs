use meeting_summarizer_lib::services::{RecordingService, WhisperService};
use meeting_summarizer_lib::database::Database;
use meeting_summarizer_lib::errors::AppResult;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_whisper_service_creation() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let model_path = temp_dir.path().join("model.bin");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    assert!(!whisper_service.is_initialized().await);
    Ok(())
}

#[tokio::test]
async fn test_whisper_initialization() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let model_path = temp_dir.path().join("model.bin");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    // 初期化
    whisper_service.initialize().await?;
    assert!(whisper_service.is_initialized().await);
    
    Ok(())
}

#[tokio::test]
async fn test_transcription_workflow() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    // サービス初期化
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    // Whisper初期化
    whisper_service.initialize().await?;
    
    // 録音作成
    let _session_id = recording_service.start_recording().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let recording = recording_service.stop_recording().await?;
    
    // 書き起こし実行
    let audio_path = PathBuf::from(&recording.file_path);
    let transcription = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), Some("ja".to_string()))
        .await?;
    
    // 結果検証
    assert_eq!(transcription.recording_id, recording.id);
    assert!(!transcription.text.is_empty());
    assert_eq!(transcription.language, "ja");
    assert!(transcription.confidence.is_some());
    assert!(transcription.processing_time_ms.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_transcription_invalid_file() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let model_path = temp_dir.path().join("model.bin");
    let recordings_dir = temp_dir.path().join("recordings");
    let invalid_audio_path = temp_dir.path().join("nonexistent.wav");
    
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    whisper_service.initialize().await?;
    
    // 存在しないファイルでの書き起こしはエラーになる
    let result = whisper_service
        .transcribe_audio_file(&invalid_audio_path, "test_id".to_string(), Some("ja".to_string()))
        .await;
    
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_transcription_different_languages() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    // サービス初期化
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    whisper_service.initialize().await?;
    
    // 録音作成
    let _session_id = recording_service.start_recording().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let recording = recording_service.stop_recording().await?;
    let audio_path = PathBuf::from(&recording.file_path);
    
    // 日本語での書き起こし
    let transcription_ja = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), Some("ja".to_string()))
        .await?;
    assert_eq!(transcription_ja.language, "ja");
    
    // 英語での書き起こし
    let transcription_en = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), Some("en".to_string()))
        .await?;
    assert_eq!(transcription_en.language, "en");
    
    Ok(())
}

#[tokio::test]
async fn test_transcription_status_lifecycle() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    // サービス初期化
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    whisper_service.initialize().await?;
    
    // 録音作成
    let _session_id = recording_service.start_recording().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let recording = recording_service.stop_recording().await?;
    
    // 書き起こし実行
    let audio_path = PathBuf::from(&recording.file_path);
    let transcription = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), None)
        .await?;
    
    // ステータスチェック
    match transcription.status {
        meeting_summarizer_lib::models::TranscriptionStatus::Completed => {
            assert!(!transcription.text.is_empty());
        }
        _ => panic!("Expected completed status"),
    }
    
    Ok(())
}