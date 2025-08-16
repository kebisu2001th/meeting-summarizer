pub mod recording;

// CMakeが必要なため、現在はモック実装を使用
// pub mod whisper;
pub mod whisper_mock;

pub use recording::RecordingService;
pub use whisper_mock::WhisperService;