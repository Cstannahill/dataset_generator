import { invoke } from "@tauri-apps/api/core";
import type { DatasetEntry, DatasetFormat } from "@/types";

export interface ValidationFeedback {
  common_issues: string[];
  improvement_suggestions: string[];
  quality_patterns: string[];
  avoid_patterns: string[];
  batch_summary: string;
}

export class PromptFeedbackService {
  /**
   * Generate prompt improvements based on a batch of generated entries
   */
  static async generatePromptImprovements(
    entries: DatasetEntry[],
    useCase: string,
    format: DatasetFormat
  ): Promise<string> {
    try {
      const improvements = await invoke<string>(
        "generate_prompt_improvements",
        {
          entries,
          useCase,
          format,
        }
      );
      return improvements;
    } catch (error) {
      console.error("Failed to generate prompt improvements:", error);
      return "";
    }
  }

  /**
   * Apply dynamic improvements to a base prompt
   */
  static applyImprovementsToPrompt(
    basePrompt: string,
    improvements: string
  ): string {
    if (!improvements.trim()) {
      return basePrompt;
    }

    // Insert improvements before the main instruction
    const sections = basePrompt.split("\n\n");

    // Add improvements after the context but before the main instruction
    if (sections.length > 1) {
      return [
        sections[0], // Context/setup
        improvements, // Dynamic improvements
        ...sections.slice(1), // Main instruction and examples
      ].join("\n\n");
    }

    // If single section, append improvements
    return `${basePrompt}\n\n${improvements}`;
  }

  /**
   * Extract actionable feedback from validation results
   */
  static formatFeedbackForDisplay(feedback: ValidationFeedback): string {
    const sections: string[] = [];

    if (feedback.batch_summary) {
      sections.push(`📊 **Batch Summary**: ${feedback.batch_summary}`);
    }

    if (feedback.common_issues.length > 0) {
      sections.push(
        `⚠️ **Common Issues Found**:\n${feedback.common_issues
          .map((issue) => `  • ${issue}`)
          .join("\n")}`
      );
    }

    if (feedback.avoid_patterns.length > 0) {
      sections.push(
        `🚫 **Patterns to Avoid**:\n${feedback.avoid_patterns
          .map((pattern) => `  • ${pattern}`)
          .join("\n")}`
      );
    }

    if (feedback.improvement_suggestions.length > 0) {
      sections.push(
        `💡 **Focus More On**:\n${feedback.improvement_suggestions
          .map((suggestion) => `  • ${suggestion}`)
          .join("\n")}`
      );
    }

    if (feedback.quality_patterns.length > 0) {
      sections.push(
        `✅ **Successful Patterns**:\n${feedback.quality_patterns
          .map((pattern) => `  • ${pattern}`)
          .join("\n")}`
      );
    }

    return (
      sections.join("\n\n") || "No specific feedback available for this batch."
    );
  }

  /**
   * Determine if feedback suggests major prompt adjustments
   */
  static requiresPromptAdjustment(feedback: ValidationFeedback): boolean {
    return (
      feedback.common_issues.length > 2 ||
      feedback.avoid_patterns.length > 1 ||
      feedback.improvement_suggestions.length > 0
    );
  }

  /**
   * Generate priority score for feedback application (0-1, higher = more urgent)
   */
  static getFeedbackPriority(feedback: ValidationFeedback): number {
    let score = 0;

    // High priority for many common issues
    score += Math.min(feedback.common_issues.length * 0.2, 0.6);

    // High priority for avoid patterns
    score += Math.min(feedback.avoid_patterns.length * 0.3, 0.5);

    // Medium priority for improvement suggestions
    score += Math.min(feedback.improvement_suggestions.length * 0.1, 0.3);

    return Math.min(score, 1.0);
  }
}
