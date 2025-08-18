use meeting_summarizer_lib::services::{AudioCapture, RecordingService};
use meeting_summarizer_lib::database::Database;
use meeting_summarizer_lib::errors::AppResult;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_audio_capture_creation() -> AppResult<()> {
    let audio_capture = AudioCapture::new()?;
    assert!(!audio_capture.is_recording());
    Ok(())
}

#[tokio::test]
async fn test_recording_service_creation() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database, recordings_dir)?;
    
    assert!(!recording_service.is_recording());
    Ok(())
}

#[tokio::test]
async fn test_start_stop_recording() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database, recordings_dir)?;
    
    // 録音開始
    let session_id = recording_service.start_recording().await?;
    assert!(!session_id.is_empty());
    assert!(recording_service.is_recording());
    
    // 録音停止
    let recording = recording_service.stop_recording().await?;
    assert!(!recording_service.is_recording());
    assert!(!recording.id.is_empty());
    assert!(!recording.filename.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_get_recordings() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database, recordings_dir)?;
    
    // 最初は空
    let recordings = recording_service.get_recordings().await?;
    assert_eq!(recordings.len(), 0);
    
    // 録音を作成
    let _session_id = recording_service.start_recording().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let recording = recording_service.stop_recording().await?;
    
    // 録音が取得できる
    let recordings = recording_service.get_recordings().await?;
    assert_eq!(recordings.len(), 1);
    assert_eq!(recordings[0].id, recording.id);
    
    Ok(())
}

#[tokio::test]
async fn test_get_audio_devices() -> AppResult<()> {
    let devices = meeting_summarizer_lib::services::audio_capture_mock::get_audio_devices()?;
    assert!(!devices.is_empty());
    assert!(devices.contains(&"Default Microphone".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_audio_capture_start_stop() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("test_recording.wav");
    
    let audio_capture = AudioCapture::new()?;
    
    // 録音開始
    audio_capture.start_recording(&output_path).await?;
    assert!(audio_capture.is_recording());
    
    // 短時間録音
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // 録音停止
    audio_capture.stop_recording().await?;
    assert!(!audio_capture.is_recording());
    
    // ファイルが作成されていることを確認
    assert!(output_path.exists());
    
    Ok(())
}

#[tokio::test]
async fn test_double_start_recording_error() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database, recordings_dir)?;
    
    // 最初の録音開始
    let _session_id = recording_service.start_recording().await?;
    
    // 2回目の録音開始はエラーになる
    let result = recording_service.start_recording().await;
    assert!(result.is_err());
    
    // クリーンアップ
    let _recording = recording_service.stop_recording().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_stop_recording_without_start_error() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database, recordings_dir)?;
    
    // 録音開始せずに停止はエラーになる
    let result = recording_service.stop_recording().await;
    assert!(result.is_err());
    
    Ok(())
}