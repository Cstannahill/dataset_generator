import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  Download, 
  RefreshCw, 
  FileJson, 
  CheckCircle, 
  Users, 
  Layers,
  HardDrive,
  Cloud,
  Sparkles,
  BarChart3,
  Bug
} from 'lucide-react';
import { GenerationProgress } from '@/types';
import { invoke } from '@tauri-apps/api/core';

interface ExportDatasetProps {
  progress: GenerationProgress | null;
  onExport: () => void;
  onStartNew: () => void;
}

export const ExportDataset: React.FC<ExportDatasetProps> = ({
  progress,
  onExport,
  onStartNew,
}) => {
  const entryCount = progress?.entries_generated || 0;
  
  const debugDataset = async () => {
    try {
      const debugInfo = await invoke('debug_dataset_state') as string;
      alert('Debug Info:\n' + debugInfo);
    } catch (error) {
      alert('Debug Error: ' + error);
    }
  };

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="p-3 bg-gradient-to-br from-green-500/20 to-blue-500/20 rounded-xl border border-green-500/20">
            <Download className="w-8 h-8 text-green-500" />
          </div>
          <div>
            <h2 className="text-3xl font-bold text-foreground">Export Dataset</h2>
            <p className="text-muted-foreground text-lg">Your fine-tuning dataset is ready for download</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Dataset Info */}
        <div className="lg:col-span-2 space-y-6">
          {/* Success Card */}
          <Card className="border-green-500/50 bg-green-500/5 shadow-green-500/20 shadow-lg">
            <CardContent className="py-8">
              <div className="flex items-center space-x-4">
                <div className="p-4 bg-green-500/10 rounded-full">
                  <CheckCircle className="w-10 h-10 text-green-500" />
                </div>
                <div>
                  <h3 className="text-green-600 dark:text-green-400 font-bold text-xl">Generation Complete!</h3>
                  <p className="text-green-600/80 dark:text-green-300 text-base mt-1">
                    Your dataset has been successfully generated with {entryCount.toLocaleString()} high-quality entries.
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Dataset Details */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground flex items-center text-xl">
                <div className="p-2 bg-blue-500/10 rounded-lg mr-3">
                  <FileJson className="w-5 h-5 text-blue-500" />
                </div>
                Dataset Information
              </CardTitle>
              <CardDescription className="text-base">
                Details about your generated fine-tuning dataset
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Card className="border-border bg-muted/30">
                  <CardContent className="p-4">
                    <div className="flex items-center space-x-3 mb-3">
                      <div className="p-2 bg-blue-500/10 rounded-lg">
                        <Users className="w-4 h-4 text-blue-500" />
                      </div>
                      <span className="text-muted-foreground text-sm font-medium">Total Entries</span>
                    </div>
                    <div className="text-2xl font-bold text-foreground">
                      {entryCount.toLocaleString()}
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">
                      Training examples generated
                    </div>
                  </CardContent>
                </Card>

                <Card className="border-border bg-muted/30">
                  <CardContent className="p-4">
                    <div className="flex items-center space-x-3 mb-3">
                      <div className="p-2 bg-purple-500/10 rounded-lg">
                        <Layers className="w-4 h-4 text-purple-500" />
                      </div>
                      <span className="text-muted-foreground text-sm font-medium">Format</span>
                    </div>
                    <div className="text-2xl font-bold text-foreground">JSON</div>
                    <div className="text-xs text-muted-foreground mt-1">
                      Structured training format
                    </div>
                  </CardContent>
                </Card>
              </div>

              <Card className="border-border bg-muted/20">
                <CardContent className="p-4">
                  <h4 className="text-foreground font-semibold mb-4 flex items-center">
                    <FileJson className="w-4 h-4 mr-2 text-blue-500" />
                    Data Structure
                  </h4>
                  <div className="space-y-3 text-sm">
                    <div className="flex items-center justify-between p-2 bg-background/50 rounded-lg">
                      <span className="text-muted-foreground font-medium">• instruction:</span>
                      <span className="text-foreground">Task description</span>
                    </div>
                    <div className="flex items-center justify-between p-2 bg-background/50 rounded-lg">
                      <span className="text-muted-foreground font-medium">• input:</span>
                      <span className="text-foreground">Context or input data</span>
                    </div>
                    <div className="flex items-center justify-between p-2 bg-background/50 rounded-lg">
                      <span className="text-muted-foreground font-medium">• output:</span>
                      <span className="text-foreground">Expected response</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </CardContent>
          </Card>

          {/* Compatibility Info */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground flex items-center text-xl">
                <div className="p-2 bg-yellow-500/10 rounded-lg mr-3">
                  <Sparkles className="w-5 h-5 text-yellow-500" />
                </div>
                Platform Compatibility
              </CardTitle>
              <CardDescription className="text-base">
                Your dataset is compatible with these fine-tuning platforms
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <Card className="border-border bg-muted/30 hover:bg-muted/50 transition-colors">
                  <CardContent className="flex items-center space-x-3 p-4">
                    <div className="p-2 bg-green-500/10 rounded-lg">
                      <HardDrive className="w-4 h-4 text-green-500" />
                    </div>
                    <span className="text-sm font-medium text-foreground">Ollama</span>
                  </CardContent>
                </Card>
                <Card className="border-border bg-muted/30 hover:bg-muted/50 transition-colors">
                  <CardContent className="flex items-center space-x-3 p-4">
                    <div className="p-2 bg-blue-500/10 rounded-lg">
                      <Cloud className="w-4 h-4 text-blue-500" />
                    </div>
                    <span className="text-sm font-medium text-foreground">OpenAI</span>
                  </CardContent>
                </Card>
                <Card className="border-border bg-muted/30 hover:bg-muted/50 transition-colors">
                  <CardContent className="flex items-center space-x-3 p-4">
                    <div className="p-2 bg-orange-500/10 rounded-lg">
                      <Layers className="w-4 h-4 text-orange-500" />
                    </div>
                    <span className="text-sm font-medium text-foreground">HuggingFace</span>
                  </CardContent>
                </Card>
                <Card className="border-border bg-muted/30 hover:bg-muted/50 transition-colors">
                  <CardContent className="flex items-center space-x-3 p-4">
                    <div className="p-2 bg-purple-500/10 rounded-lg">
                      <Sparkles className="w-4 h-4 text-purple-500" />
                    </div>
                    <span className="text-sm font-medium text-foreground">Custom</span>
                  </CardContent>
                </Card>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Actions Panel */}
        <div className="space-y-6">
          {/* Quick Stats */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground text-lg flex items-center">
                <BarChart3 className="w-5 h-5 mr-2 text-blue-500" />
                Quick Stats
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex justify-between items-center p-3 bg-muted/30 rounded-lg">
                <span className="text-muted-foreground font-medium">File Size:</span>
                <Badge variant="secondary" className="bg-blue-500/10 text-blue-500">~{Math.round((entryCount * 0.5))}KB</Badge>
              </div>
              <div className="flex justify-between items-center p-3 bg-muted/30 rounded-lg">
                <span className="text-muted-foreground font-medium">Quality:</span>
                <Badge className="bg-green-500 hover:bg-green-600 text-white">High</Badge>
              </div>
              <div className="flex justify-between items-center p-3 bg-muted/30 rounded-lg">
                <span className="text-muted-foreground font-medium">Format:</span>
                <Badge variant="outline" className="border-border bg-background">JSON</Badge>
              </div>
            </CardContent>
          </Card>

          {/* Export Actions */}
          <div className="space-y-4">
            <Button
              onClick={onExport}
              className="w-full bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700 text-white py-6 text-lg font-semibold shadow-lg hover:shadow-xl transition-all duration-300"
              size="lg"
            >
              <Download className="w-6 h-6 mr-2" />
              Download Dataset
            </Button>

            <Button
              onClick={debugDataset}
              variant="outline"
              className="w-full border-yellow-500 bg-yellow-500/10 hover:bg-yellow-500/20 text-yellow-600 dark:text-yellow-400 py-4 text-sm font-semibold transition-all duration-300"
              size="sm"
            >
              <Bug className="w-4 h-4 mr-2" />
              Debug Dataset State
            </Button>

            <Button
              onClick={onStartNew}
              variant="outline"
              className="w-full border-border bg-card hover:bg-accent py-6 text-lg font-semibold transition-all duration-300"
              size="lg"
            >
              <RefreshCw className="w-6 h-6 mr-2" />
              Generate New Dataset
            </Button>
          </div>

          {/* Tips */}
          <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
            <CardHeader className="pb-4">
              <CardTitle className="text-foreground text-lg flex items-center">
                <div className="p-2 bg-yellow-500/10 rounded-lg mr-3">
                  <Sparkles className="w-4 h-4 text-yellow-500" />
                </div>
                Pro Tips
              </CardTitle>
            </CardHeader>
            <CardContent>
              <ul className="text-sm text-muted-foreground space-y-3">
                <li className="flex items-start space-x-2">
                  <CheckCircle className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                  <span>Review entries before fine-tuning</span>
                </li>
                <li className="flex items-start space-x-2">
                  <CheckCircle className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                  <span>Consider data diversity for better results</span>
                </li>
                <li className="flex items-start space-x-2">
                  <CheckCircle className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                  <span>Validate format compatibility</span>
                </li>
                <li className="flex items-start space-x-2">
                  <CheckCircle className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                  <span>Keep backups of your datasets</span>
                </li>
              </ul>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};