import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
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
  AlertCircle
} from 'lucide-react';
import { GenerationConfig, Model } from '@/types';
import { cn } from '@/lib/utils';

interface DatasetConfigurationProps {
  config: GenerationConfig;
  selectedModel: Model | undefined;
  isGenerating: boolean;
  onConfigChange: (config: Partial<GenerationConfig>) => void;
  onBack: () => void;
  onStart: () => void;
}

export const DatasetConfiguration: React.FC<DatasetConfigurationProps> = ({
  config,
  selectedModel,
  isGenerating,
  onConfigChange,
  onBack,
  onStart,
}) => {
  const estimatedBatches = Math.ceil(config.target_entries / config.batch_size);
  const estimatedTime = Math.ceil(estimatedBatches * 2); // Rough estimate: 2 minutes per batch

  const handleInputChange = (field: keyof GenerationConfig, value: string | number) => {
    onConfigChange({ [field]: value });
  };

  const isFormValid = config.fine_tuning_goal.trim().length > 0;

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="p-3 bg-gradient-to-br from-purple-500/20 to-pink-500/20 rounded-xl border border-purple-500/20">
            <Settings className="w-8 h-8 text-purple-500" />
          </div>
          <div>
            <h2 className="text-3xl font-bold text-foreground">Dataset Configuration</h2>
            <p className="text-muted-foreground text-lg">Define the parameters for your dataset generation</p>
          </div>
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
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground flex items-center text-xl">
                <div className="p-2 bg-blue-500/10 rounded-lg mr-3">
                  <Target className="w-5 h-5 text-blue-500" />
                </div>
                Fine-tuning Objective
              </CardTitle>
              <CardDescription className="text-base">
                Describe what you want the model to learn from this dataset
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-3">
                <Label htmlFor="goal" className="text-sm font-medium text-foreground flex items-center">
                  <FileText className="w-4 h-4 mr-2 text-muted-foreground" />
                  Fine-tuning Goal *
                </Label>
                <Textarea
                  id="goal"
                  value={config.fine_tuning_goal}
                  onChange={(e) => handleInputChange('fine_tuning_goal', e.target.value)}
                  placeholder="e.g., Generate creative product descriptions for e-commerce items, focusing on benefits and emotional appeal that drive conversions"
                  rows={4}
                  className="resize-none border-border bg-background text-foreground placeholder:text-muted-foreground focus:border-blue-500 focus:ring-blue-500/20"
                />
                <p className="text-xs text-muted-foreground flex items-center">
                  <Info className="w-3 h-3 mr-1" />
                  Be specific about the task, style, and desired output format
                </p>
              </div>

              <div className="space-y-3">
                <Label htmlFor="context" className="text-sm font-medium text-foreground">
                  Domain Context (Optional)
                </Label>
                <Textarea
                  id="context"
                  value={config.domain_context}
                  onChange={(e) => handleInputChange('domain_context', e.target.value)}
                  placeholder="Additional context about your domain, industry, or specific requirements that will help guide the generation"
                  rows={3}
                  className="resize-none border-border bg-background text-foreground placeholder:text-muted-foreground focus:border-blue-500 focus:ring-blue-500/20"
                />
              </div>
            </CardContent>
          </Card>

          {/* Generation Parameters */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground flex items-center text-xl">
                <div className="p-2 bg-green-500/10 rounded-lg mr-3">
                  <Layers className="w-5 h-5 text-green-500" />
                </div>
                Generation Parameters
              </CardTitle>
              <CardDescription className="text-base">
                Configure the size and processing parameters for optimal results
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-3">
                  <Label htmlFor="entries" className="text-sm font-medium text-foreground flex items-center">
                    <BarChart3 className="w-4 h-4 mr-2 text-muted-foreground" />
                    Target Entries
                  </Label>
                  <Input
                    id="entries"
                    type="number"
                    value={config.target_entries}
                    onChange={(e) => handleInputChange('target_entries', parseInt(e.target.value))}
                    min="100"
                    max="10000"
                    className="border-border bg-background text-foreground focus:border-blue-500 focus:ring-blue-500/20"
                  />
                  <p className="text-xs text-muted-foreground">
                    Recommended: 1000-5000 for effective fine-tuning
                  </p>
                </div>

                <div className="space-y-3">
                  <Label htmlFor="batch" className="text-sm font-medium text-foreground flex items-center">
                    <Zap className="w-4 h-4 mr-2 text-muted-foreground" />
                    Batch Size
                  </Label>
                  <Input
                    id="batch"
                    type="number"
                    value={config.batch_size}
                    onChange={(e) => handleInputChange('batch_size', parseInt(e.target.value))}
                    min="10"
                    max="100"
                    className="border-border bg-background text-foreground focus:border-blue-500 focus:ring-blue-500/20"
                  />
                  <p className="text-xs text-muted-foreground">
                    Smaller batches = better quality, larger = faster processing
                  </p>
                </div>
              </div>

              {/* Quick Presets */}
              <div className="space-y-3">
                <Label className="text-sm font-medium text-foreground">Quick Presets</Label>
                <div className="flex flex-wrap gap-2">
                  {[
                    { label: 'Small Dataset', entries: 500, batch: 25 },
                    { label: 'Medium Dataset', entries: 2000, batch: 50 },
                    { label: 'Large Dataset', entries: 5000, batch: 75 },
                  ].map((preset) => (
                    <Button
                      key={preset.label}
                      variant="outline"
                      size="sm"
                      onClick={() => onConfigChange({ 
                        target_entries: preset.entries, 
                        batch_size: preset.batch 
                      })}
                      className="text-xs"
                    >
                      {preset.label}
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
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-3">
              <CardTitle className="text-foreground text-lg flex items-center">
                <CheckCircle className="w-5 h-5 mr-2 text-green-500" />
                Selected Model
              </CardTitle>
            </CardHeader>
            <CardContent>
              {selectedModel ? (
                <div className="space-y-4">
                  <div>
                    <div className="font-semibold text-foreground text-lg">{selectedModel.name}</div>
                    <div className="text-sm text-muted-foreground flex items-center mt-1">
                      <Badge variant="outline" className="mr-2">
                        {selectedModel.provider}
                      </Badge>
                      <span>{selectedModel.size}</span>
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {selectedModel.capabilities.map((cap) => (
                      <Badge key={cap} variant="secondary" className="text-xs bg-muted/50">
                        {cap}
                      </Badge>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="text-muted-foreground">No model selected</div>
              )}
            </CardContent>
          </Card>

          {/* Generation Summary */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-3">
              <CardTitle className="text-foreground text-lg flex items-center">
                <Info className="w-5 h-5 mr-2 text-blue-500" />
                Generation Summary
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-3">
                {[
                  { label: 'Total Entries', value: config.target_entries.toLocaleString(), icon: BarChart3 },
                  { label: 'Batch Size', value: config.batch_size.toString(), icon: Layers },
                  { label: 'Total Batches', value: estimatedBatches.toString(), icon: Zap },
                  { label: 'Est. Time', value: `${estimatedTime} min`, icon: Clock },
                ].map((item) => (
                  <div key={item.label} className="flex items-center justify-between p-3 bg-muted/30 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <item.icon className="w-4 h-4 text-muted-foreground" />
                      <span className="text-sm text-muted-foreground">{item.label}:</span>
                    </div>
                    <span className="text-sm font-medium text-foreground">{item.value}</span>
                  </div>
                ))}
              </div>
              
              <div className="pt-3 border-t border-border">
                <div className="flex items-center text-xs text-muted-foreground bg-blue-500/10 p-2 rounded-lg">
                  <Zap className="w-3 h-3 mr-2 text-blue-500" />
                  Processing will happen in background
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Validation Status */}
          <Card className={cn(
            "border shadow-lg",
            isFormValid 
              ? "border-green-500/50 bg-green-500/5" 
              : "border-amber-500/50 bg-amber-500/5"
          )}>
            <CardContent className="pt-6">
              <div className="flex items-center space-x-3">
                {isFormValid ? (
                  <CheckCircle className="w-5 h-5 text-green-500" />
                ) : (
                  <AlertCircle className="w-5 h-5 text-amber-500" />
                )}
                <div>
                  <div className={cn(
                    "font-medium text-sm",
                    isFormValid ? "text-green-600 dark:text-green-400" : "text-amber-600 dark:text-amber-400"
                  )}>
                    {isFormValid ? 'Ready to Generate' : 'Configuration Required'}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {isFormValid 
                      ? 'All required fields are completed'
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
              "w-full py-6 text-lg font-semibold shadow-lg transition-all duration-300",
              isFormValid
                ? "bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700 text-white hover:shadow-xl"
                : "bg-muted text-muted-foreground cursor-not-allowed"
            )}
          >
            <Play className="w-6 h-6 mr-2" />
            {isGenerating ? 'Generating...' : 'Start Generation'}
          </Button>
        </div>
      </div>
    </div>
  );
};