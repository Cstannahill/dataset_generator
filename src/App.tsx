import React from 'react';
import { useDatasetGenerator } from '@/hooks/useDatasetGenerator';
import {
  AppLayout,
  StepIndicator,
  ModelSelection,
  DatasetConfiguration,
  GenerationProgress,
  ExportDataset,
} from '@/components';

const App: React.FC = () => {
  const {
    models,
    selectedModel,
    isDiscovering,
    isGenerating,
    progress,
    currentStep,
    generationConfig,
    error,
    success,
    actions,
  } = useDatasetGenerator();

  const selectedModelData = models.find(m => m.id === selectedModel);

  const renderCurrentStep = () => {
    switch (currentStep) {
      case 'models':
        return (
          <ModelSelection
            models={models}
            selectedModel={selectedModel}
            isDiscovering={isDiscovering}
            onModelSelect={actions.setSelectedModel}
            onRefreshModels={actions.discoverModels}
            onNext={() => actions.setCurrentStep('configuration')}
          />
        );

      case 'configuration':
        return (
          <DatasetConfiguration
            config={generationConfig}
            selectedModel={selectedModelData}
            isGenerating={isGenerating}
            onConfigChange={actions.updateGenerationConfig}
            onBack={() => actions.setCurrentStep('models')}
            onStart={actions.startGeneration}
          />
        );

      case 'generating':
        return (
          <GenerationProgress
            progress={progress}
            config={generationConfig}
            isGenerating={isGenerating}
          />
        );

      case 'export':
        return (
          <ExportDataset
            progress={progress}
            onExport={actions.exportDataset}
            onStartNew={actions.resetGeneration}
          />
        );

      default:
        return null;
    }
  };

  return (
    <AppLayout error={error} success={success}>
      <StepIndicator currentStep={currentStep} />
      {renderCurrentStep()}
    </AppLayout>
  );
};

export default App;