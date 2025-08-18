use meeting_summarizer_lib::services::{RecordingService, WhisperService};
use meeting_summarizer_lib::database::Database;
use meeting_summarizer_lib::errors::AppResult;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// 完全なワークフローテスト：録音 → 書き起こし → データ取得
#[tokio::test]
async fn test_complete_recording_transcription_workflow() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("integration_test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    // サービス初期化
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    // Whisper初期化
    whisper_service.initialize().await?;
    assert!(whisper_service.is_initialized().await);
    
    // Step 1: 録音開始
    assert!(!recording_service.is_recording());
    let session_id = recording_service.start_recording().await?;
    assert!(!session_id.is_empty());
    assert!(recording_service.is_recording());
    
    // Step 2: 短時間録音
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Step 3: 録音停止
    let recording = recording_service.stop_recording().await?;
    assert!(!recording_service.is_recording());
    assert!(!recording.id.is_empty());
    assert!(!recording.filename.is_empty());
    assert!(recording.duration.is_some());
    
    // Step 4: 録音ファイルが存在することを確認
    let recording_path = PathBuf::from(&recording.file_path);
    assert!(recording_path.exists());
    
    // Step 5: 書き起こし実行
    let transcription = whisper_service
        .transcribe_audio_file(&recording_path, recording.id.clone(), Some("ja".to_string()))
        .await?;
    
    // Step 6: 書き起こし結果検証
    assert_eq!(transcription.recording_id, recording.id);
    assert!(!transcription.text.is_empty());
    assert_eq!(transcription.language, "ja");
    assert!(transcription.confidence.is_some());
    assert!(transcription.processing_time_ms.is_some());
    
    // Step 7: データベースから録音データを取得
    let retrieved_recording = recording_service
        .get_recording(&recording.id)
        .await?
        .expect("Recording should exist");
    assert_eq!(retrieved_recording.id, recording.id);
    
    // Step 8: 全録音の取得
    let all_recordings = recording_service.get_recordings().await?;
    assert_eq!(all_recordings.len(), 1);
    assert_eq!(all_recordings[0].id, recording.id);
    
    // Step 9: 録音数の確認
    let count = recording_service.get_recordings_count().await?;
    assert_eq!(count, 1);
    
    // Step 10: 録音削除
    let deleted = recording_service.delete_recording(&recording.id).await?;
    assert!(deleted);
    
    // Step 11: 削除後の確認
    let count_after_delete = recording_service.get_recordings_count().await?;
    assert_eq!(count_after_delete, 0);
    
    let deleted_recording = recording_service.get_recording(&recording.id).await?;
    assert!(deleted_recording.is_none());
    
    Ok(())
}

/// 複数の録音と書き起こしテスト
#[tokio::test]
async fn test_multiple_recordings_workflow() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("multiple_test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    whisper_service.initialize().await?;
    
    const NUM_RECORDINGS: usize = 3;
    let mut recordings = Vec::new();
    
    // 複数の録音を作成
    for i in 0..NUM_RECORDINGS {
        let _session_id = recording_service.start_recording().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(100 + i as u64 * 50)).await;
        let recording = recording_service.stop_recording().await?;
        recordings.push(recording);
    }
    
    // 録音数の確認
    assert_eq!(recording_service.get_recordings_count().await?, NUM_RECORDINGS as i64);
    
    // 各録音の書き起こし
    let mut transcriptions = Vec::new();
    for recording in &recordings {
        let audio_path = PathBuf::from(&recording.file_path);
        let transcription = whisper_service
            .transcribe_audio_file(&audio_path, recording.id.clone(), Some("ja".to_string()))
            .await?;
        transcriptions.push(transcription);
    }
    
    // 書き起こし結果の検証
    assert_eq!(transcriptions.len(), NUM_RECORDINGS);
    for (i, transcription) in transcriptions.iter().enumerate() {
        assert_eq!(transcription.recording_id, recordings[i].id);
        assert!(!transcription.text.is_empty());
    }
    
    Ok(())
}

