import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  FileText,
  Search,
  ChevronDown,
  ChevronRight,
  Hash,
  Clock,
  BarChart3,
  Layers,
  RefreshCw,
  Eye,
  Database,
  Trash2,
  AlertTriangle,
} from 'lucide-react'

interface DocumentChunk {
  chunk_id: string
  document_id: string
  content: string
  metadata: {
    title?: string
    section?: string
    page_number?: number
    keywords?: string[]
    chunk_index: number
    chunk_type: string
    heading_level?: number
    word_count: number
    content_hash: string
  }
  position: {
    start_char: number
    end_char: number
    start_line: number
    end_line: number
  }
}

interface DocumentIndexEntry {
  id: string
  path: string
  title: string
  summary?: string
  content: string
  metadata: {
    file_size?: number
    created_at?: string
    modified_at?: string
    document_type?: string
    page_count?: number
    word_count?: number
    language?: string
  }
  structure: {
    document_type: string
    sections: Array<{
      id: string
      title: string
      level: number
      content: string
      keywords: string[]
    }>
    toc?: Array<{ title: string; level: number; page?: number }>
    page_count?: number
    has_images: boolean
    has_tables: boolean
    has_code: boolean
  }
  keywords: string[]
  content_hash: string
  indexed_at: string
  index_version: number
}

interface IndexStats {
  total_documents: number
  total_chunks: number
  total_size_mb: number
  avg_chunks_per_document: number
  most_common_keywords: string[]
  processing_performance: {
    avg_processing_time_ms: number
    total_processing_time_ms: number
  }
}

interface SearchStats {
  total_chunks: number
  total_embeddings: number
  total_documents: number
  dimension: number
  memory_usage_estimate: number
}

