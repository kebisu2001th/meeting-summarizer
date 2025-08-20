import { useAtom } from 'jotai';
import { CheckCircle, AlertCircle, Clock, Loader2 } from 'lucide-react';
import { Badge } from '../ui/badge';
import { currentModelAtom, modelStatusAtom, modelTestingAtom } from '../../atoms/llm';

export function ModelStatusIndicator() {
  const [currentModel] = useAtom(currentModelAtom);
  const [modelStatus] = useAtom(modelStatusAtom);
  const [testingModel] = useAtom(modelTestingAtom);

  if (!currentModel) {
    return (
      <Badge variant="secondary" className="bg-gray-100 text-gray-600">
        <AlertCircle className="w-3 h-3 mr-1" />
        モデル未選択
      </Badge>
    );
  }

  if (testingModel === currentModel.id) {
    return (
      <Badge variant="default" className="bg-blue-100 text-blue-800">
        <Loader2 className="w-3 h-3 mr-1 animate-spin" />
        テスト中
      </Badge>
    );
  }

  if (modelStatus.isConnected) {
    return (
      <Badge variant="default" className="bg-green-100 text-green-800">
        <CheckCircle className="w-3 h-3 mr-1" />
        接続済み
      </Badge>
    );
  }

  if (modelStatus.error) {
    return (
      <Badge variant="destructive" className="bg-red-100 text-red-800">
        <AlertCircle className="w-3 h-3 mr-1" />
        エラー
      </Badge>
    );
  }

  if (currentModel.isAvailable) {
    return (
      <Badge variant="default" className="bg-yellow-100 text-yellow-800">
        <Clock className="w-3 h-3 mr-1" />
        未テスト
      </Badge>
    );
  }

  return (
    <Badge variant="secondary" className="bg-gray-100 text-gray-600">
      <AlertCircle className="w-3 h-3 mr-1" />
      未インストール
    </Badge>
  );
}