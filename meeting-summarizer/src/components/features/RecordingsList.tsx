import { useAtom, useSetAtom } from 'jotai';
import { useEffect, useState } from 'react';
import { Trash2, FileAudio, FileText, Loader2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { 
  recordingsAtom, 
  deleteRecordingAtom, 
  loadRecordingsAtom 
} from '../../atoms/recording';
import { formatDuration, formatDate, formatFileSize } from '../../lib/utils';
import { Recording, Transcription } from '../../types/recording';
import { TauriService } from '../../services/tauri';

interface RecordingItemProps {
  recording: Recording;
  onDelete: (id: string) => void;
}

function RecordingItem({ recording, onDelete }: RecordingItemProps) {
  const [transcription, setTranscription] = useState<Transcription | null>(null);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [showTranscription, setShowTranscription] = useState(false);

  const handleDelete = async () => {
    if (window.confirm(`Are you sure you want to delete "${recording.filename}"?`)) {
      try {
        console.log('Attempting to delete recording:', recording.id);
        await onDelete(recording.id);
        console.log('Successfully deleted recording:', recording.id);
      } catch (error) {
        console.error('Failed to delete recording:', error);
        alert(`削除に失敗しました: ${error}`);
      }
    }
  };

  const handleTranscribe = async () => {
    if (isTranscribing) return;

    setIsTranscribing(true);
    try {
      console.log('Starting transcription for recording:', recording.id);
      
      // Whisperサービスの初期化確認
      const isInitialized = await TauriService.isWhisperInitialized();
      if (!isInitialized) {
        console.log('Initializing Whisper service...');
        await TauriService.initializeWhisper();
      }
      
      const result = await TauriService.transcribeRecording(recording.id, 'ja');
      console.log('Transcription completed:', result);
      setTranscription(result);
      setShowTranscription(true);
    } catch (error) {
      console.error('Failed to transcribe recording:', error);
      alert(`書き起こしに失敗しました: ${error}`);
    } finally {
      setIsTranscribing(false);
    }
  };

  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3 flex-1 min-w-0">
            {/* ファイルアイコン */}
            <div className="flex-shrink-0">
              <div className="flex items-center justify-center w-10 h-10 bg-blue-100 rounded-lg">
                <FileAudio className="w-5 h-5 text-blue-600" />
              </div>
            </div>

            {/* ファイル情報 */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center space-x-2">
                <h3 className="text-sm font-medium text-gray-900 truncate">
                  {recording.filename}
                </h3>
                <span className="text-xs text-gray-500 font-mono">
                  {formatDuration(recording.duration || 0)}
                </span>
              </div>
              <div className="flex items-center space-x-4 mt-1">
                <span className="text-xs text-gray-500">
                  {formatDate(new Date(recording.created_at))}
                </span>
                {recording.file_size && (
                  <span className="text-xs text-gray-500">
                    {formatFileSize(recording.file_size)}
                  </span>
                )}
              </div>
            </div>
          </div>

          {/* アクションボタン */}
          <div className="flex items-center space-x-1 flex-shrink-0">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleTranscribe}
              disabled={isTranscribing}
              className="h-8 w-8 p-0 text-blue-600 hover:text-blue-700 hover:bg-blue-50"
              title="Transcribe audio"
            >
              {isTranscribing ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <FileText className="w-4 h-4" />
              )}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleDelete}
              className="h-8 w-8 p-0 text-red-600 hover:text-red-700 hover:bg-red-50"
              title="Delete recording"
            >
              <Trash2 className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* 書き起こし結果表示 */}
        {showTranscription && transcription && (
          <div className="mt-4 pt-4 border-t border-gray-200">
            <div className="flex items-center justify-between mb-2">
              <h4 className="text-sm font-medium text-gray-900">書き起こし結果</h4>
              <div className="flex items-center space-x-2 text-xs text-gray-500">
                {transcription.confidence && (
                  <span>信頼度: {transcription.confidence ? Math.round(transcription.confidence * 100) : 0}%</span>
                )}
                {transcription.processing_time_ms && (
                  <span>処理時間: {transcription.processing_time_ms}ms</span>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowTranscription(false)}
                  className="h-6 w-6 p-0 text-gray-400 hover:text-gray-600"
                >
                  ×
                </Button>
              </div>
            </div>
            <div className="bg-gray-50 rounded-md p-3">
              <p className="text-sm text-gray-800 whitespace-pre-wrap">
                {transcription.text || '書き起こし結果がありません'}
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function RecordingsList() {
  const [recordings] = useAtom(recordingsAtom);
  const deleteRecording = useSetAtom(deleteRecordingAtom);
  const loadRecordings = useSetAtom(loadRecordingsAtom);

  // コンポーネントマウント時に録音一覧を読み込む
  useEffect(() => {
    loadRecordings();
  }, [loadRecordings]);

  const handleDelete = async (id: string) => {
    try {
      console.log('RecordingsList: handleDelete called with id:', id);
      const success = await deleteRecording(id);
      console.log('RecordingsList: deleteRecording result:', success);
      
      if (success) {
        // 削除成功後、リストを再読み込み
        await loadRecordings();
        console.log('RecordingsList: refreshed recordings list');
      }
    } catch (error) {
      console.error('RecordingsList: handleDelete error:', error);
      alert(`削除に失敗しました: ${error}`);
    }
  };

  if (recordings.length === 0) {
    return (
      <Card>
        <CardContent className="pt-8 pb-8">
          <div className="text-center space-y-3">
            <div className="flex justify-center">
              <div className="flex items-center justify-center w-16 h-16 bg-gray-100 rounded-full">
                <FileAudio className="w-8 h-8 text-gray-400" />
              </div>
            </div>
            <div className="space-y-1">
              <h3 className="text-lg font-medium text-gray-900">No recordings yet</h3>
              <p className="text-sm text-gray-500">
                Start recording to see your audio files here
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-900">
          Recent Recordings ({recordings.length})
        </h2>
      </div>
      
      <div className="space-y-3">
        {recordings.map((recording) => (
          <RecordingItem
            key={recording.id}
            recording={recording}
            onDelete={handleDelete}
          />
        ))}
      </div>
    </div>
  );
}