use crate::errors::AppResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadableModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub file_size: Option<u64>,
    pub download_command: String,
    pub requirements: ModelRequirements,
    pub tags: Vec<String>,
    pub popularity: u32, // ダウンロード数などの指標
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_memory_mb: u64,
    pub recommended_memory_mb: u64,
    pub disk_space_mb: u64,
    pub gpu_required: bool,
    pub supported_platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub status: DownloadStatus,
    pub progress_percent: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bps: Option<u64>, // bytes per second
    pub eta_seconds: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Installing,
    Completed,
    Failed,
    Cancelled,
}

pub struct ModelDownloader {
    client: Client,
    model_catalog: HashMap<String, DownloadableModel>,
}

impl ModelDownloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5分のタイムアウト
            .build()
            .expect("Failed to create HTTP client");

        let mut downloader = Self {
            client,
            model_catalog: HashMap::new(),
        };
        
        downloader.initialize_catalog();
        downloader
    }

    /// モデルカタログの初期化
    fn initialize_catalog(&mut self) {
        let models = vec![
            // Ollama モデル
            DownloadableModel {
                id: "ollama:llama3.2:1b".to_string(),
                name: "Llama 3.2 1B".to_string(),
                description: "軽量で高速な汎用言語モデル".to_string(),
                provider: "Ollama".to_string(),
                file_size: Some(1_300_000_000), // 約1.3GB
                download_command: "ollama pull llama3.2:1b".to_string(),
                requirements: ModelRequirements {
                    min_memory_mb: 2048,
                    recommended_memory_mb: 4096,
                    disk_space_mb: 1500,
                    gpu_required: false,
                    supported_platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                },
                tags: vec!["汎用".to_string(), "軽量".to_string(), "高速".to_string()],
                popularity: 95,
            },
            DownloadableModel {
                id: "ollama:llama3.2:3b".to_string(),
                name: "Llama 3.2 3B".to_string(),
                description: "バランスの取れた汎用言語モデル".to_string(),
                provider: "Ollama".to_string(),
                file_size: Some(2_000_000_000), // 約2GB
                download_command: "ollama pull llama3.2:3b".to_string(),
                requirements: ModelRequirements {
                    min_memory_mb: 4096,
                    recommended_memory_mb: 6144,
                    disk_space_mb: 2500,
                    gpu_required: false,
                    supported_platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                },
                tags: vec!["汎用".to_string(), "バランス".to_string(), "推奨".to_string()],
                popularity: 90,
            },
            DownloadableModel {
                id: "ollama:llama3.2:7b".to_string(),
                name: "Llama 3.2 7B".to_string(),
                description: "高品質な出力を生成する汎用言語モデル".to_string(),
                provider: "Ollama".to_string(),
                file_size: Some(4_100_000_000), // 約4.1GB
                download_command: "ollama pull llama3.2:7b".to_string(),
                requirements: ModelRequirements {
                    min_memory_mb: 8192,
                    recommended_memory_mb: 16384,
                    disk_space_mb: 5000,
                    gpu_required: false,
                    supported_platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                },
                tags: vec!["汎用".to_string(), "高品質".to_string()],
                popularity: 85,
            },
            DownloadableModel {
                id: "ollama:mistral:7b".to_string(),
                name: "Mistral 7B".to_string(),
                description: "効率的で多言語対応の言語モデル".to_string(),
                provider: "Ollama".to_string(),
                file_size: Some(4_200_000_000), // 約4.2GB
                download_command: "ollama pull mistral:7b".to_string(),
                requirements: ModelRequirements {
                    min_memory_mb: 8192,
                    recommended_memory_mb: 16384,
                    disk_space_mb: 5200,
                    gpu_required: false,
                    supported_platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                },
                tags: vec!["多言語".to_string(), "効率的".to_string()],
                popularity: 80,
            },
            DownloadableModel {
                id: "ollama:codellama:7b".to_string(),
                name: "Code Llama 7B".to_string(),
                description: "コード生成に特化した言語モデル".to_string(),
                provider: "Ollama".to_string(),
                file_size: Some(3_800_000_000), // 約3.8GB
                download_command: "ollama pull codellama:7b".to_string(),
                requirements: ModelRequirements {
                    min_memory_mb: 8192,
                    recommended_memory_mb: 16384,
                    disk_space_mb: 4500,
                    gpu_required: false,
                    supported_platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                },
                tags: vec!["コード生成".to_string(), "プログラミング".to_string()],
                popularity: 75,
            },
        ];

        for model in models {
            self.model_catalog.insert(model.id.clone(), model);
        }
    }

    /// 利用可能なモデル一覧を取得
    pub fn get_downloadable_models(&self) -> Vec<&DownloadableModel> {
        let mut models: Vec<&DownloadableModel> = self.model_catalog.values().collect();
        models.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        models
    }

    /// カテゴリ別モデル一覧を取得
    pub fn get_models_by_category(&self, category: &str) -> Vec<&DownloadableModel> {
        self.model_catalog
            .values()
            .filter(|model| {
                match category {
                    "lightweight" => model.requirements.min_memory_mb <= 4096,
                    "balanced" => model.requirements.min_memory_mb > 4096 && model.requirements.min_memory_mb <= 8192,
                    "high-quality" => model.requirements.min_memory_mb > 8192,
                    "code" => model.tags.contains(&"コード生成".to_string()),
                    "multilingual" => model.tags.contains(&"多言語".to_string()),
                    _ => true,
                }
            })
            .collect()
    }

    /// システム要件チェック
    pub fn check_system_requirements(&self, model_id: &str) -> Result<SystemCompatibility, String> {
        let model = self.model_catalog.get(model_id)
            .ok_or_else(|| format!("Model not found: {}", model_id))?;

        let available_memory = self.get_available_memory();
        let available_disk = self.get_available_disk_space();
        let platform = self.get_current_platform();

        let memory_ok = available_memory >= model.requirements.min_memory_mb;
        let disk_ok = available_disk >= model.requirements.disk_space_mb;
        let platform_ok = model.requirements.supported_platforms.contains(&platform);

        let compatibility = SystemCompatibility {
            model_id: model_id.to_string(),
            memory_compatible: memory_ok,
            disk_compatible: disk_ok,
            platform_compatible: platform_ok,
            available_memory_mb: available_memory,
            required_memory_mb: model.requirements.min_memory_mb,
            available_disk_mb: available_disk,
            required_disk_mb: model.requirements.disk_space_mb,
            warnings: self.generate_compatibility_warnings(model, available_memory, available_disk),
        };

        Ok(compatibility)
    }

    /// Ollamaモデルのダウンロード開始
    pub async fn start_download_ollama(&self, model_name: &str) -> AppResult<DownloadProgress> {
        log::info!("📥 Starting Ollama model download: {}", model_name);
        
        // Ollamaが利用可能かチェック
        self.check_ollama_availability().await?;
        
        // pullコマンドを実行（実際の実装では非同期プロセス実行）
        let progress = DownloadProgress {
            model_id: format!("ollama:{}", model_name),
            status: DownloadStatus::Downloading,
            progress_percent: 0.0,
            downloaded_bytes: 0,
            total_bytes: self.model_catalog.get(&format!("ollama:{}", model_name))
                .and_then(|m| m.file_size),
            speed_bps: None,
            eta_seconds: None,
            error_message: None,
        };
        
        // 実際の実装では、ここでコマンドを非同期実行し、進捗を追跡
        log::info!("🔄 Would execute: ollama pull {}", model_name);
        
        Ok(progress)
    }

    /// GPT4Allモデルのダウンロード情報取得
    pub fn get_gpt4all_download_info(&self, model_name: &str) -> Result<String, String> {
        let download_url = match model_name {
            "orca-mini-3b" => "https://gpt4all.io/models/orca-mini-3b-gguf2-q4_0.gguf",
            "vicuna-7b" => "https://gpt4all.io/models/vicuna-7b-q4_0.gguf",
            "falcon-7b" => "https://gpt4all.io/models/falcon-7b-q4_0.gguf",
            _ => return Err(format!("Unknown GPT4All model: {}", model_name)),
        };
        
        Ok(download_url.to_string())
    }

    /// システム互換性チェック用のヘルパー関数
    fn get_available_memory(&self) -> u64 {
        // 実際の実装ではシステムメトリクスを使用
        16384 // 16GB と仮定
    }

    fn get_available_disk_space(&self) -> u64 {
        // 実際の実装ではディスク使用量を取得
        100_000 // 100GB と仮定
    }

    fn get_current_platform(&self) -> String {
        #[cfg(target_os = "windows")]
        return "windows".to_string();
        
        #[cfg(target_os = "macos")]
        return "macos".to_string();
        
        #[cfg(target_os = "linux")]
        return "linux".to_string();
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "unknown".to_string();
    }

    fn generate_compatibility_warnings(&self, model: &DownloadableModel, available_memory: u64, available_disk: u64) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if available_memory < model.requirements.recommended_memory_mb {
            warnings.push(format!(
                "推奨メモリ容量 {}MB に対して利用可能メモリが {}MB です。パフォーマンスが低下する可能性があります。",
                model.requirements.recommended_memory_mb,
                available_memory
            ));
        }
        
        if available_disk < model.requirements.disk_space_mb * 2 {
            warnings.push(format!(
                "ディスク容量に余裕がありません。{}MB 以上の空き容量を確保することを推奨します。",
                model.requirements.disk_space_mb * 2
            ));
        }
        
        if model.requirements.gpu_required {
            warnings.push("このモデルはGPUアクセラレーションを推奨します。".to_string());
        }
        
        warnings
    }

    async fn check_ollama_availability(&self) -> AppResult<()> {
        match self.client.get("http://localhost:11434/api/version").send().await {
            Ok(response) if response.status().is_success() => Ok(()),
            _ => Err(crate::errors::AppError::LLMConnectionError {
                message: "Ollama is not running. Please start Ollama first.".to_string(),
            }),
        }
    }

    /// モデル検索機能
    pub fn search_models(&self, query: &str, tags: &[String]) -> Vec<&DownloadableModel> {
        let query_lower = query.to_lowercase();
        
        self.model_catalog
            .values()
            .filter(|model| {
                // テキスト検索
                let text_match = model.name.to_lowercase().contains(&query_lower)
                    || model.description.to_lowercase().contains(&query_lower);
                
                // タグ検索
                let tag_match = tags.is_empty() || tags.iter().any(|tag| model.tags.contains(tag));
                
                text_match && tag_match
            })
            .collect()
    }

    /// 人気モデルの推奨
    pub fn get_popular_models(&self, limit: usize) -> Vec<&DownloadableModel> {
        let mut models: Vec<&DownloadableModel> = self.model_catalog.values().collect();
        models.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        models.into_iter().take(limit).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCompatibility {
    pub model_id: String,
    pub memory_compatible: bool,
    pub disk_compatible: bool,
    pub platform_compatible: bool,
    pub available_memory_mb: u64,
    pub required_memory_mb: u64,
    pub available_disk_mb: u64,
    pub required_disk_mb: u64,
    pub warnings: Vec<String>,
}

impl SystemCompatibility {
    pub fn is_fully_compatible(&self) -> bool {
        self.memory_compatible && self.disk_compatible && self.platform_compatible
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}