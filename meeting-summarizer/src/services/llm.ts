// Mock implementation for development - replace with actual Tauri invoke when integrated
const mockInvoke = async (command: string, args?: any): Promise<any> => {
  console.log('Mock Tauri invoke:', command, args);
  
  // Mock responses for different commands
  switch (command) {
    case 'discover_available_models':
      return [
        {
          id: 'ollama-llama3.2-3b',
          name: 'Llama 3.2 3B',
          provider: 'Ollama',
          description: '高速で軽量なモデル、日常的なタスクに最適',
          isAvailable: true,
          size: '2.0GB',
          languages: ['en', 'ja'],
          capabilities: ['chat', 'summarization'],
          performance: {
            speed: 'fast',
            accuracy: 'medium',
            memoryUsage: 'low'
          },
          requirements: {
            minRam: '4GB',
            diskSpace: '2.5GB',
            gpuRequired: false
          }
        }
      ];
    case 'get_model_settings':
      return {
        defaultModel: 'ollama-llama3.2-3b',
        useCaseDefaults: {
          summarization: 'ollama-llama3.2-3b'
        },
        preferences: [],
        performancePriority: 'balance',
        autoSwitchEnabled: false
      };
    default:
      return null;
  }
};

import { LLMConfig, ModelInfo, ModelBenchmark, ModelSettings, DownloadableModel } from '../types/llm';

export class LLMService {
  // Model Management
  static async discoverAvailableModels(): Promise<ModelInfo[]> {
    return mockInvoke('discover_available_models');
  }

  static async getCachedModels(): Promise<ModelInfo[]> {
    return mockInvoke('get_cached_models');
  }

  static async benchmarkModel(modelId: string, sampleText?: string): Promise<ModelBenchmark> {
    return mockInvoke('benchmark_model', { modelId, sampleText });
  }

  static async getCachedBenchmarks(): Promise<ModelBenchmark[]> {
    return mockInvoke('get_cached_benchmarks');
  }

  static async getRecommendedModels(useCase?: string): Promise<ModelInfo[]> {
    return mockInvoke('get_recommended_models', { useCase });
  }

  static async validateModelAvailability(modelId: string): Promise<boolean> {
    return mockInvoke('validate_model_availability', { modelId });
  }

  static async getModelCapabilities(modelId: string): Promise<string[]> {
    return mockInvoke('get_model_capabilities', { modelId });
  }

  static async estimateProcessingTime(modelId: string, inputLength: number): Promise<number> {
    return mockInvoke('estimate_processing_time', { modelId, inputLength });
  }

  // Model Settings
  static async getModelSettings(): Promise<ModelSettings> {
    return mockInvoke('get_model_settings');
  }

  static async saveModelSettings(settings: ModelSettings): Promise<void> {
    return mockInvoke('save_model_settings', { settings });
  }

  static async setDefaultModel(modelId: string): Promise<void> {
    return mockInvoke('set_default_model', { modelId });
  }

  static async setUseCaseDefault(useCase: string, modelId: string): Promise<void> {
    return mockInvoke('set_use_case_default', { useCase, modelId });
  }

  static async getOptimalModelForUseCase(useCase: string): Promise<string | null> {
    return mockInvoke('get_optimal_model_for_use_case', { useCase });
  }

  static async getEnabledModelsByPriority(): Promise<string[]> {
    return mockInvoke('get_enabled_models_by_priority');
  }

  // Model Downloader
  static async getDownloadableModels(): Promise<DownloadableModel[]> {
    return mockInvoke('get_downloadable_models');
  }

  static async getModelsByCategory(category: string): Promise<DownloadableModel[]> {
    return mockInvoke('get_models_by_category', { category });
  }

  static async checkSystemRequirements(modelId: string): Promise<boolean> {
    return mockInvoke('check_system_requirements', { modelId });
  }

  static async startModelDownload(modelId: string): Promise<string> {
    return mockInvoke('start_model_download', { modelId });
  }

  static async getDownloadCommand(modelId: string): Promise<string> {
    return mockInvoke('get_download_command', { modelId });
  }

  static async searchModels(query: string, category?: string): Promise<DownloadableModel[]> {
    return mockInvoke('search_models', { query, category });
  }

  static async getPopularModels(limit?: number): Promise<DownloadableModel[]> {
    return mockInvoke('get_popular_models', { limit });
  }

  // LLM Config
  static async getDefaultLLMConfig(): Promise<LLMConfig> {
    return mockInvoke('get_default_llm_config');
  }

  static async validateLLMConfig(config: LLMConfig): Promise<boolean> {
    return mockInvoke('validate_llm_config', { config });
  }

  static async getAvailableLLMProviders(): Promise<string[]> {
    return mockInvoke('get_available_llm_providers');
  }

  static async getProviderDefaultConfig(provider: string): Promise<LLMConfig> {
    return mockInvoke('get_provider_default_config', { provider });
  }

  static async checkLLMConnection(config: LLMConfig): Promise<boolean> {
    return mockInvoke('check_llm_connection', { config });
  }

  static async testSummarization(config: LLMConfig, sampleText: string): Promise<any> {
    return mockInvoke('test_summarization', { config, sampleText });
  }
}