use crate::database::Database;
use crate::models::{LLMConfig, Summary};
use crate::services::LLMService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;

type DbState = Arc<Mutex<Database>>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SummarizationProgress {
    pub stage: String,
    pub message: String,
    pub progress: f32, // 0.0 to 1.0
    pub summary_id: Option<String>,
    pub completed: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn generate_summary_with_progress(
    window: Window,
    db: State<'_, DbState>,
    transcription_text: String,
    transcription_id: String,
    model_config: Option<LLMConfig>,
) -> Result<Summary, String> {
    let database = db.lock().await;
    
    // Use provided config or default
    let config = model_config.unwrap_or_default();
    let llm_service = LLMService::new(config.clone());
    
    log::info!("🤖 Starting summarization with progress tracking for transcription: {}", transcription_id);
    
    // Emit initial progress
    let _ = window.emit("summarization-progress", SummarizationProgress {
        stage: "initializing".to_string(),
        message: "LLM接続を初期化中...".to_string(),
        progress: 0.1,
        summary_id: None,
        completed: false,
        error: None,
    });
    
    // Check LLM connection
    match llm_service.check_connection().await {
        Ok(true) => {
            let _ = window.emit("summarization-progress", SummarizationProgress {
                stage: "connected".to_string(),
                message: format!("{}に接続済み", config.model_name),
                progress: 0.2,
                summary_id: None,
                completed: false,
                error: None,
            });
        }
        Ok(false) => {
            let error_msg = format!("LLMサーバーに接続できません: {}", config.base_url);
            let _ = window.emit("summarization-progress", SummarizationProgress {
                stage: "error".to_string(),
                message: error_msg.clone(),
                progress: 0.0,
                summary_id: None,
                completed: false,
                error: Some(error_msg.clone()),
            });
            return Err(error_msg);
        }
        Err(e) => {
            let error_msg = format!("接続チェック中にエラー: {}", e);
            let _ = window.emit("summarization-progress", SummarizationProgress {
                stage: "error".to_string(),
                message: error_msg.clone(),
                progress: 0.0,
                summary_id: None,
                completed: false,
                error: Some(error_msg.clone()),
            });
            return Err(error_msg);
        }
    }
    
    // Emit processing start
    let _ = window.emit("summarization-progress", SummarizationProgress {
        stage: "processing".to_string(),
        message: format!("{}で要約を生成中...", config.model_name),
        progress: 0.3,
        summary_id: None,
        completed: false,
        error: None,
    });
    
    // Generate summary
    let result = llm_service
        .summarize_text(&transcription_text, transcription_id.clone())
        .await;
    
    match result {
        Ok(summary) => {
            // Emit processing completion
            let _ = window.emit("summarization-progress", SummarizationProgress {
                stage: "saving".to_string(),
                message: "要約をデータベースに保存中...".to_string(),
                progress: 0.8,
                summary_id: Some(summary.id.clone()),
                completed: false,
                error: None,
            });
            
            // Save to database
            match database.create_summary(&summary).await {
                Ok(_) => {
                    // Emit completion
                    let _ = window.emit("summarization-progress", SummarizationProgress {
                        stage: "completed".to_string(),
                        message: "要約の生成が完了しました".to_string(),
                        progress: 1.0,
                        summary_id: Some(summary.id.clone()),
                        completed: true,
                        error: None,
                    });
                    
                    log::info!("✅ Summary generated and saved with progress tracking: {}", summary.id);
                    Ok(summary)
                }
                Err(e) => {
                    let error_msg = format!("データベース保存エラー: {}", e);
                    let _ = window.emit("summarization-progress", SummarizationProgress {
                        stage: "error".to_string(),
                        message: error_msg.clone(),
                        progress: 0.8,
                        summary_id: Some(summary.id.clone()),
                        completed: false,
                        error: Some(error_msg.clone()),
                    });
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("要約生成エラー: {}", e);
            let _ = window.emit("summarization-progress", SummarizationProgress {
                stage: "error".to_string(),
                message: error_msg.clone(),
                progress: 0.3,
                summary_id: None,
                completed: false,
                error: Some(error_msg.clone()),
            });
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn cancel_summarization(
    window: Window,
    summary_id: Option<String>,
) -> Result<(), String> {
    // Note: In a full implementation, this would cancel the ongoing LLM request
    // For now, we just emit a cancellation event
    
    let _ = window.emit("summarization-progress", SummarizationProgress {
        stage: "cancelled".to_string(),
        message: "要約生成がキャンセルされました".to_string(),
        progress: 0.0,
        summary_id,
        completed: false,
        error: Some("User cancelled".to_string()),
    });
    
    log::info!("🛑 Summarization cancelled by user");
    Ok(())
}

#[tauri::command]
pub async fn get_summarization_status(
    summary_id: String,
) -> Result<SummarizationProgress, String> {
    // This would typically check the status of an ongoing summarization
    // For now, return a default status
    Ok(SummarizationProgress {
        stage: "unknown".to_string(),
        message: "ステータス不明".to_string(),
        progress: 0.0,
        summary_id: Some(summary_id),
        completed: false,
        error: None,
    })
}