const DocumentIndex: React.FC = () => {
  const [indexEntries, setIndexEntries] = useState<DocumentIndexEntry[]>([])
  const [selectedDocument, setSelectedDocument] = useState<DocumentIndexEntry | null>(null)
  const [documentChunks, setDocumentChunks] = useState<DocumentChunk[]>([])
  const [indexStats, setIndexStats] = useState<IndexStats | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [filterType, setFilterType] = useState<'all' | 'processed' | 'processing' | 'error'>('all')
  const [expandedDocuments, setExpandedDocuments] = useState<Set<string>>(new Set())
  const [selectedChunk, setSelectedChunk] = useState<DocumentChunk | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null)
  const [showClearConfirm, setShowClearConfirm] = useState(false)
  const [deletingDocument, setDeletingDocument] = useState<string | null>(null)

  useEffect(() => {
    loadIndexData()
  }, [])

  const loadIndexData = async () => {
    setIsLoading(true)
    try {
      // Initialize document indexer first
      try {
        await invoke('init_document_indexer', { indexDir: null })
        console.log('Document indexer initialized successfully')
      } catch (initError) {
        console.warn('Failed to initialize document indexer:', initError)
      }

      // Try to load from document indexer first
      try {
        const indexStats = await invoke<IndexStats>('get_index_stats')
        setIndexStats(indexStats)

        const documents = await invoke<DocumentIndexEntry[]>('get_all_documents')
        setIndexEntries(documents)
        console.log('Successfully loaded', documents.length, 'documents from indexer')
      } catch {
        console.log('Document indexer not available, falling back to vector stats')

        // Fallback to vector stats
        const vectorStats = await invoke<SearchStats>('get_vector_stats')

        const stats: IndexStats = {
          total_documents: vectorStats.total_documents || 0,
          total_chunks: vectorStats.total_chunks || 0,
          total_size_mb: vectorStats.memory_usage_estimate
            ? vectorStats.memory_usage_estimate / (1024 * 1024)
            : 0,
          avg_chunks_per_document:
            vectorStats.total_documents > 0
              ? Math.round(vectorStats.total_chunks / vectorStats.total_documents)
              : 0,
          most_common_keywords: [],
          processing_performance: {
            avg_processing_time_ms: 0,
            total_processing_time_ms: 0,
          },
        }
        setIndexStats(stats)
        setIndexEntries([])
      }
    } catch (error) {
      console.error('Failed to load index data:', error)
      // Set default values to prevent crashes
      setIndexStats({
        total_documents: 0,
        total_chunks: 0,
        total_size_mb: 0,
        avg_chunks_per_document: 0,
        most_common_keywords: [],
        processing_performance: {
          avg_processing_time_ms: 0,
          total_processing_time_ms: 0,
        },
      })
      setIndexEntries([])
    } finally {
      setIsLoading(false)
    }
  }

  const toggleDocumentExpansion = (documentId: string) => {
    const newExpanded = new Set(expandedDocuments)
    if (newExpanded.has(documentId)) {
      newExpanded.delete(documentId)
      if (selectedDocument?.id === documentId) {
        setSelectedDocument(null)
        setDocumentChunks([])
      }
    } else {
      newExpanded.add(documentId)
      const document = indexEntries.find(entry => entry.id === documentId)
      if (document) {
        setSelectedDocument(document)
        // Use the document's sections as chunks
        const sections = document.structure?.sections || []
        setDocumentChunks(
          sections.map(section => ({
            chunk_id: section.id,
            document_id: document.id,
            content: section.content || '',
            chunk_index: 0,
            start_char: 0,
            end_char: (section.content || '').length,
            metadata: {
              title: section.title || '',
              section: section.title || '',
              chunk_index: 0,
              chunk_type: 'section',
              heading_level: section.level || 1,
              word_count: (section.content || '').split(' ').length,
              content_hash: '',
            },
            position: {
              start_char: 0,
              end_char: (section.content || '').length,
              start_line: 0,
              end_line: 0,
            },
          }))
        )
      }
    }
    setExpandedDocuments(newExpanded)
  }

  const refreshIndex = async () => {
    setIsLoading(true)
    try {
      // Just reload the data since there's no refresh command
      await loadIndexData()
    } catch (error) {
      console.error('Failed to refresh index:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const deleteDocument = async (documentId: string) => {
    setDeletingDocument(documentId)
    try {
      // Remove from document indexer
      await invoke('remove_document_from_indexer', { documentId })

      // Also try to remove from vector store (may fail if not in vector store)
      try {
        await invoke('remove_document_from_index', { documentId })
      } catch (vectorError) {
        console.warn('Failed to remove from vector store (may not exist there):', vectorError)
      }

      // Refresh the data
      await loadIndexData()

      // Close expanded document if it was deleted
      if (expandedDocuments.has(documentId)) {
        const newExpanded = new Set(expandedDocuments)
        newExpanded.delete(documentId)
        setExpandedDocuments(newExpanded)
        if (selectedDocument?.id === documentId) {
          setSelectedDocument(null)
          setDocumentChunks([])
        }
      }

      console.log('Document deleted successfully:', documentId)
    } catch (error) {
      console.error('Failed to delete document:', error)
      alert('Failed to delete document: ' + (error as Error).message)
    } finally {
      setDeletingDocument(null)
      setShowDeleteConfirm(null)
    }
  }

  const clearAllDocuments = async () => {
    setIsLoading(true)
    try {
      const removedCount = await invoke<number>('clear_document_index')
      console.log(`Cleared ${removedCount} documents from index`)

      // Reset UI state
      setIndexEntries([])
      setSelectedDocument(null)
      setDocumentChunks([])
      setExpandedDocuments(new Set())

      // Reload data to get fresh stats
      await loadIndexData()
    } catch (error) {
      console.error('Failed to clear documents:', error)
      alert('Failed to clear documents: ' + (error as Error).message)
    } finally {
      setIsLoading(false)
      setShowClearConfirm(false)
    }
  }

  const filteredEntries = indexEntries.filter(entry => {
    const matchesSearch =
      !searchQuery ||
      (entry.title || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      (entry.path || '').toLowerCase().includes(searchQuery.toLowerCase())

    const matchesFilter = filterType === 'all' || filterType === 'processed'

    return matchesSearch && matchesFilter
  })

  const formatDocumentType = (
    documentType: string | { [key: string]: string } | null | undefined
  ): string => {
    if (!documentType) return 'Unknown'
    if (typeof documentType === 'string') return documentType
    if (typeof documentType === 'object' && documentType.Other) {
      return `Other (${documentType.Other})`
    }
    // Handle other enum variants that might be objects
    if (typeof documentType === 'object') {
      const keys = Object.keys(documentType)
      if (keys.length > 0) {
        const variant = keys[0]
        if (variant && documentType[variant] !== undefined) {
          const value = documentType[variant]
          return value ? `${variant} (${value})` : variant
        }
      }
    }
    return String(documentType)
  }

  const getChunkTypeIcon = (chunkType: string) => {
    switch (chunkType) {
      case 'heading':
        return <Hash className="text-blue-600" size={16} />
      case 'paragraph':
        return <FileText className="text-gray-600" size={16} />
      case 'list':
        return <Layers className="text-green-600" size={16} />
      case 'table':
        return <BarChart3 className="text-purple-600" size={16} />
      default:
        return <FileText className="text-gray-600" size={16} />
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold text-gray-900 flex items-center justify-center gap-3">
          <Database className="text-purple-600" />
          Document Index Browser
        </h1>
        <p className="text-gray-600">Explore indexed documents, chunks, and processing metadata</p>
      </div>

      {/* Stats Overview */}
      {indexStats && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-gradient-to-r from-blue-50 to-blue-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <FileText className="text-blue-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-blue-900">{indexStats.total_documents}</div>
                <div className="text-sm text-blue-700">Documents</div>
              </div>
            </div>
          </div>

          <div className="bg-gradient-to-r from-green-50 to-green-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <Layers className="text-green-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-green-900">{indexStats.total_chunks}</div>
                <div className="text-sm text-green-700">Chunks</div>
              </div>
            </div>
          </div>

          <div className="bg-gradient-to-r from-purple-50 to-purple-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <Database className="text-purple-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-purple-900">
                  {indexStats?.total_size_mb?.toFixed(1) || '0.0'}MB
                </div>
                <div className="text-sm text-purple-700">Index Size</div>
              </div>
            </div>
          </div>

          <div className="bg-gradient-to-r from-orange-50 to-orange-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <Clock className="text-orange-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-orange-900">
                  {formatDuration(indexStats?.processing_performance?.avg_processing_time_ms || 0)}
                </div>
                <div className="text-sm text-orange-700">Avg Processing</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Controls */}
      <div className="bg-white rounded-xl shadow-lg p-6 border border-gray-200">
        <div className="flex flex-wrap items-center gap-4 mb-4">
          <div className="flex-1 relative">
            <Search
              className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400"
              size={20}
            />
            <input
              type="text"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              placeholder="Search documents by title or path..."
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
            />
          </div>

          <select
            value={filterType}
            onChange={e =>
              setFilterType(e.target.value as 'all' | 'processed' | 'processing' | 'error')
            }
            className="border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-purple-500"
          >
            <option value="all">All Documents</option>
            <option value="processed">Processed</option>
            <option value="processing">Processing</option>
            <option value="error">Error</option>
          </select>

          <button
            onClick={refreshIndex}
            disabled={isLoading}
            className="bg-purple-600 text-white px-4 py-2 rounded-lg hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2"
          >
            <RefreshCw className={isLoading ? 'animate-spin' : ''} size={16} />
            Refresh
          </button>

          {indexEntries.length > 0 && (
            <button
              onClick={() => setShowClearConfirm(true)}
              disabled={isLoading}
              className="bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
            >
              <Trash2 size={16} />
              Clear All
            </button>
          )}
        </div>

        <div className="text-sm text-gray-600">
          Showing {filteredEntries.length} of {indexEntries.length} documents
        </div>
      </div>

      {/* Document List */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left Panel - Document List */}
        <div className="space-y-4">
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <FileText className="text-purple-600" />
            Indexed Documents
          </h2>

          <div className="space-y-2 max-h-96 overflow-y-auto">
            {filteredEntries.map(entry => (
              <div key={entry.id} className="border border-gray-200 rounded-lg">
                <div
                  className="p-4 cursor-pointer hover:bg-gray-50 transition-colors"
                  onClick={() => toggleDocumentExpansion(entry.id)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 flex-1">
                      {expandedDocuments.has(entry.id) ? (
                        <ChevronDown size={16} />
                      ) : (
                        <ChevronRight size={16} />
                      )}
                      <FileText size={16} className="text-gray-600" />
                      <div className="flex-1">
                        <div className="font-medium text-gray-900 truncate">{entry.title}</div>
                        <div className="text-sm text-gray-600 truncate">{entry.path}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                        Indexed
                      </span>
                      <button
                        onClick={e => {
                          e.stopPropagation()
                          setShowDeleteConfirm(entry.id)
                        }}
                        disabled={deletingDocument === entry.id}
                        className="p-1 text-red-600 hover:bg-red-50 rounded disabled:opacity-50"
                        title="Delete document"
                      >
                        {deletingDocument === entry.id ? (
                          <RefreshCw className="animate-spin" size={16} />
                        ) : (
                          <Trash2 size={16} />
                        )}
                      </button>
                    </div>
                  </div>
                </div>

                {expandedDocuments.has(entry.id) && (
                  <div className="border-t border-gray-200 p-4 bg-gray-50">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <div className="text-gray-600">File Size</div>
                        <div className="font-medium">
                          {entry.metadata?.file_size
                            ? formatFileSize(entry.metadata.file_size)
                            : 'Unknown'}
                        </div>
                      </div>
                      <div>
                        <div className="text-gray-600">Type</div>
                        <div className="font-medium">
                          {formatDocumentType(entry.structure?.document_type)}
                        </div>
                      </div>
                      <div>
                        <div className="text-gray-600">Sections</div>
                        <div className="font-medium">{entry.structure?.sections?.length || 0}</div>
                      </div>
                      <div>
                        <div className="text-gray-600">Keywords</div>
                        <div className="font-medium">{entry.keywords?.length || 0}</div>
                      </div>
                    </div>

                    <div className="mt-3 pt-3 border-t border-gray-200">
                      <div className="text-xs text-gray-600">
                        Indexed {new Date(entry.indexed_at).toLocaleDateString()} • Version{' '}
                        {entry.index_version}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Right Panel - Chunks Browser */}
        <div className="space-y-4">
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Layers className="text-purple-600" />
            Document Chunks
            {selectedDocument && (
              <span className="text-sm text-gray-600">({documentChunks.length} chunks)</span>
            )}
          </h2>

          {selectedDocument ? (
            <div className="space-y-4">
              {/* Selected Document Info */}
              <div className="bg-white p-4 rounded-lg border border-gray-200">
                <h3 className="font-medium text-gray-900 mb-2">{selectedDocument.title}</h3>
                <div className="grid grid-cols-2 gap-4 text-sm text-gray-600">
                  <div>Word Count: {selectedDocument.metadata?.word_count || 'Unknown'}</div>
                  <div>Sections: {selectedDocument.structure?.sections?.length || 0}</div>
                  <div>Keywords: {selectedDocument.keywords?.length || 0}</div>
                  <div>Content: {(selectedDocument.content || '').length} chars</div>
                </div>
              </div>

              {/* Chunks List */}
              <div className="space-y-2 max-h-96 overflow-y-auto">
                {documentChunks.map(chunk => (
                  <div
                    key={chunk.chunk_id}
                    className={`border rounded-lg p-3 cursor-pointer transition-colors ${
                      selectedChunk?.chunk_id === chunk.chunk_id
                        ? 'border-purple-500 bg-purple-50'
                        : 'border-gray-200 hover:bg-gray-50'
                    }`}
                    onClick={() => setSelectedChunk(chunk)}
                  >
                    <div className="flex items-start gap-2 mb-2">
                      {getChunkTypeIcon(chunk.metadata?.chunk_type || 'paragraph')}
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-sm font-medium text-gray-900">
                            Chunk {(chunk.metadata?.chunk_index || 0) + 1}
                          </span>
                          {chunk.metadata.section && (
                            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                              {chunk.metadata?.section}
                            </span>
                          )}
                          {chunk.metadata.page_number && (
                            <span className="text-xs bg-gray-100 text-gray-600 px-2 py-1 rounded">
                              Page {chunk.metadata?.page_number}
                            </span>
                          )}
                        </div>
                        <div className="text-sm text-gray-600 mb-2">
                          {(chunk.content || '').substring(0, 100)}...
                        </div>
                        <div className="flex items-center gap-4 text-xs text-gray-500">
                          <span>{chunk.metadata?.word_count || 0} words</span>
                          <span>
                            Lines {chunk.position?.start_line || 0}-{chunk.position?.end_line || 0}
                          </span>
                          {chunk.metadata?.keywords && chunk.metadata.keywords.length > 0 && (
                            <span>{chunk.metadata.keywords.length} keywords</span>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="bg-gray-50 border-2 border-dashed border-gray-300 rounded-lg p-8 text-center">
              <Eye className="mx-auto h-12 w-12 text-gray-400 mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No Document Selected</h3>
              <p className="text-gray-600">
                Click on a document from the left panel to view its chunks and processing details
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Chunk Detail Modal */}
      {selectedChunk && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-4xl w-full max-h-[80vh] overflow-hidden">
            <div className="p-6 border-b border-gray-200">
              <div className="flex items-center justify-between">
                <h3 className="text-xl font-semibold flex items-center gap-2">
                  {getChunkTypeIcon(selectedChunk.metadata?.chunk_type || 'paragraph')}
                  Chunk {(selectedChunk.metadata?.chunk_index || 0) + 1} Details
                </h3>
                <button
                  onClick={() => setSelectedChunk(null)}
                  className="text-gray-400 hover:text-gray-600"
                >
                  ✕
                </button>
              </div>
            </div>

            <div className="p-6 space-y-4 overflow-y-auto max-h-[60vh]">
              {/* Metadata */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div>
                  <div className="text-gray-600">Type</div>
                  <div className="font-medium">
                    {selectedChunk.metadata?.chunk_type || 'Unknown'}
                  </div>
                </div>
                <div>
                  <div className="text-gray-600">Word Count</div>
                  <div className="font-medium">{selectedChunk.metadata?.word_count || 0}</div>
                </div>
                <div>
                  <div className="text-gray-600">Position</div>
                  <div className="font-medium">
                    {selectedChunk.position.start_char}-{selectedChunk.position.end_char}
                  </div>
                </div>
                <div>
                  <div className="text-gray-600">Lines</div>
                  <div className="font-medium">
                    {selectedChunk.position.start_line}-{selectedChunk.position.end_line}
                  </div>
                </div>
              </div>

              {/* Keywords */}
              {selectedChunk.metadata?.keywords && selectedChunk.metadata.keywords.length > 0 && (
                <div>
                  <div className="text-sm text-gray-600 mb-2">Keywords</div>
                  <div className="flex flex-wrap gap-2">
                    {selectedChunk.metadata.keywords.map((keyword, index) => (
                      <span
                        key={index}
                        className="bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded-full"
                      >
                        {keyword}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {/* Content */}
              <div>
                <div className="text-sm text-gray-600 mb-2">Content</div>
                <div className="bg-gray-50 p-4 rounded-lg border">
                  <pre className="whitespace-pre-wrap text-sm text-gray-900 leading-relaxed">
                    {selectedChunk.content}
                  </pre>
                </div>
              </div>

              {/* Technical Details */}
              <div className="bg-blue-50 p-4 rounded-lg">
                <div className="text-sm text-blue-900 font-medium mb-2">Technical Details</div>
                <div className="grid grid-cols-2 gap-4 text-xs text-blue-800">
                  <div>
                    <div>Chunk ID</div>
                    <div className="font-mono">{selectedChunk.chunk_id}</div>
                  </div>
                  <div>
                    <div>Content Hash</div>
                    <div className="font-mono">{selectedChunk.metadata?.content_hash || 'N/A'}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-md w-full p-6">
            <div className="flex items-center gap-3 mb-4">
              <AlertTriangle className="text-red-500" size={24} />
              <h3 className="text-lg font-semibold text-gray-900">Delete Document</h3>
            </div>
            <p className="text-gray-600 mb-6">
              Are you sure you want to delete this document from the index? This action cannot be
              undone.
            </p>
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setShowDeleteConfirm(null)}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={() => deleteDocument(showDeleteConfirm)}
                disabled={deletingDocument === showDeleteConfirm}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
              >
                {deletingDocument === showDeleteConfirm ? (
                  <>
                    <RefreshCw className="animate-spin" size={16} />
                    Deleting...
                  </>
                ) : (
                  <>
                    <Trash2 size={16} />
                    Delete
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Clear All Confirmation Modal */}
      {showClearConfirm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-md w-full p-6">
            <div className="flex items-center gap-3 mb-4">
              <AlertTriangle className="text-red-500" size={24} />
              <h3 className="text-lg font-semibold text-gray-900">Clear All Documents</h3>
            </div>
            <p className="text-gray-600 mb-6">
              Are you sure you want to remove ALL {indexEntries.length} documents from the index?
              This will permanently delete all indexed documents and cannot be undone.
            </p>
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={clearAllDocuments}
                disabled={isLoading}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
              >
                {isLoading ? (
                  <>
                    <RefreshCw className="animate-spin" size={16} />
                    Clearing...
                  </>
                ) : (
                  <>
                    <Trash2 size={16} />
                    Clear All
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default DocumentIndex
