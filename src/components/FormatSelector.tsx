import React from 'react';
import { DatasetFormat } from '@/types';
import { getAllFormats, getFormatInfo } from '@/lib/dataset-formats';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface FormatSelectorProps {
    selectedFormat: DatasetFormat;
    onFormatChange: (format: DatasetFormat) => void;
}

export const FormatSelector: React.FC<FormatSelectorProps> = ({
    selectedFormat,
    onFormatChange,
}) => {
    const formats = getAllFormats();
    const selectedFormatInfo = getFormatInfo(selectedFormat);

    return (
        <div className="space-y-6">
            <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-300">
                    Dataset Format
                </label>
                <Select value={selectedFormat} onValueChange={onFormatChange}>
                    <SelectTrigger className="w-full bg-gray-800 border-gray-600 text-white">
                        <SelectValue placeholder="Select dataset format" />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-800 border-gray-600">
                        {formats.map((format) => (
                            <SelectItem
                                key={format.id}
                                value={format.id}
                                className="text-white hover:bg-gray-700"
                            >
                                <div className="flex flex-col">
                                    <span className="font-medium">{format.name}</span>
                                    <span className="text-xs text-gray-400">{format.description}</span>
                                </div>
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            {/* Selected Format Details */}
            <Card className="bg-gray-800/50 border-gray-600">
                <CardHeader>
                    <CardTitle className="text-white text-lg">{selectedFormatInfo.name}</CardTitle>
                    <CardDescription className="text-gray-300">
                        {selectedFormatInfo.description}
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div>
                        <h4 className="text-sm font-medium text-gray-300 mb-2">Structure:</h4>
                        <code className="text-xs bg-gray-900 text-green-400 p-2 rounded block overflow-x-auto">
                            {selectedFormatInfo.structure}
                        </code>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <h4 className="text-sm font-medium text-green-400 mb-2">✅ Good For:</h4>
                            <div className="flex flex-wrap gap-1">
                                {selectedFormatInfo.goodFor.map((item, index) => (
                                    <Badge key={index} variant="secondary" className="text-xs bg-green-900/20 text-green-300">
                                        {item}
                                    </Badge>
                                ))}
                            </div>
                        </div>

                        <div>
                            <h4 className="text-sm font-medium text-red-400 mb-2">🚫 Not Ideal For:</h4>
                            <div className="flex flex-wrap gap-1">
                                {selectedFormatInfo.notIdealFor.map((item, index) => (
                                    <Badge key={index} variant="secondary" className="text-xs bg-red-900/20 text-red-300">
                                        {item}
                                    </Badge>
                                ))}
                            </div>
                        </div>
                    </div>

                    <div>
                        <h4 className="text-sm font-medium text-gray-300 mb-2">Examples:</h4>
                        <div className="flex flex-wrap gap-1">
                            {selectedFormatInfo.examples.map((example, index) => (
                                <Badge key={index} variant="outline" className="text-xs text-gray-400 border-gray-600">
                                    {example}
                                </Badge>
                            ))}
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* Format Comparison Table */}
            <Card className="bg-gray-800/50 border-gray-600">
                <CardHeader>
                    <CardTitle className="text-white text-lg">Format Comparison Guide</CardTitle>
                    <CardDescription className="text-gray-300">
                        Overview of all available dataset formats and their use cases
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="overflow-x-auto">
                        <Table>
                            <TableHeader>
                                <TableRow className="border-gray-600">
                                    <TableHead className="text-gray-300">Format</TableHead>
                                    <TableHead className="text-gray-300">Best For</TableHead>
                                    <TableHead className="text-gray-300">Avoid For</TableHead>
                                    <TableHead className="text-gray-300">File Type</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {formats.map((format) => (
                                    <TableRow
                                        key={format.id}
                                        className={`border-gray-600 ${format.id === selectedFormat ? 'bg-blue-900/20' : ''}`}
                                    >
                                        <TableCell className="font-medium text-white">
                                            <div>
                                                <div className="font-semibold">{format.name}</div>
                                                <div className="text-xs text-gray-400">{format.description}</div>
                                            </div>
                                        </TableCell>
                                        <TableCell className="text-green-300 text-sm">
                                            {format.goodFor.slice(0, 2).join(', ')}
                                            {format.goodFor.length > 2 && '...'}
                                        </TableCell>
                                        <TableCell className="text-red-300 text-sm">
                                            {format.notIdealFor.slice(0, 2).join(', ')}
                                            {format.notIdealFor.length > 2 && '...'}
                                        </TableCell>
                                        <TableCell className="text-gray-400 text-sm">
                                            {format.fileExtension}
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
};

export default FormatSelector;
