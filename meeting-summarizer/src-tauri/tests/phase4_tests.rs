use meeting_summarizer_lib::services::{LLMModelManager, ModelSettingsManager, ModelDownloader};
use tempfile::TempDir;

#[tokio::test]
async fn test_model_manager_initialization() {
    let mut manager = LLMModelManager::new();
    
    // 初期状態では空のモデルリストを持つ
    let cached_models = manager.get_cached_models();
    assert!(cached_models.is_empty());
    
    // プロバイダーによる推奨モデル取得をテスト
    let recommendations = manager.get_recommended_models("テキスト要約");
    assert!(!recommendations.is_empty());
    
    println!("✅ LLMModelManager initialization test passed");
}

#[tokio::test]
async fn test_model_settings_manager() {
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");
    
    let mut manager = ModelSettingsManager::new(settings_path.clone());
    
    // 設定の保存をテスト
    let result = manager.save_settings().await;
    assert!(result.is_ok());
    
    // 設定ファイルが作成されることを確認
    assert!(settings_path.exists());
    
    println!("✅ ModelSettingsManager test passed");
}

#[tokio::test]
async fn test_model_downloader() {
    let mut downloader = ModelDownloader::new();
    
    // ダウンロード可能なモデル一覧を取得
    let models = downloader.get_downloadable_models();
    assert!(!models.is_empty());
    
    // カテゴリー別モデル取得をテスト
    let lightweight_models = downloader.get_models_by_category("lightweight");
    assert!(!lightweight_models.is_empty());
    
    // 人気モデル取得をテスト
    let popular_models = downloader.get_popular_models(5);
    assert!(!popular_models.is_empty());
    assert!(popular_models.len() <= 5);
    
    println!("✅ ModelDownloader test passed");
}

#[tokio::test]
async fn test_model_discovery_workflow() {
    let mut manager = LLMModelManager::new();
    
    // モデル発見の実行（Ollamaが動作していなくても失敗せずに空の結果を返す）
    let result = manager.discover_available_models().await;
    assert!(result.is_ok());
    
    let models = result.unwrap();
    // モデルが見つかった場合の検証
    if !models.is_empty() {
        let first_model = &models[0];
        assert!(!first_model.id.is_empty());
        assert!(!first_model.name.is_empty());
        println!("✅ Found model: {}", first_model.name);
    }
    
    println!("✅ Model discovery workflow test passed");
}

#[tokio::test]
async fn test_model_recommendations() {
    let manager = LLMModelManager::new();
    
    // 異なるユースケースでの推奨モデル取得
    let use_cases = vec![
        "テキスト要約",
        "会議記録",
        "高速処理",
        "高品質分析",
    ];
    
    for use_case in use_cases {
        let recommendations = manager.get_recommended_models(use_case);
        assert!(!recommendations.is_empty(), "No recommendations for use case: {}", use_case);
        println!("✅ {} use case has {} recommendations", use_case, recommendations.len());
    }
    
    println!("✅ Model recommendations test passed");
}