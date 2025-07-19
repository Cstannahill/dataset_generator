import React from 'react';
import { Brain, Settings, Database, Download, Check } from 'lucide-react';
import { Step, StepConfig } from '@/types';
import { cn } from '@/lib/utils';

interface StepIndicatorProps {
  currentStep: Step;
  className?: string;
}

const stepConfigs: StepConfig[] = [
  { 
    key: 'models', 
    label: 'Select Model', 
    icon: Brain,
    description: 'Choose your AI model'
  },
  { 
    key: 'configuration', 
    label: 'Configure', 
    icon: Settings,
    description: 'Set generation parameters'
  },
  { 
    key: 'generating', 
    label: 'Generate', 
    icon: Database,
    description: 'Create dataset entries'
  },
  { 
    key: 'export', 
    label: 'Export', 
    icon: Download,
    description: 'Download your dataset'
  }
];

export const StepIndicator: React.FC<StepIndicatorProps> = ({ currentStep, className }) => {
  const getCurrentStepIndex = () => stepConfigs.findIndex(step => step.key === currentStep);
  const currentStepIndex = getCurrentStepIndex();

  const getStepStatus = (stepIndex: number) => {
    if (stepIndex < currentStepIndex) return 'completed';
    if (stepIndex === currentStepIndex) return 'current';
    return 'upcoming';
  };

  return (
    <div className={cn("mb-16", className)}>
      <div className="flex items-center justify-center">
        <div className="flex items-center space-x-8">
          {stepConfigs.map((step, index) => {
            const status = getStepStatus(index);
            const Icon = step.icon;
            
            return (
              <div key={step.key} className="flex items-center">
                {/* Step Circle */}
                <div className="flex flex-col items-center">
                  <div
                    className={cn(
                      "relative flex items-center justify-center w-16 h-16 rounded-full border-2 transition-all duration-500 shadow-lg backdrop-blur-sm",
                      {
                        "border-green-400 bg-green-500 text-white shadow-green-500/25 ring-4 ring-green-500/20": status === 'completed',
                        "border-blue-400 bg-blue-500 text-white shadow-blue-500/25 ring-4 ring-blue-500/20 scale-110": status === 'current',
                        "border-border bg-muted text-muted-foreground hover:border-border/80": status === 'upcoming',
                      }
                    )}
                  >
                    {status === 'completed' ? (
                      <Check className="w-7 h-7" />
                    ) : (
                      <Icon className="w-7 h-7" />
                    )}
                    
                    {/* Glow effect for current step */}
                    {status === 'current' && (
                      <div className="absolute inset-0 rounded-full bg-blue-500/20 animate-pulse" />
                    )}
                  </div>
                  
                  {/* Step Label */}
                  <div className="mt-4 text-center">
                    <div
                      className={cn(
                        "text-sm font-semibold transition-colors",
                        {
                          "text-green-500": status === 'completed',
                          "text-blue-500": status === 'current',
                          "text-muted-foreground": status === 'upcoming',
                        }
                      )}
                    >
                      {step.label}
                    </div>
                    <div className="text-xs text-muted-foreground mt-1 max-w-24 text-center">
                      {step.description}
                    </div>
                  </div>
                </div>

                {/* Connector Line */}
                {index < stepConfigs.length - 1 && (
                  <div className="relative mx-6">
                    <div
                      className={cn(
                        "w-20 h-0.5 transition-all duration-500",
                        {
                          "bg-green-400": index < currentStepIndex,
                          "bg-gradient-to-r from-blue-400 to-border": index === currentStepIndex - 1,
                          "bg-border": index >= currentStepIndex,
                        }
                      )}
                    />
                    
                    {/* Animated progress line for current transition */}
                    {index === currentStepIndex - 1 && (
                      <div className="absolute top-0 left-0 h-0.5 bg-blue-400 rounded-full animate-pulse" 
                           style={{ width: '100%' }} />
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
      
      {/* Current Step Description */}
      <div className="text-center mt-8">
        <div className="max-w-2xl mx-auto">
          <p className="text-muted-foreground">
            {stepConfigs[currentStepIndex]?.description && 
              `Step ${currentStepIndex + 1} of ${stepConfigs.length}: ${stepConfigs[currentStepIndex].description}`
            }
          </p>
        </div>
      </div>
    </div>
  );
};