export interface Model {
  id: string;
  name: string;
  size: string;
  modified: string;
  provider: "Ollama" | "OpenAI";
  capabilities: string[];
}

export type DatasetFormat =
  | "alpaca"
  | "conversation"
  | "chain_of_thought"
  | "preference_ranking"
  | "function_call"
  | "multi_round_dialogue"
  | "code_task"
  | "reflection"
  | "retrieval_embedding"
  | "reranking";

export interface DatasetFormatInfo {
  id: DatasetFormat;
  name: string;
  description: string;
  structure: string;
  goodFor: string[];
  notIdealFor: string[];
  examples: string[];
  fileExtension: string;
}

export interface GenerationConfig {
  target_entries: number;
  batch_size: number;
  fine_tuning_goal: string;
  domain_context: string;
  selected_model?: string;
  format: DatasetFormat;
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

export interface DatasetEntry {
  data: Record<string, any>;
}

export type Step = "models" | "configuration" | "generating" | "export";

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

// Knowledge Base Types
export interface QualityScore {
  overall_score: number;
  relevance_score: number;
  coherence_score: number;
  completeness_score: number;
  format_compliance_score: number;
  issues: string[];
  tags: string[];
}

export interface ProcessingStats {
  total_entries: number;
  validated_entries: number;
  embedded_entries: number;
  stored_entries: number;
  validation_time_ms: number;
  embedding_time_ms: number;
  storage_time_ms: number;
}

export interface CollectionInfo {
  name: string;
  use_case: string;
  dataset_format: DatasetFormat;
  entry_count: number;
  created_at: number;
  last_updated: number;
}

export interface SearchResult {
  id: string;
  text: string;
  distance: number;
  metadata: Record<string, any>;
}

export interface KnowledgeBaseStats {
  total_collections: number;
  total_entries: number;
  unique_use_cases: number;
  unique_formats: number;
  oldest_entry_timestamp?: number;
  newest_entry_timestamp?: number;
  collections: CollectionInfo[];
}

export interface ImprovementSuggestion {
  suggestion_type: string;
  description: string;
  confidence: number;
  examples: SearchResult[];
}
