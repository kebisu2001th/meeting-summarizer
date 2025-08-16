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