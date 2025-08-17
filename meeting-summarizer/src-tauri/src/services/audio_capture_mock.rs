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

        // モック録音ループ
        while is_recording.lock().map(|g| *g).unwrap_or(false) {
            // サイレント音声データを生成（わずかなノイズを追加して現実的に）
            for _ in 0..samples_per_update {
                // 基本的にはサイレント、時々わずかなノイズ
                let sample = if sample_count % (SAMPLE_RATE as u64 * 5) == 0 {
                    // 5秒に一度、わずかなノイズ
                    (rand::random::<f32>() - 0.5) * 0.01 * i16::MAX as f32
                } else {
                    // 基本はサイレント
                    0.0
                };
                
                let i16_sample = sample as i16;
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