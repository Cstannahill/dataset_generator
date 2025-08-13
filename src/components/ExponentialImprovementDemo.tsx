import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
    TrendingUp,
    Zap,
    Brain,
    Target,
    ArrowUp,
    ArrowDown,
    BarChart3,
    Clock,
    CheckCircle,
    AlertCircle,
    Info
} from 'lucide-react';
import { ExponentialQualityService, BatchSizeAnalysis } from '@/lib/exponential-quality';
import { cn } from '@/lib/utils';

interface ExponentialImprovementDemoProps {
    totalEntries?: number;
    currentBatch?: number;
    className?: string;
}

export const ExponentialImprovementDemo: React.FC<ExponentialImprovementDemoProps> = ({
    totalEntries = 1000,
    currentBatch = 0,
    className,
}) => {
    const [selectedAnalysis, setSelectedAnalysis] = useState<BatchSizeAnalysis | null>(null);
    const [isSimulating, setIsSimulating] = useState(false);

    // Analyze different batch sizes
    const batchSizes = [10, 20, 50, 100, 200];
    const analyses = ExponentialQualityService.analyzeExponentialGrowth(
        totalEntries,
        currentBatch,
        batchSizes
    );

    const recommendation = ExponentialQualityService.generateOptimizationRecommendations(analyses);
    const explanation = ExponentialQualityService.getExponentialExplanation();

    useEffect(() => {
        if (!selectedAnalysis && analyses.length > 0) {
            setSelectedAnalysis(analyses.find(a => a.batchSize === 50) || analyses[0]);
        }
    }, [analyses]);

    const simulateQualityGrowth = () => {
        if (!selectedAnalysis) return;

        setIsSimulating(true);
        setTimeout(() => setIsSimulating(false), 2000);
    };

    const qualityCurve = selectedAnalysis
        ? ExponentialQualityService.simulateQualityGrowth(selectedAnalysis.batchSize, selectedAnalysis.totalBatches)
        : [];

    return (
        <div className={cn('space-y-6', className)}>
            {/* Header */}
            <Card className="border-gradient-to-r from-purple-500/20 to-blue-500/20">
                <CardHeader>
                    <div className="flex items-center space-x-3">
                        <div className="p-3 rounded-lg bg-gradient-to-br from-purple-500/20 to-blue-500/20">
                            <TrendingUp className="w-6 h-6 text-purple-500" />
                        </div>
                        <div>
                            <CardTitle className="text-xl">Exponential Quality Improvement</CardTitle>
                            <CardDescription>
                                How smaller batches create compounding quality gains through continuous feedback
                            </CardDescription>
                        </div>
                    </div>
                </CardHeader>
            </Card>

            <Tabs defaultValue="analysis" className="space-y-4">
                <TabsList className="grid w-full grid-cols-4">
                    <TabsTrigger value="analysis">Batch Analysis</TabsTrigger>
                    <TabsTrigger value="theory">Theory</TabsTrigger>
                    <TabsTrigger value="simulation">Simulation</TabsTrigger>
                    <TabsTrigger value="recommendation">Optimization</TabsTrigger>
                </TabsList>

                {/* Batch Size Analysis */}
                <TabsContent value="analysis" className="space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {analyses.map((analysis) => (
                            <Card
                                key={analysis.batchSize}
                                className={cn(
                                    'cursor-pointer transition-all hover:shadow-lg',
                                    selectedAnalysis?.batchSize === analysis.batchSize
                                        ? 'ring-2 ring-purple-500 bg-purple-500/5'
                                        : 'hover:bg-muted/50'
                                )}
                                onClick={() => setSelectedAnalysis(analysis)}
                            >
                                <CardHeader className="pb-3">
                                    <div className="flex items-center justify-between">
                                        <CardTitle className="text-lg">Batch Size: {analysis.batchSize}</CardTitle>
                                        <Badge
                                            variant={analysis.efficiencyScore > 0.7 ? 'default' : 'secondary'}
                                        >
                                            {analysis.efficiencyScore > 0.8 ? 'Optimal' :
                                                analysis.efficiencyScore > 0.7 ? 'Good' : 'Fair'}
                                        </Badge>
                                    </div>
                                    <CardDescription>
                                        {analysis.totalBatches} batches • {analysis.qualityMetrics.feedbackCycles} feedback cycles
                                    </CardDescription>
                                </CardHeader>
                                <CardContent className="space-y-3">
                                    <div className="grid grid-cols-2 gap-3">
                                        <div className="text-center p-2 rounded bg-blue-500/10">
                                            <div className="text-lg font-bold text-blue-500">
                                                {(analysis.qualityMetrics.projectedFinalQuality * 100).toFixed(1)}%
                                            </div>
                                            <div className="text-xs text-muted-foreground">Final Quality</div>
                                        </div>
                                        <div className="text-center p-2 rounded bg-green-500/10">
                                            <div className="text-lg font-bold text-green-500">
                                                {analysis.totalQualityValue.toFixed(0)}
                                            </div>
                                            <div className="text-xs text-muted-foreground">Quality Value</div>
                                        </div>
                                    </div>

                                    <div className="space-y-2">
                                        <div className="flex justify-between text-sm">
                                            <span>Improvement Rate:</span>
                                            <span className="font-medium">
                                                +{(analysis.qualityMetrics.improvementRate * 100).toFixed(1)}%/cycle
                                            </span>
                                        </div>
                                        <div className="flex justify-between text-sm">
                                            <span>Compounding:</span>
                                            <span className="font-medium">
                                                +{(analysis.qualityMetrics.compoundingFactor * 100).toFixed(1)}%
                                            </span>
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>
                        ))}
                    </div>

                    {/* Comparison Chart */}
                    {selectedAnalysis && (
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <BarChart3 className="w-5 h-5" />
                                    Quality Growth Curve (Batch Size: {selectedAnalysis.batchSize})
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="space-y-4">
                                    <div className="grid grid-cols-4 gap-4 text-sm">
                                        <div className="text-center">
                                            <div className="text-lg font-bold text-purple-500">
                                                {selectedAnalysis.totalBatches}
                                            </div>
                                            <div className="text-muted-foreground">Total Batches</div>
                                        </div>
                                        <div className="text-center">
                                            <div className="text-lg font-bold text-blue-500">
                                                {selectedAnalysis.qualityMetrics.feedbackCycles}
                                            </div>
                                            <div className="text-muted-foreground">Feedback Cycles</div>
                                        </div>
                                        <div className="text-center">
                                            <div className="text-lg font-bold text-green-500">
                                                {(selectedAnalysis.qualityMetrics.projectedFinalQuality * 100).toFixed(1)}%
                                            </div>
                                            <div className="text-muted-foreground">Final Quality</div>
                                        </div>
                                        <div className="text-center">
                                            <div className="text-lg font-bold text-orange-500">
                                                {(selectedAnalysis.efficiencyScore * 100).toFixed(0)}
                                            </div>
                                            <div className="text-muted-foreground">Efficiency Score</div>
                                        </div>
                                    </div>

                                    {/* Quality progression visualization */}
                                    <div className="space-y-2">
                                        <div className="flex justify-between text-sm text-muted-foreground">
                                            <span>Quality progression over batches:</span>
                                            <span>{qualityCurve.length} data points</span>
                                        </div>
                                        <div className="h-32 bg-muted/30 rounded-lg p-4 relative overflow-hidden">
                                            <div className="absolute inset-0 bg-gradient-to-r from-red-500/20 via-yellow-500/20 to-green-500/20 opacity-50" />
                                            <div className="relative h-full flex items-end space-x-1">
                                                {qualityCurve.slice(0, 20).map((point, index) => (
                                                    <div
                                                        key={index}
                                                        className="bg-purple-500 rounded-t-sm flex-1 min-w-[2px] transition-all"
                                                        style={{
                                                            height: `${point.quality * 100}%`,
                                                            opacity: 0.7 + (point.quality * 0.3)
                                                        }}
                                                        title={`Batch ${point.batch}: ${(point.quality * 100).toFixed(1)}%`}
                                                    />
                                                ))}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    )}
                </TabsContent>

                {/* Theory Explanation */}
                <TabsContent value="theory" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2">
                                <Brain className="w-5 h-5" />
                                {explanation.title}
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            <div className="p-4 rounded-lg bg-muted/50 font-mono text-sm">
                                {explanation.formula}
                            </div>

                            <div className="grid gap-4">
                                {explanation.factors.map((factor, index) => (
                                    <div key={index} className="flex gap-4 p-4 rounded-lg border">
                                        <div className="w-2 h-2 rounded-full bg-purple-500 mt-2 flex-shrink-0" />
                                        <div className="space-y-1">
                                            <div className="font-medium">{factor.name}</div>
                                            <div className="text-sm text-muted-foreground">{factor.description}</div>
                                            <div className="text-sm text-purple-600 font-medium">{factor.impact}</div>
                                        </div>
                                    </div>
                                ))}
                            </div>

                            <div className="p-4 rounded-lg bg-gradient-to-r from-purple-500/10 to-blue-500/10 border border-purple-500/20">
                                <div className="font-medium mb-2">Practical Example:</div>
                                <pre className="text-sm text-muted-foreground whitespace-pre-wrap">
                                    {explanation.example}
                                </pre>
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* Simulation */}
                <TabsContent value="simulation" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <div className="flex items-center justify-between">
                                <div>
                                    <CardTitle className="flex items-center gap-2">
                                        <Zap className="w-5 h-5" />
                                        Live Quality Simulation
                                    </CardTitle>
                                    <CardDescription>
                                        Watch how quality improves exponentially with feedback cycles
                                    </CardDescription>
                                </div>
                                <Button
                                    onClick={simulateQualityGrowth}
                                    disabled={isSimulating}
                                    className="flex items-center gap-2"
                                >
                                    {isSimulating ? (
                                        <>
                                            <Zap className="w-4 h-4 animate-pulse" />
                                            Simulating...
                                        </>
                                    ) : (
                                        <>
                                            <Zap className="w-4 h-4" />
                                            Run Simulation
                                        </>
                                    )}
                                </Button>
                            </div>
                        </CardHeader>
                        <CardContent>
                            {selectedAnalysis && (
                                <div className="space-y-4">
                                    <div className="text-center p-6 rounded-lg bg-gradient-to-r from-purple-500/10 to-blue-500/10">
                                        <div className="text-3xl font-bold text-purple-500 mb-2">
                                            {(selectedAnalysis.qualityMetrics.projectedFinalQuality * 100).toFixed(1)}%
                                        </div>
                                        <div className="text-muted-foreground">
                                            Projected Final Quality with {selectedAnalysis.batchSize}-entry batches
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="p-4 rounded-lg border">
                                            <div className="flex items-center gap-2 mb-2">
                                                <Clock className="w-4 h-4 text-blue-500" />
                                                <span className="font-medium">Timeline</span>
                                            </div>
                                            <div className="text-sm text-muted-foreground space-y-1">
                                                <div>Start: 60% base quality</div>
                                                <div>After {Math.floor(selectedAnalysis.totalBatches / 2)} batches: {(ExponentialQualityService.calculateQualityAtBatch(Math.floor(selectedAnalysis.totalBatches / 2), selectedAnalysis.batchSize) * 100).toFixed(1)}%</div>
                                                <div>Final: {(selectedAnalysis.qualityMetrics.projectedFinalQuality * 100).toFixed(1)}%</div>
                                            </div>
                                        </div>

                                        <div className="p-4 rounded-lg border">
                                            <div className="flex items-center gap-2 mb-2">
                                                <Target className="w-4 h-4 text-green-500" />
                                                <span className="font-medium">Improvement</span>
                                            </div>
                                            <div className="text-sm text-muted-foreground space-y-1">
                                                <div>Total gain: +{((selectedAnalysis.qualityMetrics.projectedFinalQuality - 0.6) * 100).toFixed(1)}%</div>
                                                <div>Per cycle: +{(selectedAnalysis.qualityMetrics.improvementRate * 100).toFixed(1)}%</div>
                                                <div>Compounding: +{(selectedAnalysis.qualityMetrics.compoundingFactor * 100).toFixed(1)}%</div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>

                {/* Optimization Recommendations */}
                <TabsContent value="recommendation" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2">
                                <CheckCircle className="w-5 h-5 text-green-500" />
                                Optimized Configuration
                            </CardTitle>
                            <CardDescription>
                                AI-recommended batch size for maximum quality improvement
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            <div className="p-6 rounded-lg bg-green-500/10 border border-green-500/20">
                                <div className="text-center space-y-2">
                                    <div className="text-2xl font-bold text-green-600">
                                        Batch Size: {recommendation.recommended.batchSize}
                                    </div>
                                    <div className="text-muted-foreground">
                                        Achieves {(recommendation.recommended.qualityMetrics.projectedFinalQuality * 100).toFixed(1)}% final quality
                                    </div>
                                </div>
                            </div>

                            <div className="grid gap-4">
                                <div>
                                    <div className="flex items-center gap-2 mb-3">
                                        <Info className="w-4 h-4 text-blue-500" />
                                        <span className="font-medium">Why This Works Best:</span>
                                    </div>
                                    <ul className="space-y-2 text-sm text-muted-foreground">
                                        {recommendation.reasoning.map((reason, index) => (
                                            <li key={index} className="flex items-start gap-2">
                                                <ArrowUp className="w-3 h-3 text-green-500 mt-0.5 flex-shrink-0" />
                                                {reason}
                                            </li>
                                        ))}
                                    </ul>
                                </div>

                                <div>
                                    <div className="flex items-center gap-2 mb-3">
                                        <AlertCircle className="w-4 h-4 text-orange-500" />
                                        <span className="font-medium">Trade-offs to Consider:</span>
                                    </div>
                                    <ul className="space-y-2 text-sm text-muted-foreground">
                                        {recommendation.tradeoffs.map((tradeoff, index) => (
                                            <li key={index} className="flex items-start gap-2">
                                                <ArrowDown className="w-3 h-3 text-orange-500 mt-0.5 flex-shrink-0" />
                                                {tradeoff}
                                            </li>
                                        ))}
                                    </ul>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    );
};

export default ExponentialImprovementDemo;
