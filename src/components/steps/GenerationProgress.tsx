import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { 
  Database, 
  Clock, 
  Zap, 
  CheckCircle, 
  Loader2,
  BarChart3,
  Activity
} from 'lucide-react';
import { GenerationProgress as ProgressType, GenerationConfig } from '@/types';
import { cn } from '@/lib/utils';

interface GenerationProgressProps {
  progress: ProgressType | null;
  config: GenerationConfig;
  isGenerating: boolean;
}

export const GenerationProgress: React.FC<GenerationProgressProps> = ({
  progress,
  config,
  isGenerating,
}) => {
  if (!progress) {
    return (
      <div className="space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <div className="p-3 bg-gradient-to-br from-blue-500/20 to-purple-500/20 rounded-xl border border-blue-500/20">
              <Database className="w-8 h-8 text-blue-500" />
            </div>
            <div>
              <h2 className="text-3xl font-bold text-foreground">Dataset Generation</h2>
              <p className="text-muted-foreground text-lg">Initializing generation process...</p>
            </div>
          </div>
        </div>
        
        <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
          <CardContent className="flex items-center justify-center py-16">
            <div className="text-center space-y-4">
              <div className="p-4 bg-blue-500/10 rounded-full mx-auto w-fit">
                <Loader2 className="w-12 h-12 text-blue-500 animate-spin" />
              </div>
              <div>
                <h3 className="text-xl font-semibold text-foreground mb-2">Starting Generation</h3>
                <p className="text-muted-foreground">Preparing your dataset generation...</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  const progressPercentage = (progress.entries_generated / config.target_entries) * 100;
  const isCompleted = progress.status === 'completed';
  const isRunning = isGenerating && !isCompleted;

  const getStatusColor = () => {
    if (isCompleted) return 'text-green-500';
    if (isRunning) return 'text-blue-500';
    return 'text-muted-foreground';
  };

  const getStatusIcon = () => {
    if (isCompleted) return CheckCircle;
    if (isRunning) return Loader2;
    return Activity;
  };

  const StatusIcon = getStatusIcon();

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="p-3 bg-gradient-to-br from-blue-500/20 to-purple-500/20 rounded-xl border border-blue-500/20">
            <Database className="w-8 h-8 text-blue-500" />
          </div>
          <div>
            <h2 className="text-3xl font-bold text-foreground">Dataset Generation</h2>
            <p className="text-muted-foreground text-lg">Monitoring generation progress in real-time</p>
          </div>
        </div>
      </div>

      {/* Main Progress Card */}
      <Card className="border-border bg-card/50 backdrop-blur-sm shadow-lg">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="text-foreground flex items-center text-xl">
              <div className={cn(
                "p-2 rounded-lg mr-3",
                isCompleted ? "bg-green-500/10" : isRunning ? "bg-blue-500/10" : "bg-muted/50"
              )}>
                <StatusIcon className={cn(
                  "w-5 h-5",
                  getStatusColor(),
                  isRunning && "animate-spin"
                )} />
              </div>
              Generation Progress
            </CardTitle>
            <Badge 
              variant={isCompleted ? "default" : "secondary"}
              className={cn(
                isCompleted && "bg-green-500 hover:bg-green-600 text-white",
                isRunning && "bg-blue-500 hover:bg-blue-600 text-white animate-pulse"
              )}
            >
              {progress.status}
            </Badge>
          </div>
          <CardDescription className="text-base">
            {progress.entries_generated.toLocaleString()} of {config.target_entries.toLocaleString()} entries generated
          </CardDescription>
        </CardHeader>
        
        <CardContent className="space-y-6">
          {/* Progress Bar */}
          <div className="space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-foreground font-medium">Overall Progress</span>
              <span className="text-foreground font-semibold">{Math.round(progressPercentage)}%</span>
            </div>
            <Progress 
              value={progressPercentage} 
              className="h-4"
            />
            <p className="text-xs text-muted-foreground text-center">
              {isRunning ? 'Generation in progress...' : isCompleted ? 'Generation completed successfully!' : 'Preparing to generate...'}
            </p>
          </div>

          {/* Stats Grid */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card className="border-border bg-muted/30">
              <CardContent className="p-4">
                <div className="flex items-center space-x-3 mb-3">
                  <div className="p-2 bg-blue-500/10 rounded-lg">
                    <BarChart3 className="w-4 h-4 text-blue-500" />
                  </div>
                  <span className="text-muted-foreground text-sm font-medium">Batch Progress</span>
                </div>
                <div className="text-foreground font-bold text-xl">
                  {progress.current_batch} / {progress.total_batches}
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Batches completed
                </div>
              </CardContent>
            </Card>

            <Card className="border-border bg-muted/30">
              <CardContent className="p-4">
                <div className="flex items-center space-x-3 mb-3">
                  <div className="p-2 bg-green-500/10 rounded-lg">
                    <Zap className="w-4 h-4 text-green-500" />
                  </div>
                  <span className="text-muted-foreground text-sm font-medium">Entries Generated</span>
                </div>
                <div className="text-foreground font-bold text-xl">
                  {progress.entries_generated.toLocaleString()}
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Dataset entries created
                </div>
              </CardContent>
            </Card>

            <Card className="border-border bg-muted/30">
              <CardContent className="p-4">
                <div className="flex items-center space-x-3 mb-3">
                  <div className="p-2 bg-purple-500/10 rounded-lg">
                    <Clock className="w-4 h-4 text-purple-500" />
                  </div>
                  <span className="text-muted-foreground text-sm font-medium">Time Remaining</span>
                </div>
                <div className="text-foreground font-bold text-xl">
                  {progress.estimated_completion}
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Estimated completion
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Current Status */}
          <Card className="border-border bg-muted/20">
            <CardContent className="p-4">
              <div className="flex items-center space-x-3">
                <Activity className="w-5 h-5 text-blue-500" />
                <div>
                  <div className="text-muted-foreground text-sm font-medium">Current Status</div>
                  <div className="text-foreground font-semibold capitalize">{progress.status.replace('_', ' ')}</div>
                </div>
              </div>
            </CardContent>
          </Card>
        </CardContent>
      </Card>

      {/* Completion Message */}
      {isCompleted && (
        <Card className="border-green-500/50 bg-green-500/5 shadow-green-500/20 shadow-lg">
          <CardContent className="py-6">
            <div className="flex items-center space-x-4">
              <div className="p-3 bg-green-500/10 rounded-full">
                <CheckCircle className="w-8 h-8 text-green-500" />
              </div>
              <div>
                <h3 className="text-green-600 dark:text-green-400 font-bold text-lg">Generation Complete!</h3>
                <p className="text-green-600/80 dark:text-green-300 text-base">
                  Successfully generated {progress.entries_generated.toLocaleString()} dataset entries. 
                  Your dataset is ready for export.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};