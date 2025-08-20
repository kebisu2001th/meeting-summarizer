// LLM Model types
export interface LLMConfig {
  provider: LLMProvider;
  baseUrl: string;
  modelName: string;
  temperature: number;
  maxTokens: number;
  timeoutSeconds: number;
}

export enum LLMProvider {
  Ollama = 'Ollama',
  OpenAI = 'OpenAI', 
  GPT4All = 'GPT4All',
  LMStudio = 'LMStudio',
  Custom = 'Custom'
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: LLMProvider;
  description: string;
  isAvailable: boolean;
  size: string;
  languages: string[];
  capabilities: string[];
  performance: {
    speed: 'fast' | 'medium' | 'slow';
    accuracy: 'high' | 'medium' | 'low';
    memoryUsage: 'low' | 'medium' | 'high';
  };
  requirements: {
    minRam: string;
    diskSpace: string;
    gpuRequired: boolean;
  };
}

export interface ModelBenchmark {
  modelId: string;
  processingTime: number;
  accuracy: number;
  memoryUsage: number;
  timestamp: string;
}

export interface ModelSettings {
  defaultModel: string;
  useCaseDefaults: {
    [useCase: string]: string;
  };
  preferences: ModelPreference[];
  performancePriority: PerformancePriority;
  autoSwitchEnabled: boolean;
}

export interface ModelPreference {
  modelId: string;
  priority: number;
  useCase: string;
  conditions: {
    minAccuracy?: number;
    maxResponseTime?: number;
    maxMemoryUsage?: number;
  };
}

export enum PerformancePriority {
  Speed = 'speed',
  Accuracy = 'accuracy', 
  Balance = 'balance',
  MemoryEfficiency = 'memory'
}

export interface DownloadableModel {
  id: string;
  name: string;
  provider: LLMProvider;
  category: ModelCategory;
  tags: string[];
  size: string;
  description: string;
  popularity: number;
  systemRequirements: {
    minRam: string;
    recommendedRam: string;
    diskSpace: string;
    gpuSupport: boolean;
  };
  downloadInfo: {
    url?: string;
    installCommand?: string;
    instructions: string;
  };
}

export enum ModelCategory {
  Conversation = 'conversation',
  Coding = 'coding',
  Creative = 'creative',
  Analysis = 'analysis',
  Summarization = 'summarization',
  Translation = 'translation'
}