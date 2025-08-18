use crate::errors::{AppError, AppResult};
use crate::models::{LLMConfig, LLMProvider};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: LLMProvider,
    pub description: String,
    pub parameter_count: Option<String>, // e.g., "7B", "13B", "70B"
    pub quantization: Option<String>,    // e.g., "Q4_0", "Q8_0"
    pub memory_required: Option<u64>,    // MB
    pub context_length: Option<u32>,
    pub is_available: bool,
    pub download_url: Option<String>,
    pub file_size: Option<u64>, // bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBenchmark {
    pub model_id: String,
    pub inference_speed: Option<f64>, // tokens per second
    pub memory_usage: Option<u64>,    // MB
    pub quality_score: Option<f32>,   // 0.0 - 1.0
    pub last_benchmarked: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_summarization: bool,
    pub supports_japanese: bool,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub max_context_tokens: u32,
    pub recommended_use_cases: Vec<String>,
}

pub struct LLMModelManager {
    client: Client,
    models_cache: HashMap<String, ModelInfo>,
    benchmarks_cache: HashMap<String, ModelBenchmark>,
}

impl LLMModelManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            models_cache: HashMap::new(),
            benchmarks_cache: HashMap::new(),
        }
    }

    /// 各プロバイダーから利用可能なモデル一覧を取得
    pub async fn discover_available_models(&mut self) -> AppResult<Vec<ModelInfo>> {
        log::info!("🔍 Discovering available LLM models across providers");
        
        let mut all_models = Vec::new();
        
        // Ollama models
        if let Ok(ollama_models) = self.discover_ollama_models().await {
            all_models.extend(ollama_models);
        }
        
        // GPT4All models
        if let Ok(gpt4all_models) = self.discover_gpt4all_models().await {
            all_models.extend(gpt4all_models);
        }
        
        // LM Studio models
        if let Ok(lmstudio_models) = self.discover_lmstudio_models().await {
            all_models.extend(lmstudio_models);
        }
        
        // Update cache
        for model in &all_models {
            self.models_cache.insert(model.id.clone(), model.clone());
        }
        
        log::info!("✅ Discovered {} models across all providers", all_models.len());
        Ok(all_models)
    }

    /// Ollama で利用可能なモデルを検出
    async fn discover_ollama_models(&self) -> AppResult<Vec<ModelInfo>> {
        log::debug!("🔍 Checking Ollama models at localhost:11434");
        
        match self.client.get("http://localhost:11434/api/tags").send().await {
            Ok(response) if response.status().is_success() => {
                let ollama_response: serde_json::Value = response.json().await?;
                let empty_models = vec![];
                let models = ollama_response["models"].as_array().unwrap_or(&empty_models);
                
                let mut model_infos = Vec::new();
                for model in models {
                    if let Some(name) = model["name"].as_str() {
                        let model_info = ModelInfo {
                            id: format!("ollama:{}", name),
                            name: name.to_string(),
                            provider: LLMProvider::Ollama,
                            description: format!("Ollama model: {}", name),
                            parameter_count: self.extract_parameter_count(name),
                            quantization: self.extract_quantization(name),
                            memory_required: self.estimate_memory_requirement(name),
                            context_length: self.get_context_length_for_model(name),
                            is_available: true,
                            download_url: None,
                            file_size: model["size"].as_u64(),
                        };
                        model_infos.push(model_info);
                    }
                }
                
                log::debug!("✅ Found {} Ollama models", model_infos.len());
                Ok(model_infos)
            }
            _ => {
                log::debug!("⚠️ Ollama not available at localhost:11434");
                Ok(Vec::new())
            }
        }
    }

    /// GPT4All で利用可能なモデルを検出
    async fn discover_gpt4all_models(&self) -> AppResult<Vec<ModelInfo>> {
        log::debug!("🔍 Checking GPT4All models at localhost:4891");
        
        // GPT4All API チェック
        match self.client.get("http://localhost:4891/v1/models").send().await {
            Ok(response) if response.status().is_success() => {
                let gpt4all_response: serde_json::Value = response.json().await?;
                let empty_models = vec![];
                let models = gpt4all_response["data"].as_array().unwrap_or(&empty_models);
                
                let mut model_infos = Vec::new();
                for model in models {
                    if let Some(id) = model["id"].as_str() {
                        let model_info = ModelInfo {
                            id: format!("gpt4all:{}", id),
                            name: id.to_string(),
                            provider: LLMProvider::GPT4All,
                            description: format!("GPT4All model: {}", id),
                            parameter_count: self.extract_parameter_count(id),
                            quantization: Some("Q4_0".to_string()), // GPT4All default
                            memory_required: self.estimate_memory_requirement(id),
                            context_length: Some(2048), // GPT4All typical context
                            is_available: true,
                            download_url: None,
                            file_size: None,
                        };
                        model_infos.push(model_info);
                    }
                }
                
                log::debug!("✅ Found {} GPT4All models", model_infos.len());
                Ok(model_infos)
            }
            _ => {
                log::debug!("⚠️ GPT4All not available at localhost:4891");
                Ok(Vec::new())
            }
        }
    }

    /// LM Studio で利用可能なモデルを検出
    async fn discover_lmstudio_models(&self) -> AppResult<Vec<ModelInfo>> {
        log::debug!("🔍 Checking LM Studio models at localhost:1234");
        
        match self.client.get("http://localhost:1234/v1/models").send().await {
            Ok(response) if response.status().is_success() => {
                let lmstudio_response: serde_json::Value = response.json().await?;
                let empty_models = vec![];
                let models = lmstudio_response["data"].as_array().unwrap_or(&empty_models);
                
                let mut model_infos = Vec::new();
                for model in models {
                    if let Some(id) = model["id"].as_str() {
                        let model_info = ModelInfo {
                            id: format!("lmstudio:{}", id),
                            name: id.to_string(),
                            provider: LLMProvider::LMStudio,
                            description: format!("LM Studio model: {}", id),
                            parameter_count: self.extract_parameter_count(id),
                            quantization: self.extract_quantization(id),
                            memory_required: self.estimate_memory_requirement(id),
                            context_length: Some(4096), // LM Studio typical context
                            is_available: true,
                            download_url: None,
                            file_size: None,
                        };
                        model_infos.push(model_info);
                    }
                }
                
                log::debug!("✅ Found {} LM Studio models", model_infos.len());
                Ok(model_infos)
            }
            _ => {
                log::debug!("⚠️ LM Studio not available at localhost:1234");
                Ok(Vec::new())
            }
        }
    }

    /// モデル名からパラメーター数を抽出
    fn extract_parameter_count(&self, model_name: &str) -> Option<String> {
        let name_lower = model_name.to_lowercase();
        
        if name_lower.contains("70b") || name_lower.contains("70-b") {
            Some("70B".to_string())
        } else if name_lower.contains("34b") || name_lower.contains("34-b") {
            Some("34B".to_string())
        } else if name_lower.contains("13b") || name_lower.contains("13-b") {
            Some("13B".to_string())
        } else if name_lower.contains("7b") || name_lower.contains("7-b") {
            Some("7B".to_string())
        } else if name_lower.contains("3b") || name_lower.contains("3-b") {
            Some("3B".to_string())
        } else if name_lower.contains("1b") || name_lower.contains("1-b") {
            Some("1B".to_string())
        } else {
            None
        }
    }

    /// モデル名から量子化情報を抽出
    fn extract_quantization(&self, model_name: &str) -> Option<String> {
        let name_upper = model_name.to_uppercase();
        
        if name_upper.contains("Q8_0") {
            Some("Q8_0".to_string())
        } else if name_upper.contains("Q5_K_M") {
            Some("Q5_K_M".to_string())
        } else if name_upper.contains("Q4_K_M") {
            Some("Q4_K_M".to_string())
        } else if name_upper.contains("Q4_0") {
            Some("Q4_0".to_string())
        } else if name_upper.contains("Q2_K") {
            Some("Q2_K".to_string())
        } else {
            None
        }
    }

    /// モデルの推定メモリ使用量を計算
    fn estimate_memory_requirement(&self, model_name: &str) -> Option<u64> {
        let param_count = self.extract_parameter_count(model_name)?;
        let quantization = self.extract_quantization(model_name);
        
        let base_memory = match param_count.as_str() {
            "70B" => 140_000, // 140GB for FP16
            "34B" => 68_000,  // 68GB for FP16
            "13B" => 26_000,  // 26GB for FP16
            "7B" => 14_000,   // 14GB for FP16
            "3B" => 6_000,    // 6GB for FP16
            "1B" => 2_000,    // 2GB for FP16
            _ => return None,
        };
        
        // 量子化による削減率を適用
        let memory_mb = match quantization.as_deref() {
            Some("Q2_K") => (base_memory as f64 * 0.3) as u64, // 約30%
            Some("Q4_0") | Some("Q4_K_M") => (base_memory as f64 * 0.5) as u64, // 約50%
            Some("Q5_K_M") => (base_memory as f64 * 0.65) as u64, // 約65%
            Some("Q8_0") => (base_memory as f64 * 0.8) as u64,  // 約80%
            _ => base_memory, // FP16 as default
        };
        
        Some(memory_mb)
    }

    /// モデルのコンテキスト長を取得
    fn get_context_length_for_model(&self, model_name: &str) -> Option<u32> {
        let name_lower = model_name.to_lowercase();
        
        // 一般的なモデルのコンテキスト長
        if name_lower.contains("llama") && name_lower.contains("3.2") {
            Some(128_000) // Llama 3.2 extended context
        } else if name_lower.contains("llama") {
            Some(4096) // Llama 2 standard context
        } else if name_lower.contains("mistral") {
            Some(8192) // Mistral context
        } else if name_lower.contains("codellama") {
            Some(16_384) // CodeLlama context
        } else {
            Some(2048) // Default context
        }
    }

    /// モデルのベンチマークを実行
    pub async fn benchmark_model(&mut self, model_id: &str, test_prompt: &str) -> AppResult<ModelBenchmark> {
        log::info!("🏁 Running benchmark for model: {}", model_id);
        
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage().unwrap_or(0);
        
        // テストプロンプトで推論実行
        let config = self.create_config_for_model(model_id)?;
        let test_response = self.run_inference_test(&config, test_prompt).await?;
        
        let inference_time = start_time.elapsed();
        let end_memory = self.get_memory_usage().unwrap_or(0);
        
        // トークン数を推定（簡易計算）
        let estimated_tokens = test_response.len() / 4; // 概算
        let tokens_per_second = estimated_tokens as f64 / inference_time.as_secs_f64();
        
        let benchmark = ModelBenchmark {
            model_id: model_id.to_string(),
            inference_speed: Some(tokens_per_second),
            memory_usage: Some(end_memory.saturating_sub(start_memory)),
            quality_score: None, // 品質評価は別途実装
            last_benchmarked: chrono::Utc::now(),
        };
        
        // キャッシュに保存
        self.benchmarks_cache.insert(model_id.to_string(), benchmark.clone());
        
        log::info!("✅ Benchmark completed for {}: {:.2} tokens/sec", model_id, tokens_per_second);
        Ok(benchmark)
    }

    /// モデルに対応するConfigを生成
    fn create_config_for_model(&self, model_id: &str) -> AppResult<LLMConfig> {
        let parts: Vec<&str> = model_id.split(':').collect();
        if parts.len() != 2 {
            return Err(AppError::LLMConfigError { 
                message: format!("Invalid model ID format: {}", model_id) 
            });
        }
        
        let provider_str = parts[0];
        let model_name = parts[1];
        
        let provider = match provider_str {
            "ollama" => LLMProvider::Ollama,
            "gpt4all" => LLMProvider::GPT4All,
            "lmstudio" => LLMProvider::LMStudio,
            _ => return Err(AppError::LLMConfigError { 
                message: format!("Unsupported provider: {}", provider_str) 
            }),
        };
        
        let base_url = match provider {
            LLMProvider::Ollama => "http://localhost:11434",
            LLMProvider::GPT4All => "http://localhost:4891",
            LLMProvider::LMStudio => "http://localhost:1234",
            _ => return Err(AppError::LLMConfigError { 
                message: "Unsupported provider".to_string() 
            }),
        };
        
        Ok(LLMConfig {
            provider,
            base_url: base_url.to_string(),
            model_name: model_name.to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 120,
        })
    }

    /// 推論テストを実行
    async fn run_inference_test(&self, config: &LLMConfig, test_prompt: &str) -> AppResult<String> {
        let payload = match config.provider {
            LLMProvider::Ollama => {
                serde_json::json!({
                    "model": config.model_name,
                    "prompt": test_prompt,
                    "stream": false
                })
            }
            LLMProvider::GPT4All | LLMProvider::LMStudio => {
                serde_json::json!({
                    "model": config.model_name,
                    "messages": [{"role": "user", "content": test_prompt}],
                    "stream": false,
                    "max_tokens": 100
                })
            }
            _ => return Err(AppError::LLMConfigError { 
                message: "Unsupported provider for benchmarking".to_string() 
            }),
        };
        
        let endpoint = match config.provider {
            LLMProvider::Ollama => format!("{}/api/generate", config.base_url),
            LLMProvider::GPT4All | LLMProvider::LMStudio => format!("{}/v1/chat/completions", config.base_url),
            _ => return Err(AppError::LLMConfigError { 
                message: "Unsupported provider endpoint".to_string() 
            }),
        };
        
        let response = self.client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(AppError::LLMConnectionError { 
                message: format!("HTTP error: {}", response.status()) 
            });
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        let content = match config.provider {
            LLMProvider::Ollama => {
                response_json["response"].as_str().unwrap_or("").to_string()
            }
            LLMProvider::GPT4All | LLMProvider::LMStudio => {
                response_json["choices"][0]["message"]["content"]
                    .as_str().unwrap_or("").to_string()
            }
            _ => String::new(),
        };
        
        Ok(content)
    }

    /// 現在のメモリ使用量を取得（簡易実装）
    fn get_memory_usage(&self) -> Option<u64> {
        // 実際の実装では system metrics を使用
        // ここでは概算値を返す
        Some(0)
    }

    /// 推奨モデルを取得（用途別）
    pub fn get_recommended_models(&self, use_case: &str) -> Vec<String> {
        match use_case {
            "summarization" | "テキスト要約" => vec![
                "ollama:llama3.2:3b".to_string(),
                "ollama:mistral:7b".to_string(),
            ],
            "japanese" | "会議記録" => vec![
                "ollama:llama3.2:3b".to_string(),
                "gpt4all:orca-mini".to_string(),
            ],
            "speed" | "高速処理" | "速度重視" => vec![
                "ollama:llama3.2:1b".to_string(),
                "gpt4all:orca-mini".to_string(),
            ],
            "quality" | "高品質分析" | "高品質" => vec![
                "ollama:llama3.2:7b".to_string(),
                "lmstudio:mistral-7b-instruct".to_string(),
            ],
            _ => vec![
                "ollama:llama3.2:3b".to_string(), // デフォルト推奨
            ],
        }
    }

    /// キャッシュされたモデル情報を取得
    pub fn get_cached_models(&self) -> Vec<&ModelInfo> {
        self.models_cache.values().collect()
    }

    /// キャッシュされたベンチマーク情報を取得
    pub fn get_cached_benchmarks(&self) -> Vec<&ModelBenchmark> {
        self.benchmarks_cache.values().collect()
    }
}

impl Default for LLMModelManager {
    fn default() -> Self {
        Self::new()
    }
}