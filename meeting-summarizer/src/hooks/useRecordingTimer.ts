import { useEffect, useRef } from 'react';
import { useAtom, useSetAtom } from 'jotai';
import { recordingStateAtom, updateRecordingDurationAtom } from '../atoms/recording';

export const useRecordingTimer = () => {
  const [recordingState] = useAtom(recordingStateAtom);
  const updateDuration = useSetAtom(updateRecordingDurationAtom);
  const intervalRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);

  useEffect(() => {
    if (recordingState.isRecording && !recordingState.isPaused) {
      // 録音開始
      startTimeRef.current = Date.now();
      
      intervalRef.current = setInterval(() => {
        const currentTime = Date.now();
        const elapsedSeconds = Math.floor((currentTime - startTimeRef.current) / 1000);
        updateDuration(elapsedSeconds);
      }, 1000);
    } else {
      // 録音停止または一時停止
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [recordingState.isRecording, recordingState.isPaused, updateDuration]);

  return recordingState.duration;
};