use crate::errors::AppError;
use crate::models::{Recording, Transcription};
use crate::services::{RecordingService, WhisperService};
use tauri::{AppHandle, State};
use std::sync::Arc;
use std::path::PathBuf;

// セキュリティ：基本的な認証チェック（実装は簡易版）
async fn validate_request(_app_handle: &AppHandle) -> Result<(), AppError> {
    // TODO: 実際の認証システムでは、セッショントークンやJWTの検証を行う
    // 現在は基本チェックのみ実装
    
    // 簡易実装：アプリケーションハンドルが存在することで認証済みとみなす
    // 本格的な実装では、セッション管理、JWT検証、権限チェックなどを実装
    
    Ok(())
}

// 入力の基本的なサニタイゼーション
fn sanitize_string_input(input: &str, max_length: usize) -> Result<String, AppError> {
    if input.is_empty() {
        return Err(AppError::ValidationError {
            message: "Input cannot be empty".to_string(),
        });
    }
    
    if input.len() > max_length {
        return Err(AppError::ValidationError {
            message: format!("Input too long (max: {} characters)", max_length),
        });
    }
    
    // 基本的な危険文字の除去
    let sanitized = input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect::<String>();
    
    Ok(sanitized)
}

#[tauri::command]
pub async fn start_recording(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<String, String> {
    recording_service
        .start_recording()
        .await
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
    app_handle: AppHandle,
    recording_service: State<'_, Arc<RecordingService>>,
    id: String,
) -> Result<bool, String> {
    // 認証チェック
    validate_request(&app_handle)
        .await
        .map_err(|e| e.to_string())?;
    
    // 入力の検証とサニタイゼーション
    let sanitized_id = sanitize_string_input(&id, 50)
        .map_err(|e| e.to_string())?;
    
    recording_service
        .delete_recording(&sanitized_id)
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

#[tauri::command]
pub async fn get_audio_devices(
    recording_service: State<'_, Arc<RecordingService>>,
) -> Result<Vec<String>, String> {
    recording_service
        .get_audio_devices()
        .map_err(|e| e.to_string())
}

// Whisper 書き起こし関連コマンド

#[tauri::command]
pub async fn transcribe_recording(
    app_handle: AppHandle,
    recording_service: State<'_, Arc<RecordingService>>,
    whisper_service: State<'_, Arc<WhisperService>>,
    recording_id: String,
    language: Option<String>,
) -> Result<Transcription, String> {
    // 認証チェック
    validate_request(&app_handle)
        .await
        .map_err(|e| e.to_string())?;
    
    // 入力の検証とサニタイゼーション
    let sanitized_recording_id = sanitize_string_input(&recording_id, 50)
        .map_err(|e| e.to_string())?;
    
    let sanitized_language = if let Some(lang) = language {
        Some(sanitize_string_input(&lang, 10)
            .map_err(|e| e.to_string())?)
    } else {
        None
    };
    
    // 録音ファイルの取得
    let recording = recording_service
        .get_recording(&sanitized_recording_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Recording not found".to_string())?;

    // 音声ファイルが存在するかチェック
    let audio_path = PathBuf::from(&recording.file_path);
    if !audio_path.exists() {
        return Err("Audio file not found".to_string());
    }

    // 書き起こし実行（セキュリティ検証は WhisperService 内で実行）
    whisper_service
        .transcribe_audio_file(&audio_path, sanitized_recording_id, sanitized_language)
        .await
        .map_err(|e| {
            // エラーログを記録（本番環境では詳細なエラー情報を隠蔽）
            log::error!("Transcription failed for recording {}: {}", recording_id, e);
            "Transcription failed".to_string()
        })
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