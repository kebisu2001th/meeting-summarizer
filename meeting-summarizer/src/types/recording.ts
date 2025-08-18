// Rust側のstructと完全一致するインターフェース
export interface Recording {
  id: string;
  filename: string;
  file_path: string;
  duration: number | null; // Rust側はOption<i64>
  file_size: number | null; // Rust側はOption<i64>  
  created_at: string; // Rust側はDateTime<Utc> -> ISO string
  updated_at: string; // Rust側はDateTime<Utc> -> ISO string
}

// フロントエンド専用の状態管理インターフェース
export interface RecordingState {
  isRecording: boolean;
  duration: number; // current recording duration in seconds
  isPaused: boolean;
}

// Rust側のTranscriptionと完全一致
export interface Transcription {
  id: string;
  recording_id: string;
  text: string;
  language: string;
  confidence: number | null; // Rust側はOption<f32>
  processing_time_ms: number | null; // Rust側はOption<u64>
  status: TranscriptionStatus;
  created_at: string; // Rust側はDateTime<Utc> -> ISO string
  updated_at: string; // Rust側はDateTime<Utc> -> ISO string
}

// Rust側のenumと完全一致
export type TranscriptionStatus = 
  | 'Pending'
  | 'Processing'
  | 'Completed'
  | { Failed: string };

// 型ガード関数：ランタイム検証
export function isValidRecording(obj: any): obj is Recording {
  return (
    obj &&
    typeof obj.id === 'string' &&
    typeof obj.filename === 'string' &&
    typeof obj.file_path === 'string' &&
    (obj.duration === null || typeof obj.duration === 'number') &&
    (obj.file_size === null || typeof obj.file_size === 'number') &&
    typeof obj.created_at === 'string' &&
    typeof obj.updated_at === 'string'
  );
}

export function isValidTranscription(obj: any): obj is Transcription {
  return (
    obj &&
    typeof obj.id === 'string' &&
    typeof obj.recording_id === 'string' &&
    typeof obj.text === 'string' &&
    typeof obj.language === 'string' &&
    (obj.confidence === null || typeof obj.confidence === 'number') &&
    (obj.processing_time_ms === null || typeof obj.processing_time_ms === 'number') &&
    isValidTranscriptionStatus(obj.status) &&
    typeof obj.created_at === 'string' &&
    typeof obj.updated_at === 'string'
  );
}

export function isValidTranscriptionStatus(status: any): status is TranscriptionStatus {
  if (typeof status === 'string') {
    return ['Pending', 'Processing', 'Completed'].includes(status);
  }
  
  if (typeof status === 'object' && status !== null) {
    return (
      'Failed' in status &&
      typeof status.Failed === 'string'
    );
  }
  
  return false;
}