/// エラー処理のテスト
#[tokio::test]
async fn test_error_handling_workflow() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("error_test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    // 二重録音開始のエラー
    let _session_id = recording_service.start_recording().await?;
    let second_start = recording_service.start_recording().await;
    assert!(second_start.is_err());
    let _recording = recording_service.stop_recording().await?;
    
    // 録音なしの停止エラー
    let stop_without_start = recording_service.stop_recording().await;
    assert!(stop_without_start.is_err());
    
    // Whisper初期化なしでの書き起こし（新実装では初期化が必須）
    assert!(!whisper_service.is_initialized().await);
    let _session_id = recording_service.start_recording().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let recording = recording_service.stop_recording().await?;
    
    let audio_path = PathBuf::from(&recording.file_path);
    // 初期化なしでの書き起こしはエラーになることを確認
    let transcription_result = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), Some("ja".to_string()))
        .await;
    assert!(transcription_result.is_err());
    
    // 初期化後なら成功することを確認
    whisper_service.initialize().await?;
    let transcription = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), Some("ja".to_string()))
        .await?;
    assert!(!transcription.text.is_empty());
    
    Ok(())
}

/// パフォーマンステスト
#[tokio::test]
async fn test_performance_workflow() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("perf_test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = RecordingService::new(database.clone(), recordings_dir.clone())?;
    let whisper_service = WhisperService::new(model_path, recordings_dir);
    
    whisper_service.initialize().await?;
    
    // 録音時間の測定
    let start_time = std::time::Instant::now();
    let _session_id = recording_service.start_recording().await?;
    let recording_start_duration = start_time.elapsed();
    assert!(recording_start_duration.as_millis() < 100); // 100ms以内で開始
    
    // 録音期間
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // 停止時間の測定
    let stop_time = std::time::Instant::now();
    let recording = recording_service.stop_recording().await?;
    let recording_stop_duration = stop_time.elapsed();
    assert!(recording_stop_duration.as_millis() < 1000); // 1秒以内で停止
    
    // 書き起こし時間の測定
    let audio_path = PathBuf::from(&recording.file_path);
    let transcription_start = std::time::Instant::now();
    let transcription = whisper_service
        .transcribe_audio_file(&audio_path, recording.id.clone(), Some("ja".to_string()))
        .await?;
    let transcription_duration = transcription_start.elapsed();
    
    // 書き起こし時間の確認（ローカルWhisper実装）
    assert!(transcription_duration.as_nanos() >= 1); // 何らかの時間がかかっている
    assert!(transcription_duration.as_secs() <= 30); // 最大30秒（ローカル処理）
    
    // 書き起こし結果に処理時間が記録されている
    assert!(transcription.processing_time_ms.is_some());
    let processing_time = transcription.processing_time_ms.unwrap();
    assert!(processing_time >= 0);
    assert!(processing_time <= 30000); // 最大30秒
    
    Ok(())
}

/// 同期処理テスト
#[tokio::test]
async fn test_concurrent_operations() -> AppResult<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("concurrent_test.db");
    let recordings_dir = temp_dir.path().join("recordings");
    let model_path = temp_dir.path().join("model.bin");
    
    let database = Arc::new(Database::new(db_path)?);
    let recording_service = Arc::new(RecordingService::new(database.clone(), recordings_dir.clone())?);
    let whisper_service = Arc::new(WhisperService::new(model_path, recordings_dir));
    
    whisper_service.initialize().await?;
    
    // 同時に複数のサービス操作を実行
    let recording_service_clone = recording_service.clone();
    let whisper_service_clone = whisper_service.clone();
    
    let handle1 = tokio::spawn(async move {
        // 録音とデータベース操作
        recording_service_clone.get_recordings_count().await
    });
    
    let handle2 = tokio::spawn(async move {
        // Whisper状態チェック
        whisper_service_clone.is_initialized().await
    });
    
    let (count_result, whisper_status) = tokio::join!(handle1, handle2);
    
    assert!(count_result.is_ok());
    assert_eq!(count_result.unwrap()?, 0);
    assert!(whisper_status.is_ok());
    assert!(whisper_status.unwrap());
    
    Ok(())
}