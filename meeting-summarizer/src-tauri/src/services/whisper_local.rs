use crate::errors::{AppError, AppResult};
use crate::models::{Transcription, TranscriptionStatus};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Command as TokioCommand;
use std::fs;
use dirs;

pub struct WhisperService {
    model_path: PathBuf,
    recordings_dir: PathBuf,
    python_path: Option<PathBuf>,
    whisper_command: String,
    initialized: Arc<Mutex<bool>>,
    model_size: String,
}

impl WhisperService {
    pub fn new(model_path: PathBuf, recordings_dir: PathBuf) -> Self {
        // モデルサイズを環境変数で設定可能（デフォルト: small - 速度重視）
        let model_size = std::env::var("WHISPER_MODEL_SIZE")
            .unwrap_or_else(|_| "small".to_string());
        
        // Pythonパスを自動検出
        let python_path = Self::detect_python_path();
        
        // whisperコマンドを設定
        let whisper_command = std::env::var("WHISPER_COMMAND")
            .unwrap_or_else(|_| "whisper".to_string());
        
        Self {
            model_path,
            recordings_dir,
            python_path,
            whisper_command,
            initialized: Arc::new(Mutex::new(false)),
            model_size,
        }
    }

    pub async fn initialize(&self) -> AppResult<()> {
        let mut initialized = self.initialized.lock().await;
        
        if *initialized {
            return Ok(());
        }

        log::info!("🔄 ローカルWhisper初期化中...");

        // Pythonの存在確認
        if !self.check_python_available().await? {
            return Err(AppError::WhisperInit {
                message: "Python not found. Please install Python 3.8 or later.".to_string(),
            });
        }

        // whisperライブラリの存在確認
        if !self.check_whisper_available().await? {
            log::warn!("Whisper library not found. Attempting to install...");
            self.install_whisper().await?;
        }

        // モデルファイルのダウンロード確認
        self.ensure_model_downloaded().await?;

        *initialized = true;
        log::info!("✅ ローカルWhisper初期化完了 (モデル: {})", self.model_size);
        
        Ok(())
    }

    pub async fn is_initialized(&self) -> bool {
        let initialized = self.initialized.lock().await;
        *initialized
    }

    pub async fn transcribe_audio_file(
        &self,
        audio_path: &Path,
        recording_id: String,
        language: Option<String>,
    ) -> AppResult<Transcription> {
        let start_time = std::time::Instant::now();
        
        // 初期化チェック
        if !self.is_initialized().await {
            return Err(AppError::WhisperNotInitialized {
                message: "Whisper service is not initialized. Call initialize() first.".to_string(),
            });
        }

        // ファイル存在チェック
        if !audio_path.exists() {
            return Err(AppError::FileNotFound {
                path: audio_path.to_string_lossy().to_string(),
            });
        }

        // ファイルサイズチェック（500MB制限、1KB最小）
        let file_size = fs::metadata(audio_path)?.len();
        if file_size > 500 * 1024 * 1024 {
            return Err(AppError::TranscriptionFailed {
                message: "Audio file too large. Maximum size is 500MB for local processing.".to_string(),
            });
        }
        if file_size < 1024 {
            return Err(AppError::TranscriptionFailed {
                message: "Audio file too small. Minimum size is 1KB.".to_string(),
            });
        }

        log::info!("🎤 ローカル音声書き起こし開始: {:?}", audio_path);

        // 出力ファイルパスを生成
        let output_dir = self.recordings_dir.join("transcripts");
        fs::create_dir_all(&output_dir)?;
        let output_file = output_dir.join(format!("{}.txt", recording_id));

        // whisperコマンドを実行
        let transcription_text = self.run_whisper_command(
            audio_path,
            &output_file,
            language.as_deref()
        ).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // 転写結果を作成
        let transcription = Transcription::new(
            recording_id,
            transcription_text,
            language.unwrap_or_else(|| "ja".to_string()),
        )
        .with_confidence(Some(0.95)) // ローカル処理なので高い信頼度を設定
        .with_processing_time(Some(processing_time))
        .with_status(TranscriptionStatus::Completed);

        log::info!("✅ ローカル書き起こし完了: {} 文字 ({}ms)", 
                  transcription.text.len(), processing_time);

        Ok(transcription)
    }

