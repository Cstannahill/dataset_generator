import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    Brain,
    Lightbulb,
    AlertTriangle,
    CheckCircle,
    Zap,
    Eye,
    EyeOff,
    TrendingUp,
    MessageSquare
} from 'lucide-react';
import { PromptFeedbackService, ValidationFeedback } from '@/lib/prompt-feedback';
import { DatasetEntry, DatasetFormat } from '@/types';
import { cn } from '@/lib/utils';

interface FeedbackPanelProps {
    entries: DatasetEntry[];
    useCase: string;
    format: DatasetFormat;
    onPromptImprovement?: (improvement: string) => void;
    className?: string;
}

export const FeedbackPanel: React.FC<FeedbackPanelProps> = ({
    entries,
    useCase,
    format,
    onPromptImprovement,
    className,
}) => {
    const [feedback, setFeedback] = useState<ValidationFeedback | null>(null);
    const [isAnalyzing, setIsAnalyzing] = useState(false);
    const [showDetails, setShowDetails] = useState(false);
    const [improvements, setImprovements] = useState<string>('');

    // Auto-analyze when entries change (e.g., after each batch)
    useEffect(() => {
        if (entries.length > 0) {
            analyzeBatch();
        }
    }, [entries.length]);

    const analyzeBatch = async () => {
        if (entries.length === 0) return;

        setIsAnalyzing(true);
        try {
            const improvementText = await PromptFeedbackService.generatePromptImprovements(
                entries,
                useCase,
                format
            );
            setImprovements(improvementText);

            // Parse feedback from the improvement text (simplified)
            const mockFeedback: ValidationFeedback = {
                common_issues: [],
                improvement_suggestions: [],
                quality_patterns: [],
                avoid_patterns: [],
                batch_summary: improvementText ? 'Analysis completed with suggestions' : 'No specific issues found'
            };

            setFeedback(mockFeedback);

            if (improvementText && onPromptImprovement) {
                onPromptImprovement(improvementText);
            }
        } catch (error) {
            console.error('Failed to analyze batch:', error);
            setFeedback({
                common_issues: ['Analysis failed'],
                improvement_suggestions: [],
                quality_patterns: [],
                avoid_patterns: [],
                batch_summary: 'Unable to analyze this batch'
            });
        } finally {
            setIsAnalyzing(false);
        }
    };

    const getPriorityLevel = () => {
        if (!feedback) return 'low';
        return PromptFeedbackService.getFeedbackPriority(feedback) > 0.5 ? 'high' : 'medium';
    };

    const requiresAttention = feedback && PromptFeedbackService.requiresPromptAdjustment(feedback);

    return (
        <Card className={cn(
            'border transition-all duration-300',
            requiresAttention
                ? 'border-orange-500/50 bg-orange-500/5'
                : 'border-green-500/50 bg-green-500/5',
            className
        )}>
            <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                        <div className={cn(
                            'p-2 rounded-lg',
                            requiresAttention
                                ? 'bg-orange-500/20 text-orange-500'
                                : 'bg-green-500/20 text-green-500'
                        )}>
                            <Brain className="w-5 h-5" />
                        </div>
                        <div>
                            <CardTitle className="text-lg flex items-center gap-2">
                                AI Quality Feedback
                                {isAnalyzing && <Zap className="w-4 h-4 animate-pulse text-blue-500" />}
                            </CardTitle>
                            <CardDescription>
                                {entries.length > 0
                                    ? `Analyzed ${entries.length} entries`
                                    : 'Waiting for entries to analyze'
                                }
                            </CardDescription>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        <Badge variant={requiresAttention ? 'destructive' : 'default'}>
                            {getPriorityLevel().toUpperCase()}
                        </Badge>
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setShowDetails(!showDetails)}
                        >
                            {showDetails ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                        </Button>
                    </div>
                </div>
            </CardHeader>

            <CardContent className="space-y-4">
                {/* Summary */}
                {feedback && (
                    <div className="flex items-start gap-3 p-3 rounded-lg bg-muted/50">
                        <MessageSquare className="w-5 h-5 text-muted-foreground mt-0.5" />
                        <div>
                            <p className="text-sm font-medium mb-1">Batch Summary</p>
                            <p className="text-sm text-muted-foreground">{feedback.batch_summary}</p>
                        </div>
                    </div>
                )}

                {/* Quick Actions */}
                {improvements && (
                    <div className="flex items-center justify-between p-3 rounded-lg border bg-blue-500/5 border-blue-500/20">
                        <div className="flex items-center gap-2">
                            <TrendingUp className="w-4 h-4 text-blue-500" />
                            <span className="text-sm font-medium">Prompt improvements available</span>
                        </div>
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={() => onPromptImprovement?.(improvements)}
                        >
                            Apply Now
                        </Button>
                    </div>
                )}

                {/* Detailed Feedback */}
                {showDetails && feedback && (
                    <div className="space-y-3 border-t pt-4">
                        {feedback.avoid_patterns.length > 0 && (
                            <div className="space-y-2">
                                <div className="flex items-center gap-2">
                                    <AlertTriangle className="w-4 h-4 text-red-500" />
                                    <span className="text-sm font-medium">Patterns to Avoid</span>
                                </div>
                                <ul className="space-y-1 ml-6">
                                    {feedback.avoid_patterns.map((pattern, idx) => (
                                        <li key={idx} className="text-sm text-muted-foreground">
                                            • {pattern}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        {feedback.improvement_suggestions.length > 0 && (
                            <div className="space-y-2">
                                <div className="flex items-center gap-2">
                                    <Lightbulb className="w-4 h-4 text-yellow-500" />
                                    <span className="text-sm font-medium">Focus More On</span>
                                </div>
                                <ul className="space-y-1 ml-6">
                                    {feedback.improvement_suggestions.map((suggestion, idx) => (
                                        <li key={idx} className="text-sm text-muted-foreground">
                                            • {suggestion}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        {feedback.quality_patterns.length > 0 && (
                            <div className="space-y-2">
                                <div className="flex items-center gap-2">
                                    <CheckCircle className="w-4 h-4 text-green-500" />
                                    <span className="text-sm font-medium">Successful Patterns</span>
                                </div>
                                <ul className="space-y-1 ml-6">
                                    {feedback.quality_patterns.map((pattern, idx) => (
                                        <li key={idx} className="text-sm text-muted-foreground">
                                            • {pattern}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}

                        {feedback.common_issues.length > 0 && (
                            <div className="space-y-2">
                                <div className="flex items-center gap-2">
                                    <AlertTriangle className="w-4 h-4 text-orange-500" />
                                    <span className="text-sm font-medium">Common Issues</span>
                                </div>
                                <ul className="space-y-1 ml-6">
                                    {feedback.common_issues.map((issue, idx) => (
                                        <li key={idx} className="text-sm text-muted-foreground">
                                            • {issue}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}
                    </div>
                )}

                {/* Action Buttons */}
                <div className="flex gap-2 pt-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={analyzeBatch}
                        disabled={isAnalyzing || entries.length === 0}
                    >
                        {isAnalyzing ? (
                            <>
                                <Zap className="w-4 h-4 mr-2 animate-pulse" />
                                Analyzing...
                            </>
                        ) : (
                            <>
                                <Brain className="w-4 h-4 mr-2" />
                                Re-analyze
                            </>
                        )}
                    </Button>
                </div>
            </CardContent>
        </Card>
    );
};

export default FeedbackPanel;
