use crate::errors::AppResult;
use crate::models::LLMConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub default_model: Option<String>,
    pub model_preferences: HashMap<String, ModelPreference>,
    pub use_case_defaults: HashMap<String, String>, // use_case -> model_id
    pub auto_switch_enabled: bool,
    pub performance_priority: PerformancePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreference {
    pub model_id: String,
    pub custom_config: Option<LLMConfig>,
    pub enabled: bool,
    pub priority: u8, // 1-10, higher is better
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformancePriority {
    Speed,    // 速度優先
    Quality,  // 品質優先  
    Balance,  // バランス
    Memory,   // メモリ効率
}

impl Default for ModelSettings {
    fn default() -> Self {
        let mut use_case_defaults = HashMap::new();
        use_case_defaults.insert("summarization".to_string(), "ollama:llama3.2:3b".to_string());
        use_case_defaults.insert("japanese".to_string(), "ollama:llama3.2:3b".to_string());
        use_case_defaults.insert("speed".to_string(), "ollama:llama3.2:1b".to_string());
        use_case_defaults.insert("quality".to_string(), "ollama:llama3.2:7b".to_string());

        Self {
            default_model: Some("ollama:llama3.2:3b".to_string()),
            model_preferences: HashMap::new(),
            use_case_defaults,
            auto_switch_enabled: false,
            performance_priority: PerformancePriority::Balance,
        }
    }
}

impl ModelSettings {
    /// 設定ファイルから読み込み
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            log::info!("📄 Model settings file not found, using defaults");
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(path_ref).await?;
        let settings: ModelSettings = serde_json::from_str(&content)?;
        
        log::info!("✅ Model settings loaded from: {:?}", path_ref);
        Ok(settings)
    }
    
    /// 設定ファイルに保存
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> AppResult<()> {
        let path_ref = path.as_ref();
        
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path_ref, content).await?;
        
        log::info!("💾 Model settings saved to: {:?}", path_ref);
        Ok(())
    }
    
    /// デフォルトモデルを設定
    pub fn set_default_model(&mut self, model_id: String) {
        self.default_model = Some(model_id);
    }
    
    /// 用途別デフォルトモデルを設定
    pub fn set_use_case_default(&mut self, use_case: String, model_id: String) {
        self.use_case_defaults.insert(use_case, model_id);
    }
    
    /// モデル設定を追加/更新
    pub fn set_model_preference(&mut self, model_id: String, preference: ModelPreference) {
        self.model_preferences.insert(model_id, preference);
    }
    
    /// 指定された用途に最適なモデルを取得
    pub fn get_optimal_model(&self, use_case: &str) -> Option<String> {
        // 1. 用途別デフォルトをチェック
        if let Some(model_id) = self.use_case_defaults.get(use_case) {
            if let Some(pref) = self.model_preferences.get(model_id) {
                if pref.enabled {
                    return Some(model_id.clone());
                }
            } else {
                return Some(model_id.clone());
            }
        }
        
        // 2. パフォーマンス優先度に基づく選択
        let candidates = match self.performance_priority {
            PerformancePriority::Speed => vec!["ollama:llama3.2:1b", "ollama:llama3.2:3b"],
            PerformancePriority::Quality => vec!["ollama:llama3.2:7b", "ollama:llama3.2:13b"],
            PerformancePriority::Balance => vec!["ollama:llama3.2:3b", "ollama:llama3.2:7b"],
            PerformancePriority::Memory => vec!["ollama:llama3.2:1b", "gpt4all:orca-mini"],
        };
        
        for candidate in candidates {
            if let Some(pref) = self.model_preferences.get(candidate) {
                if pref.enabled {
                    return Some(candidate.to_string());
                }
            }
        }
        
        // 3. デフォルトモデルを返す
        self.default_model.clone()
    }
    
    /// 有効なモデル一覧を取得（優先度順）
    pub fn get_enabled_models_by_priority(&self) -> Vec<String> {
        let mut models: Vec<(String, u8)> = self.model_preferences
            .iter()
            .filter(|(_, pref)| pref.enabled)
            .map(|(id, pref)| (id.clone(), pref.priority))
            .collect();
        
        // 優先度で降順ソート
        models.sort_by(|a, b| b.1.cmp(&a.1));
        
        models.into_iter().map(|(id, _)| id).collect()
    }
    
    /// モデル設定のバリデーション
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        // デフォルトモデルの検証
        if let Some(default_model) = &self.default_model {
            if !default_model.contains(':') {
                errors.push(format!("Invalid default model format: {}", default_model));
            }
        }
        
        // 用途別デフォルトの検証
        for (use_case, model_id) in &self.use_case_defaults {
            if !model_id.contains(':') {
                errors.push(format!("Invalid model format for use case '{}': {}", use_case, model_id));
            }
        }
        
        // モデル設定の検証
        for (model_id, preference) in &self.model_preferences {
            if !model_id.contains(':') {
                errors.push(format!("Invalid model ID format: {}", model_id));
            }
            
            if preference.priority > 10 {
                errors.push(format!("Invalid priority for model '{}': {} (must be 1-10)", model_id, preference.priority));
            }
        }
        
        errors
    }
    
    /// 設定のリセット
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }
    
    /// モデル設定をマージ（既存設定を保持しつつ新規追加）
    pub fn merge_with(&mut self, other: ModelSettings) {
        // デフォルトモデルの更新
        if other.default_model.is_some() {
            self.default_model = other.default_model;
        }
        
        // 用途別デフォルトをマージ
        for (use_case, model_id) in other.use_case_defaults {
            self.use_case_defaults.insert(use_case, model_id);
        }
        
        // モデル設定をマージ
        for (model_id, preference) in other.model_preferences {
            self.model_preferences.insert(model_id, preference);
        }
        
        // 設定項目を更新
        self.auto_switch_enabled = other.auto_switch_enabled;
        self.performance_priority = other.performance_priority;
    }
}

#[derive(Debug, Clone)]
pub struct ModelSettingsManager {
    settings: ModelSettings,
    settings_path: std::path::PathBuf,
}

impl ModelSettingsManager {
    pub fn new(settings_path: std::path::PathBuf) -> Self {
        Self {
            settings: ModelSettings::default(),
            settings_path,
        }
    }
    
    /// 設定を読み込み
    pub async fn load_settings(&mut self) -> AppResult<()> {
        self.settings = ModelSettings::load_from_file(&self.settings_path).await?;
        Ok(())
    }
    
    /// 設定を保存
    pub async fn save_settings(&self) -> AppResult<()> {
        self.settings.save_to_file(&self.settings_path).await
    }
    
    /// 現在の設定を取得
    pub fn get_settings(&self) -> &ModelSettings {
        &self.settings
    }
    
    /// 設定を更新
    pub fn update_settings<F>(&mut self, updater: F) 
    where
        F: FnOnce(&mut ModelSettings),
    {
        updater(&mut self.settings);
    }
    
    /// 最適なモデルを取得
    pub fn get_optimal_model(&self, use_case: &str) -> Option<String> {
        self.settings.get_optimal_model(use_case)
    }
    
    /// 設定の自動保存（変更検出付き）
    pub async fn auto_save_if_changed(&mut self, new_settings: ModelSettings) -> AppResult<bool> {
        let current_json = serde_json::to_string(&self.settings)?;
        let new_json = serde_json::to_string(&new_settings)?;
        
        if current_json != new_json {
            self.settings = new_settings;
            self.save_settings().await?;
            log::info!("🔄 Model settings auto-saved due to changes");
            Ok(true)
        } else {
            Ok(false)
        }
    }
}