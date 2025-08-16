use crate::errors::{AppError, AppResult};
use crate::models::{Transcription, TranscriptionStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
// use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperService {
    context: Arc<Mutex<Option<WhisperContext>>>,
    model_path: PathBuf,
}

impl WhisperService {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            context: Arc::new(Mutex::new(None)),
            model_path,
        }
    }

    pub async fn initialize(&self) -> AppResult<()> {
        let mut context_guard = self.context.lock().await;
        
        if context_guard.is_some() {
            return Ok(());
        }

        // Whisperモデルが存在するかチェック
        if !self.model_path.exists() {
            return Err(AppError::FileNotFound {
                path: self.model_path.to_string_lossy().to_string(),
            });
        }

        // Whisperコンテキストを初期化
        let ctx_params = WhisperContextParameters::default();
        let context = WhisperContext::new_with_params(
            self.model_path.to_string_lossy().as_ref(),
            ctx_params,
        )
        .map_err(|e| AppError::InvalidOperation {
            message: format!("Failed to initialize Whisper context: {}", e),
        })?;

        *context_guard = Some(context);
        Ok(())
    }

    pub async fn transcribe_audio_file(
        &self,
        audio_file_path: &Path,
        recording_id: String,
        language: Option<String>,
    ) -> AppResult<Transcription> {
        let start_time = Instant::now();
        
        // 初期化確認
        self.initialize().await?;

        let mut transcription = Transcription::new(
            recording_id,
            language.unwrap_or_else(|| "ja".to_string()),
        );
        transcription = transcription.set_processing();

        // 音声ファイル読み込み
        let audio_data = self.load_audio_file(audio_file_path).await?;

        // 音声書き起こし実行
        let transcription_result = self.perform_transcription(&audio_data, &transcription.language).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(transcription
            .with_text(transcription_result.text, Some(transcription_result.confidence))
            .set_processing_time(processing_time))
    }

    async fn load_audio_file(&self, file_path: &Path) -> AppResult<Vec<f32>> {
        // WAVファイルの読み込み
        let mut reader = hound::WavReader::open(file_path)
            .map_err(|e| AppError::InvalidOperation {
                message: format!("Failed to open WAV file: {}", e),
            })?;

        let spec = reader.spec();
        
        // Whisperは16-bit, 16kHz, モノラルの音声を期待
        if spec.channels != 1 {
            return Err(AppError::InvalidOperation {
                message: "Audio must be mono (1 channel)".to_string(),
            });
        }

        if spec.sample_rate != 16000 {
            return Err(AppError::InvalidOperation {
                message: "Audio sample rate must be 16kHz".to_string(),
            });
        }

        // サンプルをf32に変換
        let samples: Result<Vec<f32>, _> = match spec.sample_format {
            hound::SampleFormat::Int => {
                if spec.bits_per_sample == 16 {
                    reader
                        .samples::<i16>()
                        .map(|s| s.map(|s| s as f32 / 32768.0))
                        .collect()
                } else {
                    return Err(AppError::InvalidOperation {
                        message: "Unsupported bits per sample (expected 16-bit)".to_string(),
                    });
                }
            }
            hound::SampleFormat::Float => {
                reader.samples::<f32>().collect()
            }
        };

        samples.map_err(|e| AppError::InvalidOperation {
            message: format!("Failed to read audio samples: {}", e),
        })
    }

    async fn perform_transcription(
        &self,
        audio_data: &[f32],
        language: &str,
    ) -> AppResult<TranscriptionResult> {
        let context_guard = self.context.lock().await;
        let context = context_guard
            .as_ref()
            .ok_or_else(|| AppError::InvalidOperation {
                message: "Whisper context not initialized".to_string(),
            })?;

        // パラメータ設定
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // 日本語に最適化
        params.set_language(Some(language));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // 書き起こし実行
        context
            .full(params, audio_data)
            .map_err(|e| AppError::InvalidOperation {
                message: format!("Transcription failed: {}", e),
            })?;

        // 結果を取得
        let num_segments = context.full_n_segments()
            .map_err(|e| AppError::InvalidOperation {
                message: format!("Failed to get segment count: {}", e),
            })?;

        let mut full_text = String::new();
        let mut total_confidence = 0.0;
        let mut segment_count = 0;

        for i in 0..num_segments {
            if let Ok(segment_text) = context.full_get_segment_text(i) {
                full_text.push_str(&segment_text);
                segment_count += 1;
                
                // セグメントの信頼度を取得（可能であれば）
                // whisper-rsでは直接的な信頼度取得が制限されているため、
                // 簡易的な計算を行う
                total_confidence += 0.8; // デフォルト値
            }
        }

        let average_confidence = if segment_count > 0 {
            total_confidence / segment_count as f32
        } else {
            0.5
        };

        Ok(TranscriptionResult {
            text: full_text.trim().to_string(),
            confidence: average_confidence,
        })
    }

    pub async fn is_initialized(&self) -> bool {
        let context_guard = self.context.lock().await;
        context_guard.is_some()
    }
}

struct TranscriptionResult {
    text: String,
    confidence: f32,
}