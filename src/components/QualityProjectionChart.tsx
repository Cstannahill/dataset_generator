import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
    TrendingUp,
    BarChart3,
    Zap,
    Target,
    Brain,
    ArrowRight,
    Plus,
    Minus
} from 'lucide-react';
import { cn } from '@/lib/utils';

interface QualityProjectionProps {
    currentBatch: number;
    totalBatches: number;
    batchSize: number;
    className?: string;
}

export const QualityProjectionChart: React.FC<QualityProjectionProps> = ({
    currentBatch,
    totalBatches,
    batchSize,
    className,
}) => {
    const [projectedBatchSize, setProjectedBatchSize] = useState(batchSize);
    const [showComparison, setShowComparison] = useState(false);

    // Calculate exponential improvement factors
    const calculateQualityProjection = (batchNum: number, batchSz: number) => {
        const improvementRate = 0.15; // 15% improvement per feedback cycle
        const compoundingFactor = Math.min(0.05 * Math.log(batchNum + 1), 0.3); // Diminishing returns

        // Smaller batches = more feedback cycles = more improvements
        const feedbackCycles = Math.floor(batchNum / Math.max(1, batchSz / 10));
        const baseQuality = 0.6; // Starting quality

        return Math.min(
            baseQuality + (improvementRate * feedbackCycles) + compoundingFactor,
            0.95 // Cap at 95% quality
        );
    };

    const calculateDatasetValue = (batchSz: number) => {
        const totalEntries = totalBatches * batchSz;
        let totalQualityScore = 0;

        for (let i = 0; i < totalBatches; i++) {
            const quality = calculateQualityProjection(i, batchSz);
            totalQualityScore += quality * batchSz;
        }

        return {
            totalEntries,
            averageQuality: totalQualityScore / totalEntries,
            qualityWeightedEntries: totalQualityScore,
        };
    };

    const currentScenario = calculateDatasetValue(batchSize);
    const smallerBatchScenario = calculateDatasetValue(Math.max(5, Math.floor(batchSize / 2)));
    const largerBatchScenario = calculateDatasetValue(batchSize * 2);

    const getQualityTrend = () => {
        const data = [];
        for (let i = 0; i <= Math.min(currentBatch + 5, totalBatches); i++) {
            data.push({
                batch: i,
                quality: calculateQualityProjection(i, batchSize) * 100,
                qualitySmaller: calculateQualityProjection(i, Math.floor(batchSize / 2)) * 100,
                qualityLarger: calculateQualityProjection(i, batchSize * 2) * 100,
            });
        }
        return data;
    };

    const trendData = getQualityTrend();
    const currentQuality = calculateQualityProjection(currentBatch, batchSize);
    const projectedFinalQuality = calculateQualityProjection(totalBatches, batchSize);

    return (
        <Card className={cn('border-gradient', className)}>
            <CardHeader className="pb-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                        <div className="p-2 rounded-lg bg-gradient-to-br from-purple-500/20 to-blue-500/20">
                            <TrendingUp className="w-5 h-5 text-purple-500" />
                        </div>
                        <div>
                            <CardTitle className="text-lg">Exponential Quality Improvement</CardTitle>
                            <CardDescription>
                                How smaller batches create compounding quality gains
                            </CardDescription>
                        </div>
                    </div>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setShowComparison(!showComparison)}
                    >
                        {showComparison ? 'Hide' : 'Show'} Comparison
                    </Button>
                </div>
            </CardHeader>

            <CardContent className="space-y-6">
                {/* Current Progress */}
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">Current Quality Progress</span>
                        <Badge variant={currentQuality > 0.8 ? 'default' : 'secondary'}>
                            {(currentQuality * 100).toFixed(1)}% Quality
                        </Badge>
                    </div>
                    <Progress value={currentQuality * 100} className="h-2" />
                    <div className="flex justify-between text-xs text-muted-foreground">
                        <span>Batch {currentBatch}/{totalBatches}</span>
                        <span>Target: {(projectedFinalQuality * 100).toFixed(1)}%</span>
                    </div>
                </div>

                {/* Quality Improvement Factors */}
                <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
                        <div className="flex items-center gap-2 mb-2">
                            <Brain className="w-4 h-4 text-blue-500" />
                            <span className="text-sm font-medium">Feedback Cycles</span>
                        </div>
                        <div className="text-lg font-bold text-blue-500">
                            {Math.floor(currentBatch / Math.max(1, batchSize / 10))}
                        </div>
                        <div className="text-xs text-muted-foreground">
                            More cycles = Better learning
                        </div>
                    </div>

                    <div className="p-3 rounded-lg bg-green-500/10 border border-green-500/20">
                        <div className="flex items-center gap-2 mb-2">
                            <Target className="w-4 h-4 text-green-500" />
                            <span className="text-sm font-medium">Improvement Rate</span>
                        </div>
                        <div className="text-lg font-bold text-green-500">
                            +{((currentQuality - 0.6) * 100).toFixed(1)}%
                        </div>
                        <div className="text-xs text-muted-foreground">
                            From baseline quality
                        </div>
                    </div>
                </div>

                {/* Batch Size Impact */}
                {showComparison && (
                    <div className="space-y-4 pt-4 border-t">
                        <h4 className="text-sm font-medium flex items-center gap-2">
                            <BarChart3 className="w-4 h-4" />
                            Batch Size Impact Analysis
                        </h4>

                        <div className="grid grid-cols-1 gap-3">
                            {/* Smaller Batches */}
                            <div className="flex items-center justify-between p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                                <div>
                                    <div className="font-medium text-emerald-700">Smaller Batches (Size: {Math.floor(batchSize / 2)})</div>
                                    <div className="text-sm text-muted-foreground">
                                        {smallerBatchScenario.totalEntries} entries • Avg: {(smallerBatchScenario.averageQuality * 100).toFixed(1)}%
                                    </div>
                                </div>
                                <div className="text-right">
                                    <div className="text-lg font-bold text-emerald-600">
                                        +{((smallerBatchScenario.qualityWeightedEntries - currentScenario.qualityWeightedEntries) / currentScenario.qualityWeightedEntries * 100).toFixed(1)}%
                                    </div>
                                    <div className="text-xs text-emerald-600">Quality Value</div>
                                </div>
                            </div>

                            {/* Current Batches */}
                            <div className="flex items-center justify-between p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
                                <div>
                                    <div className="font-medium text-blue-700">Current Batches (Size: {batchSize})</div>
                                    <div className="text-sm text-muted-foreground">
                                        {currentScenario.totalEntries} entries • Avg: {(currentScenario.averageQuality * 100).toFixed(1)}%
                                    </div>
                                </div>
                                <div className="text-right">
                                    <div className="text-lg font-bold text-blue-600">Baseline</div>
                                    <div className="text-xs text-blue-600">Reference</div>
                                </div>
                            </div>

                            {/* Larger Batches */}
                            <div className="flex items-center justify-between p-3 rounded-lg bg-orange-500/10 border border-orange-500/20">
                                <div>
                                    <div className="font-medium text-orange-700">Larger Batches (Size: {batchSize * 2})</div>
                                    <div className="text-sm text-muted-foreground">
                                        {largerBatchScenario.totalEntries} entries • Avg: {(largerBatchScenario.averageQuality * 100).toFixed(1)}%
                                    </div>
                                </div>
                                <div className="text-right">
                                    <div className="text-lg font-bold text-orange-600">
                                        {((largerBatchScenario.qualityWeightedEntries - currentScenario.qualityWeightedEntries) / currentScenario.qualityWeightedEntries * 100).toFixed(1)}%
                                    </div>
                                    <div className="text-xs text-orange-600">Quality Value</div>
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {/* Key Insight */}
                <div className="p-4 rounded-lg bg-gradient-to-r from-purple-500/10 to-blue-500/10 border border-purple-500/20">
                    <div className="flex items-start gap-3">
                        <Zap className="w-5 h-5 text-purple-500 mt-0.5" />
                        <div>
                            <div className="font-medium text-purple-700 mb-1">Exponential Quality Formula</div>
                            <div className="text-sm text-muted-foreground space-y-1">
                                <div>• <strong>More batches</strong> = More feedback cycles</div>
                                <div>• <strong>More feedback</strong> = Better prompt improvements</div>
                                <div>• <strong>Better prompts</strong> = Higher quality in subsequent batches</div>
                                <div>• <strong>Compounding effect</strong> = Exponential quality growth</div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Interactive Batch Size Adjuster */}
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">Test Batch Size Impact</span>
                        <div className="flex items-center gap-2">
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => setProjectedBatchSize(Math.max(5, projectedBatchSize - 5))}
                            >
                                <Minus className="w-3 h-3" />
                            </Button>
                            <Badge variant="outline" className="min-w-[60px] justify-center">
                                {projectedBatchSize}
                            </Badge>
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => setProjectedBatchSize(projectedBatchSize + 5)}
                            >
                                <Plus className="w-3 h-3" />
                            </Button>
                        </div>
                    </div>

                    <div className="p-3 rounded-lg bg-muted/50">
                        <div className="text-sm">
                            With batch size <strong>{projectedBatchSize}</strong>:
                            Final quality would be <strong>{(calculateQualityProjection(totalBatches, projectedBatchSize) * 100).toFixed(1)}%</strong>
                            {projectedBatchSize < batchSize && (
                                <span className="text-green-600 ml-2">
                                    (+{((calculateQualityProjection(totalBatches, projectedBatchSize) - projectedFinalQuality) * 100).toFixed(1)}% improvement!)
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
};

export default QualityProjectionChart;
