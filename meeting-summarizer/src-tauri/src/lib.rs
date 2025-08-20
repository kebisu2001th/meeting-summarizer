mod commands;
pub mod database;
pub mod errors;
pub mod models;
pub mod services;

use crate::commands::{*, file_management, llm, streaming, model_management, model_settings, model_downloader};
use crate::database::Database;
use crate::services::{RecordingService, WhisperService, LLMModelManager, ModelSettingsManager, ModelDownloader};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

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

            // データベースを初期化（LLM用のMutex包装版）
            let database = Arc::new(Mutex::new(Database::new(&db_path).expect("Failed to initialize database")));

            // 録音サービス用のデータベース（独立インスタンス）
            let recording_db = Arc::new(Database::new(&db_path).expect("Failed to initialize recording database"));
            
            // 録音サービスを初期化
            let recording_service = Arc::new(
                RecordingService::new(recording_db, recordings_dir.clone())
                    .expect("Failed to initialize recording service")
            );

            // Whisperモデルパス（アプリケーションデータディレクトリ内）
            let whisper_model_path = app_data_dir.join("models").join("ggml-base.bin");
            
            // Whisperサービスを初期化（セキュリティ強化：許可されたディレクトリを指定）
            let whisper_service = Arc::new(WhisperService::new(whisper_model_path, recordings_dir));

            // LLMモデル管理サービスを初期化
            let llm_model_manager = Arc::new(Mutex::new(LLMModelManager::new()));

            // モデル設定管理サービスを初期化
            let model_settings_path = app_data_dir.join("model_settings.json");
            let model_settings_manager = ModelSettingsManager::new(model_settings_path);
            
            // 設定の読み込みは後でランタイム時に行う
            let model_settings_manager = Arc::new(Mutex::new(model_settings_manager));

            // モデルダウンロードサービスを初期化
            let model_downloader = Arc::new(Mutex::new(ModelDownloader::new()));

            // サービスをアプリケーション状態に追加
            app.manage(database);
            app.manage(recording_service);
            app.manage(whisper_service);
            app.manage(llm_model_manager);
            app.manage(model_settings_manager);
            app.manage(model_downloader);

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
            is_whisper_initialized,
            // File management commands (Phase 2)
            file_management::get_all_recordings_fm,
            file_management::get_recording_by_id,
            file_management::search_recordings,
            file_management::update_recording_metadata,
            file_management::delete_recording_fm,
            file_management::get_recording_stats,
            file_management::get_all_categories,
            file_management::get_all_tags,
            file_management::get_transcriptions_by_recording,
            file_management::get_transcription_by_id,
            file_management::export_recording_data,
            file_management::get_recordings_count_fm,
            file_management::cleanup_orphaned_files,
            // LLM commands (Phase 3)
            llm::generate_summary,
            llm::get_summary_by_id,
            llm::get_summaries_for_transcription,
            llm::update_summary,
            llm::delete_summary,
            llm::check_llm_connection,
            llm::get_default_llm_config,
            llm::validate_llm_config,
            llm::get_available_llm_providers,
            llm::get_provider_default_config,
            llm::test_summarization,
            // Streaming commands (Phase 3)
            streaming::generate_summary_with_progress,
            streaming::cancel_summarization,
            streaming::get_summarization_status,
            // Model Management commands (Phase 4)
            model_management::discover_available_models,
            model_management::get_cached_models,
            model_management::benchmark_model,
            model_management::get_cached_benchmarks,
            model_management::get_recommended_models,
            model_management::validate_model_availability,
            model_management::get_model_capabilities,
            model_management::estimate_processing_time,
            // Model Settings commands (Phase 4)
            model_settings::get_model_settings,
            model_settings::save_model_settings,
            model_settings::set_default_model,
            model_settings::set_use_case_default,
            model_settings::add_model_preference,
            model_settings::remove_model_preference,
            model_settings::set_performance_priority,
            model_settings::set_auto_switch_enabled,
            model_settings::get_optimal_model_for_use_case,
            model_settings::get_enabled_models_by_priority,
            model_settings::validate_model_settings,
            model_settings::reset_model_settings,
            model_settings::export_model_settings,
            model_settings::import_model_settings,
            model_settings::get_performance_recommendations,
            // Model Downloader commands (Phase 4)
            model_downloader::get_downloadable_models,
            model_downloader::get_models_by_category,
            model_downloader::check_system_requirements,
            model_downloader::start_model_download,
            model_downloader::get_download_command,
            model_downloader::search_models,
            model_downloader::get_popular_models,
            model_downloader::get_gpt4all_download_info,
            model_downloader::validate_model_download_requirements,
            model_downloader::get_recommended_models_for_system,
            model_downloader::estimate_download_time,
            model_downloader::get_model_categories,
            model_downloader::get_model_tags
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
