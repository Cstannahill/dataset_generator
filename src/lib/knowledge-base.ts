import { invoke } from "@tauri-apps/api/core";
import type {
  KnowledgeBaseStats,
  SearchResult,
  CollectionInfo,
  ImprovementSuggestion,
  DatasetEntry,
  DatasetFormat,
} from "@/types";

export class KnowledgeBaseService {
  /**
   * Initialize the knowledge base system
   */
  static async initialize(): Promise<void> {
    try {
      await invoke("initialize_knowledge_base");
    } catch (error) {
      throw new Error(`Failed to initialize knowledge base: ${error}`);
    }
  }

  /**
   * Get comprehensive statistics about the knowledge base
   */
  static async getStats(): Promise<KnowledgeBaseStats> {
    try {
      return await invoke("get_knowledge_base_stats");
    } catch (error) {
      throw new Error(`Failed to get knowledge base stats: ${error}`);
    }
  }

  /**
   * Search for similar entries in the knowledge base
   */
  static async search(
    queryText: string,
    options: {
      useCaseFilter?: string;
      formatFilter?: DatasetFormat;
      minQualityScore?: number;
      limit?: number;
    } = {}
  ): Promise<SearchResult[]> {
    try {
      return await invoke("search_knowledge_base", {
        queryText,
        useCaseFilter: options.useCaseFilter,
        formatFilter: options.formatFilter
          ? this.formatToString(options.formatFilter)
          : undefined,
        minQualityScore: options.minQualityScore,
        limit: options.limit,
      });
    } catch (error) {
      throw new Error(`Failed to search knowledge base: ${error}`);
    }
  }

  /**
   * Get improvement suggestions based on current entries
   */
  static async getImprovementSuggestions(
    entries: DatasetEntry[],
    useCase: string,
    format: DatasetFormat
  ): Promise<ImprovementSuggestion[]> {
    try {
      return await invoke("get_improvement_suggestions", {
        entries,
        useCase,
        format: this.formatToString(format),
      });
    } catch (error) {
      throw new Error(`Failed to get improvement suggestions: ${error}`);
    }
  }

  /**
   * List all collections in the knowledge base
   */
  static async listCollections(): Promise<CollectionInfo[]> {
    try {
      return await invoke("list_collections");
    } catch (error) {
      throw new Error(`Failed to list collections: ${error}`);
    }
  }

  /**
   * Find similar examples for inspiration
   */
  static async findSimilarExamples(
    useCase: string,
    format: DatasetFormat,
    queryText: string,
    limit: number = 5
  ): Promise<SearchResult[]> {
    return this.search(queryText, {
      useCaseFilter: useCase,
      formatFilter: format,
      minQualityScore: 0.8, // Only high-quality examples
      limit,
    });
  }

  /**
   * Get insights about knowledge base coverage
   */
  static async getCoverageInsights(): Promise<{
    strongAreas: string[];
    weakAreas: string[];
    recommendations: string[];
  }> {
    try {
      const stats = await this.getStats();
      const collections = stats.collections;

      // Analyze coverage by use case and format
      const useCaseCounts = new Map<string, number>();
      const formatCounts = new Map<DatasetFormat, number>();

      collections.forEach((collection) => {
        useCaseCounts.set(
          collection.use_case,
          (useCaseCounts.get(collection.use_case) || 0) + collection.entry_count
        );
        formatCounts.set(
          collection.dataset_format,
          (formatCounts.get(collection.dataset_format) || 0) +
            collection.entry_count
        );
      });

      // Sort by entry count
      const sortedUseCases = Array.from(useCaseCounts.entries()).sort(
        (a, b) => b[1] - a[1]
      );
      const sortedFormats = Array.from(formatCounts.entries()).sort(
        (a, b) => b[1] - a[1]
      );

      const strongAreas = sortedUseCases
        .slice(0, 3)
        .map(([useCase, count]) => `${useCase} (${count} entries)`);

      const weakAreas = sortedUseCases
        .slice(-3)
        .map(([useCase, count]) => `${useCase} (${count} entries)`);

      const recommendations = [];

      if (stats.total_entries < 100) {
        recommendations.push(
          "Consider generating more diverse examples to improve knowledge base coverage"
        );
      }

      if (stats.unique_use_cases < 5) {
        recommendations.push(
          "Expand into more use cases to build a comprehensive knowledge base"
        );
      }

      if (stats.unique_formats < 3) {
        recommendations.push(
          "Try different dataset formats to support various training scenarios"
        );
      }

      return {
        strongAreas,
        weakAreas,
        recommendations,
      };
    } catch (error) {
      throw new Error(`Failed to get coverage insights: ${error}`);
    }
  }

  /**
   * Convert DatasetFormat enum to string for backend
   */
  private static formatToString(format: DatasetFormat): string {
    const formatMap: Record<DatasetFormat, string> = {
      alpaca: "Alpaca",
      conversation: "Conversation",
      chain_of_thought: "ChainOfThought",
      preference_ranking: "PreferenceRanking",
      function_call: "FunctionCall",
      multi_round_dialogue: "MultiRoundDialogue",
      code_task: "CodeTask",
      reflection: "Reflection",
      retrieval_embedding: "RetrievalEmbedding",
      reranking: "Reranking",
    };
    return formatMap[format] || "Alpaca";
  }
}
