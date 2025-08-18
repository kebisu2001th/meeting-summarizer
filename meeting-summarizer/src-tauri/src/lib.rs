mod commands;
pub mod database;
pub mod errors;
pub mod models;
pub mod services;

use crate::commands::*;
use crate::database::Database;
use crate::services::{RecordingService, WhisperService};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	// Initialize logger so that `log::info!` etc. are printed to the terminal
	let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // アプリケーションデータディレクトリを取得
            let app_data_dir = app.path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // データディレクトリが存在しない場合は作成
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir)
                    .expect("Failed to create app data directory");
            }

            // データベースファイルパス
            let db_path = app_data_dir.join("recordings.db");
            
            // 録音ファイル保存ディレクトリ
            let recordings_dir = app_data_dir.join("recordings");

            // データベースを初期化
            let database = Arc::new(Database::new(db_path).expect("Failed to initialize database"));

            // 録音サービスを初期化
            let recording_service = Arc::new(
                RecordingService::new(database, recordings_dir.clone())
                    .expect("Failed to initialize recording service")
            );

            // Whisperモデルパス（アプリケーションデータディレクトリ内）
            let whisper_model_path = app_data_dir.join("models").join("ggml-base.bin");
            
            // Whisperサービスを初期化（セキュリティ強化：許可されたディレクトリを指定）
            let whisper_service = Arc::new(WhisperService::new(whisper_model_path, recordings_dir));

            // サービスをアプリケーション状態に追加
            app.manage(recording_service);
            app.manage(whisper_service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_recordings,
            get_recording,
            delete_recording,
            is_recording,
            get_recordings_count,
            get_audio_devices,
            transcribe_recording,
            initialize_whisper,
            is_whisper_initialized
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
