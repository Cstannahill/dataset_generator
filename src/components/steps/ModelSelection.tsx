import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Brain, RefreshCw, ChevronRight, Zap, Cloud, HardDrive, CheckCircle, AlertTriangle, Monitor, Server } from 'lucide-react';
import { Model } from '@/types';
import { cn } from '@/lib/utils';

interface ModelSelectionProps {
  models: Model[];
  selectedModel: string;
  isDiscovering: boolean;
  onModelSelect: (modelId: string) => void;
  onRefreshModels: () => void;
  onNext: () => void;
}

export const ModelSelection: React.FC<ModelSelectionProps> = ({
  models,
  selectedModel,
  isDiscovering,
  onModelSelect,
  onRefreshModels,
  onNext,
}) => {
  const selectedModelData = models.find(m => m.id === selectedModel);

  const getProviderIcon = (provider: string) => {
    return provider === 'Ollama' ? Server : Cloud;
  };

  const getProviderColor = (provider: string) => {
    return provider === 'Ollama' ? 'text-green-500' : 'text-blue-500';
  };

  const formatModelDisplayName = (model: Model) => {
    return `${model.name} (${model.provider} • ${model.size})`;
  };

  return (
    <div className="space-y-8 max-w-4xl mx-auto">
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center">
          <div className="p-4 bg-gradient-to-br from-blue-500/20 to-purple-500/20 rounded-2xl border border-blue-500/20">
            <Brain className="w-10 h-10 text-blue-500" />
          </div>
        </div>
        <div>
          <h2 className="text-4xl font-bold text-foreground mb-2">Select AI Model</h2>
          <p className="text-muted-foreground text-xl max-w-2xl mx-auto">
            Choose the AI model that will generate your fine-tuning dataset
          </p>
        </div>
      </div>

      {/* Models Selection */}
      {models.length === 0 ? (
        <Card className="border-border bg-card/50 backdrop-blur-sm max-w-2xl mx-auto">
          <CardContent className="flex flex-col items-center justify-center py-20">
            <div className="p-6 bg-muted/50 rounded-full mb-8">
              <AlertTriangle className="w-16 h-16 text-muted-foreground" />
            </div>
            <CardTitle className="text-3xl text-foreground mb-4">No Models Found</CardTitle>
            <CardDescription className="text-center max-w-md text-lg mb-8 leading-relaxed">
              No AI models were discovered. Make sure Ollama is running locally or check your OpenAI API configuration.
            </CardDescription>
            <Button onClick={onRefreshModels} size="lg" variant="outline" className="px-8 py-4">
              <RefreshCw className="w-5 h-5 mr-2" />
              Try Again
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-8">
          {/* Model Selection Card */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-xl">
            <CardHeader className="pb-6">
              <div className="flex items-center justify-between">
                <CardTitle className="text-2xl text-foreground flex items-center">
                  <Monitor className="w-6 h-6 mr-3 text-blue-500" />
                  Available Models
                </CardTitle>
                <div className="flex items-center space-x-4">
                  <Badge variant="secondary" className="px-3 py-1">
                    {models.length} models found
                  </Badge>
                  <Button
                    onClick={onRefreshModels}
                    disabled={isDiscovering}
                    variant="outline"
                    size="sm"
                  >
                    <RefreshCw className={cn("w-4 h-4 mr-2", isDiscovering && "animate-spin")} />
                    {isDiscovering ? 'Refreshing...' : 'Refresh'}
                  </Button>
                </div>
              </div>
              <CardDescription className="text-lg">
                Select from locally available Ollama models or cloud-based OpenAI models
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-3">
                <Label htmlFor="model-select" className="text-base font-medium text-foreground">
                  Choose Model
                </Label>
                <Select value={selectedModel} onValueChange={onModelSelect}>
                  <SelectTrigger className="w-full h-14 text-left border-2 hover:border-blue-500/50 focus:border-blue-500 transition-colors">
                    <SelectValue placeholder="Select an AI model for dataset generation...">
                      {selectedModelData && (
                        <div className="flex items-center space-x-3">
                          <div className={cn(
                            "p-2 rounded-lg flex-shrink-0",
                            selectedModelData.provider === 'Ollama' ? 'bg-green-500/10' : 'bg-blue-500/10'
                          )}>
                            {React.createElement(getProviderIcon(selectedModelData.provider), { 
                              className: cn("w-5 h-5", getProviderColor(selectedModelData.provider)) 
                            })}
                          </div>
                          <div className="flex-1 min-w-0">
                            <div className="font-semibold text-foreground truncate">
                              {selectedModelData.name}
                            </div>
                            <div className="text-sm text-muted-foreground truncate">
                              {selectedModelData.provider} • {selectedModelData.size}
                            </div>
                          </div>
                        </div>
                      )}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent className="w-full max-h-80">
                    {models.map((model) => {
                      const ProviderIcon = getProviderIcon(model.provider);
                      return (
                        <SelectItem 
                          key={model.id} 
                          value={model.id}
                          className="p-4 cursor-pointer"
                        >
                          <div className="flex items-center space-x-3 w-full">
                            <div className={cn(
                              "p-2 rounded-lg flex-shrink-0",
                              model.provider === 'Ollama' ? 'bg-green-500/10' : 'bg-blue-500/10'
                            )}>
                              <ProviderIcon className={cn("w-5 h-5", getProviderColor(model.provider))} />
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="font-semibold text-foreground truncate">
                                {model.name}
                              </div>
                              <div className="text-sm text-muted-foreground truncate">
                                {model.provider} • {model.size}
                                {model.modified && ` • Modified ${model.modified}`}
                              </div>
                            </div>
                            <div className="flex-shrink-0">
                              <Badge 
                                variant="outline" 
                                className={cn(
                                  "font-medium text-xs",
                                  model.provider === 'Ollama' 
                                    ? 'bg-green-500/10 text-green-500 border-green-500/20' 
                                    : 'bg-blue-500/10 text-blue-500 border-blue-500/20'
                                )}
                              >
                                {model.provider}
                              </Badge>
                            </div>
                          </div>
                        </SelectItem>
                      );
                    })}
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          {/* Selected Model Details */}
          {selectedModelData && (
            <Card className="border-blue-500/50 bg-blue-500/5 shadow-lg">
              <CardHeader className="pb-4">
                <CardTitle className="text-xl text-foreground flex items-center">
                  <CheckCircle className="w-6 h-6 mr-3 text-blue-500" />
                  Selected Model Details
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-4">
                    <div>
                      <Label className="text-sm font-medium text-muted-foreground">Model Name</Label>
                      <p className="text-lg font-semibold text-foreground">{selectedModelData.name}</p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium text-muted-foreground">Provider</Label>
                      <div className="flex items-center space-x-2 mt-1">
                        {React.createElement(getProviderIcon(selectedModelData.provider), { 
                          className: cn("w-5 h-5", getProviderColor(selectedModelData.provider))
                        })}
                        <span className="text-lg font-medium text-foreground">{selectedModelData.provider}</span>
                      </div>
                    </div>
                  </div>
                  <div className="space-y-4">
                    <div>
                      <Label className="text-sm font-medium text-muted-foreground">Model Size</Label>
                      <p className="text-lg font-semibold text-foreground">{selectedModelData.size}</p>
                    </div>
                    {selectedModelData.modified && (
                      <div>
                        <Label className="text-sm font-medium text-muted-foreground">Last Modified</Label>
                        <p className="text-lg font-medium text-foreground">{selectedModelData.modified}</p>
                      </div>
                    )}
                  </div>
                </div>
                
                {selectedModelData.capabilities.length > 0 && (
                  <div>
                    <Label className="text-sm font-medium text-muted-foreground mb-3 block">Capabilities</Label>
                    <div className="flex flex-wrap gap-2">
                      {selectedModelData.capabilities.map((capability) => (
                        <Badge
                          key={capability}
                          variant="secondary"
                          className="bg-blue-500/10 text-blue-700 dark:text-blue-300 border-blue-500/20 px-3 py-1"
                        >
                          <Zap className="w-3 h-3 mr-1" />
                          {capability}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Continue Button */}
      {selectedModel && (
        <div className="flex justify-center pt-4">
          <Button
            onClick={onNext}
            size="lg"
            className="bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white px-12 py-6 text-xl font-semibold shadow-lg hover:shadow-xl transition-all duration-300 rounded-xl"
          >
            Configure Dataset Generation
            <ChevronRight className="w-6 h-6 ml-3" />
          </Button>
        </div>
      )}
    </div>
  );
};