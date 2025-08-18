use crate::services::{ModelDownloader, DownloadableModel, SystemCompatibility, DownloadProgress};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type ModelDownloaderState = Arc<Mutex<ModelDownloader>>;

#[tauri::command]
pub async fn get_downloadable_models(
    downloader: State<'_, ModelDownloaderState>,
) -> Result<Vec<DownloadableModel>, String> {
    let downloader = downloader.lock().await;
    let models: Vec<DownloadableModel> = downloader.get_downloadable_models()
        .into_iter()
        .cloned()
        .collect();
    
    log::info!("📋 Retrieved {} downloadable models", models.len());
    Ok(models)
}

#[tauri::command]
pub async fn get_models_by_category(
    downloader: State<'_, ModelDownloaderState>,
    category: String,
) -> Result<Vec<DownloadableModel>, String> {
    let downloader = downloader.lock().await;
    let models: Vec<DownloadableModel> = downloader.get_models_by_category(&category)
        .into_iter()
        .cloned()
        .collect();
    
    log::info!("📂 Retrieved {} models for category: {}", models.len(), category);
    Ok(models)
}

#[tauri::command]
pub async fn check_system_requirements(
    downloader: State<'_, ModelDownloaderState>,
    model_id: String,
) -> Result<SystemCompatibility, String> {
    let downloader = downloader.lock().await;
    let compatibility = downloader.check_system_requirements(&model_id)
        .map_err(|e| e.to_string())?;
    
    log::info!("🔍 System compatibility check for {}: compatible={}", 
               model_id, compatibility.is_fully_compatible());
    Ok(compatibility)
}

#[tauri::command]
pub async fn start_model_download(
    downloader: State<'_, ModelDownloaderState>,
    model_id: String,
) -> Result<DownloadProgress, String> {
    log::info!("📥 Starting download for model: {}", model_id);
    
    let downloader = downloader.lock().await;
    
    // モデルIDを分解
    let parts: Vec<&str> = model_id.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid model ID format".to_string());
    }
    
    let provider = parts[0];
    let model_name = parts[1];
    
    match provider {
        "ollama" => {
            downloader.start_download_ollama(model_name)
                .await
                .map_err(|e| e.to_string())
        }
        _ => {
            Err(format!("Download not supported for provider: {}", provider))
        }
    }
}

#[tauri::command]
pub async fn get_download_command(
    downloader: State<'_, ModelDownloaderState>,
    model_id: String,
) -> Result<String, String> {
    let downloader = downloader.lock().await;
    let models = downloader.get_downloadable_models();
    
    let model = models.iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Model not found: {}", model_id))?;
    
    log::info!("📋 Retrieved download command for: {}", model_id);
    Ok(model.download_command.clone())
}

#[tauri::command]
pub async fn search_models(
    downloader: State<'_, ModelDownloaderState>,
    query: String,
    tags: Vec<String>,
) -> Result<Vec<DownloadableModel>, String> {
    let downloader = downloader.lock().await;
    let models: Vec<DownloadableModel> = downloader.search_models(&query, &tags)
        .into_iter()
        .cloned()
        .collect();
    
    log::info!("🔍 Search '{}' with tags {:?} returned {} models", query, tags, models.len());
    Ok(models)
}

#[tauri::command]
pub async fn get_popular_models(
    downloader: State<'_, ModelDownloaderState>,
    limit: Option<u32>,
) -> Result<Vec<DownloadableModel>, String> {
    let downloader = downloader.lock().await;
    let limit = limit.unwrap_or(10) as usize;
    let models: Vec<DownloadableModel> = downloader.get_popular_models(limit)
        .into_iter()
        .cloned()
        .collect();
    
    log::info!("⭐ Retrieved {} popular models", models.len());
    Ok(models)
}

#[tauri::command]
pub async fn get_gpt4all_download_info(
    downloader: State<'_, ModelDownloaderState>,
    model_name: String,
) -> Result<String, String> {
    let downloader = downloader.lock().await;
    let download_url = downloader.get_gpt4all_download_info(&model_name)
        .map_err(|e| e.to_string())?;
    
    log::info!("📥 GPT4All download info for {}: {}", model_name, download_url);
    Ok(download_url)
}

