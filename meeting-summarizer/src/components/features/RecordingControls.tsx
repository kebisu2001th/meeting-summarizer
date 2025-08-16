import { useAtom, useSetAtom } from 'jotai';
import { Mic, Square } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { 
  recordingStateAtom, 
  startRecordingAtom, 
  stopRecordingAtom 
} from '../../atoms/recording';
import { useRecordingTimer } from '../../hooks/useRecordingTimer';
import { formatDuration, cn } from '../../lib/utils';

export function RecordingControls() {
  const [recordingState] = useAtom(recordingStateAtom);
  const startRecording = useSetAtom(startRecordingAtom);
  const stopRecording = useSetAtom(stopRecordingAtom);
  const duration = useRecordingTimer();

  const handleRecordingToggle = async () => {
    try {
      if (recordingState.isRecording) {
        await stopRecording();
      } else {
        await startRecording();
      }
    } catch (error) {
      console.error('Recording error:', error);
      // TODO: Show user-friendly error message
    }
  };

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
                "w-24 h-24 rounded-full transition-all duration-200 ease-in-out",
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
              <div className="absolute inset-0 rounded-full bg-red-400 animate-ping opacity-25" />
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