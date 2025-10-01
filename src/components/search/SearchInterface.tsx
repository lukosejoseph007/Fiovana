/**
 * Advanced Search Interface
 *
 * Hybrid semantic + keyword search UI with real-time suggestions,
 * filtering, and confidence-scored results
 */

import React, { useState, useCallback, useEffect, useMemo } from 'react'
import { searchService } from '../../services/searchService'
import { documentService } from '../../services/documentService'
import Input from '../ui/Input'
import Button from '../ui/Button'
import Badge from '../ui/Badge'
import Card from '../ui/Card'
import Dropdown from '../ui/Dropdown'
import Icon from '../ui/Icon'
import { colors, spacing, typography, shadows } from '../../styles/tokens'
import type { SearchQuery, SearchResult, SearchResultItem, SearchFilter } from '../../types'

// Filter configuration for document types
const DOCUMENT_TYPES = [
  { value: 'all', label: 'All Types' },
  { value: 'pdf', label: 'PDF' },
  { value: 'docx', label: 'Word' },
  { value: 'txt', label: 'Text' },
  { value: 'md', label: 'Markdown' },
]

// Content category options
const CONTENT_CATEGORIES = [
  { value: 'all', label: 'All Categories' },
  { value: 'concept', label: 'Concepts' },
  { value: 'procedure', label: 'Procedures' },
  { value: 'example', label: 'Examples' },
  { value: 'definition', label: 'Definitions' },
  { value: 'explanation', label: 'Explanations' },
  { value: 'reference', label: 'References' },
]

// Sort options
const SORT_OPTIONS = [
  { value: 'relevance', label: 'Relevance' },
  { value: 'date', label: 'Date Modified' },
  { value: 'title', label: 'Title' },
  { value: 'score', label: 'Confidence Score' },
]

