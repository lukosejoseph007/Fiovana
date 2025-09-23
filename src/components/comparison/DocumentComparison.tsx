import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  FileText,
  GitCompare,
  Zap,
  AlertCircle,
  ArrowRight,
  Brain,
  BarChart3,
  FileSearch,
} from 'lucide-react'

interface ComparisonOptions {
  comparison_type: 'TextDiff' | 'StructuralDiff' | 'SemanticSimilarity' | 'Comprehensive'
  include_metadata: boolean
  similarity_threshold: number
}

// Exact match to backend enums
type DifferenceType =
  | 'Addition'
  | 'Deletion'
  | 'Modification'
  | 'Structural'
  | 'Semantic'
  | 'Formatting'
type DifferenceSeverity = 'Minor' | 'Major' | 'Critical'

interface DifferenceLocation {
  section?: string
  paragraph?: number
  line?: number
  page?: number
  character_offset?: number
}

interface DocumentDifference {
  diff_type: DifferenceType
  severity: DifferenceSeverity
  location: DifferenceLocation
  description: string
  before?: string
  after?: string
  confidence: number
}

interface ComparisonSummary {
  total_differences: number
  major_changes: number
  minor_changes: number
  additions: number
  deletions: number
  modifications: number
  overall_similarity: number
  content_similarity: number
  structural_similarity: number
}

interface MetadataComparison {
  // Add fields as needed based on backend implementation
  [key: string]: unknown
}

interface ComparisonResult {
  comparison_id: string
  comparison_type: string
  summary: ComparisonSummary
  differences: DocumentDifference[]
  similarity_score: number
  processing_time_ms: number
  metadata_comparison?: MetadataComparison
}

interface ComparisonResponse {
  success: boolean
  result?: ComparisonResult
  error?: string
  processing_time_ms: number
}

interface DocumentInfo {
  id: string
  title: string
  path: string
  size: number
  modified: string
}

interface DocumentIndexEntry {
  id: string
  path: string
  title: string
  content: string
  indexed_at: string
}

