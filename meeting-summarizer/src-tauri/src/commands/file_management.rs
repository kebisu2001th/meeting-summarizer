use crate::database::Database;
use crate::models::{Recording, Transcription, RecordingQuery, RecordingStats, SortBy, SortOrder};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type DbState = Arc<Mutex<Database>>;

#[tauri::command]
pub async fn get_all_recordings_fm(db: State<'_, DbState>) -> Result<Vec<Recording>, String> {
    let database = db.lock().await;
    database.get_all_recordings().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording_by_id(db: State<'_, DbState>, id: String) -> Result<Option<Recording>, String> {
    let database = db.lock().await;
    database.get_recording(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_recordings(
    db: State<'_, DbState>,
    search_text: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    date_from: Option<String>,
    date_to: Option<String>,
    min_duration: Option<i64>,
    max_duration: Option<i64>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Recording>, String> {
    let database = db.lock().await;
    
    // Parse dates
    let date_from_parsed = if let Some(date_str) = date_from {
        Some(chrono::DateTime::parse_from_rfc3339(&date_str)
            .map_err(|e| format!("Invalid date_from format: {}", e))?
            .with_timezone(&chrono::Utc))
    } else {
        None
    };

    let date_to_parsed = if let Some(date_str) = date_to {
        Some(chrono::DateTime::parse_from_rfc3339(&date_str)
            .map_err(|e| format!("Invalid date_to format: {}", e))?
            .with_timezone(&chrono::Utc))
    } else {
        None
    };

    // Parse sort_by
    let sort_by_parsed = match sort_by.as_deref().unwrap_or("created_at") {
        "created_at" => SortBy::CreatedAt,
        "updated_at" => SortBy::UpdatedAt,
        "filename" => SortBy::Filename,
        "duration" => SortBy::Duration,
        "file_size" => SortBy::FileSize,
        _ => SortBy::CreatedAt,
    };

    // Parse sort_order
    let sort_order_parsed = match sort_order.as_deref().unwrap_or("desc") {
        "asc" => SortOrder::Asc,
        "desc" => SortOrder::Desc,
        _ => SortOrder::Desc,
    };

    let query = RecordingQuery {
        search_text,
        category,
        tags: tags.unwrap_or_default(),
        date_from: date_from_parsed,
        date_to: date_to_parsed,
        min_duration,
        max_duration,
        limit: Some(limit.unwrap_or(50)),
        offset: Some(offset.unwrap_or(0)),
        sort_by: sort_by_parsed,
        sort_order: sort_order_parsed,
    };

    database.search_recordings(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_recording_metadata(
    db: State<'_, DbState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let database = db.lock().await;
    
    // Get existing recording
    let mut recording = database
        .get_recording(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Recording with id {} not found", id))?;

    // Update fields
    if let Some(title) = title {
        recording.title = Some(title);
    }
    if let Some(description) = description {
        recording.description = Some(description);
    }
    if let Some(category) = category {
        recording.category = Some(category);
    }
    if let Some(tags) = tags {
        recording.tags = tags;
    }

    database.update_recording(&recording).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_recording_fm(db: State<'_, DbState>, id: String) -> Result<bool, String> {
    let database = db.lock().await;
    database.delete_recording(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording_stats(db: State<'_, DbState>) -> Result<RecordingStats, String> {
    let database = db.lock().await;
    database.get_recording_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_categories(db: State<'_, DbState>) -> Result<Vec<String>, String> {
    let database = db.lock().await;
    database.get_all_categories().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_tags(db: State<'_, DbState>) -> Result<Vec<String>, String> {
    let database = db.lock().await;
    database.get_all_tags().await.map_err(|e| e.to_string())
}

// Transcription management commands
#[tauri::command]
pub async fn get_transcriptions_by_recording(
    db: State<'_, DbState>,
    recording_id: String,
) -> Result<Vec<Transcription>, String> {
    let database = db.lock().await;
    database
        .get_transcriptions_by_recording(&recording_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_transcription_by_id(
    db: State<'_, DbState>,
    id: String,
) -> Result<Option<Transcription>, String> {
    let database = db.lock().await;
    database.get_transcription(&id).await.map_err(|e| e.to_string())
}

// File export functionality
#[tauri::command]
pub async fn export_recording_data(
    db: State<'_, DbState>,
    recording_id: String,
    format: String,
) -> Result<String, String> {
    let database = db.lock().await;
    
    let recording = database
        .get_recording(&recording_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Recording with id {} not found", recording_id))?;

    let transcriptions = database
        .get_transcriptions_by_recording(&recording_id)
        .await
        .map_err(|e| e.to_string())?;

    match format.as_str() {
        "json" => {
            let export_data = serde_json::json!({
                "recording": recording,
                "transcriptions": transcriptions,
                "exported_at": chrono::Utc::now().to_rfc3339(),
            });
            Ok(serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?)
        }
        "text" => {
            let mut result = String::new();
            result.push_str(&format!("=== Recording: {} ===\n", recording.filename));
            result.push_str(&format!("Created: {}\n", recording.created_at.format("%Y-%m-%d %H:%M:%S")));
            
            if let Some(title) = &recording.title {
                result.push_str(&format!("Title: {}\n", title));
            }
            if let Some(description) = &recording.description {
                result.push_str(&format!("Description: {}\n", description));
            }
            if let Some(category) = &recording.category {
                result.push_str(&format!("Category: {}\n", category));
            }
            if !recording.tags.is_empty() {
                result.push_str(&format!("Tags: {}\n", recording.tags.join(", ")));
            }
            if let Some(duration) = recording.duration {
                result.push_str(&format!("Duration: {}s\n", duration));
            }

            result.push_str("\n=== Transcriptions ===\n");
            for transcription in transcriptions {
                result.push_str(&format!("\n--- {} (Confidence: {:.2}) ---\n", 
                    transcription.language,
                    transcription.confidence.unwrap_or(0.0)
                ));
                result.push_str(&transcription.text);
                result.push_str("\n");
            }

            Ok(result)
        }
        _ => Err(format!("Unsupported export format: {}", format)),
    }
}

// File management utility functions
#[tauri::command]
pub async fn get_recordings_count_fm(db: State<'_, DbState>) -> Result<i64, String> {
    let database = db.lock().await;
    database.get_recordings_count().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_orphaned_files(
    db: State<'_, DbState>,
    recordings_dir: String,
) -> Result<Vec<String>, String> {
    let database = db.lock().await;
    let recordings = database.get_all_recordings().await.map_err(|e| e.to_string())?;
    
    let mut orphaned_files = Vec::new();
    let recordings_path = std::path::Path::new(&recordings_dir);
    
    if recordings_path.exists() && recordings_path.is_dir() {
        let entries = std::fs::read_dir(recordings_path).map_err(|e| e.to_string())?;
        
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_path = entry.path();
            
            if file_path.is_file() {
                let file_path_str = file_path.to_string_lossy().to_string();
                
                // Check if this file is referenced by any recording
                let is_referenced = recordings.iter().any(|r| r.file_path == file_path_str);
                
                if !is_referenced {
                    orphaned_files.push(file_path_str);
                }
            }
        }
    }
    
    Ok(orphaned_files)
}