export const SearchInterface: React.FC = React.memo(() => {
  const [searchText, setSearchText] = useState('')
  const [searchType, setSearchType] = useState<'semantic' | 'keyword' | 'hybrid'>('hybrid')
  const [selectedDocType, setSelectedDocType] = useState('all')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [sortBy, setSortBy] = useState('relevance')

  const [results, setResults] = useState<SearchResult | null>(null)
  const [suggestions, setSuggestions] = useState<string[]>([])
  const [savedSearches, setSavedSearches] = useState<Array<{ name: string; query: SearchQuery }>>(
    []
  )
  const [searchHistory, setSearchHistory] = useState<SearchQuery[]>([])

  const [isSearching, setIsSearching] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedResultId, setSelectedResultId] = useState<string | null>(null)

  // Load saved searches and history on mount
  useEffect(() => {
    const loadData = async () => {
      try {
        const [savedRes, historyRes] = await Promise.all([
          searchService.getSavedQueries(),
          searchService.getSearchHistory(10),
        ])

        if (savedRes.success && savedRes.data) {
          setSavedSearches(savedRes.data as Array<{ name: string; query: SearchQuery }>)
        }

        if (historyRes.success && historyRes.data) {
          setSearchHistory(historyRes.data)
        }
      } catch (err) {
        console.error('Failed to load search data:', err)
      }
    }

    loadData()
  }, [])

  // Get search suggestions as user types
  useEffect(() => {
    if (searchText.length < 2) {
      setSuggestions([])
      return
    }

    const timeoutId = setTimeout(async () => {
      try {
        const response = await searchService.getSearchSuggestions(searchText)
        if (response.success && response.data) {
          setSuggestions(response.data)
        }
      } catch (err) {
        console.error('Failed to get suggestions:', err)
      }
    }, 300)

    return () => clearTimeout(timeoutId)
  }, [searchText])

  // Build filters from UI state
  const buildFilters = useCallback((): SearchFilter[] => {
    const filters: SearchFilter[] = []

    if (selectedDocType !== 'all') {
      filters.push({
        field: 'type',
        operator: 'equals',
        value: selectedDocType,
      })
    }

    if (selectedCategory !== 'all') {
      filters.push({
        field: 'category',
        operator: 'equals',
        value: selectedCategory,
      })
    }

    return filters
  }, [selectedDocType, selectedCategory])

  // Perform search
  const performSearch = useCallback(async () => {
    if (!searchText.trim()) {
      setError('Please enter a search query')
      return
    }

    setIsSearching(true)
    setError(null)

    try {
      const query: SearchQuery = {
        text: searchText,
        type: searchType,
        filters: buildFilters(),
        options: {
          limit: 50,
          sortBy: sortBy === 'relevance' ? 'score' : sortBy,
          sortOrder: sortBy === 'date' ? 'desc' : 'asc',
          includeMetadata: true,
          includeContent: true,
          threshold: 0.3, // Minimum similarity threshold
        },
      }

      const response = await searchService.search(query)

      // Check if response has valid SearchResult structure
      if (
        response.success &&
        response.data &&
        'results' in response.data &&
        Array.isArray(response.data.results)
      ) {
        setResults(response.data)
        setError(null)
      } else {
        // Handle mock/undefined response gracefully
        setResults({
          query,
          results: [],
          totalCount: 0,
          executionTime: 0,
          metadata: {
            algorithm: 'hybrid',
            indexVersion: '1.0',
            performance: {
              queryTime: 0,
              indexTime: 0,
              postProcessingTime: 0,
              totalDocuments: 0,
              documentsScanned: 0,
            },
          },
        })
        setError(response.error || 'No results found (backend not connected)')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed')
      console.error('Search error:', err)
    } finally {
      setIsSearching(false)
    }
  }, [searchText, searchType, buildFilters, sortBy])

  // Handle enter key
  const handleKeyPress = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        performSearch()
      }
    },
    [performSearch]
  )

  // Save current search
  const handleSaveSearch = useCallback(async () => {
    const name = prompt('Enter a name for this search:')
    if (!name) return

    try {
      const query: SearchQuery = {
        text: searchText,
        type: searchType,
        filters: buildFilters(),
        options: { limit: 50 },
      }

      const response = await searchService.saveSearchQuery(query, name)
      if (response.success) {
        setSavedSearches(prev => [...prev, { name, query }])
      }
    } catch (err) {
      console.error('Failed to save search:', err)
    }
  }, [searchText, searchType, buildFilters])

  // Load a saved search
  const loadSavedSearch = useCallback((search: { name: string; query: SearchQuery }) => {
    setSearchText(search.query.text)
    setSearchType(search.query.type)
    if (search.query.filters) {
      search.query.filters.forEach(filter => {
        if (filter.field === 'type') setSelectedDocType(String(filter.value))
        if (filter.field === 'category') setSelectedCategory(String(filter.value))
      })
    }
  }, [])

  // Open document from result
  const handleOpenDocument = useCallback(async (item: SearchResultItem) => {
    try {
      setSelectedResultId(item.id)
      // Get the document to view it
      await documentService.getDocument(item.documentId)
      // In a real implementation, this would navigate to the document viewer
      console.log('Opening document:', item.documentId)
    } catch (err) {
      console.error('Failed to open document:', err)
    }
  }, [])

  // Get confidence color based on score
  const getConfidenceColor = useCallback((score: number): string => {
    if (score >= 0.8) return colors.confidence.high
    if (score >= 0.6) return colors.confidence.medium
    if (score >= 0.4) return colors.confidence.low
    return colors.confidence.critical
  }, [])

  // Memoized result count
  const resultCount = useMemo(() => results?.totalCount ?? 0, [results])

  return (
    <div style={styles.container}>
      {/* Search Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Advanced Search</h2>
        <p style={styles.subtitle}>Hybrid semantic + keyword search across all documents</p>
      </div>

      {/* Search Bar and Controls */}
      <div style={styles.searchControls}>
        <div style={styles.searchInputWrapper}>
          <Icon name="Search" style={styles.searchIcon} />
          <Input
            type="text"
            value={searchText}
            onChange={e => setSearchText(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Search documents, concepts, procedures..."
            style={styles.searchInput}
            autoFocus
          />
          {isSearching && (
            <div style={styles.loadingIndicator}>
              <Icon name="Loader" />
            </div>
          )}
        </div>

        <div style={styles.searchActions}>
          <Button
            variant="primary"
            onClick={performSearch}
            disabled={isSearching || !searchText.trim()}
            style={styles.searchButton}
          >
            {isSearching ? 'Searching...' : 'Search'}
          </Button>
          <Button
            variant="ghost"
            onClick={handleSaveSearch}
            disabled={!searchText.trim()}
            title="Save this search"
          >
            <Icon name="BookOpen" />
          </Button>
        </div>
      </div>

      {/* Search Suggestions */}
      {suggestions.length > 0 && !results && (
        <div style={styles.suggestions}>
          <p style={styles.suggestionsLabel}>Suggestions:</p>
          <div style={styles.suggestionsList}>
            {suggestions.map((suggestion, index) => (
              <button
                key={index}
                style={styles.suggestionChip}
                onClick={() => {
                  setSearchText(suggestion)
                  setSuggestions([])
                }}
              >
                {suggestion}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Filters Bar */}
      <div style={styles.filtersBar}>
        <div style={styles.filterGroup}>
          <label style={styles.filterLabel}>Search Type:</label>
          <div style={styles.searchTypeButtons}>
            {(['hybrid', 'semantic', 'keyword'] as const).map(type => (
              <button
                key={type}
                style={{
                  ...styles.typeButton,
                  ...(searchType === type ? styles.typeButtonActive : {}),
                }}
                onClick={() => setSearchType(type)}
              >
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </button>
            ))}
          </div>
        </div>

        <div style={styles.filterGroup}>
          <label style={styles.filterLabel}>Document Type:</label>
          <Dropdown
            value={selectedDocType}
            onChange={value => setSelectedDocType(value)}
            options={DOCUMENT_TYPES}
          />
        </div>

        <div style={styles.filterGroup}>
          <label style={styles.filterLabel}>Category:</label>
          <Dropdown
            value={selectedCategory}
            onChange={value => setSelectedCategory(value)}
            options={CONTENT_CATEGORIES}
          />
        </div>

        <div style={styles.filterGroup}>
          <label style={styles.filterLabel}>Sort By:</label>
          <Dropdown value={sortBy} onChange={value => setSortBy(value)} options={SORT_OPTIONS} />
        </div>
      </div>

      {/* Saved Searches & History */}
      <div style={styles.savedSection}>
        {savedSearches.length > 0 && (
          <div style={styles.savedSearches}>
            <p style={styles.savedLabel}>Saved Searches:</p>
            <div style={styles.savedList}>
              {savedSearches.slice(0, 5).map((search, index) => (
                <button
                  key={index}
                  style={styles.savedChip}
                  onClick={() => loadSavedSearch(search)}
                  title={search.query.text}
                >
                  <Icon name="BookOpen" style={styles.savedIcon} />
                  {search.name}
                </button>
              ))}
            </div>
          </div>
        )}

        {searchHistory.length > 0 && !results && (
          <div style={styles.historySection}>
            <p style={styles.savedLabel}>Recent Searches:</p>
            <div style={styles.savedList}>
              {searchHistory.slice(0, 5).map((query, index) => (
                <button
                  key={index}
                  style={styles.historyChip}
                  onClick={() => setSearchText(query.text)}
                >
                  <Icon name="RefreshCcw" style={styles.savedIcon} />
                  {query.text}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div style={styles.error}>
          <Icon name="AlertCircle" style={styles.errorIcon} />
          {error}
        </div>
      )}

      {/* Search Results */}
      {results && (
        <div style={styles.resultsSection}>
          {/* Results Header */}
          <div style={styles.resultsHeader}>
            <div style={styles.resultsInfo}>
              <span style={styles.resultsCount}>
                {resultCount} {resultCount === 1 ? 'result' : 'results'}
              </span>
              {results.executionTime && (
                <span style={styles.executionTime}>in {results.executionTime.toFixed(2)}ms</span>
              )}
              {results.metadata && results.metadata.algorithm && (
                <Badge variant="status" style={styles.algorithmBadge}>
                  {results.metadata.algorithm}
                </Badge>
              )}
            </div>
          </div>

          {/* Results List */}
          <div style={styles.resultsList}>
            {!results.results || results.results.length === 0 ? (
              <div style={styles.noResults}>
                <Icon name="Search" style={styles.noResultsIcon} />
                <p style={styles.noResultsText}>No results found</p>
                <p style={styles.noResultsHint}>Try adjusting your search query or filters</p>
              </div>
            ) : (
              results.results.map(item => (
                <Card
                  key={item.id}
                  style={{
                    ...styles.resultCard,
                    ...(selectedResultId === item.id ? styles.resultCardSelected : {}),
                  }}
                  onClick={() => handleOpenDocument(item)}
                >
                  {/* Result Header */}
                  <div style={styles.resultHeader}>
                    <div style={styles.resultTitle}>
                      <Icon name="FileText" style={styles.resultIcon} />
                      <h3 style={styles.resultTitleText}>{item.title}</h3>
                    </div>
                    <div style={styles.resultMeta}>
                      <Badge
                        variant="success"
                        style={{
                          ...styles.scoreBadge,
                          backgroundColor: getConfidenceColor(item.score),
                        }}
                      >
                        {(item.score * 100).toFixed(0)}%
                      </Badge>
                    </div>
                  </div>

                  {/* Result Content Preview */}
                  <div style={styles.resultContent}>
                    <p style={styles.resultText}>{item.content.substring(0, 200)}...</p>
                  </div>

                  {/* Result Path */}
                  <div style={styles.resultPath}>
                    <Icon name="Folder" style={styles.pathIcon} />
                    <span style={styles.pathText}>{item.path}</span>
                  </div>

                  {/* Highlights */}
                  {item.highlights && item.highlights.length > 0 && (
                    <div style={styles.highlights}>
                      {item.highlights.slice(0, 2).map((highlight, idx) => (
                        <div key={idx} style={styles.highlightSection}>
                          {highlight.fragments.slice(0, 1).map((fragment, fragIdx) => (
                            <p
                              key={fragIdx}
                              style={styles.highlightText}
                              dangerouslySetInnerHTML={{ __html: fragment }}
                            />
                          ))}
                        </div>
                      ))}
                    </div>
                  )}
                </Card>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  )
})

SearchInterface.displayName = 'SearchInterface'

// Styles
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    padding: spacing[6],
    backgroundColor: colors.surface.primary,
    overflow: 'hidden',
  },
  header: {
    marginBottom: spacing[6],
  },
  title: {
    fontSize: typography.fontSize['2xl'],
    fontWeight: typography.fontWeight.semibold,
    color: colors.text.primary,
    margin: 0,
    marginBottom: spacing[2],
  },
  subtitle: {
    fontSize: typography.fontSize.sm,
    color: colors.text.secondary,
    margin: 0,
  },
  searchControls: {
    display: 'flex',
    gap: spacing[3],
    marginBottom: spacing[4],
  },
  searchInputWrapper: {
    flex: 1,
    position: 'relative',
    display: 'flex',
    alignItems: 'center',
  },
  searchIcon: {
    position: 'absolute',
    left: spacing[3],
    color: colors.text.tertiary,
    width: '20px',
    height: '20px',
    pointerEvents: 'none',
  },
  searchInput: {
    width: '100%',
    paddingLeft: spacing[10],
    paddingRight: spacing[10],
    backgroundColor: colors.surface.secondary,
    color: colors.text.primary,
    border: `1px solid ${colors.border.subtle}`,
  },
  loadingIndicator: {
    position: 'absolute',
    right: spacing[3],
    color: colors.accent.ai,
    animation: 'spin 1s linear infinite',
  },
  searchActions: {
    display: 'flex',
    gap: spacing[2],
  },
  searchButton: {
    minWidth: '120px',
  },
  suggestions: {
    marginBottom: spacing[4],
    padding: spacing[4],
    backgroundColor: colors.surface.secondary,
    borderRadius: '8px',
  },
  suggestionsLabel: {
    fontSize: typography.fontSize.sm,
    color: colors.text.secondary,
    marginBottom: spacing[2],
  },
  suggestionsList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: spacing[2],
  },
  suggestionChip: {
    padding: `${spacing[1]} ${spacing[3]}`,
    backgroundColor: colors.surface.tertiary,
    border: `1px solid ${colors.border.subtle}`,
    borderRadius: '16px',
    color: colors.text.primary,
    fontSize: typography.fontSize.sm,
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  filtersBar: {
    display: 'flex',
    gap: spacing[4],
    flexWrap: 'wrap',
    marginBottom: spacing[4],
    padding: spacing[4],
    backgroundColor: colors.surface.secondary,
    borderRadius: '8px',
  },
  filterGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: spacing[2],
  },
  filterLabel: {
    fontSize: typography.fontSize.sm,
    color: colors.text.secondary,
    fontWeight: typography.fontWeight.medium,
  },
  searchTypeButtons: {
    display: 'flex',
    gap: spacing[1],
    backgroundColor: colors.surface.primary,
    padding: spacing[1],
    borderRadius: '6px',
  },
  typeButton: {
    padding: `${spacing[2]} ${spacing[4]}`,
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '4px',
    color: colors.text.secondary,
    fontSize: typography.fontSize.sm,
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  typeButtonActive: {
    backgroundColor: colors.accent.ai,
    color: colors.surface.primary,
    fontWeight: typography.fontWeight.medium,
  },
  filterDropdown: {
    minWidth: '150px',
  },
  savedSection: {
    marginBottom: spacing[4],
  },
  savedSearches: {
    marginBottom: spacing[3],
  },
  historySection: {
    marginBottom: spacing[3],
  },
  savedLabel: {
    fontSize: typography.fontSize.sm,
    color: colors.text.tertiary,
    marginBottom: spacing[2],
  },
  savedList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: spacing[2],
  },
  savedChip: {
    display: 'flex',
    alignItems: 'center',
    gap: spacing[1],
    padding: `${spacing[1]} ${spacing[3]}`,
    backgroundColor: colors.surface.secondary,
    border: `1px solid ${colors.accent.ai}`,
    borderRadius: '16px',
    color: colors.text.primary,
    fontSize: typography.fontSize.sm,
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  historyChip: {
    display: 'flex',
    alignItems: 'center',
    gap: spacing[1],
    padding: `${spacing[1]} ${spacing[3]}`,
    backgroundColor: colors.surface.secondary,
    border: `1px solid ${colors.border.subtle}`,
    borderRadius: '16px',
    color: colors.text.secondary,
    fontSize: typography.fontSize.sm,
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  savedIcon: {
    width: '14px',
    height: '14px',
  },
  error: {
    display: 'flex',
    alignItems: 'center',
    gap: spacing[2],
    padding: spacing[4],
    backgroundColor: colors.surface.secondary,
    border: `1px solid ${colors.accent.alert}`,
    borderRadius: '8px',
    color: colors.accent.alert,
    marginBottom: spacing[4],
  },
  errorIcon: {
    width: '20px',
    height: '20px',
  },
  resultsSection: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
  },
  resultsHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing[4],
    paddingBottom: spacing[3],
    borderBottom: `1px solid ${colors.border.subtle}`,
  },
  resultsInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: spacing[3],
  },
  resultsCount: {
    fontSize: typography.fontSize.base,
    fontWeight: typography.fontWeight.semibold,
    color: colors.text.primary,
  },
  executionTime: {
    fontSize: typography.fontSize.sm,
    color: colors.text.tertiary,
  },
  algorithmBadge: {
    textTransform: 'uppercase',
  },
  resultsList: {
    flex: 1,
    overflowY: 'auto',
    display: 'flex',
    flexDirection: 'column',
    gap: spacing[3],
  },
  noResults: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: spacing[12],
  },
  noResultsIcon: {
    width: '64px',
    height: '64px',
    color: colors.text.tertiary,
    marginBottom: spacing[4],
  },
  noResultsText: {
    fontSize: typography.fontSize.lg,
    fontWeight: typography.fontWeight.medium,
    color: colors.text.secondary,
    marginBottom: spacing[2],
  },
  noResultsHint: {
    fontSize: typography.fontSize.sm,
    color: colors.text.tertiary,
  },
  resultCard: {
    cursor: 'pointer',
    transition: 'all 0.2s',
    border: `1px solid ${colors.border.subtle}`,
  },
  resultCardSelected: {
    border: `1px solid ${colors.accent.ai}`,
    boxShadow: shadows.glassSubtle,
  },
  resultHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: spacing[3],
  },
  resultTitle: {
    display: 'flex',
    alignItems: 'center',
    gap: spacing[2],
    flex: 1,
  },
  resultIcon: {
    width: '20px',
    height: '20px',
    color: colors.accent.ai,
  },
  resultTitleText: {
    fontSize: typography.fontSize.base,
    fontWeight: typography.fontWeight.semibold,
    color: colors.text.primary,
    margin: 0,
  },
  resultMeta: {
    display: 'flex',
    gap: spacing[2],
  },
  scoreBadge: {
    fontWeight: typography.fontWeight.bold,
  },
  resultContent: {
    marginBottom: spacing[3],
  },
  resultText: {
    fontSize: typography.fontSize.sm,
    color: colors.text.secondary,
    lineHeight: typography.lineHeight.relaxed,
    margin: 0,
  },
  resultPath: {
    display: 'flex',
    alignItems: 'center',
    gap: spacing[1],
    marginBottom: spacing[2],
  },
  pathIcon: {
    width: '14px',
    height: '14px',
    color: colors.text.tertiary,
  },
  pathText: {
    fontSize: typography.fontSize.xs,
    color: colors.text.tertiary,
  },
  highlights: {
    display: 'flex',
    flexDirection: 'column',
    gap: spacing[2],
    paddingTop: spacing[3],
    borderTop: `1px solid ${colors.border.subtle}`,
  },
  highlightSection: {
    padding: spacing[2],
    backgroundColor: colors.surface.primary,
    borderRadius: '4px',
  },
  highlightText: {
    fontSize: typography.fontSize.sm,
    color: colors.text.secondary,
    margin: 0,
  },
}

export default SearchInterface
