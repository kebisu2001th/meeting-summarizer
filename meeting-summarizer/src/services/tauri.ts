import { invoke } from '@tauri-apps/api/core';
import { Recording } from '../types/recording';

export interface TauriRecording {
  id: string;
  filename: string;
  file_path: string;
  duration: number | null;
  file_size: number | null;
  created_at: string; // ISO string
  updated_at: string; // ISO string
}

// Tauri Recording を フロントエンド Recording に変換
function convertTauriRecording(tauriRecording: TauriRecording): Recording {
  return {
    id: tauriRecording.id,
    filename: tauriRecording.filename,
    duration: tauriRecording.duration || 0,
    createdAt: new Date(tauriRecording.created_at),
    size: tauriRecording.file_size || undefined,
  };
}

export class TauriService {
  static async startRecording(): Promise<string> {
    try {
      return await invoke<string>('start_recording');
    } catch (error) {
      throw new Error(`Failed to start recording: ${error}`);
    }
  }

  static async stopRecording(): Promise<Recording> {
    try {
      const tauriRecording = await invoke<TauriRecording>('stop_recording');
      return convertTauriRecording(tauriRecording);
    } catch (error) {
      throw new Error(`Failed to stop recording: ${error}`);
    }
  }

  static async getRecordings(): Promise<Recording[]> {
    try {
      const tauriRecordings = await invoke<TauriRecording[]>('get_recordings');
      return tauriRecordings.map(convertTauriRecording);
    } catch (error) {
      throw new Error(`Failed to get recordings: ${error}`);
    }
  }

  static async getRecording(id: string): Promise<Recording | null> {
    try {
      const tauriRecording = await invoke<TauriRecording | null>('get_recording', { id });
      return tauriRecording ? convertTauriRecording(tauriRecording) : null;
    } catch (error) {
      throw new Error(`Failed to get recording: ${error}`);
    }
  }

  static async deleteRecording(id: string): Promise<boolean> {
    try {
      return await invoke<boolean>('delete_recording', { id });
    } catch (error) {
      throw new Error(`Failed to delete recording: ${error}`);
    }
  }

  static async isRecording(): Promise<boolean> {
    try {
      return await invoke<boolean>('is_recording');
    } catch (error) {
      console.warn('Failed to check recording status:', error);
      return false;
    }
  }

  static async getRecordingsCount(): Promise<number> {
    try {
      return await invoke<number>('get_recordings_count');
    } catch (error) {
      throw new Error(`Failed to get recordings count: ${error}`);
    }
  }
}