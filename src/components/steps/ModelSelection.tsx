import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Brain, RefreshCw, ChevronRight, Zap, Cloud, HardDrive, CheckCircle, AlertTriangle } from 'lucide-react';
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
  const getProviderIcon = (provider: string) => {
    return provider === 'Ollama' ? HardDrive : Cloud;
  };

  const getProviderColor = (provider: string) => {
    return provider === 'Ollama' ? 'text-green-500' : 'text-blue-500';
  };

  const getProviderBadgeColor = (provider: string) => {
    return provider === 'Ollama' 
      ? 'bg-green-500/10 text-green-500 border-green-500/20' 
      : 'bg-blue-500/10 text-blue-500 border-blue-500/20';
  };

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="p-3 bg-gradient-to-br from-blue-500/20 to-purple-500/20 rounded-xl border border-blue-500/20">
            <Brain className="w-8 h-8 text-blue-500" />
          </div>
          <div>
            <h2 className="text-3xl font-bold text-foreground">Select AI Model</h2>
            <p className="text-muted-foreground text-lg">Choose the model that will generate your dataset</p>
          </div>
        </div>
        
        <Button
          onClick={onRefreshModels}
          disabled={isDiscovering}
          variant="outline"
          size="lg"
          className="border-border bg-card hover:bg-accent"
        >
          <RefreshCw className={cn("w-5 h-5 mr-2", isDiscovering && "animate-spin")} />
          {isDiscovering ? 'Discovering...' : 'Refresh Models'}
        </Button>
      </div>

      {/* Models Grid */}
      {models.length === 0 ? (
        <Card className="border-border bg-card/50 backdrop-blur-sm">
          <CardContent className="flex flex-col items-center justify-center py-16">
            <div className="p-4 bg-muted/50 rounded-full mb-6">
              <AlertTriangle className="w-12 h-12 text-muted-foreground" />
            </div>
            <CardTitle className="text-2xl text-foreground mb-3">No Models Found</CardTitle>
            <CardDescription className="text-center max-w-md text-lg mb-6">
              No AI models were discovered. Make sure Ollama is running locally or check your OpenAI API configuration.
            </CardDescription>
            <Button onClick={onRefreshModels} size="lg" variant="outline">
              <RefreshCw className="w-5 h-5 mr-2" />
              Try Again
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          {/* Models Count */}
          <div className="text-center">
            <p className="text-muted-foreground">
              Found <span className="font-semibold text-foreground">{models.length}</span> available models
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {models.map((model) => {
              const ProviderIcon = getProviderIcon(model.provider);
              const isSelected = selectedModel === model.id;
              
              return (
                <Card
                  key={model.id}
                  className={cn(
                    "cursor-pointer transition-all duration-300 hover:scale-[1.02] border-2 bg-card/50 backdrop-blur-sm hover:shadow-xl",
                    isSelected
                      ? "border-blue-500 bg-blue-500/5 shadow-blue-500/20 shadow-lg ring-1 ring-blue-500/20"
                      : "border-border hover:border-accent-foreground/20 hover:bg-accent/5"
                  )}
                  onClick={() => onModelSelect(model.id)}
                >
                  <CardHeader className="pb-4">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center space-x-3 mb-3">
                          <div className={cn(
                            "p-2 rounded-lg",
                            model.provider === 'Ollama' ? 'bg-green-500/10' : 'bg-blue-500/10'
                          )}>
                            <ProviderIcon className={cn("w-5 h-5", getProviderColor(model.provider))} />
                          </div>
                          <Badge 
                            variant="outline" 
                            className={cn("font-medium", getProviderBadgeColor(model.provider))}
                          >
                            {model.provider}
                          </Badge>
                        </div>
                        
                        <CardTitle className="text-foreground text-xl font-semibold mb-2 leading-tight">
                          {model.name}
                        </CardTitle>
                        
                        <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                          <span>Size: {model.size}</span>
                          {model.modified && (
                            <>
                              <span>•</span>
                              <span>{model.modified}</span>
                            </>
                          )}
                        </div>
                      </div>
                      
                      {isSelected && (
                        <div className="p-2 bg-blue-500 rounded-full">
                          <CheckCircle className="w-5 h-5 text-white" />
                        </div>
                      )}
                    </div>
                  </CardHeader>
                  
                  <CardContent className="pt-0">
                    <div className="space-y-4">
                      {/* Capabilities */}
                      <div>
                        <h4 className="text-sm font-medium text-foreground mb-2">Capabilities</h4>
                        <div className="flex flex-wrap gap-2">
                          {model.capabilities.map((capability) => (
                            <Badge
                              key={capability}
                              variant="secondary"
                              className="bg-muted/50 text-muted-foreground hover:bg-muted/70 text-xs"
                            >
                              <Zap className="w-3 h-3 mr-1" />
                              {capability}
                            </Badge>
                          ))}
                        </div>
                      </div>
                      
                      {/* Selection Indicator */}
                      {isSelected && (
                        <div className="flex items-center space-x-2 p-3 bg-blue-500/10 rounded-lg border border-blue-500/20">
                          <CheckCircle className="w-4 h-4 text-blue-500" />
                          <span className="text-sm font-medium text-blue-500">Selected</span>
                        </div>
                      )}
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </>
      )}

      {/* Continue Button */}
      {selectedModel && (
        <div className="flex justify-center pt-8">
          <Button
            onClick={onNext}
            size="lg"
            className="bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white px-8 py-6 text-lg font-semibold shadow-lg hover:shadow-xl transition-all duration-300"
          >
            Configure Dataset Generation
            <ChevronRight className="w-6 h-6 ml-2" />
          </Button>
        </div>
      )}
    </div>
  );
};