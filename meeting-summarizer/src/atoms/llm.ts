import { atom } from 'jotai';
import { ModelInfo, ModelSettings, LLMConfig } from '../types/llm';

// Current selected model
export const selectedModelAtom = atom<string>('');

// Available models
export const availableModelsAtom = atom<ModelInfo[]>([]);

// Model settings
export const modelSettingsAtom = atom<ModelSettings | null>(null);

// Current LLM configuration
export const llmConfigAtom = atom<LLMConfig | null>(null);

// Loading states
export const modelsLoadingAtom = atom<boolean>(false);
export const modelTestingAtom = atom<string>(''); // Model ID being tested

// Model selection status
export const modelStatusAtom = atom<{
  isConnected: boolean;
  lastTested: string | null;
  error: string | null;
}>({
  isConnected: false,
  lastTested: null,
  error: null
});

// Derived atom for current model info
export const currentModelAtom = atom((get) => {
  const selectedId = get(selectedModelAtom);
  const models = get(availableModelsAtom);
  return models.find(model => model.id === selectedId) || null;
});

// Actions
export const updateModelSelectionAtom = atom(
  null,
  (_, set, modelId: string) => {
    set(selectedModelAtom, modelId);
    // Reset status when changing models
    set(modelStatusAtom, {
      isConnected: false,
      lastTested: null,
      error: null
    });
  }
);

export const setModelTestingAtom = atom(
  null,
  (_, set, modelId: string) => {
    set(modelTestingAtom, modelId);
  }
);

export const updateModelStatusAtom = atom(
  null,
  (get, set, status: { isConnected: boolean; lastTested?: string; error?: string | null }) => {
    const current = get(modelStatusAtom);
    set(modelStatusAtom, {
      ...current,
      ...status,
      lastTested: status.lastTested || new Date().toISOString()
    });
  }
);