const DocumentComparison: React.FC = () => {
  const [documentA, setDocumentA] = useState<DocumentInfo | null>(null)
  const [documentB, setDocumentB] = useState<DocumentInfo | null>(null)
  const [availableDocuments, setAvailableDocuments] = useState<DocumentInfo[]>([])
  const [comparisonOptions, setComparisonOptions] = useState<ComparisonOptions>({
    comparison_type: 'Comprehensive',
    include_metadata: true,
    similarity_threshold: 0.8,
  })
  const [result, setResult] = useState<ComparisonResult | null>(null)
  const [processingTime, setProcessingTime] = useState<number>(0)
  const [isComparing, setIsComparing] = useState(false)
  const [activeTab, setActiveTab] = useState<'text' | 'structure' | 'semantic' | 'overview'>(
    'overview'
  )

  useEffect(() => {
    loadAvailableDocuments()
  }, [])

  const loadAvailableDocuments = async () => {
    try {
      // Get all indexed documents
      const documents = await invoke<DocumentIndexEntry[]>('get_all_documents')

      const docInfos: DocumentInfo[] = documents.map(doc => ({
        id: doc.id,
        title: doc.title || doc.path.split('/').pop() || 'Unknown Document',
        path: doc.path,
        size: doc.content?.length || 0,
        modified: new Date(doc.indexed_at).toLocaleDateString(),
      }))

      setAvailableDocuments(docInfos)
    } catch (error) {
      console.error('Failed to load documents:', error)
      // Fallback to empty array
      setAvailableDocuments([])
    }
  }

  const performComparison = async () => {
    if (!documentA || !documentB) return

    setIsComparing(true)
    try {
      const request = {
        document_a_id: documentA.id,
        document_b_id: documentB.id,
        comparison_type: comparisonOptions.comparison_type,
        include_metadata: comparisonOptions.include_metadata,
        similarity_threshold: comparisonOptions.similarity_threshold,
      }

      const response = await invoke<ComparisonResponse>('compare_documents', {
        request,
      })

      if (response.success && response.result) {
        setResult(response.result)
        setProcessingTime(response.processing_time_ms)
        setActiveTab('overview')
      } else {
        console.error('Comparison failed:', response.error)
        alert(`Comparison failed: ${response.error || 'Unknown error'}`)
      }
    } catch (error) {
      console.error('Comparison error:', error)
      alert(`Comparison error: ${error}`)
    } finally {
      setIsComparing(false)
    }
  }

  const getChangeTypeColor = (changeType: DifferenceType) => {
    switch (changeType) {
      case 'Addition':
        return 'text-green-700 bg-green-100'
      case 'Deletion':
        return 'text-red-700 bg-red-100'
      case 'Modification':
        return 'text-blue-700 bg-blue-100'
      case 'Structural':
        return 'text-purple-700 bg-purple-100'
      case 'Semantic':
        return 'text-indigo-700 bg-indigo-100'
      case 'Formatting':
        return 'text-yellow-700 bg-yellow-100'
      default:
        return 'text-gray-700 bg-gray-100'
    }
  }

  const getSimilarityColor = (score: number) => {
    if (score >= 0.8) return 'text-green-600'
    if (score >= 0.6) return 'text-yellow-600'
    return 'text-red-600'
  }

  // Helper functions to filter differences by type
  const getTextChanges = (differences: DocumentDifference[]) => {
    return differences.filter(
      diff =>
        diff.diff_type === 'Addition' ||
        diff.diff_type === 'Deletion' ||
        diff.diff_type === 'Modification'
    )
  }

  const getStructuralChanges = (differences: DocumentDifference[]) => {
    return differences.filter(diff => diff.diff_type === 'Structural')
  }

  const getSemanticChanges = (differences: DocumentDifference[]) => {
    return differences.filter(diff => diff.diff_type === 'Semantic')
  }

  const renderDocumentSelector = (
    label: string,
    selectedDoc: DocumentInfo | null,
    onSelect: (doc: DocumentInfo) => void,
    excludeId?: string
  ) => (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700">{label}</label>
      <select
        value={selectedDoc?.id || ''}
        onChange={e => {
          const doc = availableDocuments.find(d => d.id === e.target.value)
          if (doc) onSelect(doc)
        }}
        className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-purple-500"
      >
        <option value="">Select a document...</option>
        {availableDocuments
          .filter(doc => doc.id !== excludeId)
          .map(doc => (
            <option key={doc.id} value={doc.id}>
              {doc.title} ({(doc.size / 1024).toFixed(1)} KB)
            </option>
          ))}
      </select>
      {selectedDoc && (
        <div className="text-sm text-gray-600 bg-gray-50 p-2 rounded">
          <div className="flex items-center gap-2">
            <FileText size={16} />
            <span>{selectedDoc.path}</span>
          </div>
          <div className="text-xs text-gray-500 mt-1">Modified: {selectedDoc.modified}</div>
        </div>
      )}
    </div>
  )

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold text-gray-900 flex items-center justify-center gap-3">
          <GitCompare className="text-purple-600" />
          Intelligent Document Comparison
        </h1>
        <p className="text-gray-600">
          AI-powered document analysis with semantic understanding and structural insights
        </p>
      </div>

      {/* Document Selection */}
      <div className="bg-white rounded-xl shadow-lg p-6 border border-gray-200">
        <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
          <FileSearch className="text-purple-600" />
          Select Documents to Compare
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          {renderDocumentSelector('Document A (Original)', documentA, setDocumentA, documentB?.id)}
          {renderDocumentSelector(
            'Document B (Comparison)',
            documentB,
            setDocumentB,
            documentA?.id
          )}
        </div>

        {documentA && documentB && (
          <div className="flex items-center justify-center">
            <div className="flex items-center gap-4 bg-gradient-to-r from-purple-50 to-blue-50 p-4 rounded-lg">
              <div className="text-center">
                <div className="font-medium text-gray-900">{documentA.title}</div>
                <div className="text-sm text-gray-600">{(documentA.size / 1024).toFixed(1)} KB</div>
              </div>
              <ArrowRight className="text-purple-600" />
              <div className="text-center">
                <div className="font-medium text-gray-900">{documentB.title}</div>
                <div className="text-sm text-gray-600">{(documentB.size / 1024).toFixed(1)} KB</div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Comparison Options */}
      <div className="bg-white rounded-xl shadow-lg p-6 border border-gray-200">
        <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
          <Zap className="text-purple-600" />
          Comparison Settings
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Comparison Type</label>
            <select
              value={comparisonOptions.comparison_type}
              onChange={e =>
                setComparisonOptions(prev => ({
                  ...prev,
                  comparison_type: e.target.value as
                    | 'TextDiff'
                    | 'StructuralDiff'
                    | 'SemanticSimilarity'
                    | 'Comprehensive',
                }))
              }
              className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-purple-500"
            >
              <option value="Comprehensive">🧠 Comprehensive (All Analysis)</option>
              <option value="TextDiff">📝 Text Differences</option>
              <option value="StructuralDiff">🏗️ Structural Changes</option>
              <option value="SemanticSimilarity">🔍 Semantic Similarity</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Similarity Threshold
            </label>
            <select
              value={comparisonOptions.similarity_threshold}
              onChange={e =>
                setComparisonOptions(prev => ({
                  ...prev,
                  similarity_threshold: parseFloat(e.target.value),
                }))
              }
              className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-purple-500"
            >
              <option value={0.9}>90% - Very High</option>
              <option value={0.8}>80% - High</option>
              <option value={0.7}>70% - Medium</option>
              <option value={0.6}>60% - Low</option>
            </select>
          </div>

          <div className="flex items-center">
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={comparisonOptions.include_metadata}
                onChange={e =>
                  setComparisonOptions(prev => ({
                    ...prev,
                    include_metadata: e.target.checked,
                  }))
                }
                className="rounded border-gray-300 text-purple-600 focus:ring-purple-500"
              />
              <span className="text-sm text-gray-700">Include metadata analysis</span>
            </label>
          </div>
        </div>

        <div className="mt-6">
          <button
            onClick={performComparison}
            disabled={!documentA || !documentB || isComparing}
            className="bg-purple-600 text-white px-8 py-3 rounded-lg hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
          >
            {isComparing ? (
              <>
                <div className="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"></div>
                Analyzing Documents...
              </>
            ) : (
              <>
                <GitCompare size={20} />
                Compare Documents
              </>
            )}
          </button>
        </div>
      </div>

      {/* Results */}
      {result && (
        <div className="bg-white rounded-xl shadow-lg border border-gray-200">
          {/* Results Header */}
          <div className="p-6 border-b border-gray-200">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold">Comparison Results</h2>
              <div className="flex items-center gap-4 text-sm text-gray-600">
                <span>Processed in {processingTime}ms</span>
                <span>{result.summary.total_differences} changes detected</span>
              </div>
            </div>

            {/* Summary Cards */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-gradient-to-r from-blue-50 to-blue-100 p-4 rounded-lg">
                <div className="flex items-center gap-2">
                  <BarChart3 className="text-blue-600" />
                  <div>
                    <div
                      className={`text-2xl font-bold ${getSimilarityColor(result.similarity_score)}`}
                    >
                      {(result.similarity_score * 100).toFixed(1)}%
                    </div>
                    <div className="text-sm text-blue-700">Overall Similarity</div>
                  </div>
                </div>
              </div>

              <div className="bg-gradient-to-r from-green-50 to-green-100 p-4 rounded-lg">
                <div className="flex items-center gap-2">
                  <Brain className="text-green-600" />
                  <div>
                    <div
                      className={`text-2xl font-bold ${getSimilarityColor(result.summary.content_similarity)}`}
                    >
                      {(result.summary.content_similarity * 100).toFixed(1)}%
                    </div>
                    <div className="text-sm text-green-700">Semantic Similarity</div>
                  </div>
                </div>
              </div>

              <div className="bg-gradient-to-r from-purple-50 to-purple-100 p-4 rounded-lg">
                <div className="flex items-center gap-2">
                  <AlertCircle className="text-purple-600" />
                  <div>
                    <div className="text-2xl font-bold text-purple-900">
                      {result.summary.total_differences}
                    </div>
                    <div className="text-sm text-purple-700">Total Changes</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Tabs */}
          <div className="border-b border-gray-200">
            <nav className="flex">
              {[
                { id: 'overview', label: 'Overview', icon: Brain },
                {
                  id: 'text',
                  label: `Text Changes (${getTextChanges(result.differences).length})`,
                  icon: FileText,
                },
                {
                  id: 'structure',
                  label: `Structural (${getStructuralChanges(result.differences).length})`,
                  icon: GitCompare,
                },
                { id: 'semantic', label: 'Semantic Analysis', icon: Zap },
              ].map(({ id, label, icon: Icon }) => (
                <button
                  key={id}
                  onClick={() => setActiveTab(id as 'text' | 'structure' | 'semantic' | 'overview')}
                  className={`flex items-center gap-2 px-6 py-3 text-sm font-medium border-b-2 transition-colors ${
                    activeTab === id
                      ? 'border-purple-500 text-purple-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700'
                  }`}
                >
                  <Icon size={16} />
                  {label}
                </button>
              ))}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            {activeTab === 'overview' && (
              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
                    <Brain className="text-purple-600" />
                    Comparison Overview
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="bg-gradient-to-r from-blue-50 to-purple-50 p-6 rounded-lg">
                      <h4 className="font-semibold text-gray-900 mb-3">Summary Statistics</h4>
                      <div className="space-y-2 text-sm">
                        <div className="flex justify-between">
                          <span>Total Differences:</span>
                          <span className="font-medium">{result.summary.total_differences}</span>
                        </div>
                        <div className="flex justify-between">
                          <span>Major Changes:</span>
                          <span className="font-medium text-red-600">
                            {result.summary.major_changes}
                          </span>
                        </div>
                        <div className="flex justify-between">
                          <span>Minor Changes:</span>
                          <span className="font-medium text-yellow-600">
                            {result.summary.minor_changes}
                          </span>
                        </div>
                        <div className="flex justify-between">
                          <span>Additions:</span>
                          <span className="font-medium text-green-600">
                            {result.summary.additions}
                          </span>
                        </div>
                        <div className="flex justify-between">
                          <span>Deletions:</span>
                          <span className="font-medium text-red-600">
                            {result.summary.deletions}
                          </span>
                        </div>
                        <div className="flex justify-between">
                          <span>Modifications:</span>
                          <span className="font-medium text-blue-600">
                            {result.summary.modifications}
                          </span>
                        </div>
                      </div>
                    </div>

                    <div className="bg-gradient-to-r from-green-50 to-blue-50 p-6 rounded-lg">
                      <h4 className="font-semibold text-gray-900 mb-3">Similarity Scores</h4>
                      <div className="space-y-3">
                        <div>
                          <div className="flex justify-between text-sm mb-1">
                            <span>Overall Similarity</span>
                            <span className="font-medium">
                              {(result.summary.overall_similarity * 100).toFixed(1)}%
                            </span>
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-2">
                            <div
                              className="bg-blue-600 h-2 rounded-full"
                              style={{ width: `${result.summary.overall_similarity * 100}%` }}
                            ></div>
                          </div>
                        </div>
                        <div>
                          <div className="flex justify-between text-sm mb-1">
                            <span>Content Similarity</span>
                            <span className="font-medium">
                              {(result.summary.content_similarity * 100).toFixed(1)}%
                            </span>
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-2">
                            <div
                              className="bg-green-600 h-2 rounded-full"
                              style={{ width: `${result.summary.content_similarity * 100}%` }}
                            ></div>
                          </div>
                        </div>
                        <div>
                          <div className="flex justify-between text-sm mb-1">
                            <span>Structural Similarity</span>
                            <span className="font-medium">
                              {(result.summary.structural_similarity * 100).toFixed(1)}%
                            </span>
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-2">
                            <div
                              className="bg-purple-600 h-2 rounded-full"
                              style={{ width: `${result.summary.structural_similarity * 100}%` }}
                            ></div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                {result.differences.length > 0 && (
                  <div>
                    <h3 className="text-lg font-semibold mb-3">Recent Changes</h3>
                    <div className="space-y-3">
                      {result.differences.slice(0, 5).map((diff, index) => (
                        <div key={index} className="border border-gray-200 rounded-lg p-4">
                          <div className="flex items-center gap-2 mb-2">
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${getChangeTypeColor(diff.diff_type)}`}
                            >
                              {diff.diff_type}
                            </span>
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${
                                diff.severity === 'Critical'
                                  ? 'bg-red-100 text-red-700'
                                  : diff.severity === 'Major'
                                    ? 'bg-orange-100 text-orange-700'
                                    : 'bg-yellow-100 text-yellow-700'
                              }`}
                            >
                              {diff.severity}
                            </span>
                            {diff.location.line && (
                              <span className="text-sm text-gray-600">
                                Line {diff.location.line}
                              </span>
                            )}
                          </div>
                          <p className="text-gray-700 text-sm">{diff.description}</p>
                          {(diff.before || diff.after) && (
                            <div className="mt-2 grid grid-cols-1 gap-2">
                              {diff.before && (
                                <div className="bg-red-50 p-2 rounded text-sm">
                                  <div className="text-xs text-red-700 font-medium mb-1">
                                    Before:
                                  </div>
                                  <div className="text-red-800 font-mono">{diff.before}</div>
                                </div>
                              )}
                              {diff.after && (
                                <div className="bg-green-50 p-2 rounded text-sm">
                                  <div className="text-xs text-green-700 font-medium mb-1">
                                    After:
                                  </div>
                                  <div className="text-green-800 font-mono">{diff.after}</div>
                                </div>
                              )}
                            </div>
                          )}
                        </div>
                      ))}
                      {result.differences.length > 5 && (
                        <div className="text-center py-4">
                          <p className="text-gray-600">
                            And {result.differences.length - 5} more changes...
                          </p>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'text' && (
              <div className="space-y-4">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                  <FileText className="text-purple-600" />
                  Text Changes
                </h3>
                {(() => {
                  const textChanges = getTextChanges(result.differences)
                  return textChanges.length === 0 ? (
                    <p className="text-gray-500 text-center py-8">No text changes detected</p>
                  ) : (
                    <div className="space-y-3">
                      {textChanges.map((change, index) => (
                        <div key={index} className="border border-gray-200 rounded-lg p-4">
                          <div className="flex items-center gap-2 mb-2">
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${getChangeTypeColor(change.diff_type)}`}
                            >
                              {change.diff_type}
                            </span>
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${
                                change.severity === 'Critical'
                                  ? 'bg-red-100 text-red-700'
                                  : change.severity === 'Major'
                                    ? 'bg-orange-100 text-orange-700'
                                    : 'bg-yellow-100 text-yellow-700'
                              }`}
                            >
                              {change.severity}
                            </span>
                            {change.location.line && (
                              <span className="text-sm text-gray-600">
                                Line {change.location.line}
                              </span>
                            )}
                            {change.location.section && (
                              <span className="text-sm text-gray-600">
                                Section: {change.location.section}
                              </span>
                            )}
                          </div>
                          <p className="text-gray-700 mb-3">{change.description}</p>
                          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                            {change.before && (
                              <div className="bg-red-50 p-3 rounded">
                                <div className="text-sm text-red-700 font-medium mb-1">Before:</div>
                                <div className="text-red-800 font-mono text-sm whitespace-pre-wrap">
                                  {change.before}
                                </div>
                              </div>
                            )}
                            {change.after && (
                              <div className="bg-green-50 p-3 rounded">
                                <div className="text-sm text-green-700 font-medium mb-1">
                                  After:
                                </div>
                                <div className="text-green-800 font-mono text-sm whitespace-pre-wrap">
                                  {change.after}
                                </div>
                              </div>
                            )}
                          </div>
                          <div className="mt-2 text-xs text-gray-500">
                            Confidence: {(change.confidence * 100).toFixed(1)}%
                          </div>
                        </div>
                      ))}
                    </div>
                  )
                })()}
              </div>
            )}

            {activeTab === 'structure' && (
              <div className="space-y-4">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                  <GitCompare className="text-purple-600" />
                  Structural Changes
                </h3>
                {(() => {
                  const structuralChanges = getStructuralChanges(result.differences)
                  return structuralChanges.length === 0 ? (
                    <p className="text-gray-500 text-center py-8">No structural changes detected</p>
                  ) : (
                    <div className="space-y-3">
                      {structuralChanges.map((change, index) => (
                        <div key={index} className="border border-gray-200 rounded-lg p-4">
                          <div className="flex items-center gap-2 mb-2">
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${getChangeTypeColor(change.diff_type)}`}
                            >
                              {change.diff_type}
                            </span>
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${
                                change.severity === 'Critical'
                                  ? 'bg-red-100 text-red-700'
                                  : change.severity === 'Major'
                                    ? 'bg-orange-100 text-orange-700'
                                    : 'bg-yellow-100 text-yellow-700'
                              }`}
                            >
                              {change.severity}
                            </span>
                            <span className="text-sm text-gray-600">
                              Confidence: {(change.confidence * 100).toFixed(0)}%
                            </span>
                          </div>
                          <p className="text-gray-700 mb-3">{change.description}</p>
                          {change.location.section && (
                            <div className="text-gray-900 font-medium mb-2">
                              Section: {change.location.section}
                            </div>
                          )}
                          {(change.before || change.after) && (
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                              {change.before && (
                                <div className="bg-red-50 p-3 rounded">
                                  <div className="text-sm text-red-700 font-medium mb-1">
                                    Before:
                                  </div>
                                  <div className="text-red-800 text-sm whitespace-pre-wrap">
                                    {change.before}
                                  </div>
                                </div>
                              )}
                              {change.after && (
                                <div className="bg-green-50 p-3 rounded">
                                  <div className="text-sm text-green-700 font-medium mb-1">
                                    After:
                                  </div>
                                  <div className="text-green-800 text-sm whitespace-pre-wrap">
                                    {change.after}
                                  </div>
                                </div>
                              )}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )
                })()}
              </div>
            )}

            {activeTab === 'semantic' && (
              <div className="space-y-6">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                  <Zap className="text-purple-600" />
                  Semantic Analysis
                </h3>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="bg-gradient-to-r from-blue-50 to-purple-50 p-6 rounded-lg">
                    <h4 className="font-semibold text-gray-900 mb-3">Content Similarity Score</h4>
                    <div
                      className={`text-4xl font-bold ${getSimilarityColor(result.summary.content_similarity)} mb-2`}
                    >
                      {(result.summary.content_similarity * 100).toFixed(1)}%
                    </div>
                    <p className="text-gray-600">
                      Based on semantic understanding of document content
                    </p>
                  </div>

                  <div className="bg-gradient-to-r from-green-50 to-blue-50 p-6 rounded-lg">
                    <h4 className="font-semibold text-gray-900 mb-3">Structural Similarity</h4>
                    <div
                      className={`text-4xl font-bold ${getSimilarityColor(result.summary.structural_similarity)} mb-2`}
                    >
                      {(result.summary.structural_similarity * 100).toFixed(1)}%
                    </div>
                    <p className="text-gray-600">
                      How similar the document structure and organization is
                    </p>
                  </div>
                </div>

                {(() => {
                  const semanticChanges = getSemanticChanges(result.differences)
                  return semanticChanges.length > 0 ? (
                    <div>
                      <h4 className="text-lg font-semibold mb-3">Semantic Changes</h4>
                      <div className="space-y-3">
                        {semanticChanges.map((change, index) => (
                          <div key={index} className="border border-gray-200 rounded-lg p-4">
                            <div className="flex items-center gap-2 mb-2">
                              <span
                                className={`px-2 py-1 rounded-full text-xs font-medium ${getChangeTypeColor(change.diff_type)}`}
                              >
                                {change.diff_type}
                              </span>
                              <span
                                className={`px-2 py-1 rounded-full text-xs font-medium ${
                                  change.severity === 'Critical'
                                    ? 'bg-red-100 text-red-700'
                                    : change.severity === 'Major'
                                      ? 'bg-orange-100 text-orange-700'
                                      : 'bg-yellow-100 text-yellow-700'
                                }`}
                              >
                                {change.severity}
                              </span>
                              <span className="text-sm text-gray-600">
                                Confidence: {(change.confidence * 100).toFixed(0)}%
                              </span>
                            </div>
                            <p className="text-gray-700">{change.description}</p>
                          </div>
                        ))}
                      </div>
                    </div>
                  ) : null
                })()}

                <div className="bg-gray-50 p-6 rounded-lg">
                  <h4 className="font-semibold text-gray-900 mb-3">Understanding the Scores</h4>
                  <div className="grid gap-3 text-sm">
                    <div className="flex items-center gap-2">
                      <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                      <span>
                        <strong>High Similarity (80-100%):</strong> Documents are very similar with
                        minor differences
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
                      <span>
                        <strong>Medium Similarity (60-79%):</strong> Documents share common themes
                        but have notable differences
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                      <span>
                        <strong>Low Similarity (0-59%):</strong> Documents are significantly
                        different or cover different topics
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

export default DocumentComparison
