import { invoke } from '@tauri-apps/api/core';
import { type Recording, type Transcription, isValidRecording, isValidTranscription } from '../types/recording';

// Tauri呼び出し時の型安全性を確保
function validateAndParseRecording(obj: any): Recording {
  if (!isValidRecording(obj)) {
    throw new Error(`Invalid recording data received from Tauri: ${JSON.stringify(obj)}`);
  }
  return obj;
}

function validateAndParseTranscription(obj: any): Transcription {
  if (!isValidTranscription(obj)) {
    throw new Error(`Invalid transcription data received from Tauri: ${JSON.stringify(obj)}`);
  }
  return obj;
}

function validateAndParseRecordings(arr: any): Recording[] {
  if (!Array.isArray(arr)) {
    throw new Error(`Expected array of recordings, got: ${typeof arr}`);
  }
  
  return arr.map((item, index) => {
    try {
      return validateAndParseRecording(item);
    } catch (error) {
      throw new Error(`Invalid recording at index ${index}: ${error}`);
    }
  });
}

// biome-ignore lint/complexity/noStaticOnlyClass: <explanation>
export class TauriService {
  static async startRecording(): Promise<string> {

    console.log('startRecording');

    try {
      const result = await invoke<string>('start_recording');
      console.log('result', result);
      return result;
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw new Error(`Failed to start recording: ${error}`);
    }
  }

  static async stopRecording(): Promise<Recording> {
    try {
      const result = await invoke<any>('stop_recording');
      return validateAndParseRecording(result);
    } catch (error) {
      throw new Error(`Failed to stop recording: ${error}`);
    }
  }

  static async getRecordings(): Promise<Recording[]> {
    try {
      const result = await invoke<any>('get_recordings');
      return validateAndParseRecordings(result);
    } catch (error) {
      throw new Error(`Failed to get recordings: ${error}`);
    }
  }

  static async getRecording(id: string): Promise<Recording | null> {
    try {
      if (typeof id !== 'string' || id.trim() === '') {
        throw new Error('Recording ID must be a non-empty string');
      }
      
      const result = await invoke<any>('get_recording', { id: id.trim() });
      return result ? validateAndParseRecording(result) : null;
    } catch (error) {
      throw new Error(`Failed to get recording: ${error}`);
    }
  }

  static async deleteRecording(id: string): Promise<boolean> {
    try {
      if (typeof id !== 'string' || id.trim() === '') {
        throw new Error('Recording ID must be a non-empty string');
      }
      
      const result = await invoke<boolean>('delete_recording', { id: id.trim() });
      if (typeof result !== 'boolean') {
        throw new Error(`Expected boolean result, got: ${typeof result}`);
      }
      return result;
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

  // Whisper 書き起こし関連メソッド

  static async initializeWhisper(): Promise<void> {
    try {
      await invoke('initialize_whisper');
    } catch (error) {
      throw new Error(`Failed to initialize Whisper: ${error}`);
    }
  }

  static async isWhisperInitialized(): Promise<boolean> {
    try {
      return await invoke<boolean>('is_whisper_initialized');
    } catch (error) {
      console.warn('Failed to check Whisper status:', error);
      return false;
    }
  }

  static async transcribeRecording(
    recordingId: string, 
    language?: string
  ): Promise<Transcription> {
    try {
      if (typeof recordingId !== 'string' || recordingId.trim() === '') {
        throw new Error('Recording ID must be a non-empty string');
      }
      
      if (language !== undefined && (typeof language !== 'string' || language.trim() === '')) {
        throw new Error('Language must be a non-empty string or undefined');
      }
      
      const result = await invoke<any>('transcribe_recording', {
        recordingId: recordingId.trim(),
        language: language?.trim() || 'ja',
      });
      
      return validateAndParseTranscription(result);
    } catch (error) {
      throw new Error(`Failed to transcribe recording: ${error}`);
    }
  }

  static async getAudioDevices(): Promise<string[]> {
    try {
      const result = await invoke<any>('get_audio_devices');
      if (!Array.isArray(result)) {
        throw new Error(`Expected array of devices, got: ${typeof result}`);
      }
      
      return result.filter(device => typeof device === 'string');
    } catch (error) {
      throw new Error(`Failed to get audio devices: ${error}`);
    }
  }
}