#[tauri::command]
pub async fn validate_model_download_requirements(
    downloader: State<'_, ModelDownloaderState>,
    model_ids: Vec<String>,
) -> Result<Vec<(String, SystemCompatibility)>, String> {
    let downloader = downloader.lock().await;
    let mut results = Vec::new();
    
    for model_id in model_ids {
        match downloader.check_system_requirements(&model_id) {
            Ok(compatibility) => {
                results.push((model_id, compatibility));
            }
            Err(e) => {
                log::warn!("❌ Failed to check requirements for {}: {}", model_id, e);
                // エラーの場合は互換性なしとして追加
                let compatibility = SystemCompatibility {
                    model_id: model_id.clone(),
                    memory_compatible: false,
                    disk_compatible: false,
                    platform_compatible: false,
                    available_memory_mb: 0,
                    required_memory_mb: 0,
                    available_disk_mb: 0,
                    required_disk_mb: 0,
                    warnings: vec![format!("Requirements check failed: {}", e)],
                };
                results.push((model_id, compatibility));
            }
        }
    }
    
    log::info!("✅ Validated requirements for {} models", results.len());
    Ok(results)
}

#[tauri::command]
pub async fn get_recommended_models_for_system() -> Result<Vec<String>, String> {
    log::info!("🎯 Getting recommended models for current system");
    
    // システム仕様に基づく推奨（簡易実装）
    let available_memory = 16384u64; // 実際の実装ではシステム情報取得
    
    let recommendations = if available_memory >= 32768 {
        // 32GB以上 - 高性能モデル推奨
        vec![
            "ollama:llama3.2:7b".to_string(),
            "ollama:mistral:7b".to_string(),
            "ollama:codellama:7b".to_string(),
        ]
    } else if available_memory >= 16384 {
        // 16GB以上 - バランス型推奨
        vec![
            "ollama:llama3.2:3b".to_string(),
            "ollama:llama3.2:7b".to_string(),
            "ollama:mistral:7b".to_string(),
        ]
    } else if available_memory >= 8192 {
        // 8GB以上 - 軽量モデル推奨
        vec![
            "ollama:llama3.2:1b".to_string(),
            "ollama:llama3.2:3b".to_string(),
        ]
    } else {
        // 8GB未満 - 最軽量のみ
        vec![
            "ollama:llama3.2:1b".to_string(),
        ]
    };
    
    log::info!("🎯 Generated {} system-specific recommendations", recommendations.len());
    Ok(recommendations)
}

#[tauri::command]
pub async fn estimate_download_time(
    downloader: State<'_, ModelDownloaderState>,
    model_id: String,
    connection_speed_mbps: f64, // Mbps
) -> Result<u64, String> {
    let downloader = downloader.lock().await;
    let models = downloader.get_downloadable_models();
    
    let model = models.iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Model not found: {}", model_id))?;
    
    let file_size_mb = match model.file_size {
        Some(bytes) => bytes as f64 / (1024.0 * 1024.0),
        None => return Err("Model file size unknown".to_string()),
    };
    
    // ダウンロード時間を秒単位で計算
    let download_time_seconds = (file_size_mb * 8.0) / connection_speed_mbps;
    
    log::info!("⏱️ Estimated download time for {}: {:.1}s", model_id, download_time_seconds);
    Ok(download_time_seconds as u64)
}

#[tauri::command]
pub async fn get_model_categories() -> Result<Vec<String>, String> {
    let categories = vec![
        "lightweight".to_string(),
        "balanced".to_string(),
        "high-quality".to_string(),
        "code".to_string(),
        "multilingual".to_string(),
    ];
    
    log::info!("📂 Retrieved {} model categories", categories.len());
    Ok(categories)
}

#[tauri::command]
pub async fn get_model_tags() -> Result<Vec<String>, String> {
    let tags = vec![
        "汎用".to_string(),
        "軽量".to_string(),
        "高速".to_string(),
        "バランス".to_string(),
        "推奨".to_string(),
        "高品質".to_string(),
        "多言語".to_string(),
        "効率的".to_string(),
        "コード生成".to_string(),
        "プログラミング".to_string(),
    ];
    
    log::info!("🏷️ Retrieved {} model tags", tags.len());
    Ok(tags)
}