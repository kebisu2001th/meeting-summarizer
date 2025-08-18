// pub mod audio_capture;  // 実際の音声キャプチャ（Send+Sync問題のため一時無効化）
pub mod audio_capture_mock;
pub mod audio_capture_cpal;    // CPAL音声キャプチャ実装
pub mod recording;

// ローカルWhisper実装（Python whisperライブラリ使用）
pub mod whisper;
pub mod whisper_local;
pub mod whisper_mock;

// LLM統合サービス
pub mod llm;
pub mod llm_manager;
pub mod model_settings;
pub mod model_downloader;

pub use audio_capture_cpal::AudioCapture;
pub use recording::RecordingService;
pub use whisper_local::WhisperService;
pub use llm::LLMService;
pub use llm_manager::{LLMModelManager, ModelInfo, ModelBenchmark, ModelCapabilities};
pub use model_settings::{ModelSettings, ModelPreference, PerformancePriority, ModelSettingsManager};
pub use model_downloader::{ModelDownloader, DownloadableModel, SystemCompatibility, DownloadProgress, DownloadStatus};
