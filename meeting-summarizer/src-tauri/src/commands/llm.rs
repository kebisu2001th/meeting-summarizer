use crate::database::Database;
use crate::models::{LLMConfig, LLMProvider, Summary};
use crate::services::LLMService;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type DbState = Arc<Mutex<Database>>;

#[tauri::command]
pub async fn generate_summary(
    db: State<'_, DbState>,
    transcription_text: String,
    transcription_id: String,
    model_config: Option<LLMConfig>,
) -> Result<Summary, String> {
    let database = db.lock().await;
    
    // Use provided config or default
    let config = model_config.unwrap_or_default();
    let llm_service = LLMService::new(config);
    
    log::info!("🤖 Generating summary for transcription: {}", transcription_id);
    
    // Generate summary using LLM
    let result = llm_service
        .summarize_text(&transcription_text, transcription_id.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // Save summary to database
    database
        .create_summary(&result)
        .await
        .map_err(|e| e.to_string())?;
    
    log::info!("✅ Summary generated and saved: {}", result.id);
    Ok(result)
}

#[tauri::command]
pub async fn get_summary_by_id(
    db: State<'_, DbState>,
    id: String,
) -> Result<Option<Summary>, String> {
    let database = db.lock().await;
    database.get_summary(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_summaries_for_transcription(
    db: State<'_, DbState>,
    transcription_id: String,
) -> Result<Vec<Summary>, String> {
    let database = db.lock().await;
    database
        .get_summaries_by_transcription(&transcription_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_summary(
    db: State<'_, DbState>,
    summary: Summary,
) -> Result<(), String> {
    let database = db.lock().await;
    database.update_summary(&summary).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_summary(
    db: State<'_, DbState>,
    id: String,
) -> Result<bool, String> {
    let database = db.lock().await;
    database.delete_summary(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_llm_connection(
    config: LLMConfig,
) -> Result<bool, String> {
    let llm_service = LLMService::new(config);
    llm_service.check_connection().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_default_llm_config() -> Result<LLMConfig, String> {
    Ok(LLMConfig::default())
}

#[tauri::command]
pub async fn validate_llm_config(
    config: LLMConfig,
) -> Result<bool, String> {
    // Basic validation
    if config.base_url.is_empty() || config.model_name.is_empty() {
        return Ok(false);
    }
    
    if config.timeout_seconds == 0 || config.timeout_seconds > 600 {
        return Ok(false);
    }
    
    if config.temperature < 0.0 || config.temperature > 2.0 {
        return Ok(false);
    }
    
    if config.max_tokens == 0 || config.max_tokens > 8192 {
        return Ok(false);
    }
    
    // Try to connect to validate the configuration
    let llm_service = LLMService::new(config);
    llm_service.check_connection().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_llm_providers() -> Result<Vec<String>, String> {
    Ok(vec![
        "Ollama".to_string(),
        "OpenAI".to_string(),
        "GPT4All".to_string(),
        "LMStudio".to_string(),
        "Custom".to_string(),
    ])
}

#[tauri::command]
pub async fn get_provider_default_config(
    provider: String,
) -> Result<LLMConfig, String> {
    let provider_enum = match provider.as_str() {
        "Ollama" => LLMProvider::Ollama,
        "OpenAI" => LLMProvider::OpenAI,
        "GPT4All" => LLMProvider::GPT4All,
        "LMStudio" => LLMProvider::LMStudio,
        "Custom" => LLMProvider::Custom,
        _ => return Err("Invalid provider".to_string()),
    };

    let config = match provider_enum {
        LLMProvider::Ollama => LLMConfig {
            provider: LLMProvider::Ollama,
            base_url: "http://localhost:11434".to_string(),
            model_name: "llama3.2:3b".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 120,
        },
        LLMProvider::OpenAI => LLMConfig {
            provider: LLMProvider::OpenAI,
            base_url: "https://api.openai.com".to_string(),
            model_name: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 60,
        },
        LLMProvider::GPT4All => LLMConfig {
            provider: LLMProvider::GPT4All,
            base_url: "http://localhost:4891".to_string(),
            model_name: "gpt4all-13b-snoozy".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 120,
        },
        LLMProvider::LMStudio => LLMConfig {
            provider: LLMProvider::LMStudio,
            base_url: "http://localhost:1234".to_string(),
            model_name: "local-model".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 120,
        },
        LLMProvider::Custom => LLMConfig {
            provider: LLMProvider::Custom,
            base_url: "http://localhost:8080".to_string(),
            model_name: "custom-model".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 120,
        },
    };

    Ok(config)
}

#[tauri::command]
pub async fn test_summarization(
    config: LLMConfig,
    sample_text: String,
) -> Result<Summary, String> {
    let llm_service = LLMService::new(config);
    
    // Create a test transcription ID
    let test_transcription_id = "test-transcription".to_string();
    
    log::info!("🧪 Testing summarization with sample text");
    
    llm_service
        .summarize_text(&sample_text, test_transcription_id)
        .await
        .map_err(|e| e.to_string())
}