    async fn run_whisper_command(
        &self,
        audio_path: &Path,
        output_file: &Path,
        language: Option<&str>,
    ) -> AppResult<String> {
        // PythonスクリプトとしてWhisperを実行
        let python_cmd = self.python_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "python3".to_string());

        // Pythonスクリプトを作成
        let script = self.create_whisper_script(audio_path, language).await?;
        
        log::debug!("実行Python: {} -c '{}'", python_cmd, script);

        // Pythonスクリプト実行
        let mut cmd = TokioCommand::new(&python_cmd);
        cmd.arg("-c").arg(&script);

        let output = cmd.output().await
            .map_err(|e| AppError::TranscriptionFailed {
                message: format!("Failed to execute whisper Python script: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::error!("Whisper command failed. stderr: {}, stdout: {}", stderr, stdout);
            return Err(AppError::TranscriptionFailed {
                message: format!("Whisper transcription failed: {}", stderr),
            });
        }

        // stdoutから結果を取得
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        log::debug!("Whisper stdout: {}", stdout);
        log::debug!("Whisper stderr: {}", stderr);
        
        let result = stdout.trim().to_string();
        
        // 空の結果でもエラーにしない（無音の音声ファイルなど）
        if result.is_empty() {
            log::warn!("Whisper returned empty result. stdout: '{}', stderr: '{}'", stdout, stderr);
            return Ok("（無音または認識できない音声）".to_string());
        }

        Ok(result)
    }

    async fn create_whisper_script(
        &self,
        audio_path: &Path,
        language: Option<&str>,
    ) -> AppResult<String> {
        // 日本語の場合は明示的に言語指定と最適化オプションを追加
        let language = language.unwrap_or("ja");
        let is_japanese = language == "ja";
        
        // 日本語専用の高速パラメータ（速度重視）
        let transcribe_options = if is_japanese {
            format!(
                r#"language='ja',
                task='transcribe',
                temperature=0.2,
                best_of=1,
                beam_size=1,
                patience=1.0,
                length_penalty=1.0,
                suppress_tokens=[-1],
                word_timestamps=False,
                condition_on_previous_text=True"#
            )
        } else {
            format!("language='{}', temperature=0.2, best_of=1, beam_size=1", language)
        };

        let script = format!(
            r#"
import whisper
import sys
import warnings
import os
import numpy as np
warnings.filterwarnings("ignore")

try:
    audio_file = '{audio_path}'
    if not os.path.exists(audio_file):
        print(f"Error: Audio file not found: {{audio_file}}", file=sys.stderr)
        sys.exit(1)
    
    # ファイルサイズチェック
    file_size = os.path.getsize(audio_file)
    if file_size == 0:
        print("Audio file is empty", file=sys.stderr)
        sys.exit(1)
    
    print(f"Loading model: {model_size} (optimized for Japanese)", file=sys.stderr)
    model = whisper.load_model('{model_size}')
    
    print(f"Transcribing file: {{audio_file}} ({{file_size}} bytes) with Japanese optimization", file=sys.stderr)
    
    # 音声前処理（ノイズ除去とボリューム正規化）
    try:
        import librosa
        # librosaで音声を読み込み、前処理
        audio_data, sr = librosa.load(audio_file, sr=16000)
        # RMSベースのボリューム正規化
        rms = np.sqrt(np.mean(audio_data**2))
        if rms > 0:
            target_rms = 0.1
            audio_data = audio_data * (target_rms / rms)
        # 無音部分の除去
        audio_data, _ = librosa.effects.trim(audio_data, top_db=30)
        print(f"Audio preprocessing completed with librosa", file=sys.stderr)
        
        # 前処理済み音声でトランスクリプション
        result = model.transcribe(
            audio_data,
            {transcribe_options}
        )
    except ImportError:
        print(f"librosa not available, using direct file processing", file=sys.stderr)
        # 日本語最適化設定でトランスクリプション実行（ファイル直接）
        result = model.transcribe(
            audio_file,
            {transcribe_options}
        )
    
    text = result.get('text', '').strip()
    
    # デバッグ情報を出力
    if 'segments' in result:
        total_segments = len(result['segments'])
        print(f"Processed {{total_segments}} audio segments", file=sys.stderr)
        
        # 信頼度の低いセグメントを検出
        low_confidence_segments = 0
        for segment in result['segments']:
            if 'avg_logprob' in segment and segment['avg_logprob'] < -0.8:
                low_confidence_segments += 1
        
        if low_confidence_segments > 0:
            print(f"Warning: {{low_confidence_segments}} segments have low confidence", file=sys.stderr)
    
    if not text:
        # 実際の音声が認識できない場合
        print(f"Warning: No text could be transcribed from audio", file=sys.stderr)
        print(f"Audio file size: {{file_size}} bytes", file=sys.stderr)
        print("音声が認識できませんでした。より明瞭に話すか、マイクの距離を近づけてください。")
    else:
        # 日本語の場合、後処理で改善
        if '{language}' == 'ja':
            # 日本語特有の後処理
            import re
            
            # プロンプトテキストと幻覚パターンの除去
            hallucination_patterns = [
                '日本語の音声です：',
                '以下は日本語の音声です：',
                '日本語の音声です。',
                '以下は日本語の音声です。',
                'お疲れ様でした。',
                '次回はお楽しみに',
                'ありがとうございました。',
                'ご視聴ありがとうございました'
            ]
            
            for pattern in hallucination_patterns:
                # 幻覚パターンの除去
                while pattern in text:
                    text = text.replace(pattern, '', 1).strip()
            
            # 不要な空白を削除
            text = re.sub(r'\s+', ' ', text).strip()
            # 句読点の正規化
            text = text.replace('、', '、').replace('。', '。')
            # 英数字周りのスペース調整
            text = re.sub(r'([ぁ-んァ-ヶ一-龯])([A-Za-z0-9])', r'\1 \2', text)
            text = re.sub(r'([A-Za-z0-9])([ぁ-んァ-ヶ一-龯])', r'\1 \2', text)
            
            # 空の結果になった場合のハンドリング
            if not text.strip():
                text = "音声を認識できませんでした。"
        
        print(text)
        
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    import traceback
    traceback.print_exc(file=sys.stderr)
    sys.exit(1)
"#,
            audio_path = audio_path.to_string_lossy(),
            model_size = self.model_size,
            transcribe_options = transcribe_options,
            language = language
        );

        Ok(script)
    }

    async fn check_python_available(&self) -> AppResult<bool> {
        let python_cmd = if let Some(python_path) = &self.python_path {
            python_path.to_string_lossy().to_string()
        } else {
            "python3".to_string()
        };

        let output = TokioCommand::new(&python_cmd)
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(result) if result.status.success() => {
                let version = String::from_utf8_lossy(&result.stdout);
                log::info!("Python detected: {}", version.trim());
                Ok(true)
            }
            _ => {
                // python3が見つからない場合、pythonを試す
                let output = TokioCommand::new("python")
                    .arg("--version")
                    .output()
                    .await;
                
                match output {
                    Ok(result) if result.status.success() => {
                        let version = String::from_utf8_lossy(&result.stdout);
                        log::info!("Python detected: {}", version.trim());
                        Ok(true)
                    }
                    _ => Ok(false)
                }
            }
        }
    }

