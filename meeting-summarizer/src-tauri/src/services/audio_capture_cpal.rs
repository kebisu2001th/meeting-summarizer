use crate::errors::{AppError, AppResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::thread::{self, JoinHandle};

const SAMPLE_RATE: u32 = 16000; // 16kHz for Whisper compatibility
const CHANNELS: u16 = 1; // Mono

/// CPAL音声キャプチャ実装（スレッドベース）
pub struct AudioCapture {
    is_recording: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

// TODO: https://chatgpt.com/c/68a1cb5b-ed9c-832e-91a2-e2277eb5cb10
// ↑を見て修正を入れる
impl AudioCapture {
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            is_recording: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            thread_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start_recording(&mut self, output_path: &Path) -> AppResult<()> {
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

        // オーディオバッファをクリア
        {
            let mut buffer = self.audio_buffer.lock()
                .map_err(|_| AppError::Recording {
                    message: "Failed to acquire audio buffer lock".to_string(),
                })?;
            buffer.clear();
        }

        // 出力パスの事前検証（親ディレクトリ作成＋書き込み可否テスト）
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Recording { message: format!("Failed to create parent dir: {e}") })?;
        }
        {
            // 一度開けるか確認（競合を避けるなら OpenOptions で create(true), write(true), truncate(false) なども可）
            use std::fs::OpenOptions;
            OpenOptions::new().create(true).write(true).open(output_path)
                .map_err(|e| AppError::Recording { message: format!("Cannot open output file for write: {e}") })?;
        }

        // CPALを使った音声録音をスレッドで開始
        let output_path_clone = output_path.to_path_buf();
        let output_path_log = output_path.to_path_buf();
        let is_recording_clone = self.is_recording.clone();
        let audio_buffer_clone = self.audio_buffer.clone();

        // 録音スレッドを開始（チャネル通知なしでUIブロック回避）
        let handle = thread::spawn(move || {
            log::info!("Recording thread starting for file: {:?}", output_path_clone);
            if let Err(e) = Self::record_audio_thread(output_path_clone, is_recording_clone, audio_buffer_clone) {
                log::error!("Audio recording thread failed: {}", e);
            } else {
                log::info!("Recording thread completed successfully");
            }
        });

        // スレッドハンドルを保存
        {
            let mut thread_handle = self.thread_handle.lock()
                .map_err(|_| AppError::Recording {
                    message: "Failed to acquire thread handle lock".to_string(),
                })?;
            *thread_handle = Some(handle);
        }

        // 短時間だけ待ってスレッドが開始されたことを確認
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        log::info!("CPAL audio recording started: {:?}", output_path_log.file_name().unwrap_or_default());
        Ok(())
    }

    pub async fn stop_recording(&mut self) -> AppResult<()> {
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

        // スレッドハンドルを取得して終了を待つ
        let handle = {
            let mut thread_handle = self.thread_handle.lock()
                .map_err(|_| AppError::Recording {
                    message: "Failed to acquire thread handle lock".to_string(),
                })?;
            thread_handle.take()
        };

        if let Some(handle) = handle {
            // 非同期でスレッドの終了を待つ
            tokio::task::spawn_blocking(move || {
                if let Err(e) = handle.join() {
                    log::error!("Recording thread panicked: {:?}", e);
                } else {
                    log::info!("Recording thread joined successfully");
                }
            }).await
            .map_err(|e| AppError::Recording {
                message: format!("Failed to join recording thread: {}", e),
            })?;
        }

        log::info!("CPAL audio recording stopped");
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

    // 別スレッドで実行される録音機能
    fn record_audio_thread(
        output_path: std::path::PathBuf,
        is_recording: Arc<Mutex<bool>>,
        _audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    ) -> AppResult<()> {
        log::info!("Recording thread started, output path: {:?}", output_path);
        
        let host = cpal::default_host();
        log::info!("Got CPAL host");
        
        let device = host.default_input_device()
            .ok_or_else(|| AppError::Recording {
                message: "No default input device available".to_string(),
            })?;

        log::info!("Using audio device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));

        // デバイスがサポートする設定を取得
        let supported_configs = device.supported_input_configs()
            .map_err(|e| AppError::Recording {
                message: format!("Failed to get supported input configs: {}", e),
            })?;
        
        // サポートされている設定を確認してログ出力
        let mut available_configs = Vec::new();
        for config_range in supported_configs {
            log::info!("Supported config: channels={}, sample_rate={:?}, format={:?}", 
                config_range.channels(), 
                config_range.min_sample_rate().0..=config_range.max_sample_rate().0,
                config_range.sample_format()
            );
            available_configs.push(config_range);
        }

        // 最適な設定を選択（できるだけ16kHzに近い設定を選ぶ）
        let config = if let Some(config_range) = available_configs.first() {
            let sample_rate = if config_range.min_sample_rate().0 <= SAMPLE_RATE && 
                              config_range.max_sample_rate().0 >= SAMPLE_RATE {
                // 16kHzがサポートされている場合
                SampleRate(SAMPLE_RATE)
            } else {
                // 16kHzがサポートされていない場合、最も近い値を選択
                let min_rate = config_range.min_sample_rate().0;
                let max_rate = config_range.max_sample_rate().0;
                
                if min_rate > SAMPLE_RATE {
                    config_range.min_sample_rate()
                } else {
                    config_range.max_sample_rate()
                }
            };
            
            let channels = std::cmp::min(config_range.channels(), CHANNELS);
            log::info!("Selected config: channels={}, sample_rate={}", channels, sample_rate.0);
            
            StreamConfig {
                channels,
                sample_rate,
                buffer_size: cpal::BufferSize::Default,
            }
        } else {
            return Err(AppError::Recording {
                message: "No supported input configurations found".to_string(),
            });
        };

        // 録音データ用のバッファ
        let recorded_samples = Arc::new(Mutex::new(Vec::<f32>::new()));
        let recorded_samples_clone = recorded_samples.clone();
        let is_recording_for_callback = is_recording.clone();

        log::info!("Creating audio stream with config: channels={}, sample_rate={}", config.channels, config.sample_rate.0);

        // 音声ストリームを作成
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let is_recording_status = is_recording_for_callback.lock().unwrap();
                if *is_recording_status {
                    let mut samples = recorded_samples_clone.lock().unwrap();
                    for &sample in data {
                        // 音声ゲインを調整（小さい音声を増幅）
                        let amplified_sample = if sample.abs() > 0.001 {
                            sample * 3.0  // 3倍に増幅
                        } else {
                            sample
                        };
                        samples.push(amplified_sample.clamp(-1.0, 1.0));
                    }
                    if samples.len() % 16000 == 0 {  // ログを1秒ごとに出力
                        log::info!("Recorded {} samples", samples.len());
                    }
                }
            },
            move |err| {
                log::error!("Audio stream error: {}", err);
            },
            None,
        ).map_err(|e| AppError::Recording {
            message: format!("Failed to build audio stream: {}", e),
        })?;

        log::info!("Audio stream created successfully");

        // ストリームを開始
        stream.play().map_err(|e| AppError::Recording {
            message: format!("Failed to start audio stream: {}", e),
        })?;

        log::info!("Audio stream started, beginning recording loop");

        // ここで開始成功を通知（ただし、既にスレッド関数の戻り値で通知済みなので、このタイミングでの通知は不要）

        // 録音が停止されるまで待機
        loop {
            thread::sleep(std::time::Duration::from_millis(100));
            
            let is_recording_status = {
                let guard = is_recording.lock().unwrap();
                *guard
            };
            
            if !is_recording_status {
                break;
            }
        }

        // ストリームを停止
        drop(stream);

        // 録音データをファイルに保存
        let samples = {
            let guard = recorded_samples.lock().unwrap();
            guard.clone()
        };

        if samples.is_empty() {
            return Err(AppError::Recording {
                message: "No audio data recorded".to_string(),
            });
        }

        Self::save_samples_to_file(&samples, &output_path)?;

        // ファイル作成確認
        if output_path.exists() {
            let file_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
            log::info!("CPAL recording completed: {} samples saved to {:?}, file size: {} bytes", samples.len(), output_path, file_size);
        } else {
            log::error!("CPAL recording failed: file not created at {:?}", output_path);
            return Err(AppError::Recording {
                message: format!("Output file was not created: {:?}", output_path),
            });
        }

        Ok(())
    }

    fn save_samples_to_file(samples: &[f32], output_path: &Path) -> AppResult<()> {
        log::info!("Saving {} samples to file: {:?}", samples.len(), output_path);
        
        // 親ディレクトリが存在することを確認
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                log::info!("Creating parent directory: {:?}", parent);
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::Recording {
                        message: format!("Failed to create parent directory: {}", e),
                    })?;
            }
        }

        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let file = File::create(output_path)
            .map_err(|e| AppError::Recording {
                message: format!("Failed to create output file {:?}: {}", output_path, e),
            })?;
        
        let mut writer = WavWriter::new(BufWriter::new(file), spec)
            .map_err(|e| AppError::Recording {
                message: format!("Failed to create WAV writer: {}", e),
            })?;

        // f32 サンプルを i16 に変換してファイルに書き込み
        for &sample in samples {
            let i16_sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(i16_sample)
                .map_err(|e| AppError::Recording {
                    message: format!("Failed to write audio sample: {}", e),
                })?;
        }

        writer.finalize()
            .map_err(|e| AppError::Recording {
                message: format!("Failed to finalize WAV file: {}", e),
            })?;

        Ok(())
    }
}

// 利用可能なオーディオデバイスを取得
pub fn get_audio_devices() -> AppResult<Vec<String>> {
    let host = cpal::default_host();
    let mut device_names = Vec::new();
    
    // 入力デバイスを列挙
    let input_devices = host.input_devices()
        .map_err(|e| AppError::Recording {
            message: format!("Failed to enumerate input devices: {}", e),
        })?;

    for device in input_devices {
        if let Ok(name) = device.name() {
            device_names.push(name);
        }
    }

    if device_names.is_empty() {
        device_names.push("Default Microphone".to_string());
    }

    Ok(device_names)
}