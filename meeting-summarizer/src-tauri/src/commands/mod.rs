use crate::models::Recording;
use crate::services::RecordingService;
use tauri::State;
use std::sync::Arc;

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