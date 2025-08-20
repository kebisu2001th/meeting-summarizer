import { useState } from 'react';
import { useAtom } from 'jotai';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { Loader2, Sparkles, AlertCircle, Info } from 'lucide-react';
import { currentModelAtom, modelStatusAtom } from '../../atoms/llm';
import { LLMService } from '../../services/llm';

interface SummarizationPanelProps {
  transcriptionText: string;
  onSummaryGenerated: (summary: any) => void;
}

export function SummarizationPanel({ 
  transcriptionText, 
  onSummaryGenerated 
}: SummarizationPanelProps) {
  const [currentModel] = useAtom(currentModelAtom);
  const [modelStatus] = useAtom(modelStatusAtom);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerateSummary = async () => {
    if (!currentModel) {
      setError('LLMモデルが選択されていません。設定画面からモデルを選択してください。');
      return;
    }

    if (!currentModel.isAvailable) {
      setError(`モデル "${currentModel.name}" がインストールされていません。`);
      return;
    }

    setIsGenerating(true);
    setError(null);

    try {
      // Get current model configuration
      const config = await LLMService.getProviderDefaultConfig(currentModel.provider);
      config.modelName = currentModel.name;

      // Generate summary with progress tracking
      const summary = await LLMService.testSummarization(config, transcriptionText);
      
      onSummaryGenerated(summary);
    } catch (err: any) {
      console.error('Summarization failed:', err);
      setError(err?.message || '要約生成中にエラーが発生しました');
    } finally {
      setIsGenerating(false);
    }
  };

  const canGenerateSummary = currentModel?.isAvailable && !isGenerating && transcriptionText.length > 50;

  return (
    <Card className="p-4">
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900">AI要約生成</h3>
          {currentModel && (
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-600">使用モデル:</span>
              <Badge variant="outline">{currentModel.name}</Badge>
              {modelStatus.isConnected ? (
                <Badge variant="default" className="bg-green-100 text-green-800">接続済み</Badge>
              ) : (
                <Badge variant="secondary">未テスト</Badge>
              )}
            </div>
          )}
        </div>

        {/* Model Status Messages */}
        {!currentModel && (
          <div className="flex items-center gap-2 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
            <AlertCircle className="w-5 h-5 text-yellow-600" />
            <p className="text-sm text-yellow-800">
              LLMモデルが選択されていません。ヘッダーの「LLM設定」からモデルを選択してください。
            </p>
          </div>
        )}

        {currentModel && !currentModel.isAvailable && (
          <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg">
            <AlertCircle className="w-5 h-5 text-red-600" />
            <p className="text-sm text-red-800">
              選択されたモデル "{currentModel.name}" がインストールされていません。
              モデルの管理画面からインストールしてください。
            </p>
          </div>
        )}

        {error && (
          <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg">
            <AlertCircle className="w-5 h-5 text-red-600" />
            <p className="text-sm text-red-800">{error}</p>
          </div>
        )}

        {transcriptionText.length < 50 && (
          <div className="flex items-center gap-2 p-3 bg-blue-50 border border-blue-200 rounded-lg">
            <Info className="w-5 h-5 text-blue-600" />
            <p className="text-sm text-blue-800">
              要約を生成するには、最低50文字以上の書き起こしテキストが必要です。
            </p>
          </div>
        )}

        {/* Summary Generation Button */}
        <div className="flex justify-center">
          <Button
            onClick={handleGenerateSummary}
            disabled={!canGenerateSummary}
            className="flex items-center gap-2 px-6 py-3"
          >
            {isGenerating ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Sparkles className="w-4 h-4" />
            )}
            {isGenerating ? 'AI要約生成中...' : 'AI要約を生成'}
          </Button>
        </div>

        {/* Model Performance Info */}
        {currentModel && currentModel.isAvailable && (
          <div className="p-3 bg-gray-50 rounded-lg">
            <h4 className="text-sm font-medium text-gray-900 mb-2">モデル性能情報</h4>
            <div className="grid grid-cols-3 gap-4 text-xs text-gray-600">
              <div>
                <span className="font-medium">速度:</span> {currentModel.performance.speed}
              </div>
              <div>
                <span className="font-medium">精度:</span> {currentModel.performance.accuracy}
              </div>
              <div>
                <span className="font-medium">メモリ:</span> {currentModel.performance.memoryUsage}
              </div>
            </div>
            <div className="mt-2 text-xs text-gray-600">
              <span className="font-medium">要件:</span> 
              RAM {currentModel.requirements.minRam}、
              容量 {currentModel.requirements.diskSpace}
              {currentModel.requirements.gpuRequired && '、GPU必須'}
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}