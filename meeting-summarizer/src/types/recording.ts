export interface Recording {
  id: string;
  filename: string;
  duration: number; // in seconds
  createdAt: Date;
  size?: number; // in bytes
}

export interface RecordingState {
  isRecording: boolean;
  duration: number; // current recording duration in seconds
  isPaused: boolean;
}

export interface Transcription {
  id: string;
  recording_id: string;
  text: string;
  language: string;
  confidence?: number;
  processing_time_ms?: number;
  status: TranscriptionStatus;
  created_at: string;
  updated_at: string;
}

export type TranscriptionStatus = 
  | 'Pending'
  | 'Processing'
  | 'Completed'
  | { Failed: string };