use crate::services::{LLMModelManager, ModelInfo, ModelBenchmark};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type ModelManagerState = Arc<Mutex<LLMModelManager>>;

#[tauri::command]
pub async fn discover_available_models(
    model_manager: State<'_, ModelManagerState>,
) -> Result<Vec<ModelInfo>, String> {
    log::info!("🔍 Discovering available LLM models");
    
    let mut manager = model_manager.lock().await;
    match manager.discover_available_models().await {
        Ok(models) => {
            log::info!("✅ Successfully discovered {} models", models.len());
            Ok(models)
        }
        Err(e) => {
            log::error!("❌ Failed to discover models: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_cached_models(
    model_manager: State<'_, ModelManagerState>,
) -> Result<Vec<ModelInfo>, String> {
    let manager = model_manager.lock().await;
    let cached_models: Vec<ModelInfo> = manager.get_cached_models()
        .into_iter()
        .cloned()
        .collect();
    
    log::debug!("📋 Retrieved {} cached models", cached_models.len());
    Ok(cached_models)
}

#[tauri::command]
pub async fn benchmark_model(
    model_manager: State<'_, ModelManagerState>,
    model_id: String,
    test_prompt: Option<String>,
) -> Result<ModelBenchmark, String> {
    log::info!("🏁 Starting benchmark for model: {}", model_id);
    
    let prompt = test_prompt.unwrap_or_else(|| {
        "以下のテキストを要約してください：今日は天気が良く、散歩に出かけました。公園では桜が咲いていて、とても美しかったです。".to_string()
    });
    
    let mut manager = model_manager.lock().await;
    match manager.benchmark_model(&model_id, &prompt).await {
        Ok(benchmark) => {
            log::info!("✅ Benchmark completed for {}", model_id);
            Ok(benchmark)
        }
        Err(e) => {
            log::error!("❌ Benchmark failed for {}: {}", model_id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_cached_benchmarks(
    model_manager: State<'_, ModelManagerState>,
) -> Result<Vec<ModelBenchmark>, String> {
    let manager = model_manager.lock().await;
    let cached_benchmarks: Vec<ModelBenchmark> = manager.get_cached_benchmarks()
        .into_iter()
        .cloned()
        .collect();
    
    log::debug!("📊 Retrieved {} cached benchmarks", cached_benchmarks.len());
    Ok(cached_benchmarks)
}

#[tauri::command]
pub async fn get_recommended_models(
    model_manager: State<'_, ModelManagerState>,
    use_case: String,
) -> Result<Vec<String>, String> {
    let manager = model_manager.lock().await;
    let recommendations = manager.get_recommended_models(&use_case);
    
    log::debug!("🎯 Found {} recommendations for use case: {}", recommendations.len(), use_case);
    Ok(recommendations)
}

#[tauri::command]
pub async fn validate_model_availability(
    model_id: String,
) -> Result<bool, String> {
    log::debug!("🔍 Validating availability of model: {}", model_id);
    
    // モデルIDを分解
    let parts: Vec<&str> = model_id.split(':').collect();
    if parts.len() != 2 {
        return Ok(false);
    }
    
    let provider = parts[0];
    let model_name = parts[1];
    
    // プロバイダーごとの検証
    let is_available = match provider {
        "ollama" => validate_ollama_model(model_name).await,
        "gpt4all" => validate_gpt4all_model(model_name).await,
        "lmstudio" => validate_lmstudio_model(model_name).await,
        _ => false,
    };
    
    log::debug!("✓ Model {} availability: {}", model_id, is_available);
    Ok(is_available)
}

async fn validate_ollama_model(model_name: &str) -> bool {
    let client = reqwest::Client::new();
    match client.post("http://localhost:11434/api/show")
        .json(&serde_json::json!({"name": model_name}))
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

async fn validate_gpt4all_model(model_name: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get("http://localhost:4891/v1/models")
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(models) = json["data"].as_array() {
                    return models.iter().any(|m| {
                        m["id"].as_str().map_or(false, |id| id == model_name)
                    });
                }
            }
            false
        }
        _ => false,
    }
}

async fn validate_lmstudio_model(model_name: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get("http://localhost:1234/v1/models")
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(models) = json["data"].as_array() {
                    return models.iter().any(|m| {
                        m["id"].as_str().map_or(false, |id| id == model_name)
                    });
                }
            }
            false
        }
        _ => false,
    }
}

#[tauri::command]
pub async fn get_model_capabilities(
    model_id: String,
) -> Result<crate::services::ModelCapabilities, String> {
    log::debug!("🔍 Getting capabilities for model: {}", model_id);
    
    // モデル名に基づく機能判定（簡易版）
    let model_name = model_id.split(':').nth(1).unwrap_or("");
    let model_lower = model_name.to_lowercase();
    
    let capabilities = crate::services::ModelCapabilities {
        supports_summarization: true, // 全モデル対応と仮定
        supports_japanese: model_lower.contains("llama") || model_lower.contains("mistral"),
        supports_streaming: true, // 多くのモデルが対応
        supports_function_calling: model_lower.contains("llama") && model_lower.contains("3."),
        max_context_tokens: if model_lower.contains("3.2") { 128_000 } else { 4096 },
        recommended_use_cases: get_use_cases_for_model(&model_lower),
    };
    
    Ok(capabilities)
}

fn get_use_cases_for_model(model_name: &str) -> Vec<String> {
    let mut use_cases = Vec::new();
    
    if model_name.contains("3b") || model_name.contains("1b") {
        use_cases.push("速度重視".to_string());
        use_cases.push("軽量タスク".to_string());
    }
    
    if model_name.contains("7b") {
        use_cases.push("バランス型".to_string());
        use_cases.push("一般的な要約".to_string());
    }
    
    if model_name.contains("13b") || model_name.contains("70b") {
        use_cases.push("高品質".to_string());
        use_cases.push("複雑な分析".to_string());
    }
    
    if model_name.contains("code") {
        use_cases.push("コード生成".to_string());
        use_cases.push("技術文書".to_string());
    }
    
    if model_name.contains("instruct") || model_name.contains("chat") {
        use_cases.push("会話".to_string());
        use_cases.push("指示応答".to_string());
    }
    
    use_cases.push("テキスト要約".to_string()); // 全モデル共通
    
    use_cases
}

#[tauri::command]
pub async fn estimate_processing_time(
    model_id: String,
    text_length: u32,
) -> Result<f64, String> {
    log::debug!("⏱️ Estimating processing time for model: {} (text length: {})", model_id, text_length);
    
    // モデルサイズに基づく処理速度の推定
    let model_name = model_id.split(':').nth(1).unwrap_or("");
    let tokens_per_second = if model_name.contains("1b") {
        50.0 // 高速
    } else if model_name.contains("3b") {
        30.0 // 中速
    } else if model_name.contains("7b") {
        15.0 // 標準
    } else if model_name.contains("13b") {
        8.0 // やや低速
    } else if model_name.contains("70b") {
        2.0 // 低速
    } else {
        20.0 // デフォルト
    };
    
    // テキスト長からトークン数を推定（1トークン ≈ 4文字）
    let estimated_tokens = text_length as f64 / 4.0;
    let estimated_time = estimated_tokens / tokens_per_second;
    
    log::debug!("⏱️ Estimated processing time: {:.2}s", estimated_time);
    Ok(estimated_time)
}