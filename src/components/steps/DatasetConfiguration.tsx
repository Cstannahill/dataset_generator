import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { FormatSelector } from '@/components/FormatSelector';
import {
  Settings,
  ArrowLeft,
  Play,
  Target,
  Layers,
  Clock,
  Zap,
  Info,
  FileText,
  BarChart3,
  CheckCircle,
  AlertCircle,
  Sparkles,
  Loader2,
  Database,
  Lightbulb,
  Copy
} from 'lucide-react';
import { GenerationConfig, Model, DatasetFormat } from '@/types';
import { cn } from '@/lib/utils';

interface DatasetConfigurationProps {
  config: GenerationConfig;
  selectedModel: Model | undefined;
  isGenerating: boolean;
  onConfigChange: (config: Partial<GenerationConfig>) => void;
  onBack: () => void;
  onStart: () => void;
  onImprovePrompt: (prompt: string) => Promise<string | null>;
  onGenerateUseCases: (format: DatasetFormat, domainContext: string) => Promise<string[] | null>;
}

export const DatasetConfiguration: React.FC<DatasetConfigurationProps> = ({
  config,
  selectedModel,
  isGenerating,
  onConfigChange,
  onBack,
  onStart,
  onImprovePrompt,
  onGenerateUseCases,
}) => {
  const [isImproving, setIsImproving] = useState(false);
  const [isGeneratingUseCases, setIsGeneratingUseCases] = useState(false);
  const [useCaseSuggestions, setUseCaseSuggestions] = useState<string[]>([]);
  const [showUseCases, setShowUseCases] = useState(false);
  const estimatedBatches = Math.ceil(config.target_entries / config.batch_size);
  const estimatedTime = Math.ceil(estimatedBatches * 2); // Rough estimate: 2 minutes per batch

  const handleInputChange = (field: keyof GenerationConfig, value: string | number | DatasetFormat) => {
    onConfigChange({ [field]: value });
  };

  const handleImprovePrompt = async () => {
    if (!config.fine_tuning_goal.trim()) {
      return;
    }

    setIsImproving(true);
    try {
      const improvedPrompt = await onImprovePrompt(config.fine_tuning_goal);
      if (improvedPrompt) {
        onConfigChange({ fine_tuning_goal: improvedPrompt });
      }
    } finally {
      setIsImproving(false);
    }
  };

  const handleGenerateUseCases = async () => {
    setIsGeneratingUseCases(true);
    try {
      const suggestions = await onGenerateUseCases(config.format, config.domain_context);
      if (suggestions && suggestions.length > 0) {
        setUseCaseSuggestions(suggestions);
        setShowUseCases(true);
      }
    } finally {
      setIsGeneratingUseCases(false);
    }
  };

  const handleSelectUseCase = (useCase: string) => {
    onConfigChange({ fine_tuning_goal: useCase });
    setShowUseCases(false);
  };

  const isFormValid = config.fine_tuning_goal.trim().length > 0;

  return (
    <div className="space-y-8 max-w-6xl mx-auto">
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center">
          <div className="p-4 bg-gradient-to-br from-purple-500/20 to-pink-500/20 rounded-2xl border border-purple-500/20">
            <Settings className="w-10 h-10 text-purple-500" />
          </div>
        </div>
        <div>
          <h2 className="text-4xl font-bold text-foreground mb-2">Dataset Configuration</h2>
          <p className="text-muted-foreground text-xl max-w-3xl mx-auto">
            Define the parameters and objectives for your dataset generation
          </p>
        </div>
        <Button
          onClick={onBack}
          variant="outline"
          size="lg"
          className="border-border bg-card hover:bg-accent"
        >
          <ArrowLeft className="w-5 h-5 mr-2" />
          Back to Models
        </Button>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-8">
        {/* Configuration Form */}
        <div className="xl:col-span-2 space-y-6">
          {/* Goal Configuration */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-xl">
            <CardHeader className="pb-6">
              <CardTitle className="text-foreground flex items-center text-2xl">
                <div className="p-3 bg-blue-500/10 rounded-xl mr-4">
                  <Target className="w-6 h-6 text-blue-500" />
                </div>
                Fine-tuning Objective
              </CardTitle>
              <CardDescription className="text-lg leading-relaxed">
                Describe what you want the model to learn from this dataset. Be specific about the task, style, and desired output format.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-8">
              <div className="space-y-4">
                <Label htmlFor="goal" className="text-base font-semibold text-foreground flex items-center">
                  <FileText className="w-5 h-5 mr-2 text-blue-500" />
                  Fine-tuning Goal *
                </Label>
                <div className="space-y-3">
                  <div className="relative">
                    <Textarea
                      id="goal"
                      value={config.fine_tuning_goal}
                      onChange={(e) => handleInputChange('fine_tuning_goal', e.target.value)}
                      placeholder="e.g., Generate creative product descriptions for e-commerce items, focusing on benefits and emotional appeal that drive conversions"
                      rows={5}
                      className="resize-none border-2 border-border bg-background text-foreground placeholder:text-muted-foreground focus:border-blue-500 focus:ring-blue-500/20 transition-colors text-base leading-relaxed pr-32"
                    />
                    <div className="absolute top-3 right-3">
                      <Button
                        type="button"
                        onClick={handleImprovePrompt}
                        disabled={isImproving || !config.fine_tuning_goal.trim()}
                        size="sm"
                        variant="outline"
                        className="bg-gradient-to-r from-purple-500/10 to-blue-500/10 hover:from-purple-500/20 hover:to-blue-500/20 border-purple-500/30 hover:border-purple-500/50 text-purple-700 dark:text-purple-300 transition-all duration-200"
                      >
                        {isImproving ? (
                          <>
                            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                            Improving...
                          </>
                        ) : (
                          <>
                            <Sparkles className="w-4 h-4 mr-2" />
                            Improve with AI
                          </>
                        )}
                      </Button>
                    </div>
                  </div>
                  <div className="bg-blue-50 dark:bg-blue-950/30 border border-blue-200 dark:border-blue-800/50 rounded-lg p-4">
                    <p className="text-sm text-blue-700 dark:text-blue-300 flex items-start">
                      <Info className="w-4 h-4 mr-2 mt-0.5 flex-shrink-0" />
                      <span>Provide clear, specific instructions. Include examples of desired output style, tone, and format. The more detailed your description, the better the generated dataset will match your needs.</span>
                    </p>
                  </div>
                  <div className="bg-purple-50 dark:bg-purple-950/30 border border-purple-200 dark:border-purple-800/50 rounded-lg p-3">
                    <p className="text-sm text-purple-700 dark:text-purple-300 flex items-start">
                      <Sparkles className="w-4 h-4 mr-2 mt-0.5 flex-shrink-0" />
                      <span>Click "Improve with AI" to enhance your prompt using GPT-4.1-nano for better structure, clarity, and effectiveness.</span>
                    </p>
                  </div>
                </div>
              </div>

              <div className="space-y-4">
                <Label htmlFor="context" className="text-base font-semibold text-foreground">
                  Domain Context (Optional)
                </Label>
                <div className="space-y-3">
                  <Textarea
                    id="context"
                    value={config.domain_context}
                    onChange={(e) => handleInputChange('domain_context', e.target.value)}
                    placeholder="Additional context about your domain, industry, or specific requirements that will help guide the generation"
                    rows={4}
                    className="resize-none border-2 border-border bg-background text-foreground placeholder:text-muted-foreground focus:border-blue-500 focus:ring-blue-500/20 transition-colors text-base leading-relaxed"
                  />
                  <p className="text-sm text-muted-foreground">
                    Provide industry-specific terminology, audience information, or any constraints that should influence the generated content.
                  </p>
                </div>
              </div>

              {/* Use Case Suggestions */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label className="text-base font-semibold text-foreground flex items-center">
                    <Lightbulb className="w-5 h-5 mr-2 text-yellow-500" />
                    AI-Suggested Use Cases
                  </Label>
                  <Button
                    type="button"
                    onClick={handleGenerateUseCases}
                    disabled={isGeneratingUseCases}
                    size="sm"
                    variant="outline"
                    className="bg-gradient-to-r from-yellow-500/10 to-orange-500/10 hover:from-yellow-500/20 hover:to-orange-500/20 border-yellow-500/30 hover:border-yellow-500/50 text-yellow-700 dark:text-yellow-300 transition-all duration-200"
                  >
                    {isGeneratingUseCases ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        Generating...
                      </>
                    ) : (
                      <>
                        <Lightbulb className="w-4 h-4 mr-2" />
                        Get Suggestions
                      </>
                    )}
                  </Button>
                </div>

                {showUseCases && useCaseSuggestions.length > 0 && (
                  <div className="bg-yellow-50 dark:bg-yellow-950/30 border border-yellow-200 dark:border-yellow-800/50 rounded-lg p-4 space-y-3">
                    <p className="text-sm text-yellow-700 dark:text-yellow-300 font-medium">
                      Based on your selected format ({config.format}) and domain context, here are some suggested use cases:
                    </p>
                    <div className="space-y-2">
                      {useCaseSuggestions.map((useCase, index) => (
                        <div
                          key={index}
                          className="bg-white dark:bg-gray-800 border border-yellow-200 dark:border-yellow-700 rounded-md p-3 hover:bg-yellow-50 dark:hover:bg-yellow-900/20 transition-colors cursor-pointer group"
                          onClick={() => handleSelectUseCase(useCase)}
                        >
                          <div className="flex items-start justify-between">
                            <p className="text-sm text-gray-700 dark:text-gray-300 flex-1 pr-2">
                              {useCase}
                            </p>
                            <Copy className="w-4 h-4 text-gray-400 group-hover:text-yellow-600 dark:group-hover:text-yellow-400 opacity-0 group-hover:opacity-100 transition-all flex-shrink-0" />
                          </div>
                        </div>
                      ))}
                    </div>
                    <p className="text-xs text-yellow-600 dark:text-yellow-400">
                      Click on any suggestion to use it as your fine-tuning goal.
                    </p>
                  </div>
                )}

                {!showUseCases && (
                  <div className="bg-gray-50 dark:bg-gray-950/30 border border-gray-200 dark:border-gray-800/50 rounded-lg p-3">
                    <p className="text-sm text-gray-600 dark:text-gray-400 flex items-start">
                      <Lightbulb className="w-4 h-4 mr-2 mt-0.5 flex-shrink-0 text-yellow-500" />
                      <span>Click "Get Suggestions" to generate AI-powered use case ideas based on your selected format and domain context.</span>
                    </p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          {/* Dataset Format Selection */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-xl">
            <CardHeader className="pb-6">
              <CardTitle className="text-foreground flex items-center text-2xl">
                <div className="p-3 bg-purple-500/10 rounded-xl mr-4">
                  <Database className="w-6 h-6 text-purple-500" />
                </div>
                Dataset Format
              </CardTitle>
              <CardDescription className="text-lg leading-relaxed">
                Choose the format that best matches your fine-tuning objectives and model architecture.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <FormatSelector
                selectedFormat={config.format}
                onFormatChange={(format: DatasetFormat) => handleInputChange('format', format)}
              />
            </CardContent>
          </Card>

          {/* Generation Parameters */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-xl">
            <CardHeader className="pb-6">
              <CardTitle className="text-foreground flex items-center text-2xl">
                <div className="p-3 bg-green-500/10 rounded-xl mr-4">
                  <Layers className="w-6 h-6 text-green-500" />
                </div>
                Generation Parameters
              </CardTitle>
              <CardDescription className="text-lg leading-relaxed">
                Configure the size and processing parameters for optimal results
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-8">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="space-y-4">
                  <Label htmlFor="entries" className="text-base font-semibold text-foreground flex items-center">
                    <BarChart3 className="w-5 h-5 mr-2 text-green-500" />
                    Target Entries
                  </Label>
                  <div className="space-y-3">
                    <Input
                      id="entries"
                      type="number"
                      value={config.target_entries}
                      onChange={(e) => handleInputChange('target_entries', parseInt(e.target.value))}
                      min="100"
                      max="10000"
                      className="h-12 border-2 border-border bg-background text-foreground focus:border-green-500 focus:ring-green-500/20 transition-colors text-lg"
                    />
                    <div className="bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800/50 rounded-lg p-3">
                      <p className="text-sm text-green-700 dark:text-green-300">
                        Recommended: 1000-5000 entries for effective fine-tuning
                      </p>
                    </div>
                  </div>
                </div>

                <div className="space-y-4">
                  <Label htmlFor="batch" className="text-base font-semibold text-foreground flex items-center">
                    <Zap className="w-5 h-5 mr-2 text-green-500" />
                    Batch Size
                  </Label>
                  <div className="space-y-3">
                    <Input
                      id="batch"
                      type="number"
                      value={config.batch_size}
                      onChange={(e) => handleInputChange('batch_size', parseInt(e.target.value))}
                      min="10"
                      max="100"
                      className="h-12 border-2 border-border bg-background text-foreground focus:border-green-500 focus:ring-green-500/20 transition-colors text-lg"
                    />
                    <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 rounded-lg p-3">
                      <p className="text-sm text-amber-700 dark:text-amber-300">
                        Smaller batches = better quality, larger = faster processing
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              {/* Quick Presets */}
              <div className="space-y-4">
                <Label className="text-base font-semibold text-foreground">Quick Presets</Label>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  {[
                    { label: 'Small Dataset', entries: 500, batch: 25, desc: 'Quick testing' },
                    { label: 'Medium Dataset', entries: 2000, batch: 50, desc: 'Balanced approach' },
                    { label: 'Large Dataset', entries: 5000, batch: 75, desc: 'Production ready' },
                  ].map((preset) => (
                    <Button
                      key={preset.label}
                      variant="outline"
                      onClick={() => onConfigChange({
                        target_entries: preset.entries,
                        batch_size: preset.batch
                      })}
                      className="h-auto p-4 flex flex-col items-start space-y-2 hover:bg-green-50 dark:hover:bg-green-950/30 hover:border-green-500/50 transition-colors"
                    >
                      <div className="font-semibold text-left">{preset.label}</div>
                      <div className="text-xs text-muted-foreground text-left">
                        {preset.entries.toLocaleString()} entries • Batch {preset.batch}
                      </div>
                      <div className="text-xs text-muted-foreground text-left">
                        {preset.desc}
                      </div>
                    </Button>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Summary Panel */}
        <div className="space-y-6">
          {/* Selected Model */}
          <Card className="border-blue-500/30 bg-blue-500/5 backdrop-blur-sm shadow-xl">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground text-xl flex items-center">
                <CheckCircle className="w-6 h-6 mr-3 text-blue-500" />
                Selected Model
              </CardTitle>
            </CardHeader>
            <CardContent>
              {selectedModel ? (
                <div className="space-y-6">
                  <div className="text-center">
                    <div className="font-bold text-foreground text-xl mb-2">{selectedModel.name}</div>
                    <div className="flex items-center justify-center space-x-3">
                      <Badge variant="outline" className="bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/30 px-3 py-1">
                        {selectedModel.provider}
                      </Badge>
                      <span className="text-muted-foreground font-medium">{selectedModel.size}</span>
                    </div>
                  </div>
                  {selectedModel.capabilities.length > 0 && (
                    <div>
                      <Label className="text-sm font-medium text-muted-foreground mb-2 block">Capabilities</Label>
                      <div className="flex flex-wrap gap-2">
                        {selectedModel.capabilities.map((cap) => (
                          <Badge key={cap} variant="secondary" className="text-xs bg-muted/50 px-2 py-1">
                            {cap}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <div className="text-center py-8">
                  <AlertCircle className="w-12 h-12 text-muted-foreground mx-auto mb-3" />
                  <div className="text-muted-foreground">No model selected</div>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Generation Summary */}
          <Card className="border-purple-500/30 bg-purple-500/5 backdrop-blur-sm shadow-xl">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground text-xl flex items-center">
                <Info className="w-6 h-6 mr-3 text-purple-500" />
                Generation Summary
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                {[
                  {
                    label: 'Total Entries',
                    value: config.target_entries.toLocaleString(),
                    icon: BarChart3,
                    color: 'text-blue-500'
                  },
                  {
                    label: 'Batch Size',
                    value: config.batch_size.toString(),
                    icon: Layers,
                    color: 'text-green-500'
                  },
                  {
                    label: 'Format',
                    value: config.format.charAt(0).toUpperCase() + config.format.slice(1).replace('_', ' '),
                    icon: Database,
                    color: 'text-purple-500'
                  },
                  {
                    label: 'Total Batches',
                    value: estimatedBatches.toString(),
                    icon: Zap,
                    color: 'text-orange-500'
                  },
                  {
                    label: 'Est. Time',
                    value: `${estimatedTime} min`,
                    icon: Clock,
                    color: 'text-red-500'
                  },
                  {
                    label: 'Model',
                    value: selectedModel?.name || 'Not selected',
                    icon: Target,
                    color: 'text-cyan-500'
                  },
                ].map((item) => (
                  <div key={item.label} className="text-center p-4 bg-muted/30 rounded-xl border border-muted/50">
                    <item.icon className={cn("w-6 h-6 mx-auto mb-2", item.color)} />
                    <div className="text-xs text-muted-foreground mb-1">{item.label}</div>
                    <div className="text-lg font-bold text-foreground">{item.value}</div>
                  </div>
                ))}
              </div>

              <div className="bg-purple-50 dark:bg-purple-950/30 border border-purple-200 dark:border-purple-800/50 rounded-lg p-4">
                <div className="flex items-center text-sm text-purple-700 dark:text-purple-300">
                  <Zap className="w-4 h-4 mr-2 flex-shrink-0" />
                  <span>Processing will happen in background with real-time progress updates</span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Validation Status */}
          <Card className={cn(
            "border-2 shadow-xl backdrop-blur-sm transition-all duration-300",
            isFormValid
              ? "border-green-500/50 bg-green-500/10"
              : "border-amber-500/50 bg-amber-500/10"
          )}>
            <CardContent className="p-6">
              <div className="flex items-center space-x-4">
                <div className={cn(
                  "p-3 rounded-full",
                  isFormValid ? "bg-green-500/20" : "bg-amber-500/20"
                )}>
                  {isFormValid ? (
                    <CheckCircle className="w-6 h-6 text-green-500" />
                  ) : (
                    <AlertCircle className="w-6 h-6 text-amber-500" />
                  )}
                </div>
                <div>
                  <div className={cn(
                    "font-bold text-lg",
                    isFormValid ? "text-green-600 dark:text-green-400" : "text-amber-600 dark:text-amber-400"
                  )}>
                    {isFormValid ? 'Ready to Generate' : 'Configuration Required'}
                  </div>
                  <div className="text-muted-foreground">
                    {isFormValid
                      ? 'All required fields are completed. Ready to start generation!'
                      : 'Please provide a fine-tuning goal to continue'
                    }
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Start Button */}
          <Button
            onClick={onStart}
            disabled={!isFormValid || isGenerating}
            size="lg"
            className={cn(
              "w-full py-8 text-xl font-bold shadow-xl transition-all duration-300 rounded-2xl",
              isFormValid
                ? "bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700 text-white hover:shadow-2xl hover:scale-[1.02]"
                : "bg-muted text-muted-foreground cursor-not-allowed"
            )}
          >
            <Play className="w-7 h-7 mr-3" />
            {isGenerating ? 'Generating Dataset...' : 'Start Generation'}
          </Button>
        </div>
      </div>
    </div>
  );
};