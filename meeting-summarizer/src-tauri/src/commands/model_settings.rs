use crate::services::{ModelSettings, ModelPreference, PerformancePriority, ModelSettingsManager};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type ModelSettingsState = Arc<Mutex<ModelSettingsManager>>;

#[tauri::command]
pub async fn get_model_settings(
    settings_manager: State<'_, ModelSettingsState>,
) -> Result<ModelSettings, String> {
    let manager = settings_manager.lock().await;
    Ok(manager.get_settings().clone())
}

#[tauri::command]
pub async fn save_model_settings(
    settings_manager: State<'_, ModelSettingsState>,
    new_settings: ModelSettings,
) -> Result<(), String> {
    log::info!("💾 Saving model settings");
    
    let mut manager = settings_manager.lock().await;
    let changed = manager.auto_save_if_changed(new_settings).await
        .map_err(|e| e.to_string())?;
    
    if changed {
        log::info!("✅ Model settings saved successfully");
    } else {
        log::debug!("📋 No changes detected in model settings");
    }
    
    Ok(())
}

#[tauri::command]
pub async fn set_default_model(
    settings_manager: State<'_, ModelSettingsState>,
    model_id: String,
) -> Result<(), String> {
    log::info!("🎯 Setting default model to: {}", model_id);
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.set_default_model(model_id.clone());
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Default model updated to: {}", model_id);
    
    Ok(())
}

#[tauri::command]
pub async fn set_use_case_default(
    settings_manager: State<'_, ModelSettingsState>,
    use_case: String,
    model_id: String,
) -> Result<(), String> {
    log::info!("🎯 Setting default model for '{}' to: {}", use_case, model_id);
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.set_use_case_default(use_case.clone(), model_id.clone());
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Use case default updated: {} -> {}", use_case, model_id);
    
    Ok(())
}

#[tauri::command]
pub async fn add_model_preference(
    settings_manager: State<'_, ModelSettingsState>,
    model_id: String,
    enabled: bool,
    priority: u8,
    notes: Option<String>,
) -> Result<(), String> {
    log::info!("⚙️ Adding model preference: {} (enabled: {}, priority: {})", model_id, enabled, priority);
    
    if priority > 10 {
        return Err("Priority must be between 1 and 10".to_string());
    }
    
    let preference = ModelPreference {
        model_id: model_id.clone(),
        custom_config: None,
        enabled,
        priority,
        notes,
    };
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.set_model_preference(model_id.clone(), preference);
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Model preference added for: {}", model_id);
    
    Ok(())
}

#[tauri::command]
pub async fn remove_model_preference(
    settings_manager: State<'_, ModelSettingsState>,
    model_id: String,
) -> Result<(), String> {
    log::info!("🗑️ Removing model preference: {}", model_id);
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.model_preferences.remove(&model_id);
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Model preference removed for: {}", model_id);
    
    Ok(())
}

#[tauri::command]
pub async fn set_performance_priority(
    settings_manager: State<'_, ModelSettingsState>,
    priority: String,
) -> Result<(), String> {
    log::info!("⚡ Setting performance priority to: {}", priority);
    
    let priority_enum = match priority.as_str() {
        "speed" => PerformancePriority::Speed,
        "quality" => PerformancePriority::Quality,
        "balance" => PerformancePriority::Balance,
        "memory" => PerformancePriority::Memory,
        _ => return Err("Invalid performance priority".to_string()),
    };
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.performance_priority = priority_enum;
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Performance priority updated to: {}", priority);
    
    Ok(())
}

#[tauri::command]
pub async fn set_auto_switch_enabled(
    settings_manager: State<'_, ModelSettingsState>,
    enabled: bool,
) -> Result<(), String> {
    log::info!("🔄 Setting auto-switch to: {}", enabled);
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.auto_switch_enabled = enabled;
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Auto-switch updated to: {}", enabled);
    
    Ok(())
}

#[tauri::command]
pub async fn get_optimal_model_for_use_case(
    settings_manager: State<'_, ModelSettingsState>,
    use_case: String,
) -> Result<Option<String>, String> {
    let manager = settings_manager.lock().await;
    let optimal_model = manager.get_optimal_model(&use_case);
    
    log::debug!("🎯 Optimal model for '{}': {:?}", use_case, optimal_model);
    Ok(optimal_model)
}

#[tauri::command]
pub async fn get_enabled_models_by_priority(
    settings_manager: State<'_, ModelSettingsState>,
) -> Result<Vec<String>, String> {
    let manager = settings_manager.lock().await;
    let models = manager.get_settings().get_enabled_models_by_priority();
    
    log::debug!("📋 Enabled models by priority: {} models", models.len());
    Ok(models)
}

