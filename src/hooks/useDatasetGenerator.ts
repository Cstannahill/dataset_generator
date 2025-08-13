import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import {
  Model,
  GenerationConfig,
  GenerationProgress,
  Step,
  AppState,
} from "@/types";
import { getFormatInfo } from "@/lib/dataset-formats";

export const useDatasetGenerator = () => {
  const [state, setState] = useState<AppState>({
    models: [],
    selectedModel: "",
    isDiscovering: false,
    isGenerating: false,
    progress: null,
    currentStep: "models",
    generationConfig: {
      target_entries: 2000,
      batch_size: 50,
      fine_tuning_goal: "",
      domain_context: "",
      format: "alpaca",
    },
    error: null,
    success: null,
  });

  // Clear alerts after 5 seconds
  useEffect(() => {
    if (state.error || state.success) {
      const timer = setTimeout(() => {
        setState((prev) => ({ ...prev, error: null, success: null }));
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [state.error, state.success]);

  // Progress polling when generating
  useEffect(() => {
    if (state.isGenerating) {
      const interval = setInterval(async () => {
        try {
          const currentProgress = (await invoke(
            "get_progress"
          )) as GenerationProgress;
          setState((prev) => ({ ...prev, progress: currentProgress }));

          if (currentProgress.status === "completed") {
            setState((prev) => ({
              ...prev,
              isGenerating: false,
              currentStep: "export",
              success: "Dataset generation completed!",
            }));
          }
        } catch (error) {
          setState((prev) => ({
            ...prev,
            error: "Error fetching progress.",
            isGenerating: false,
          }));
        }
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [state.isGenerating]);

  const discoverModels = useCallback(async () => {
    setState((prev) => ({
      ...prev,
      isDiscovering: true,
      error: null,
      success: null,
    }));

    try {
      const discoveredModels = (await invoke("discover_models")) as Model[];
      setState((prev) => ({
        ...prev,
        models: discoveredModels,
        selectedModel:
          discoveredModels.length > 0 ? discoveredModels[0].id : "",
        isDiscovering: false,
      }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        error: "Failed to discover models",
        isDiscovering: false,
      }));
    }
  }, []);

  const startGeneration = useCallback(async () => {
    if (
      !state.selectedModel ||
      !state.generationConfig.fine_tuning_goal.trim()
    ) {
      setState((prev) => ({
        ...prev,
        error: "Please select a model and provide a fine-tuning goal",
      }));
      return;
    }

    setState((prev) => ({
      ...prev,
      isGenerating: true,
      currentStep: "generating",
      error: null,
      success: null,
    }));

    try {
      const config = {
        ...state.generationConfig,
        selected_model: state.selectedModel,
      };

      await invoke("start_generation", { config });
      setState((prev) => ({ ...prev, success: "Generation started!" }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        error: "Failed to start generation",
        isGenerating: false,
      }));
    }
  }, [state.selectedModel, state.generationConfig]);

  const exportDataset = useCallback(async () => {
    console.log("Export dataset function called");

    try {
      setState((prev) => ({ ...prev, error: null, success: null }));
      console.log("Calling Tauri export_dataset command...");

      const datasetJson = (await invoke("export_dataset")) as string;
      console.log("Received dataset JSON, length:", datasetJson.length);

      if (!datasetJson || datasetJson.trim() === "" || datasetJson === "[]") {
        throw new Error("Dataset is empty or invalid");
      }

      const formatInfo = getFormatInfo(state.generationConfig.format);
      const fileExtension =
        formatInfo.fileExtension === ".jsonl" ? "jsonl" : "json";
      const fileName = `fine_tuning_dataset_${state.generationConfig.format}.${fileExtension}`;

      // Try Tauri save dialog first
      console.log("Attempting to use Tauri save dialog...");
      try {
        const filePath = await save({
          filters: [
            {
              name: `${fileExtension.toUpperCase()} Files`,
              extensions: [fileExtension],
            },
          ],
          defaultPath: fileName,
        });

        if (filePath) {
          await writeTextFile(filePath, datasetJson);
          console.log("File saved to:", filePath);
          setState((prev) => ({
            ...prev,
            success: `Dataset exported to ${filePath}`,
          }));
          return;
        } else {
          console.log("User cancelled file save dialog");
          return;
        }
      } catch (saveError) {
        console.warn("Tauri save failed:", saveError);
        console.log("Falling back to browser download...");
      }

      // Fallback to browser download
      console.log("Creating browser download...");
      const mimeType =
        fileExtension === "jsonl"
          ? "application/x-jsonlines"
          : "application/json";
      const blob = new Blob([datasetJson], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      a.style.display = "none";
      document.body.appendChild(a);

      console.log("Triggering download click...");
      a.click();

      // Clean up after a short delay
      setTimeout(() => {
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        console.log("Cleanup completed");
      }, 100);

      console.log("Browser download triggered successfully");
      setState((prev) => ({
        ...prev,
        success: "Dataset downloaded to your Downloads folder!",
      }));
    } catch (error) {
      console.error("Export error:", error);
      const errorMessage =
        error instanceof Error ? error.message : "Failed to export dataset";
      setState((prev) => ({ ...prev, error: errorMessage }));
    }
  }, []);

  const updateGenerationConfig = useCallback(
    (config: Partial<GenerationConfig>) => {
      setState((prev) => ({
        ...prev,
        generationConfig: { ...prev.generationConfig, ...config },
      }));
    },
    []
  );

  const setCurrentStep = useCallback((step: Step) => {
    setState((prev) => ({ ...prev, currentStep: step }));
  }, []);

  const setSelectedModel = useCallback((modelId: string) => {
    setState((prev) => ({ ...prev, selectedModel: modelId }));
  }, []);

  const resetGeneration = useCallback(() => {
    setState((prev) => ({
      ...prev,
      currentStep: "models",
      progress: null,
      isGenerating: false,
      error: null,
      success: null,
    }));
  }, []);

  const improvePrompt = useCallback(async (prompt: string) => {
    if (!prompt.trim()) {
      setState((prev) => ({
        ...prev,
        error: "Please provide a fine-tuning goal to improve",
      }));
      return null;
    }

    try {
      const improvedPrompt = (await invoke("improve_prompt", {
        prompt,
      })) as string;
      setState((prev) => ({
        ...prev,
        success: "Prompt improved successfully!",
      }));
      return improvedPrompt;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to improve prompt";
      setState((prev) => ({
        ...prev,
        error: errorMessage,
      }));
      return null;
    }
  }, []);

  const generateUseCaseSuggestions = useCallback(
    async (domainContext: string, format: string) => {
      if (!state.selectedModel) {
        setState((prev) => ({
          ...prev,
          error: "Please select a model first",
        }));
        return null;
      }

      try {
        const suggestions = (await invoke("generate_use_case_suggestions", {
          domainContext,
          format,
          modelId: state.selectedModel,
        })) as string[];
        setState((prev) => ({
          ...prev,
          success: "Use case suggestions generated successfully!",
        }));
        return suggestions;
      } catch (error) {
        const errorMessage =
          error instanceof Error
            ? error.message
            : "Failed to generate suggestions";
        setState((prev) => ({
          ...prev,
          error: errorMessage,
        }));
        return null;
      }
    },
    [state.selectedModel]
  );

  // Auto-discover models on mount
  useEffect(() => {
    discoverModels();
  }, [discoverModels]);

  return {
    ...state,
    actions: {
      discoverModels,
      startGeneration,
      exportDataset,
      updateGenerationConfig,
      setCurrentStep,
      setSelectedModel,
      resetGeneration,
      improvePrompt,
      generateUseCaseSuggestions,
    },
  };
};
