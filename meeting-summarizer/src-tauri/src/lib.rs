mod commands;
mod database;
mod errors;
mod models;
mod services;

use crate::commands::*;
use crate::database::Database;
use crate::services::{RecordingService, WhisperService};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
                RecordingService::new(database, recordings_dir)
                    .expect("Failed to initialize recording service")
            );

            // Whisperモデルパス（アプリケーションデータディレクトリ内）
            let whisper_model_path = app_data_dir.join("models").join("ggml-base.bin");
            
            // Whisperサービスを初期化
            let whisper_service = Arc::new(WhisperService::new(whisper_model_path));

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
            transcribe_recording,
            initialize_whisper,
            is_whisper_initialized
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
