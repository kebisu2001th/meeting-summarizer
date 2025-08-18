use crate::errors::{AppError, AppResult};
use crate::models::{LLMConfig, LLMProvider, Summary, SummaryStatus};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::time::timeout;

pub struct LLMService {
    config: LLMConfig,
    client: Client,
}

impl LLMService {
    pub fn new(config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    pub async fn summarize_text(&self, transcription_text: &str, transcription_id: String) -> AppResult<Summary> {
        let start_time = Instant::now();
        
        log::info!("🤖 Starting LLM summarization with {} model", self.config.model_name);

        // Create summary instance
        let mut summary = Summary::new(transcription_id, self.config.model_name.clone())
            .set_processing();

        // Generate prompt for Japanese summarization
        let prompt = self.create_japanese_summary_prompt(transcription_text);
        
        // Call LLM based on provider
        let llm_response = match self.config.provider {
            LLMProvider::Ollama => self.call_ollama(&prompt).await,
            LLMProvider::OpenAI => self.call_openai_compatible(&prompt).await,
            LLMProvider::GPT4All => self.call_gpt4all(&prompt).await,
            LLMProvider::LMStudio => self.call_lmstudio(&prompt).await,
            LLMProvider::Custom => self.call_custom_api(&prompt).await,
        };

        match llm_response {
            Ok(response_text) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                
                // Parse structured response
                let (summary_text, key_points, action_items) = self.parse_summary_response(&response_text);
                
                summary = summary
                    .with_content(summary_text, key_points, action_items)
                    .with_processing_time(processing_time);

                log::info!("✅ LLM summarization completed in {}ms", processing_time);
                Ok(summary)
            }
            Err(error) => {
                log::error!("❌ LLM summarization failed: {}", error);
                Ok(summary.with_error(error.to_string()))
            }
        }
    }

    fn create_japanese_summary_prompt(&self, text: &str) -> String {
        format!(
            r#"以下は会議や音声から書き起こしたテキストです。このテキストを分析して、以下の形式で日本語で要約してください：

## 要約
（全体的な内容を3-5文で簡潔にまとめてください）

## 重要ポイント
- （重要な議論点や決定事項を箇条書きで）
- （最大5-8個程度）

## アクションアイテム
- （具体的な行動項目があれば箇条書きで）
- （担当者や期限が分かる場合は含める）

---書き起こしテキスト---
{text}
---
上記のテキストを分析して、指定された形式で要約を作成してください。"#,
            text = text
        )
    }

    async fn call_ollama(&self, prompt: &str) -> AppResult<String> {
        let url = format!("{}/api/generate", self.config.base_url);
        
        let payload = json!({
            "model": self.config.model_name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens
            }
        });

