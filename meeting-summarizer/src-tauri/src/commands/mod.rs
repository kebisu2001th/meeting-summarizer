use crate::models::{Recording, Transcription};
use crate::services::{RecordingService, WhisperService};
use tauri::State;
use std::sync::Arc;
use std::path::PathBuf;

#[tauri::command]
pub async fn start_recording(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<String, String> {
    recording_service
        .start_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<Recording, String> {
    recording_service
        .stop_recording()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recordings(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<Vec<Recording>, String> {
    recording_service
        .get_recordings()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording(
    recording_service: State<'_, Arc<RecordingService>>,
    id: String,
) -> Result<Option<Recording>, String> {
    recording_service
        .get_recording(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_recording(
    recording_service: State<'_, Arc<RecordingService>>,
    id: String,
) -> Result<bool, String> {
    recording_service
        .delete_recording(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_recording(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<bool, String> {
    Ok(recording_service.is_recording())
}

#[tauri::command]
pub async fn get_recordings_count(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<i64, String> {
    recording_service
        .get_recordings_count()
        .await
        .map_err(|e| e.to_string())
}

// Whisper 書き起こし関連コマンド

#[tauri::command]
pub async fn transcribe_recording(
    recording_service: State<'_, Arc<RecordingService>>,
    whisper_service: State<'_, Arc<WhisperService>>,
    recording_id: String,
    language: Option<String>,
) -> Result<Transcription, String> {
    // 録音ファイルの取得
    let recording = recording_service
        .get_recording(&recording_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Recording not found".to_string())?;

    // 音声ファイルが存在するかチェック
    let audio_path = PathBuf::from(&recording.file_path);
    if !audio_path.exists() {
        return Err("Audio file not found".to_string());
    }

    // 書き起こし実行
    whisper_service
        .transcribe_audio_file(&audio_path, recording_id, language)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn initialize_whisper(
    whisper_service: State<'_, Arc<WhisperService>>,
) -> Result<(), String> {
    whisper_service
        .initialize()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_whisper_initialized(
    whisper_service: State<'_, Arc<WhisperService>>,
) -> Result<bool, String> {
    Ok(whisper_service.is_initialized().await)
}