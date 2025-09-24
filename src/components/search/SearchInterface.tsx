import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Search,
  Zap,
  FileText,
  Clock,
  TrendingUp,
  Sparkles,
  RefreshCw,
  AlertTriangle,
  Activity,
} from 'lucide-react'

interface SearchResult {
  chunk: {
    id: string
    document_id: string
    content: string
    chunk_index: number
    start_char: number
    end_char: number
    metadata: { [key: string]: string }
  }
  similarity: number
  explanation: string
}

interface SearchResponse {
  success: boolean
  results: SearchResult[]
  query_time_ms: number
  error?: string
}

interface SearchStats {
  total_chunks: number
  total_embeddings: number
  total_documents: number
  dimension: number
  memory_usage_estimate: number
}

const SearchInterface: React.FC = () => {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const [searchType, setSearchType] = useState<'keyword' | 'semantic'>('semantic')
  const [maxResults, setMaxResults] = useState(10)
  const [queryTime, setQueryTime] = useState<number>(0)
  const [searchStats, setSearchStats] = useState<SearchStats | null>(null)
  const [recentQueries, setRecentQueries] = useState<string[]>([])
  const [isSyncing, setIsSyncing] = useState(false)
  const [syncError, setSyncError] = useState<string | null>(null)
  const [isRunningDiagnostics, setIsRunningDiagnostics] = useState(false)
  const [diagnosticResults, setDiagnosticResults] = useState<string | null>(null)

  useEffect(() => {
    loadSearchStats()
    loadRecentQueries()
  }, [])

  const loadSearchStats = async () => {
    try {
      const stats = await invoke<SearchStats>('get_vector_stats')
      setSearchStats(stats)
    } catch (error) {
      console.error('Failed to load search stats:', error)
      // Set default stats to prevent UI from breaking
      setSearchStats({
        total_chunks: 0,
        total_embeddings: 0,
        total_documents: 0,
        dimension: 384,
        memory_usage_estimate: 0,
      })
    }
  }

  const loadRecentQueries = () => {
    const stored = localStorage.getItem('recent_search_queries')
    if (stored) {
      setRecentQueries(JSON.parse(stored))
    }
  }

  const saveRecentQuery = (query: string) => {
    const updated = [query, ...recentQueries.filter(q => q !== query)].slice(0, 5)
    setRecentQueries(updated)
    localStorage.setItem('recent_search_queries', JSON.stringify(updated))
  }

  const syncDocuments = async () => {
    setIsSyncing(true)
    setSyncError(null)

    // Create an AbortController for cancellation
    const controller = new AbortController()
    const timeoutId = setTimeout(() => {
      controller.abort()
    }, 120000) // 2-minute timeout to prevent system hangs

    try {
      console.log('Starting document sync with 2-minute timeout...')

      // First ensure vector system is initialized with timeout
      const initPromise = invoke('init_vector_system')
      await Promise.race([
        initPromise,
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error('Vector system initialization timed out after 30 seconds')), 30000)
        )
      ])
      console.log('Vector system initialized successfully')

      // Then sync documents with strict timeout
      const syncPromise = invoke<string>('sync_documents_to_vector_system')
      const syncResult = await Promise.race([
        syncPromise,
        new Promise<string>((_, reject) =>
          setTimeout(() => reject(new Error('Document sync timed out after 90 seconds to prevent system hang')), 90000)
        )
      ])
      console.log('Document sync result:', syncResult)

      // Reload stats after sync
      await loadSearchStats()

      if (searchStats && searchStats.total_documents > 0) {
        console.log(`Successfully synced ${searchStats.total_documents} documents to vector system`)
      }
    } catch (error: unknown) {
      console.error('Failed to sync documents:', error)
      setSyncError(error instanceof Error ? error.message : String(error))
    } finally {
      clearTimeout(timeoutId)
      setIsSyncing(false)
    }
  }

  const runDiagnostics = async () => {
    setIsRunningDiagnostics(true)
    setDiagnosticResults(null)

    try {
      const results = await invoke<string>('diagnose_document_vector_system')
      setDiagnosticResults(results)
    } catch (error: unknown) {
      console.error('Diagnostics failed:', error)
      setDiagnosticResults(
        `Diagnostics Error: ${error instanceof Error ? error.message : String(error)}`
      )
    } finally {
      setIsRunningDiagnostics(false)
    }
  }

  const performSearch = async () => {
    if (!query.trim()) return

    setIsSearching(true)
    try {
      const searchRequest = {
        query: query.trim(),
        document_id: null,
        max_results: maxResults,
      }

      const commandName = searchType === 'semantic' ? 'semantic_search' : 'keyword_search'
      const response = await invoke<SearchResponse>(commandName, { request: searchRequest })

      if (response.success) {
        setResults(response.results)
        setQueryTime(response.query_time_ms)
        saveRecentQuery(query.trim())
      } else {
        console.error('Search failed:', response.error)
        setResults([])
      }
    } catch (error) {
      console.error('Search error:', error)
      setResults([])
      // Optionally show user-friendly error message
      alert(
        'Search functionality is currently unavailable. Please ensure documents are indexed first.'
      )
    } finally {
      setIsSearching(false)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      performSearch()
    }
  }

  const formatRelevanceScore = (similarity: number) => {
    const percentage = Math.round(similarity * 100)
    if (percentage >= 80) return { score: percentage, color: 'text-green-600', bg: 'bg-green-100' }
    if (percentage >= 60) return { score: percentage, color: 'text-blue-600', bg: 'bg-blue-100' }
    if (percentage >= 40)
      return { score: percentage, color: 'text-yellow-600', bg: 'bg-yellow-100' }
    return { score: percentage, color: 'text-gray-600', bg: 'bg-gray-100' }
  }

  const highlightQuery = (text: string, query: string) => {
    if (!query.trim()) return text

    const queryTerms = query.toLowerCase().split(/\s+/)
    let highlightedText = text

    queryTerms.forEach(term => {
      const regex = new RegExp(`(${term})`, 'gi')
      highlightedText = highlightedText.replace(
        regex,
        '<mark class="bg-yellow-200 px-1 rounded">$1</mark>'
      )
    })

    return highlightedText
  }

  return (
    <div className="max-w-6xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold text-gray-900 flex items-center justify-center gap-3">
          <Sparkles className="text-purple-600" />
          Intelligent Document Search
        </h1>
        <p className="text-gray-600">
          Advanced semantic search with TF-IDF scoring and phrase matching
        </p>
      </div>

      {/* Search Stats */}
      {searchStats && (
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="bg-gradient-to-r from-blue-50 to-blue-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <FileText className="text-blue-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-blue-900">
                  {searchStats.total_documents}
                </div>
                <div className="text-sm text-blue-700">Documents Indexed</div>
              </div>
            </div>
          </div>
          <div className="bg-gradient-to-r from-green-50 to-green-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <TrendingUp className="text-green-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-green-900">{searchStats.total_chunks}</div>
                <div className="text-sm text-green-700">Searchable Chunks</div>
              </div>
            </div>
          </div>
          <div className="bg-gradient-to-r from-purple-50 to-purple-100 p-4 rounded-lg">
            <div className="flex items-center gap-2">
              <Zap className="text-purple-600" size={20} />
              <div>
                <div className="text-2xl font-bold text-purple-900">
                  {(searchStats.memory_usage_estimate / (1024 * 1024)).toFixed(1)}MB
                </div>
                <div className="text-sm text-purple-700">Memory Usage</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Sync Controls */}
      {searchStats && searchStats.total_documents === 0 && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
          <div className="flex items-start gap-3">
            <AlertTriangle className="text-yellow-600 mt-1" size={20} />
            <div className="flex-1">
              <h3 className="font-medium text-yellow-800 mb-2">No documents indexed for search</h3>
              <p className="text-yellow-700 text-sm mb-3">
                You need to sync documents from the Document Index to enable intelligent search.
              </p>
              <div className="flex gap-3 flex-wrap">
                <button
                  onClick={syncDocuments}
                  disabled={isSyncing}
                  className="bg-yellow-600 text-white px-4 py-2 rounded-lg hover:bg-yellow-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm"
                >
                  {isSyncing ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
                      Syncing Documents...
                    </>
                  ) : (
                    <>
                      <RefreshCw size={16} />
                      Sync Documents from Index
                    </>
                  )}
                </button>
                <button
                  onClick={runDiagnostics}
                  disabled={isRunningDiagnostics}
                  className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm"
                >
                  {isRunningDiagnostics ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
                      Running Diagnostics...
                    </>
                  ) : (
                    <>
                      <Activity size={16} />
                      Run System Diagnostics
                    </>
                  )}
                </button>
              </div>
              {syncError && (
                <div className="mt-3 p-3 bg-red-100 border border-red-200 rounded text-red-800 text-sm">
                  <strong>Sync Error:</strong> {syncError}
                  {syncError.includes('CPU-intensive') && (
                    <div className="mt-2">
                      <strong>Recommendation:</strong> Configure OPENROUTER_API_KEY or
                      OPENAI_API_KEY in your .env file for better performance.
                    </div>
                  )}
                </div>
              )}
              {diagnosticResults && (
                <div className="mt-3 p-3 bg-blue-50 border border-blue-200 rounded text-blue-900 text-sm">
                  <strong>System Diagnostics:</strong>
                  <pre className="mt-2 whitespace-pre-wrap font-mono text-xs leading-relaxed">
                    {diagnosticResults}
                  </pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Search Interface */}
      <div className="bg-white rounded-xl shadow-lg p-6 border border-gray-200">
        <div className="space-y-4">
          {/* Search Input */}
          <div className="relative">
            <Search
              className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400"
              size={20}
            />
            <input
              type="text"
              value={query}
              onChange={e => setQuery(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Search your documents with natural language..."
              className="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 text-lg"
            />
          </div>

          {/* Search Controls */}
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex items-center gap-2">
              <label className="text-sm font-medium text-gray-700">Search Type:</label>
              <select
                value={searchType}
                onChange={e => setSearchType(e.target.value as 'keyword' | 'semantic')}
                className="border border-gray-300 rounded px-3 py-1 text-sm focus:ring-2 focus:ring-purple-500"
              >
                <option value="semantic">🧠 Semantic (AI-Enhanced)</option>
                <option value="keyword">📝 Keyword (Traditional)</option>
              </select>
            </div>

            <div className="flex items-center gap-2">
              <label className="text-sm font-medium text-gray-700">Max Results:</label>
              <select
                value={maxResults}
                onChange={e => setMaxResults(Number(e.target.value))}
                className="border border-gray-300 rounded px-3 py-1 text-sm focus:ring-2 focus:ring-purple-500"
              >
                <option value={5}>5</option>
                <option value={10}>10</option>
                <option value={20}>20</option>
                <option value={50}>50</option>
              </select>
            </div>

            <button
              onClick={performSearch}
              disabled={isSearching || !query.trim()}
              className="bg-purple-600 text-white px-6 py-2 rounded-lg hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
            >
              {isSearching ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
                  Searching...
                </>
              ) : (
                <>
                  <Search size={16} />
                  Search
                </>
              )}
            </button>
          </div>

          {/* Recent Queries */}
          {recentQueries.length > 0 && (
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-sm text-gray-500">Recent:</span>
              {recentQueries.map((recentQuery, index) => (
                <button
                  key={index}
                  onClick={() => setQuery(recentQuery)}
                  className="text-sm bg-gray-100 hover:bg-gray-200 px-3 py-1 rounded-full transition-colors"
                >
                  {recentQuery}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Search Results */}
      {queryTime > 0 && (
        <div className="flex items-center gap-2 text-sm text-gray-600">
          <Clock size={16} />
          Found {results.length} results in {queryTime}ms
          {searchType === 'semantic' && (
            <span className="bg-purple-100 text-purple-800 px-2 py-1 rounded-full text-xs ml-2">
              AI-Enhanced
            </span>
          )}
        </div>
      )}

      <div className="space-y-4">
        {results.map((result, _index) => {
          const relevance = formatRelevanceScore(result.similarity)
          return (
            <div
              key={result.chunk.id}
              className="bg-white rounded-lg shadow-md border border-gray-200 p-6 hover:shadow-lg transition-shadow"
            >
              {/* Result Header */}
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-gray-900 mb-1">
                    {result.chunk.metadata.title || result.chunk.document_id}
                  </h3>
                  <div className="flex items-center gap-4 text-sm text-gray-600">
                    {result.chunk.metadata.section && (
                      <span className="flex items-center gap-1">
                        <FileText size={14} />
                        {result.chunk.metadata.section}
                      </span>
                    )}
                    {result.chunk.metadata.page_number && (
                      <span>Page {result.chunk.metadata.page_number}</span>
                    )}
                    <span className="text-gray-400">Chunk {result.chunk.chunk_index + 1}</span>
                  </div>
                </div>

                <div
                  className={`${relevance.bg} ${relevance.color} px-3 py-1 rounded-full text-sm font-medium`}
                >
                  {relevance.score}% match
                </div>
              </div>

              {/* Content Preview */}
              <div className="mb-3">
                <p
                  className="text-gray-800 leading-relaxed"
                  dangerouslySetInnerHTML={{
                    __html: highlightQuery(
                      result.chunk.content.substring(0, 300) +
                        (result.chunk.content.length > 300 ? '...' : ''),
                      query
                    ),
                  }}
                />
              </div>

              {/* Keywords */}
              {result.chunk.metadata.keywords && (
                <div className="flex flex-wrap gap-2 mb-3">
                  {result.chunk.metadata.keywords
                    .split(',')
                    .slice(0, 5)
                    .map((keyword, idx) => (
                      <span
                        key={idx}
                        className="bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded-full"
                      >
                        {keyword.trim()}
                      </span>
                    ))}
                </div>
              )}

              {/* AI Explanation */}
              {searchType === 'semantic' && result.explanation && (
                <div className="bg-gradient-to-r from-purple-50 to-blue-50 p-3 rounded-lg border-l-4 border-purple-400">
                  <div className="flex items-start gap-2">
                    <Sparkles className="text-purple-600 mt-0.5" size={16} />
                    <div>
                      <div className="text-sm font-medium text-purple-900 mb-1">AI Insight</div>
                      <p className="text-sm text-purple-800">{result.explanation}</p>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>

      {/* Empty State */}
      {results.length === 0 && query && !isSearching && (
        <div className="text-center py-12">
          <Search className="mx-auto h-12 w-12 text-gray-400 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No results found</h3>
          <p className="text-gray-600 mb-4">
            Try adjusting your search terms or using different keywords
          </p>
          <button
            onClick={() => setSearchType(searchType === 'semantic' ? 'keyword' : 'semantic')}
            className="text-purple-600 hover:text-purple-800 font-medium"
          >
            Try {searchType === 'semantic' ? 'keyword' : 'semantic'} search instead
          </button>
        </div>
      )}
    </div>
  )
}

export default SearchInterface