        log::debug!("📡 Calling Ollama API: {}", url);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client.post(&url).json(&payload).send()
        ).await
        .map_err(|_| AppError::LLMTimeout {
            message: format!("Ollama request timed out after {} seconds", self.config.timeout_seconds),
        })?
        .map_err(|e| AppError::LLMConnectionError {
            message: format!("Failed to connect to Ollama: {}", e),
        })?;

        if !response.status().is_success() {
            return Err(AppError::LLMError {
                message: format!("Ollama API returned status: {}", response.status()),
            });
        }

        let json_response: Value = response.json().await
            .map_err(|e| AppError::LLMError {
                message: format!("Failed to parse Ollama response: {}", e),
            })?;

        json_response["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::LLMError {
                message: "Invalid response format from Ollama".to_string(),
            })
    }

    async fn call_openai_compatible(&self, prompt: &str) -> AppResult<String> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        
        let payload = json!({
            "model": self.config.model_name,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        log::debug!("📡 Calling OpenAI-compatible API: {}", url);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client.post(&url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
        ).await
        .map_err(|_| AppError::LLMTimeout {
            message: format!("OpenAI-compatible API request timed out after {} seconds", self.config.timeout_seconds),
        })?
        .map_err(|e| AppError::LLMConnectionError {
            message: format!("Failed to connect to OpenAI-compatible API: {}", e),
        })?;

        if !response.status().is_success() {
            return Err(AppError::LLMError {
                message: format!("OpenAI-compatible API returned status: {}", response.status()),
            });
        }

        let json_response: Value = response.json().await
            .map_err(|e| AppError::LLMError {
                message: format!("Failed to parse OpenAI-compatible response: {}", e),
            })?;

        json_response["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::LLMError {
                message: "Invalid response format from OpenAI-compatible API".to_string(),
            })
    }

    async fn call_gpt4all(&self, prompt: &str) -> AppResult<String> {
        // GPT4All API format (similar to OpenAI)
        self.call_openai_compatible(prompt).await
    }

    async fn call_lmstudio(&self, prompt: &str) -> AppResult<String> {
        // LM Studio uses OpenAI-compatible format
        self.call_openai_compatible(prompt).await
    }

    async fn call_custom_api(&self, prompt: &str) -> AppResult<String> {
        // Default to OpenAI-compatible format for custom APIs
        self.call_openai_compatible(prompt).await
    }

    fn parse_summary_response(&self, response: &str) -> (String, Vec<String>, Vec<String>) {
        let mut summary_text = String::new();
        let mut key_points = Vec::new();
        let mut action_items = Vec::new();
        
        let lines: Vec<&str> = response.lines().collect();
        let mut current_section = "summary";
        
        for line in lines {
            let trimmed_line = line.trim();
            
            // Section detection
            if trimmed_line.contains("## 要約") || trimmed_line.contains("要約") {
                current_section = "summary";
                continue;
            } else if trimmed_line.contains("## 重要ポイント") || trimmed_line.contains("重要ポイント") {
                current_section = "key_points";
                continue;
            } else if trimmed_line.contains("## アクションアイテム") || trimmed_line.contains("アクションアイテム") {
                current_section = "action_items";
                continue;
            }
            
            // Content parsing
            if !trimmed_line.is_empty() && !trimmed_line.starts_with("##") && !trimmed_line.starts_with("---") {
                match current_section {
                    "summary" => {
                        if !summary_text.is_empty() {
                            summary_text.push(' ');
                        }
                        summary_text.push_str(trimmed_line);
                    }
                    "key_points" => {
                        if trimmed_line.starts_with("- ") || trimmed_line.starts_with("・") {
                            key_points.push(trimmed_line.trim_start_matches("- ").trim_start_matches("・").to_string());
                        } else if !trimmed_line.starts_with("（") {
                            key_points.push(trimmed_line.to_string());
                        }
                    }
                    "action_items" => {
                        if trimmed_line.starts_with("- ") || trimmed_line.starts_with("・") {
                            action_items.push(trimmed_line.trim_start_matches("- ").trim_start_matches("・").to_string());
                        } else if !trimmed_line.starts_with("（") {
                            action_items.push(trimmed_line.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Fallback: if no structured content found, use entire response as summary
        if summary_text.is_empty() && key_points.is_empty() && action_items.is_empty() {
            summary_text = response.to_string();
        }
        
        (summary_text, key_points, action_items)
    }

    pub async fn check_connection(&self) -> AppResult<bool> {
        match self.config.provider {
            LLMProvider::Ollama => self.check_ollama_connection().await,
            _ => self.check_generic_connection().await,
        }
    }

    async fn check_ollama_connection(&self) -> AppResult<bool> {
        let url = format!("{}/api/tags", self.config.base_url);
        
        match timeout(
            Duration::from_secs(5), // Short timeout for connection check
            self.client.get(&url).send()
        ).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    }

    async fn check_generic_connection(&self) -> AppResult<bool> {
        let url = format!("{}/v1/models", self.config.base_url);
        
        match timeout(
            Duration::from_secs(5),
            self.client.get(&url).send()
        ).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    }

    pub fn get_config(&self) -> &LLMConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: LLMConfig) {
        self.config = new_config;
        // Recreate client with new timeout
        self.client = Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .build()
            .expect("Failed to recreate HTTP client");
    }
}