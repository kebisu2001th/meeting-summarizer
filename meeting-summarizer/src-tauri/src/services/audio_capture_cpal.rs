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
use std::thread;

const SAMPLE_RATE: u32 = 16000; // 16kHz for Whisper compatibility
const CHANNELS: u16 = 1; // Mono

/// CPAL音声キャプチャ実装（スレッドベース）
pub struct AudioCapture {
    is_recording: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    stream_handle: Arc<Mutex<Option<Box<dyn Send + 'static>>>>,
}

impl AudioCapture {
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            is_recording: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            stream_handle: Arc::new(Mutex::new(None)),
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

        // CPALを使った音声録音をスレッドで開始
        let output_path_clone = output_path.to_path_buf();
        let output_path_log = output_path.to_path_buf();
        let is_recording_clone = self.is_recording.clone();
        let audio_buffer_clone = self.audio_buffer.clone();

        // 録音スレッドを開始（非同期ではなく別スレッドで実行）
        let _handle = thread::spawn(move || {
            if let Err(e) = Self::record_audio_thread(output_path_clone, is_recording_clone, audio_buffer_clone) {
                log::error!("Audio recording thread failed: {}", e);
            }
        });

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

        // 録音スレッドが終了するまで少し待つ
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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
        let host = cpal::default_host();
        
        let device = host.default_input_device()
            .ok_or_else(|| AppError::Recording {
                message: "No default input device available".to_string(),
            })?;

        log::info!("Using audio device: {}", device.name().unwrap_or("Unknown".to_string()));

        let config = StreamConfig {
            channels: CHANNELS,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        // 録音データ用のバッファ
        let recorded_samples = Arc::new(Mutex::new(Vec::<f32>::new()));
        let recorded_samples_clone = recorded_samples.clone();
        let is_recording_for_callback = is_recording.clone();

        // 音声ストリームを作成
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let is_recording_status = is_recording_for_callback.lock().unwrap();
                if *is_recording_status {
                    let mut samples = recorded_samples_clone.lock().unwrap();
                    for &sample in data {
                        samples.push(sample);
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

        // ストリームを開始
        stream.play().map_err(|e| AppError::Recording {
            message: format!("Failed to start audio stream: {}", e),
        })?;

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

        log::info!("CPAL recording completed: {} samples saved to {:?}", samples.len(), output_path);
        Ok(())
    }

    fn save_samples_to_file(samples: &[f32], output_path: &Path) -> AppResult<()> {
        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let file = File::create(output_path)
            .map_err(|e| AppError::Io(e))?;
        
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