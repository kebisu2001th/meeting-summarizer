import { useState } from 'react';
import { Mic, Settings } from 'lucide-react';
import { Button } from '../ui/button';
import { ModelSelectionModal } from '../features/ModelSelectionModal';
import { ModelStatusIndicator } from '../features/ModelStatusIndicator';

export function Header() {
  const [isModelModalOpen, setIsModelModalOpen] = useState(false);

  return (
    <header className="bg-white border-b border-gray-200 px-6 py-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 bg-blue-100 rounded-lg">
            <Mic className="w-5 h-5 text-blue-600" />
          </div>
          <div>
            <h1 className="text-xl font-semibold text-gray-900">Meeting Summarizer</h1>
            <p className="text-sm text-gray-500">Record and summarize your meetings</p>
          </div>
        </div>
        
        <div className="flex items-center gap-3">
          <ModelStatusIndicator />
          <Button
            variant="outline"
            size="sm"
            onClick={() => setIsModelModalOpen(true)}
            className="flex items-center gap-2"
          >
            <Settings className="w-4 h-4" />
            LLM 設定
          </Button>
        </div>
      </div>

      <ModelSelectionModal
        isOpen={isModelModalOpen}
        onClose={() => setIsModelModalOpen(false)}
      />
    </header>
  );
}