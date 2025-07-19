import React from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ModeToggle } from '@/components/ui/mode-toggle';
import { AlertCircle, CheckCircle, Sparkles, Github, ExternalLink } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface AppLayoutProps {
  children: React.ReactNode;
  error?: string | null;
  success?: string | null;
}

export const AppLayout: React.FC<AppLayoutProps> = ({ children, error, success }) => {
  return (
    <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
      {/* Navigation Bar */}
      <nav className="sticky top-0 z-50 border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container mx-auto px-4 h-16 flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg">
              <Sparkles className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-xl font-bold text-foreground">Dataset Generator</h1>
              <p className="text-xs text-muted-foreground">AI-Powered Fine-tuning</p>
            </div>
          </div>
          
          <div className="flex items-center space-x-3">
            <Button variant="ghost" size="sm" className="text-muted-foreground hover:text-foreground">
              <Github className="w-4 h-4 mr-2" />
              GitHub
              <ExternalLink className="w-3 h-3 ml-1" />
            </Button>
            <ModeToggle />
          </div>
        </div>
      </nav>

      <div className="container mx-auto px-4 py-8 max-w-7xl">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-4xl md:text-6xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 to-blue-800 dark:from-blue-400 dark:via-purple-400 dark:to-blue-600 bg-clip-text text-transparent mb-6">
              AI Dataset Generator
            </h1>
            <p className="text-lg md:text-xl text-muted-foreground max-w-3xl mx-auto leading-relaxed">
              Generate high-quality fine-tuning datasets using local and cloud AI models with intelligent batch processing and advanced optimization
            </p>
            
            {/* Feature Pills */}
            <div className="flex flex-wrap justify-center gap-3 mt-8">
              {[
                '🤖 Multiple AI Models',
                '⚡ Batch Processing',
                '🎯 Custom Fine-tuning',
                '📊 Real-time Progress',
                '💾 Export Ready'
              ].map((feature, index) => (
                <div
                  key={index}
                  className="px-4 py-2 bg-muted/50 dark:bg-muted/20 rounded-full text-sm text-muted-foreground border border-border/50"
                >
                  {feature}
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Alerts */}
        {error && (
          <Alert className="mb-6 border-destructive/50 bg-destructive/10 text-destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {success && (
          <Alert className="mb-6 border-green-500/50 bg-green-500/10 text-green-600 dark:text-green-400">
            <CheckCircle className="h-4 w-4" />
            <AlertDescription>{success}</AlertDescription>
          </Alert>
        )}

        {/* Main Content Card */}
        <div className="bg-card/50 backdrop-blur-xl rounded-3xl border border-border/50 shadow-2xl p-8 md:p-12">
          <div className="relative">
            {/* Decorative gradient overlay */}
            <div className="absolute inset-0 bg-gradient-to-r from-blue-500/5 to-purple-500/5 rounded-3xl pointer-events-none" />
            
            {/* Content */}
            <div className="relative z-10">
              {children}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-12 text-center">
          <div className="flex flex-col md:flex-row items-center justify-center gap-4 text-sm text-muted-foreground">
            <p className="flex items-center gap-2">
              <span>Supports Ollama local models and OpenAI API</span>
              <span className="hidden md:inline">•</span>
            </p>
            <p>Advanced batch processing for optimal efficiency</p>
          </div>
          
          <div className="mt-4 pt-4 border-t border-border/50">
            <p className="text-xs text-muted-foreground">
              Built with React, TypeScript, Tauri, and Tailwind CSS
            </p>
          </div>
        </div>
      </div>

      {/* Background Decorations */}
      <div className="fixed inset-0 pointer-events-none overflow-hidden">
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-gradient-to-r from-blue-500/10 to-purple-500/10 rounded-full blur-3xl" />
        <div className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl" />
      </div>
    </div>
  );
};