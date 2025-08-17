use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::models::{Recording, RecordingSession};
use crate::services::audio_capture_cpal::AudioCapture;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RecordingService {
    db: Arc<Database>,
    recordings_dir: PathBuf,
    current_session: Arc<Mutex<Option<RecordingSession>>>,
    audio_capture: Arc<Mutex<AudioCapture>>,
}

impl RecordingService {
    pub fn new(db: Arc<Database>, recordings_dir: PathBuf) -> AppResult<Self> {
        // 録音ディレクトリが存在しない場合は作成
        if !recordings_dir.exists() {
            fs::create_dir_all(&recordings_dir)?;
        }

        // オーディオキャプチャを初期化
        let audio_capture = AudioCapture::new()?;

        Ok(Self {
            db,
            recordings_dir,
            current_session: Arc::new(Mutex::new(None)),
            audio_capture: Arc::new(Mutex::new(audio_capture)),
        })
    }

    pub async fn start_recording(&self) -> AppResult<String> {
        // セッション状態をチェック
        {
            let current_session = self.current_session.lock().await;

            // 既に録音中の場合はエラー
            if current_session.is_some() {
                return Err(AppError::Recording { 
                    message: "Recording is already in progress".to_string() 
                });
            }
        } // current_sessionガードをここでdrop

        // 一時ファイル名を生成
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::InvalidOperation { 
                message: "Failed to get system time".to_string() 
            })?
            .as_secs();

        let temp_filename = format!("recording_temp_{}.wav", timestamp);
        let temp_file_path = self.recordings_dir.join(&temp_filename);

        // 録音セッションを開始
        let session = RecordingSession::new(temp_file_path.to_string_lossy().to_string());
        let session_id = session.id.clone();

        // 実際の音声録音を開始
        {
            let mut audio_capture = self.audio_capture.lock().await;
            audio_capture.start_recording(&temp_file_path).await?;
        } // Mutexガードがここでdropされる

        // セッションを設定
        {
            let mut current_session = self.current_session.lock().await;
            *current_session = Some(session);
        }

        Ok(session_id)
    }

    pub async fn stop_recording(&self) -> AppResult<Recording> {
        // セッション情報を読み取り（まだクリアしない）
        let session = {
            let current_session = self.current_session.lock().await;
            current_session.clone().ok_or_else(|| AppError::Recording {
                message: "No active recording session".to_string(),
            })?
        };

        // 実際の音声録音を停止
        {
            let mut audio_capture = self.audio_capture.lock().await;
            audio_capture.stop_recording().await?;
        } // Mutexガードがここでdropされる

        // 録音時間を計算（秒）
        let duration = chrono::Utc::now()
            .signed_duration_since(session.start_time)
            .num_seconds();

        // 一時ファイルを最終的な場所に移動（セッションIDを含めて一意化）
        let final_filename = format!(
            "recording_{}_{}.wav",
            session.start_time.format("%Y%m%d_%H%M%S"),
            session.id
        );
        let final_path = self.recordings_dir.join(&final_filename);

        fs::rename(&session.temp_file_path, &final_path)?;

        // ファイルサイズを取得
        let file_size = fs::metadata(&final_path)?.len() as i64;

        // Recording オブジェクトを作成
        let recording = Recording::new(
            final_filename,
            final_path.to_string_lossy().to_string(),
        )
        .with_duration(duration)
        .with_file_size(file_size);

        // データベースに保存
        self.db.create_recording(&recording).await?;

        // ここまで成功したら、セッションをクリア
        {
            let mut current_session = self.current_session.lock().await;
            *current_session = None;
        }

        Ok(recording)
    }

    pub async fn get_recordings(&self) -> AppResult<Vec<Recording>> {
        self.db.get_all_recordings().await
    }

    pub async fn get_recording(&self, id: &str) -> AppResult<Option<Recording>> {
        self.db.get_recording(id).await
    }

    pub async fn delete_recording(&self, id: &str) -> AppResult<bool> {
        // データベースから録音情報を取得
        if let Some(recording) = self.db.get_recording(id).await? {
            // ファイルを削除
            let file_path = Path::new(&recording.file_path);
            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
            
            // データベースから削除
            self.db.delete_recording(id).await
        } else {
            Ok(false)
        }
    }

    pub fn is_recording(&self) -> bool {
        // セッション状態とオーディオキャプチャ状態の両方をチェック
        let session_active = self.current_session.try_lock()
            .map(|session| session.is_some())
            .unwrap_or(false);

        let audio_active = self.audio_capture.try_lock()
            .map(|capture| capture.is_recording())
            .unwrap_or(false);

        session_active && audio_active
    }

    pub async fn get_recordings_count(&self) -> AppResult<i64> {
        self.db.get_recordings_count().await
    }

    pub async fn get_recording_file_path(&self, id: &str) -> AppResult<Option<PathBuf>> {
        if let Some(recording) = self.db.get_recording(id).await? {
            let path = PathBuf::from(&recording.file_path);
            if path.exists() {
                Ok(Some(path))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    // オーディオデバイス情報を取得
    pub fn get_audio_devices(&self) -> AppResult<Vec<String>> {
        crate::services::audio_capture_mock::get_audio_devices()
    }
}