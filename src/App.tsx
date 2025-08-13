import React, { useState } from 'react';
import { useDatasetGenerator } from '@/hooks/useDatasetGenerator';
import {
  AppLayout,
  StepIndicator,
  ModelSelection,
  DatasetConfiguration,
  GenerationProgress,
  ExportDataset,
} from '@/components';
import { KnowledgeBaseDashboard } from '@/components/KnowledgeBaseDashboard';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Database, Zap } from 'lucide-react';

const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState('generator');
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
            onImprovePrompt={actions.improvePrompt}
            onGenerateUseCases={actions.generateUseCaseSuggestions}
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
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-2 mb-8">
          <TabsTrigger value="generator" className="flex items-center gap-2">
            <Zap className="h-4 w-4" />
            Dataset Generator
          </TabsTrigger>
          <TabsTrigger value="knowledge-base" className="flex items-center gap-2">
            <Database className="h-4 w-4" />
            Knowledge Base
          </TabsTrigger>
        </TabsList>

        <TabsContent value="generator">
          <StepIndicator currentStep={currentStep} />
          {renderCurrentStep()}
        </TabsContent>

        <TabsContent value="knowledge-base">
          <KnowledgeBaseDashboard />
        </TabsContent>
      </Tabs>
    </AppLayout>
  );
};

export default App;