#[tauri::command]
pub async fn validate_model_settings(
    settings_manager: State<'_, ModelSettingsState>,
) -> Result<Vec<String>, String> {
    let manager = settings_manager.lock().await;
    let errors = manager.get_settings().validate();
    
    if errors.is_empty() {
        log::info!("✅ Model settings validation passed");
    } else {
        log::warn!("⚠️ Model settings validation found {} issues", errors.len());
    }
    
    Ok(errors)
}

#[tauri::command]
pub async fn reset_model_settings(
    settings_manager: State<'_, ModelSettingsState>,
) -> Result<(), String> {
    log::info!("🔄 Resetting model settings to defaults");
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        settings.reset_to_defaults();
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Model settings reset to defaults");
    
    Ok(())
}

#[tauri::command]
pub async fn export_model_settings(
    settings_manager: State<'_, ModelSettingsState>,
) -> Result<String, String> {
    let manager = settings_manager.lock().await;
    let settings_json = serde_json::to_string_pretty(manager.get_settings())
        .map_err(|e| e.to_string())?;
    
    log::info!("📤 Model settings exported");
    Ok(settings_json)
}

#[tauri::command]
pub async fn import_model_settings(
    settings_manager: State<'_, ModelSettingsState>,
    settings_json: String,
    merge_with_existing: bool,
) -> Result<(), String> {
    log::info!("📥 Importing model settings (merge: {})", merge_with_existing);
    
    let imported_settings: ModelSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Invalid settings format: {}", e))?;
    
    // 設定のバリデーション
    let validation_errors = imported_settings.validate();
    if !validation_errors.is_empty() {
        return Err(format!("Settings validation failed: {:?}", validation_errors));
    }
    
    let mut manager = settings_manager.lock().await;
    manager.update_settings(|settings| {
        if merge_with_existing {
            settings.merge_with(imported_settings);
        } else {
            *settings = imported_settings;
        }
    });
    
    manager.save_settings().await.map_err(|e| e.to_string())?;
    log::info!("✅ Model settings imported successfully");
    
    Ok(())
}

#[tauri::command]
pub async fn get_performance_recommendations(
    use_case: String,
    text_length: u32,
    available_memory_mb: Option<u32>,
    speed_priority: f32, // 0.0-1.0, higher = prioritize speed
) -> Result<Vec<String>, String> {
    log::debug!("🎯 Getting performance recommendations for: {} (length: {}, speed_priority: {})", 
               use_case, text_length, speed_priority);
    
    let mut recommendations = Vec::new();
    
    // メモリ制約の考慮
    let memory_limit = available_memory_mb.unwrap_or(8192); // デフォルト8GB
    
    // テキスト長に基づく推奨
    if text_length < 1000 {
        // 短いテキスト - 高速モデル推奨
        if speed_priority > 0.7 {
            recommendations.extend(vec![
                "ollama:llama3.2:1b".to_string(),
                "gpt4all:orca-mini".to_string(),
            ]);
        }
        recommendations.push("ollama:llama3.2:3b".to_string());
    } else if text_length < 10000 {
        // 中程度のテキスト - バランス型推奨
        recommendations.extend(vec![
            "ollama:llama3.2:3b".to_string(),
            "ollama:mistral:7b".to_string(),
        ]);
        
        if speed_priority < 0.5 && memory_limit >= 16000 {
            recommendations.push("ollama:llama3.2:7b".to_string());
        }
    } else {
        // 長いテキスト - 高品質モデル推奨
        if memory_limit >= 16000 {
            recommendations.push("ollama:llama3.2:7b".to_string());
        }
        recommendations.extend(vec![
            "ollama:llama3.2:3b".to_string(),
            "ollama:mistral:7b".to_string(),
        ]);
    }
    
    // 用途別フィルタ
    match use_case.as_str() {
        "japanese" => {
            // 日本語対応を優先
            recommendations.retain(|model| {
                model.contains("llama") || model.contains("mistral")
            });
        }
        "code" => {
            // コード関連を優先  
            recommendations.insert(0, "ollama:codellama:7b".to_string());
        }
        _ => {}
    }
    
    // 重複除去と最大5個に制限
    recommendations.dedup();
    recommendations.truncate(5);
    
    log::debug!("🎯 Generated {} recommendations", recommendations.len());
    Ok(recommendations)
}