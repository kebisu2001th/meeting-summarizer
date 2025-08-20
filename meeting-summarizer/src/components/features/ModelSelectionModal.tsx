import { useState, useEffect } from 'react';
import { useAtom } from 'jotai';
import { Modal } from '../ui/modal';
import { Select } from '../ui/select';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { Loader2, CheckCircle, AlertCircle, Settings, Zap, Brain, MemoryStick } from 'lucide-react';
import { LLMService } from '../../services/llm';
import { ModelInfo, PerformancePriority } from '../../types/llm';
import { 
  availableModelsAtom, 
  modelSettingsAtom, 
  selectedModelAtom, 
  modelsLoadingAtom,
  modelTestingAtom,
  updateModelSelectionAtom,
  setModelTestingAtom,
  updateModelStatusAtom 
} from '../../atoms/llm';

interface ModelSelectionModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ModelSelectionModal({ isOpen, onClose }: ModelSelectionModalProps) {
  const [models, setModels] = useAtom(availableModelsAtom);
  const [settings, setSettings] = useAtom(modelSettingsAtom);
  const [loading, setLoading] = useAtom(modelsLoadingAtom);
  const [selectedModel] = useAtom(selectedModelAtom);
  const [testingModel] = useAtom(modelTestingAtom);
  const [, updateSelection] = useAtom(updateModelSelectionAtom);
  const [, setTesting] = useAtom(setModelTestingAtom);
  const [, updateStatus] = useAtom(updateModelStatusAtom);
  const [activeTab, setActiveTab] = useState<'selection' | 'settings' | 'download'>('selection');

  useEffect(() => {
    if (isOpen) {
      loadData();
    }
  }, [isOpen]);

