import { useAtom, useSetAtom } from 'jotai';
import { Mic, Square, Volume2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { 
  recordingStateAtom, 
  startRecordingAtom, 
  stopRecordingAtom 
} from '../../atoms/recording';
import { useRecordingTimer } from '../../hooks/useRecordingTimer';
import { formatDuration, cn } from '../../lib/utils';
import { useState, useEffect } from 'react';

export function RecordingControls() {
  const [recordingState] = useAtom(recordingStateAtom);
  const startRecording = useSetAtom(startRecordingAtom);
  const stopRecording = useSetAtom(stopRecordingAtom);
  const duration = useRecordingTimer();
  const [audioLevel, setAudioLevel] = useState(0);

  const handleRecordingToggle = async () => {

    console.log('handleRecordingToggle');
    try {
      if (recordingState.isRecording) {
        console.log('stopRecording');
        await stopRecording();
      } else {
        console.log('startRecording');
        await startRecording();
      }
    } catch (error) {
      console.error('Recording error:', error);
      // TODO: Show user-friendly error message
    }
  };

  // シミュレートされた音声レベル（実際の実装では音声入力レベルを取得）
  useEffect(() => {
    if (recordingState.isRecording) {
      const interval = setInterval(() => {
        // ランダムな音声レベルをシミュレート（実際の実装では実際の音声レベルを取得）
        const level = Math.random() * 100;
        setAudioLevel(level);
      }, 100);
      
      return () => clearInterval(interval);
    } else {
      setAudioLevel(0);
    }
  }, [recordingState.isRecording]);

  const getRecordingStatus = () => {
    if (recordingState.isRecording) {
      return recordingState.isPaused ? 'Paused' : 'Recording...';
    }
    return 'Ready to record';
  };

  const getRecordingStatusColor = () => {
    if (recordingState.isRecording) {
      return recordingState.isPaused ? 'text-yellow-600' : 'text-red-600';
    }
    return 'text-gray-600';
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardContent className="pt-8 pb-8">
        <div className="flex flex-col items-center space-y-6">
          {/* 録音ボタン */}
          <div className="relative">
            <Button
              onClick={handleRecordingToggle}
              size="lg"
              className={cn(
                "relative z-10 w-24 h-24 rounded-full transition-all duration-200 ease-in-out cursor-pointer",
                "hover:scale-105 active:scale-95",
                recordingState.isRecording
                  ? "bg-red-500 hover:bg-red-600 text-white shadow-red-200 shadow-lg"
                  : "bg-green-500 hover:bg-green-600 text-white shadow-green-200 shadow-lg"
              )}
            >
              {recordingState.isRecording ? (
                <Square className="w-8 h-8" fill="currentColor" /> 
              ) : (
                <Mic className="w-8 h-8" />
              )}
            </Button>
            
            {/* 録音中のパルスエフェクト */}
            {recordingState.isRecording && !recordingState.isPaused && (
              <div className="pointer-events-none absolute inset-0 z-0 rounded-full bg-red-400 animate-ping opacity-25" />
            )}
          </div>

          {/* 録音状態表示 */}
          <div className="text-center space-y-2">
            <div className={cn("text-lg font-medium", getRecordingStatusColor())}>
              {getRecordingStatus()}
            </div>
            
            {/* 録音時間表示 */}
            <div className="text-3xl font-mono font-bold text-gray-900">
              {formatDuration(duration)}
            </div>

            {/* 音声レベル表示 */}
            {recordingState.isRecording && (
              <div className="w-full max-w-xs">
                <div className="flex items-center space-x-2 mb-1">
                  <Volume2 className="w-4 h-4 text-gray-500" />
                  <span className="text-xs text-gray-500">音声レベル</span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div 
                    className="bg-green-500 h-2 rounded-full transition-all duration-100"
                    style={{ width: `${Math.min(audioLevel, 100)}%` }}
                  />
                </div>
                <div className="flex justify-between text-xs text-gray-400 mt-1">
                  <span>無音</span>
                  <span>最大</span>
                </div>
              </div>
            )}
          </div>

          {/* 録音の説明 */}
          <div className="text-center text-sm text-gray-500 max-w-xs">
            {recordingState.isRecording 
              ? "Click the red button to stop recording"
              : "Click the green button to start recording"
            }
          </div>
        </div>
      </CardContent>
    </Card>
  );
}