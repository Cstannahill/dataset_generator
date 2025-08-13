import { DatasetFormat, DatasetFormatInfo } from "@/types";

export const DATASET_FORMATS: Record<DatasetFormat, DatasetFormatInfo> = {
  alpaca: {
    id: "alpaca",
    name: "Alpaca Format",
    description:
      "Standard instruction-following format with instruction, optional input, and output",
    structure: '{"instruction": "...", "input": "...", "output": "..."}',
    goodFor: [
      "General instruction following",
      "Task-based training",
      "Simple Q&A",
    ],
    notIdealFor: [
      "Multi-turn conversations",
      "Complex reasoning chains",
      "Tool usage",
    ],
    examples: ["Stanford Alpaca", "Dolly-15k", "OpenAssistant"],
    fileExtension: ".jsonl",
  },

  conversation: {
    id: "conversation",
    name: "Conversation/Chat Format",
    description: "Multi-turn conversation format with roles (user/assistant)",
    structure:
      '[{"role": "user", "content": "..."}, {"role": "assistant", "content": "..."}]',
    goodFor: [
      "Chatbots and assistants",
      "Multi-turn context retention",
      "Tone/personality alignment",
    ],
    notIdealFor: [
      "Direct task solving",
      "Ranking training",
      "Single-turn instructions",
    ],
    examples: ["ShareGPT", "OpenChat", "UltraChat", "OpenOrca"],
    fileExtension: ".jsonl",
  },

  chain_of_thought: {
    id: "chain_of_thought",
    name: "Chain-of-Thought Reasoning",
    description:
      "Step-by-step reasoning format showing intermediate thinking steps",
    structure:
      '{"question": "...", "answer": "Step 1: ... Step 2: ... Final Answer: ..."}',
    goodFor: [
      "Mathematical reasoning",
      "Logic problems",
      "Debugging",
      "Algorithm questions",
    ],
    notIdealFor: [
      "Creative writing",
      "Concise outputs",
      "Simple factual answers",
    ],
    examples: ["CoT GSM8K", "Flan-CoT", "Math-Instruct"],
    fileExtension: ".jsonl",
  },

  preference_ranking: {
    id: "preference_ranking",
    name: "Preference/Ranking Format",
    description:
      "Format comparing good vs bad responses for preference learning",
    structure: '{"prompt": "...", "chosen": "...", "rejected": "..."}',
    goodFor: ["RLHF training", "Response quality alignment", "Tone correction"],
    notIdealFor: [
      "Direct instruction tuning",
      "Initial model training",
      "Task learning",
    ],
    examples: ["OpenAssistant RLHF", "Anthropic HH-RLHF", "TRL DPO"],
    fileExtension: ".jsonl",
  },

  function_call: {
    id: "function_call",
    name: "Function Call/Tool Use",
    description: "Format for teaching models to call APIs and use tools",
    structure:
      '{"messages": [...], "function": {"name": "...", "arguments": "..."}}',
    goodFor: [
      "Autonomous agents",
      "API integration",
      "Tool usage",
      "Plugin systems",
    ],
    notIdealFor: ["General conversation", "Creative tasks", "Simple Q&A"],
    examples: ["Toolformer", "Gorilla", "ToolBench"],
    fileExtension: ".jsonl",
  },

  multi_round_dialogue: {
    id: "multi_round_dialogue",
    name: "Multi-Round Dialogue",
    description: "Complex dialogue simulations with specific instructions",
    structure:
      '{"instruction": "...", "conversation": [{"role": "...", "content": "..."}]}',
    goodFor: [
      "Dialogue simulations",
      "Multi-agent modeling",
      "Goal-based interactions",
    ],
    notIdealFor: ["Simple tasks", "Single-turn responses", "Factual Q&A"],
    examples: ["DialogueSum", "Self-Instruct", "CAMEL-AI"],
    fileExtension: ".jsonl",
  },

  code_task: {
    id: "code_task",
    name: "Code Task Format",
    description: "Code-specific format with prompts, existing code, and output",
    structure: '{"prompt": "...", "code": "...", "output": "..."}',
    goodFor: [
      "Code transformation",
      "Bug fixing",
      "Style matching",
      "Code generation",
    ],
    notIdealFor: [
      "Text reasoning",
      "General conversation",
      "Non-technical tasks",
    ],
    examples: ["CodeAlpaca", "HumanEval+", "DeepSeek Coder-Instruct"],
    fileExtension: ".jsonl",
  },

  reflection: {
    id: "reflection",
    name: "Reflection Format",
    description:
      "Self-correction format with initial response, reflection, and correction",
    structure:
      '{"instruction": "...", "output": "...", "reflection": "...", "corrected": "..."}',
    goodFor: [
      "Self-correction training",
      "Quality improvement",
      "Error analysis",
    ],
    notIdealFor: [
      "Simple tasks",
      "Time-sensitive responses",
      "Factual lookups",
    ],
    examples: ["Self-Refine", "Constitutional AI", "Reflection datasets"],
    fileExtension: ".jsonl",
  },

  retrieval_embedding: {
    id: "retrieval_embedding",
    name: "Retrieval/Embedding Format",
    description:
      "Format for training retrieval and embedding models with query-passage pairs",
    structure:
      '{"query": "What is the capital of France?", "positive_passage": "Paris is the capital city of France...", "negative_passages": ["Berlin is the capital of Germany...", "Madrid is the capital of Spain..."]}',
    goodFor: [
      "RAG systems",
      "Semantic search",
      "Embedding model training",
      "Information retrieval",
      "Document ranking",
    ],
    notIdealFor: [
      "Generative LLMs directly",
      "Conversational AI",
      "Creative writing",
      "Code generation",
    ],
    examples: ["BEIR", "MTEB", "MSMARCO", "Natural Questions"],
    fileExtension: ".jsonl",
  },

  reranking: {
    id: "reranking",
    name: "Reranking/Cross-Encoder Format",
    description:
      "Pairwise format for training reranking models with query, positive, and negative document pairs",
    structure:
      '{"query": "What is Rust ownership?", "positive": "Rust uses a unique system of ownership to manage memory.", "negative": "JavaScript runs in the browser."}',
    goodFor: [
      "Cross-encoder rerankers",
      "LLM-based rerankers",
      "BGE/E5 reranker fine-tuning",
      "MonoT5 reranker training",
      "Contrastive learning",
      "Search result reranking",
      "Document relevance scoring",
    ],
    notIdealFor: [
      "Bi-encoder training",
      "Embedding models",
      "Generative text tasks",
      "Conversational AI",
    ],
    examples: [
      "MS MARCO pairs",
      "BGE reranker data",
      "MonoT5 datasets",
      "Custom relevance pairs",
    ],
    fileExtension: ".jsonl",
  },
};

export const getFormatInfo = (format: DatasetFormat): DatasetFormatInfo => {
  return DATASET_FORMATS[format];
};

export const getAllFormats = (): DatasetFormatInfo[] => {
  return Object.values(DATASET_FORMATS);
};