  const loadData = async () => {
    setLoading(true);
    try {
      const [availableModels, currentSettings] = await Promise.all([
        LLMService.discoverAvailableModels(),
        LLMService.getModelSettings()
      ]);
      
      setModels(availableModels);
      setSettings(currentSettings);
      updateSelection(currentSettings.defaultModel || '');
    } catch (error) {
      console.error('Failed to load model data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleModelChange = async (modelId: string) => {
    updateSelection(modelId);
    try {
      await LLMService.setDefaultModel(modelId);
      if (settings) {
        setSettings({ ...settings, defaultModel: modelId });
      }
      updateStatus({ isConnected: false, error: null });
    } catch (error: any) {
      console.error('Failed to set default model:', error);
      updateStatus({ isConnected: false, error: error?.message || 'Unknown error' });
    }
  };

  const handleTestModel = async (modelId: string) => {
    setTesting(modelId);
    try {
      const config = await LLMService.getProviderDefaultConfig('Ollama');
      const result = await LLMService.testSummarization(config, 
        'これはテスト用のサンプルテキストです。LLMの応答速度と品質を確認しています。');
      console.log('Test result:', result);
      updateStatus({ isConnected: true, error: null });
    } catch (error: any) {
      console.error('Model test failed:', error);
      updateStatus({ isConnected: false, error: error?.message || 'Test failed' });
    } finally {
      setTesting('');
    }
  };

  const getPerformanceIcon = (performance: string) => {
    switch (performance) {
      case 'fast': return <Zap className="w-4 h-4 text-green-500" />;
      case 'high': return <Brain className="w-4 h-4 text-blue-500" />;
      case 'low': return <MemoryStick className="w-4 h-4 text-gray-500" />;
      default: return <Settings className="w-4 h-4 text-gray-400" />;
    }
  };

  const getStatusBadge = (model: ModelInfo) => {
    if (model.isAvailable) {
      return <Badge variant="default" className="bg-green-100 text-green-800">利用可能</Badge>;
    } else {
      return <Badge variant="secondary" className="bg-gray-100 text-gray-600">未インストール</Badge>;
    }
  };

  if (loading) {
    return (
      <Modal isOpen={isOpen} onClose={onClose} title="LLM モデル設定">
        <div className="flex items-center justify-center py-8">
          <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
          <span className="ml-2 text-gray-600">モデル情報を読み込み中...</span>
        </div>
      </Modal>
    );
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="LLM モデル設定">
      <div className="space-y-6">
        {/* Tab Navigation */}
        <div className="flex space-x-1 bg-gray-100 p-1 rounded-lg">
          <button
            className={`flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors ${
              activeTab === 'selection' 
                ? 'bg-white text-gray-900 shadow-sm' 
                : 'text-gray-600 hover:text-gray-900'
            }`}
            onClick={() => setActiveTab('selection')}
          >
            モデル選択
          </button>
          <button
            className={`flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors ${
              activeTab === 'settings' 
                ? 'bg-white text-gray-900 shadow-sm' 
                : 'text-gray-600 hover:text-gray-900'
            }`}
            onClick={() => setActiveTab('settings')}
          >
            詳細設定
          </button>
          <button
            className={`flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors ${
              activeTab === 'download' 
                ? 'bg-white text-gray-900 shadow-sm' 
                : 'text-gray-600 hover:text-gray-900'
            }`}
            onClick={() => setActiveTab('download')}
          >
            モデル管理
          </button>
        </div>

        {/* Model Selection Tab */}
        {activeTab === 'selection' && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                現在のデフォルトモデル
              </label>
              <Select
                options={models.map(model => ({
                  value: model.id,
                  label: model.name,
                  description: `${model.provider} • ${model.size} • ${model.description}`
                }))}
                value={selectedModel}
                onChange={handleModelChange}
                placeholder="モデルを選択してください"
              />
            </div>

            {/* Model Cards */}
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {models.map((model) => (
                <Card key={model.id} className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="font-medium text-gray-900">{model.name}</h3>
                        {getStatusBadge(model)}
                        {selectedModel === model.id && (
                          <Badge variant="default" className="bg-blue-100 text-blue-800">
                            <CheckCircle className="w-3 h-3 mr-1" />
                            選択中
                          </Badge>
                        )}
                      </div>
                      
                      <p className="text-sm text-gray-600 mb-3">{model.description}</p>
                      
                      <div className="flex items-center gap-4 text-xs text-gray-500">
                        <div className="flex items-center gap-1">
                          <span className="font-medium">プロバイダー:</span>
                          <span>{model.provider}</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <span className="font-medium">サイズ:</span>
                          <span>{model.size}</span>
                        </div>
                      </div>

                      {/* Performance Indicators */}
                      <div className="flex items-center gap-4 mt-3">
                        <div className="flex items-center gap-1">
                          {getPerformanceIcon(model.performance.speed)}
                          <span className="text-xs text-gray-600">速度: {model.performance.speed}</span>
                        </div>
                        <div className="flex items-center gap-1">
                          {getPerformanceIcon(model.performance.accuracy)}
                          <span className="text-xs text-gray-600">精度: {model.performance.accuracy}</span>
                        </div>
                        <div className="flex items-center gap-1">
                          {getPerformanceIcon(model.performance.memoryUsage)}
                          <span className="text-xs text-gray-600">メモリ: {model.performance.memoryUsage}</span>
                        </div>
                      </div>

                      {/* System Requirements */}
                      <div className="mt-3 p-2 bg-gray-50 rounded text-xs text-gray-600">
                        <div className="flex items-center gap-4">
                          <span>RAM: {model.requirements.minRam}</span>
                          <span>容量: {model.requirements.diskSpace}</span>
                          {model.requirements.gpuRequired && (
                            <span className="flex items-center gap-1">
                              <AlertCircle className="w-3 h-3" />
                              GPU必須
                            </span>
                          )}
                        </div>
                      </div>
                    </div>

                    <div className="flex flex-col gap-2 ml-4">
                      {model.isAvailable ? (
                        <>
                          <Button
                            size="sm"
                            variant={selectedModel === model.id ? "default" : "outline"}
                            onClick={() => handleModelChange(model.id)}
                          >
                            {selectedModel === model.id ? '選択中' : '選択'}
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => handleTestModel(model.id)}
                            disabled={testingModel === model.id}
                          >
                            {testingModel === model.id ? (
                              <Loader2 className="w-3 h-3 animate-spin mr-1" />
                            ) : null}
                            テスト
                          </Button>
                        </>
                      ) : (
                        <Button
                          size="sm"
                          variant="outline"
                          disabled
                          className="text-gray-400"
                        >
                          インストール必要
                        </Button>
                      )}
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          </div>
        )}

        {/* Settings Tab */}
        {activeTab === 'settings' && settings && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                パフォーマンス優先度
              </label>
              <Select
                options={[
                  { value: PerformancePriority.Speed, label: '速度優先', description: '応答速度を重視' },
                  { value: PerformancePriority.Accuracy, label: '精度優先', description: '回答品質を重視' },
                  { value: PerformancePriority.Balance, label: 'バランス', description: '速度と精度のバランス' },
                  { value: PerformancePriority.MemoryEfficiency, label: 'メモリ効率', description: 'メモリ使用量を最小化' }
                ]}
                value={settings.performancePriority}
                onChange={(value) => setSettings({ ...settings, performancePriority: value as PerformancePriority })}
              />
            </div>

            <div className="flex items-center gap-3">
              <input
                type="checkbox"
                id="autoSwitch"
                checked={settings.autoSwitchEnabled}
                onChange={(e) => setSettings({ ...settings, autoSwitchEnabled: e.target.checked })}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <label htmlFor="autoSwitch" className="text-sm text-gray-700">
                用途に応じた自動モデル切り替えを有効にする
              </label>
            </div>

            <div>
              <h4 className="font-medium text-gray-900 mb-2">用途別デフォルトモデル</h4>
              <div className="space-y-3">
                {Object.entries(settings.useCaseDefaults).map(([useCase, modelId]) => (
                  <div key={useCase} className="flex items-center gap-3">
                    <span className="text-sm text-gray-600 w-24 capitalize">{useCase}:</span>
                    <Select
                      options={models.map(model => ({
                        value: model.id,
                        label: model.name
                      }))}
                      value={modelId}
                      onChange={(value) => setSettings({
                        ...settings,
                        useCaseDefaults: { ...settings.useCaseDefaults, [useCase]: value }
                      })}
                    />
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Download Tab */}
        {activeTab === 'download' && (
          <div className="space-y-4">
            <div className="text-center py-8 text-gray-500">
              <Settings className="w-12 h-12 mx-auto mb-2 text-gray-400" />
              <p>モデルのダウンロードと管理機能</p>
              <p className="text-sm">近日公開予定</p>
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="flex justify-end gap-3 pt-4 border-t border-gray-200">
          <Button variant="outline" onClick={onClose}>
            キャンセル
          </Button>
          <Button onClick={onClose}>
            保存
          </Button>
        </div>
      </div>
    </Modal>
  );
}