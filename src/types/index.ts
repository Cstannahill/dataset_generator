export interface Model {
  id: string;
  name: string;
  size: string;
  modified: string;
  provider: 'Ollama' | 'OpenAI';
  capabilities: string[];
}

export interface GenerationConfig {
  target_entries: number;
  batch_size: number;
  fine_tuning_goal: string;
  domain_context: string;
  selected_model?: string;
}

export interface GenerationProgress {
  current_batch: number;
  total_batches: number;
  entries_generated: number;
  estimated_completion: string;
  status: string;
  generation_id?: string;
  concurrent_batches: number;
  entries_per_second: number;
  errors_count: number;
  retries_count: number;
}

export type Step = 'models' | 'configuration' | 'generating' | 'export';

export interface StepConfig {
  key: Step;
  label: string;
  icon: React.ComponentType<any>;
  description?: string;
}

export interface AppState {
  models: Model[];
  selectedModel: string;
  isDiscovering: boolean;
  isGenerating: boolean;
  progress: GenerationProgress | null;
  currentStep: Step;
  generationConfig: GenerationConfig;
  error: string | null;
  success: string | null;
}