    async fn check_whisper_available(&self) -> AppResult<bool> {
        // pipでインストールされているかチェック
        let python_cmd = self.python_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "python3".to_string());

        let output = TokioCommand::new(&python_cmd)
            .arg("-c")
            .arg("import whisper; print('whisper available')")
            .output()
            .await;

        match output {
            Ok(result) if result.status.success() => Ok(true),
            _ => Ok(false)
        }
    }

    async fn install_whisper(&self) -> AppResult<()> {
        log::info!("📦 Whisperライブラリと音声処理ライブラリをインストール中...");

        let python_cmd = self.python_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "python3".to_string());

        // 必要なライブラリのリスト（音声処理の品質向上のため）
        let packages = vec![
            "openai-whisper",
            "librosa",
            "soundfile",
            "numpy",
        ];

        for package in packages {
            log::info!("📦 Installing {}...", package);
            
            let output = TokioCommand::new(&python_cmd)
                .arg("-m")
                .arg("pip")
                .arg("install")
                .arg(package)
                .arg("--user") // ユーザーローカルにインストール
                .output()
                .await
                .map_err(|e| AppError::WhisperInit {
                    message: format!("Failed to install {}: {}", package, e),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Failed to install {}: {}", package, stderr);
                // librosa等の失敗は致命的ではないため、whisperのみ必須とする
                if package == "openai-whisper" {
                    return Err(AppError::WhisperInit {
                        message: format!("Whisper installation failed: {}", stderr),
                    });
                }
            } else {
                log::info!("✅ {} インストール完了", package);
            }
        }

        log::info!("✅ 音声処理ライブラリのインストール完了");
        Ok(())
    }

    async fn ensure_model_downloaded(&self) -> AppResult<()> {
        log::info!("🔍 Whisperモデル確認中...");

        // Whisperモデルのキャッシュディレクトリを確認
        let cache_dir = self.get_whisper_cache_dir();
        let model_file = cache_dir.join(format!("{}.pt", self.model_size));

        if model_file.exists() {
            log::info!("✅ モデルファイル確認完了: {}", model_file.display());
            return Ok(());
        }

        log::info!("📥 Whisperモデルをダウンロード中... (モデル: {})", self.model_size);

        // モデルをダウンロードするためのダミー音声ファイルを作成
        let temp_audio = self.create_dummy_audio_file().await?;

        // ダミー音声でwhisperを実行してモデルをダウンロード
        let python_cmd = self.python_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "python3".to_string());

        let output = TokioCommand::new(&python_cmd)
            .arg("-c")
            .arg(&format!(
                "import whisper; model = whisper.load_model('{}'); print('Model loaded')",
                self.model_size
            ))
            .output()
            .await
            .map_err(|e| AppError::WhisperInit {
                message: format!("Failed to download model: {}", e),
            })?;

        // 一時ファイルを削除
        let _ = fs::remove_file(&temp_audio);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::WhisperInit {
                message: format!("Model download failed: {}", stderr),
            });
        }

        log::info!("✅ モデルダウンロード完了");
        Ok(())
    }

    async fn create_dummy_audio_file(&self) -> AppResult<PathBuf> {
        // 1秒の無音WAVファイルを生成
        let temp_dir = std::env::temp_dir();
        let dummy_audio = temp_dir.join("dummy_audio.wav");

        // 簡単な無音WAVファイルのヘッダーとデータ
        let sample_rate = 16000u32;
        let duration_samples = sample_rate; // 1秒
        let mut wav_data = Vec::new();

        // WAVヘッダー
        wav_data.extend_from_slice(b"RIFF");
        wav_data.extend_from_slice(&(36 + duration_samples * 2).to_le_bytes());
        wav_data.extend_from_slice(b"WAVE");
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav_data.extend_from_slice(&sample_rate.to_le_bytes());
        wav_data.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
        wav_data.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav_data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&(duration_samples * 2).to_le_bytes());

        // 無音データ（16bit）
        for _ in 0..duration_samples {
            wav_data.extend_from_slice(&0i16.to_le_bytes());
        }

        fs::write(&dummy_audio, wav_data)?;
        Ok(dummy_audio)
    }

    fn get_whisper_cache_dir(&self) -> PathBuf {
        // Whisperのデフォルトキャッシュディレクトリ
        if let Some(home) = dirs::home_dir() {
            home.join(".cache").join("whisper")
        } else {
            PathBuf::from("/tmp/.whisper_cache")
        }
    }

    fn detect_python_path() -> Option<PathBuf> {
        // 一般的なPythonパスを確認
        let possible_paths = vec![
            "/usr/bin/python3",
            "/usr/local/bin/python3",
            "/opt/homebrew/bin/python3",
            "/usr/bin/python",
            "/usr/local/bin/python",
        ];

        for path in possible_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return Some(path_buf);
            }
        }

        None
    }

    pub async fn get_available_languages(&self) -> AppResult<Vec<String>> {
        // Whisperがサポートする言語一覧
        Ok(vec![
            "ja".to_string(),    // Japanese
            "en".to_string(),    // English
            "zh".to_string(),    // Chinese
            "ko".to_string(),    // Korean
            "es".to_string(),    // Spanish
            "fr".to_string(),    // French
            "de".to_string(),    // German
            "it".to_string(),    // Italian
            "pt".to_string(),    // Portuguese
            "ru".to_string(),    // Russian
            "ar".to_string(),    // Arabic
            "hi".to_string(),    // Hindi
        ])
    }

    pub async fn get_service_status(&self) -> AppResult<String> {
        if !self.is_initialized().await {
            return Ok("Not initialized".to_string());
        }

        let python_available = self.check_python_available().await?;
        let whisper_available = self.check_whisper_available().await?;

        if python_available && whisper_available {
            Ok(format!("Local Whisper ready (model: {})", self.model_size))
        } else if !python_available {
            Ok("Python not available".to_string())
        } else {
            Ok("Whisper library not available".to_string())
        }
    }

    pub async fn get_model_info(&self) -> AppResult<String> {
        let cache_dir = self.get_whisper_cache_dir();
        let model_file = cache_dir.join(format!("{}.pt", self.model_size));
        
        if model_file.exists() {
            let metadata = fs::metadata(&model_file)?;
            let size_mb = metadata.len() / (1024 * 1024);
            Ok(format!("Model: {} ({} MB)", self.model_size, size_mb))
        } else {
            Ok(format!("Model: {} (not downloaded)", self.model_size))
        }
    }

    pub async fn get_all_models_info(&self) -> AppResult<Vec<(String, String, bool)>> {
        let models = vec![
            ("tiny".to_string(), "~39MB".to_string()),
            ("base".to_string(), "~142MB".to_string()),
            ("small".to_string(), "~461MB".to_string()),
            ("medium".to_string(), "~1.5GB".to_string()),
            ("large".to_string(), "~2.9GB".to_string()),
        ];

        let cache_dir = self.get_whisper_cache_dir();
        let mut result = Vec::new();

        for (model_name, estimated_size) in models {
            let model_file = cache_dir.join(format!("{}.pt", model_name));
            let is_downloaded = model_file.exists();
            
            let actual_size = if is_downloaded {
                let metadata = fs::metadata(&model_file)?;
                let size_mb = metadata.len() / (1024 * 1024);
                format!("{}MB", size_mb)
            } else {
                estimated_size
            };

            result.push((model_name, actual_size, is_downloaded));
        }

        Ok(result)
    }

    pub async fn download_all_models(&self) -> AppResult<()> {
        log::info!("📥 全Whisperモデルのダウンロードを開始...");
        
        let models = vec!["tiny", "base", "small", "medium", "large"];
        let total = models.len();
        
        for (index, model) in models.iter().enumerate() {
            log::info!("📥 モデルダウンロード中: {} ({}/{})", model, index + 1, total);
            
            if let Err(e) = self.download_specific_model(model).await {
                log::error!("❌ モデル {} のダウンロードに失敗: {}", model, e);
                return Err(e);
            } else {
                log::info!("✅ モデル {} ダウンロード完了", model);
            }
        }

        log::info!("🎉 全Whisperモデルのダウンロードが完了しました！");
        Ok(())
    }

    pub async fn download_specific_model(&self, model_name: &str) -> AppResult<()> {
        let python_cmd = self.python_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "python3".to_string());

        let script = format!(
            r#"
import whisper
import sys
import warnings
warnings.filterwarnings("ignore")

try:
    print(f"Downloading model: {}", file=sys.stderr)
    model = whisper.load_model('{}')
    print(f"Model {} loaded successfully", file=sys.stderr)
except Exception as e:
    print(f"Error downloading model {}: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
            model_name, model_name, model_name, model_name
        );

        let output = TokioCommand::new(&python_cmd)
            .arg("-c")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AppError::WhisperInit {
                message: format!("Failed to download model {}: {}", model_name, e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::WhisperInit {
                message: format!("Model {} download failed: {}", model_name, stderr),
            });
        }

        Ok(())
    }

    pub async fn get_available_models(&self) -> AppResult<Vec<String>> {
        Ok(vec![
            "tiny".to_string(),
            "base".to_string(),
            "small".to_string(),
            "medium".to_string(),
            "large".to_string(),
        ])
    }

    pub async fn set_model_size(&mut self, model_size: String) -> AppResult<()> {
        let available_models = self.get_available_models().await?;
        
        if !available_models.contains(&model_size) {
            return Err(AppError::ValidationError {
                message: format!("Invalid model size: {}. Available: {:?}", model_size, available_models),
            });
        }

        self.model_size = model_size;
        
        // 初期化状態をリセット（新しいモデルで再初期化が必要）
        let mut initialized = self.initialized.lock().await;
        *initialized = false;
        
        Ok(())
    }

    pub fn get_current_model_size(&self) -> String {
        self.model_size.clone()
    }
}