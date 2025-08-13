import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { KnowledgeBaseService } from '@/lib/knowledge-base';
import type {
    KnowledgeBaseStats,
    SearchResult,
    CollectionInfo,
    ImprovementSuggestion,
    DatasetFormat
} from '@/types';
import {
    Database,
    Search,
    TrendingUp,
    Lightbulb,
    Target,
    BookOpen,
    BarChart3,
    Clock
} from 'lucide-react';

interface KnowledgeBaseDashboardProps {
    className?: string;
}

export const KnowledgeBaseDashboard: React.FC<KnowledgeBaseDashboardProps> = ({
    className = ""
}) => {
    const [stats, setStats] = useState<KnowledgeBaseStats | null>(null);
    const [collections, setCollections] = useState<CollectionInfo[]>([]);
    const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
    const [searchQuery, setSearchQuery] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isInitialized, setIsInitialized] = useState(false);

    useEffect(() => {
        initializeKnowledgeBase();
    }, []);

    const initializeKnowledgeBase = async () => {
        setIsLoading(true);
        setError(null);

        try {
            await KnowledgeBaseService.initialize();
            setIsInitialized(true);
            await loadStats();
            await loadCollections();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to initialize knowledge base');
        } finally {
            setIsLoading(false);
        }
    };

    const loadStats = async () => {
        try {
            const kbStats = await KnowledgeBaseService.getStats();
            setStats(kbStats);
        } catch (err) {
            console.error('Failed to load stats:', err);
        }
    };

    const loadCollections = async () => {
        try {
            const collectionsData = await KnowledgeBaseService.listCollections();
            setCollections(collectionsData);
        } catch (err) {
            console.error('Failed to load collections:', err);
        }
    };

    const handleSearch = async () => {
        if (!searchQuery.trim()) return;

        setIsLoading(true);
        try {
            const results = await KnowledgeBaseService.search(searchQuery, {
                limit: 10,
                minQualityScore: 0.5,
            });
            setSearchResults(results);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Search failed');
        } finally {
            setIsLoading(false);
        }
    };

    const formatTimestamp = (timestamp: number) => {
        return new Date(timestamp * 1000).toLocaleDateString();
    };

    const getQualityBadge = (score: number) => {
        if (score >= 0.9) return <Badge variant="default" className="bg-green-500">Excellent</Badge>;
        if (score >= 0.8) return <Badge variant="default" className="bg-blue-500">Good</Badge>;
        if (score >= 0.7) return <Badge variant="default" className="bg-yellow-500">Fair</Badge>;
        return <Badge variant="destructive">Poor</Badge>;
    };

    if (!isInitialized && !error) {
        return (
            <Card className={className}>
                <CardContent className="flex items-center justify-center p-8">
                    <div className="text-center">
                        <Database className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
                        <p className="text-lg font-medium">Initializing Knowledge Base...</p>
                        <p className="text-sm text-muted-foreground mt-2">
                            Setting up quality validation, embeddings, and vector storage
                        </p>
                    </div>
                </CardContent>
            </Card>
        );
    }

    if (error) {
        return (
            <Card className={className}>
                <CardContent className="p-6">
                    <Alert>
                        <AlertDescription>
                            {error}
                            <Button
                                onClick={initializeKnowledgeBase}
                                className="ml-4"
                                size="sm"
                            >
                                Retry
                            </Button>
                        </AlertDescription>
                    </Alert>
                </CardContent>
            </Card>
        );
    }

    return (
        <div className={`space-y-6 ${className}`}>
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-bold">Knowledge Base</h2>
                    <p className="text-muted-foreground">
                        AI-powered dataset quality validation, embeddings, and intelligent storage
                    </p>
                </div>
                <Button onClick={loadStats} disabled={isLoading}>
                    <BarChart3 className="w-4 h-4 mr-2" />
                    Refresh
                </Button>
            </div>

            <Tabs defaultValue="overview" className="w-full">
                <TabsList className="grid w-full grid-cols-4">
                    <TabsTrigger value="overview">Overview</TabsTrigger>
                    <TabsTrigger value="search">Search</TabsTrigger>
                    <TabsTrigger value="collections">Collections</TabsTrigger>
                    <TabsTrigger value="insights">Insights</TabsTrigger>
                </TabsList>

                <TabsContent value="overview" className="space-y-4">
                    {stats && (
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Total Entries</CardTitle>
                                    <Database className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">{stats.total_entries}</div>
                                    <p className="text-xs text-muted-foreground">
                                        Across {stats.total_collections} collections
                                    </p>
                                </CardContent>
                            </Card>

                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Use Cases</CardTitle>
                                    <Target className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">{stats.unique_use_cases}</div>
                                    <p className="text-xs text-muted-foreground">
                                        Different training objectives
                                    </p>
                                </CardContent>
                            </Card>

                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Formats</CardTitle>
                                    <BookOpen className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">{stats.unique_formats}</div>
                                    <p className="text-xs text-muted-foreground">
                                        Dataset structures covered
                                    </p>
                                </CardContent>
                            </Card>

                            <Card>
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                    <CardTitle className="text-sm font-medium">Latest Update</CardTitle>
                                    <Clock className="h-4 w-4 text-muted-foreground" />
                                </CardHeader>
                                <CardContent>
                                    <div className="text-2xl font-bold">
                                        {stats.newest_entry_timestamp
                                            ? formatTimestamp(stats.newest_entry_timestamp)
                                            : 'N/A'
                                        }
                                    </div>
                                    <p className="text-xs text-muted-foreground">
                                        Most recent addition
                                    </p>
                                </CardContent>
                            </Card>
                        </div>
                    )}
                </TabsContent>

                <TabsContent value="search" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center">
                                <Search className="w-5 h-5 mr-2" />
                                Search Knowledge Base
                            </CardTitle>
                            <CardDescription>
                                Find similar examples, patterns, and high-quality entries
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex space-x-2">
                                <Input
                                    placeholder="Search for examples, use cases, or content..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
                                />
                                <Button onClick={handleSearch} disabled={isLoading}>
                                    Search
                                </Button>
                            </div>

                            {searchResults.length > 0 && (
                                <div className="space-y-3">
                                    <h4 className="font-medium">Search Results</h4>
                                    {searchResults.map((result, index) => (
                                        <Card key={result.id} className="border-l-4 border-l-blue-500">
                                            <CardContent className="p-4">
                                                <div className="flex justify-between items-start mb-2">
                                                    <div className="flex items-center space-x-2">
                                                        <span className="text-sm font-medium">#{index + 1}</span>
                                                        {result.metadata.overall_score &&
                                                            getQualityBadge(result.metadata.overall_score as number)
                                                        }
                                                    </div>
                                                    <Badge variant="outline">
                                                        Distance: {result.distance.toFixed(3)}
                                                    </Badge>
                                                </div>
                                                <p className="text-sm text-muted-foreground mb-2">
                                                    {result.text.substring(0, 200)}...
                                                </p>
                                                <div className="flex flex-wrap gap-1">
                                                    {result.metadata.tags && Array.isArray(result.metadata.tags) &&
                                                        result.metadata.tags.map((tag: string, tagIndex: number) => (
                                                            <Badge key={tagIndex} variant="secondary" className="text-xs">
                                                                {tag}
                                                            </Badge>
                                                        ))
                                                    }
                                                </div>
                                            </CardContent>
                                        </Card>
                                    ))}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="collections" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Collections</CardTitle>
                            <CardDescription>
                                Organized by use case and dataset format
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-3">
                                {collections.map((collection) => (
                                    <Card key={collection.name} className="border">
                                        <CardContent className="p-4">
                                            <div className="flex justify-between items-start">
                                                <div>
                                                    <h4 className="font-medium">{collection.name}</h4>
                                                    <p className="text-sm text-muted-foreground">
                                                        Use Case: {collection.use_case}
                                                    </p>
                                                    <p className="text-sm text-muted-foreground">
                                                        Format: {collection.dataset_format}
                                                    </p>
                                                </div>
                                                <div className="text-right">
                                                    <div className="text-lg font-bold">{collection.entry_count}</div>
                                                    <div className="text-xs text-muted-foreground">entries</div>
                                                </div>
                                            </div>
                                            <div className="mt-2 text-xs text-muted-foreground">
                                                Created: {formatTimestamp(collection.created_at)}
                                            </div>
                                        </CardContent>
                                    </Card>
                                ))}
                                {collections.length === 0 && (
                                    <div className="text-center py-8 text-muted-foreground">
                                        No collections found. Generate and export datasets to build your knowledge base.
                                    </div>
                                )}
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="insights" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center">
                                <Lightbulb className="w-5 h-5 mr-2" />
                                Knowledge Base Insights
                            </CardTitle>
                            <CardDescription>
                                AI-powered analysis and recommendations
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-4">
                                <Alert>
                                    <TrendingUp className="h-4 w-4" />
                                    <AlertDescription>
                                        Your knowledge base contains {stats?.total_entries || 0} validated entries
                                        across {stats?.unique_use_cases || 0} use cases.
                                        {stats && stats.total_entries > 100
                                            ? "You have excellent coverage for training dataset AI!"
                                            : "Consider generating more diverse examples to improve coverage."
                                        }
                                    </AlertDescription>
                                </Alert>

                                {stats && stats.total_entries > 0 && (
                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <Card>
                                            <CardHeader>
                                                <CardTitle className="text-sm">Coverage Analysis</CardTitle>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="space-y-2">
                                                    <div className="flex justify-between text-sm">
                                                        <span>Quality Coverage:</span>
                                                        <span className="font-medium">
                                                            {Math.round((stats.total_entries / Math.max(stats.total_entries, 100)) * 100)}%
                                                        </span>
                                                    </div>
                                                    <div className="flex justify-between text-sm">
                                                        <span>Format Diversity:</span>
                                                        <span className="font-medium">
                                                            {Math.round((stats.unique_formats / 9) * 100)}%
                                                        </span>
                                                    </div>
                                                </div>
                                            </CardContent>
                                        </Card>

                                        <Card>
                                            <CardHeader>
                                                <CardTitle className="text-sm">Recommendations</CardTitle>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="space-y-2 text-sm">
                                                    {stats.unique_use_cases < 5 && (
                                                        <p>• Explore more diverse use cases</p>
                                                    )}
                                                    {stats.unique_formats < 3 && (
                                                        <p>• Try different dataset formats</p>
                                                    )}
                                                    {stats.total_entries < 50 && (
                                                        <p>• Generate more examples for better AI training</p>
                                                    )}
                                                    {stats.total_entries >= 100 && (
                                                        <p>• Excellent foundation for dataset generation AI!</p>
                                                    )}
                                                </div>
                                            </CardContent>
                                        </Card>
                                    </div>
                                )}
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    );
};
