import { atom } from 'jotai';
import { Recording, RecordingState } from '../types/recording';
import { TauriService } from '../services/tauri';

// 録音状態の管理
export const recordingStateAtom = atom<RecordingState>({
  isRecording: false,
  duration: 0,
  isPaused: false,
});

// 録音中の状態を監視するDerivedAtom
export const isRecordingAtom = atom(
  (get) => get(recordingStateAtom).isRecording
);

// 録音時間を監視するDerivedAtom
export const recordingDurationAtom = atom(
  (get) => get(recordingStateAtom).duration
);

// 録音ファイル一覧の管理
export const recordingsAtom = atom<Recording[]>([]);

// 現在の録音セッション情報
export const currentRecordingAtom = atom<Recording | null>(null);

// 録音ファイル読み込みAction（Tauri統合）
export const loadRecordingsAtom = atom(
  null,
  async (_get, set) => {
    try {
      const recordings = await TauriService.getRecordings();
      set(recordingsAtom, recordings);
    } catch (error) {
      console.error('Failed to load recordings:', error);
    }
  }
);

// 録音開始Action（Tauri統合）
export const startRecordingAtom = atom(
  null,
  async (get, set) => {
    const currentState = get(recordingStateAtom);
    if (currentState.isRecording) {
      throw new Error('Recording is already in progress');
    }

    try {
      const sessionId = await TauriService.startRecording();
      
      set(recordingStateAtom, {
        isRecording: true,
        duration: 0,
        isPaused: false,
      });

      // 現在の録音セッション情報を設定
      set(currentRecordingAtom, {
        id: sessionId,
        filename: 'Recording...',
        duration: 0,
        createdAt: new Date(),
      });
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  }
);

// 録音停止Action（Tauri統合）
export const stopRecordingAtom = atom(
  null,
  async (get, set) => {
    const currentState = get(recordingStateAtom);
    if (!currentState.isRecording) {
      throw new Error('No active recording session');
    }

    try {
      const completedRecording = await TauriService.stopRecording();
      
      // 録音状態をリセット
      set(recordingStateAtom, {
        isRecording: false,
        duration: 0,
        isPaused: false,
      });

      // 完了した録音をリストの先頭に追加
      const recordings = get(recordingsAtom);
      set(recordingsAtom, [completedRecording, ...recordings]);
      set(currentRecordingAtom, null);

      return completedRecording;
    } catch (error) {
      console.error('Failed to stop recording:', error);
      throw error;
    }
  }
);

// 録音時間更新Action
export const updateRecordingDurationAtom = atom(
  null,
  (get, set, duration: number) => {
    const currentState = get(recordingStateAtom);
    if (currentState.isRecording) {
      set(recordingStateAtom, {
        ...currentState,
        duration,
      });
    }
  }
);

// 録音削除Action（Tauri統合）
export const deleteRecordingAtom = atom(
  null,
  async (get, set, recordingId: string) => {
    try {
      const success = await TauriService.deleteRecording(recordingId);
      
      if (success) {
        const recordings = get(recordingsAtom);
        const updatedRecordings = recordings.filter(r => r.id !== recordingId);
        set(recordingsAtom, updatedRecordings);
      }
      
      return success;
    } catch (error) {
      console.error('Failed to delete recording:', error);
      throw error;
    }
  }
);