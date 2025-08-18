use crate::errors::{AppError, AppResult};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

const SAMPLE_RATE: u32 = 16000; // 16kHz for Whisper compatibility
const CHANNELS: u16 = 1; // Mono

/// モック音声キャプチャ実装
/// 実際の音声録音機能の代わりに、サイレント音声ファイルを生成します
pub struct AudioCapture {
    is_recording: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl AudioCapture {
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            is_recording: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start_recording(&self, output_path: &Path) -> AppResult<()> {
        {
            let mut is_recording = self.is_recording.lock()
                .map_err(|_| AppError::Recording {
                    message: "Failed to acquire recording lock".to_string(),
                })?;

            if *is_recording {
                return Err(AppError::Recording {
                    message: "Recording is already in progress".to_string(),
                });
            }

            *is_recording = true;
        }

        // 開始時刻を記録
        {
            let mut start_time = self.start_time.lock()
                .map_err(|_| AppError::Recording {
                    message: "Failed to acquire start time lock".to_string(),
                })?;
            *start_time = Some(Instant::now());
        }

        // 出力ファイルを事前作成して、停止直後のリネーム失敗を防ぐ
        File::create(output_path).map_err(|e| AppError::Recording {
            message: format!("Failed to create output file: {}", e),
        })?;
        
        // バックグラウンドでモック録音を開始
        let output_path = output_path.to_path_buf();
        let is_recording_clone = self.is_recording.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::mock_recording_loop(output_path, is_recording_clone).await {
                log::error!("Mock recording failed: {}", e);
            }
        });

        Ok(())
    }

    pub async fn stop_recording(&self) -> AppResult<()> {
        {
            let mut is_recording = self.is_recording.lock()
                .map_err(|_| AppError::Recording {
                    message: "Failed to acquire recording lock".to_string(),
                })?;

            if !*is_recording {
                return Err(AppError::Recording {
                    message: "No recording in progress".to_string(),
                });
            }

            *is_recording = false;
        }

        // 録音が完全に停止するまで少し待つ
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    pub fn get_recording_duration(&self) -> Duration {
        let start_time = self.start_time.lock()
            .ok()
            .and_then(|guard| *guard);

        if let Some(start) = start_time {
            start.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }

    async fn mock_recording_loop(
        output_path: std::path::PathBuf,
        is_recording: Arc<Mutex<bool>>,
    ) -> AppResult<()> {
        // WAVファイルの仕様を設定
        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let file = File::create(&output_path)
            .map_err(|e| AppError::Io(e))?;
        
        let mut writer = WavWriter::new(BufWriter::new(file), spec)
            .map_err(|e| AppError::Recording {
                message: format!("Failed to create WAV writer: {}", e),
            })?;

        let mut sample_count = 0u64;
        let samples_per_update = SAMPLE_RATE / 10; // 0.1秒分のサンプル

        // モック録音ループ - 日本語音声パターンを生成
        while is_recording.lock().map(|g| *g).unwrap_or(false) {
            // 日本語話者の音声特性に似たパターンを生成
            for i in 0..samples_per_update {
                let time = (sample_count + i as u64) as f32 / SAMPLE_RATE as f32;
                
                // 日本語の音韻特性を模擬した周波数パターン
                // 日本語の平均基本周波数: 男性 ~120Hz, 女性 ~220Hz
                let base_freq = 180.0 + 40.0 * (time * 0.8).sin(); // 基本周波数（日本語話者の中間）
                
                // 日本語の子音・母音パターンを模擬
                let vowel_pattern = 400.0 + 200.0 * (time * 3.0).cos(); // 母音フォルマント
                let consonant_pattern = 800.0 + 400.0 * (time * 7.0).sin(); // 子音成分
                
                // 日本語特有のピッチ変動パターン
                let pitch_variation = 1.0 + 0.3 * (time * 1.5).sin() + 0.2 * (time * 4.0).cos();
                
                // 音韻の強弱変化（日本語の拍リズム）
                let mora_rhythm = 0.8 + 0.4 * (time * 6.0).sin().abs();
                
                // 全体の振幅（声の大きさ）
                let amplitude = 0.25 * mora_rhythm * pitch_variation;
                
                // 複数の音声成分を合成
                let fundamental = amplitude * (2.0 * std::f32::consts::PI * base_freq * time).sin();
                let vowel_component = amplitude * 0.6 * (2.0 * std::f32::consts::PI * vowel_pattern * time).sin();
                let consonant_component = amplitude * 0.3 * (2.0 * std::f32::consts::PI * consonant_pattern * time).sin();
                
                // 呼吸音・摩擦音のノイズ成分
                let breath_noise = amplitude * 0.1 * (rand::random::<f32>() - 0.5);
                
                // 最終的な音声信号
                let combined_sample = fundamental + vowel_component + consonant_component + breath_noise;
                let i16_sample = (combined_sample * i16::MAX as f32).clamp(-32767.0, 32767.0) as i16;
                
                writer.write_sample(i16_sample)
                    .map_err(|e| AppError::Recording {
                        message: format!("Failed to write audio sample: {}", e),
                    })?;
                
                sample_count += 1;
            }

            // 0.1秒待機
            sleep(Duration::from_millis(100)).await;
        }

        writer.finalize()
            .map_err(|e| AppError::Recording {
                message: format!("Failed to finalize WAV file: {}", e),
            })?;

        log::info!("Mock recording completed: {} samples written", sample_count);
        Ok(())
    }
}

// モック用のランダム数生成
mod rand {
    use std::cell::Cell;
    
    thread_local! {
        static RNG_STATE: Cell<u64> = Cell::new(1);
    }
    
    pub fn random<T: From<f32>>() -> T {
        RNG_STATE.with(|state| {
            let mut x = state.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            state.set(x);
            
            T::from((x as f32) / (u64::MAX as f32))
        })
    }
}

// サポートされているオーディオデバイスを取得する関数（モック）
pub fn get_audio_devices() -> AppResult<Vec<String>> {
    // モック実装：ダミーデバイス名を返す
    Ok(vec![
        "Default Microphone".to_string(),
        "Built-in Microphone".to_string(),
        "External USB Microphone".to_string(),
    ])
}