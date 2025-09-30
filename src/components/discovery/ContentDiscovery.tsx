import React, { useState, useEffect, useCallback, useMemo } from 'react'
import Card from '../ui/Card'
import Button from '../ui/Button'
import Badge from '../ui/Badge'
import {
  smartOrganizerService,
  relationshipService,
  knowledgeAnalyzerService,
} from '../../services'
import { colors, spacing, typography } from '../../styles/tokens'
import type { KnowledgeGap, DocumentRelationship, OrganizationSuggestion } from '../../types'

interface ContentSuggestion {
  type: 'similar' | 'related' | 'gap' | 'collection'
  title: string
  description: string
  items: Array<{
    id: string
    path: string
    title: string
    metadata?: string
    score?: number
  }>
  priority: 'high' | 'medium' | 'low'
}

interface ContentDiscoveryProps {
  currentDocumentId?: string
  workspaceId: string
}

export const ContentDiscovery: React.FC<ContentDiscoveryProps> = ({
  currentDocumentId,
  workspaceId,
}) => {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [suggestions, setSuggestions] = useState<ContentSuggestion[]>([])
  const [selectedType, setSelectedType] = useState<
    'all' | 'similar' | 'related' | 'gap' | 'collection'
  >('all')
  const [expandedSuggestions, setExpandedSuggestions] = useState<Set<number>>(new Set())

  const loadContentSuggestions = useCallback(async () => {
    setLoading(true)
    setError(null)

    try {
      const suggestionsList: ContentSuggestion[] = []

      // Load similar documents if a current document is selected
      if (currentDocumentId) {
        try {
          // Use relationship service to find similar documents
          const similarResponse = await relationshipService.findSimilarDocuments(
            currentDocumentId,
            0.7,
            5
          )
          const similar = similarResponse.data || []

          if (similar && similar.length > 0) {
            suggestionsList.push({
              type: 'similar',
              title: 'Similar Documents',
              description: 'Documents with similar content and structure',
              items: similar.map((rel: DocumentRelationship) => ({
                id: rel.targetDocumentId || rel.id || '',
                path: '',
                title: `Document ${rel.targetDocumentId}`,
                metadata: rel.type?.subcategory || 'similar',
                score: rel.strength || 0,
              })),
              priority: 'high',
            })
          }
        } catch (err) {
          console.warn('Failed to load similar documents:', err)
        }

        // Load related content based on relationships
        try {
          const relationshipsResponse =
            await relationshipService.findDocumentDependencies(currentDocumentId)
          const relationships = relationshipsResponse.data || []

          if (relationships && relationships.length > 0) {
            suggestionsList.push({
              type: 'related',
              title: 'Related Content',
              description: 'Content connected through references and concepts',
              items: relationships.map((rel: DocumentRelationship) => ({
                id: rel.targetDocumentId || rel.id || '',
                path: '',
                title: `Document ${rel.targetDocumentId}`,
                metadata: `${rel.type?.subcategory || 'related'} (${Math.round((rel.strength || 0) * 100)}%)`,
                score: rel.strength || 0,
              })),
              priority: 'medium',
            })
          }
        } catch (err) {
          console.warn('Failed to load related content:', err)
        }
      }

      // Load knowledge gaps
      try {
        const gapsResponse = await knowledgeAnalyzerService.analyzeKnowledgeGaps(workspaceId)
        const gapsAnalysis = gapsResponse.data
        const gaps = gapsAnalysis?.gaps || []

        if (gaps && gaps.length > 0) {
          const gapSuggestions = gaps
            .filter((gap: KnowledgeGap) => gap.severity !== 'low')
            .slice(0, 3)
            .map((gap: KnowledgeGap) => ({
              id: gap.id || `gap-${Date.now()}`,
              path: '',
              title: `${gap.type} gap`,
              metadata: gap.description,
              score: gap.severity === 'high' || gap.severity === 'critical' ? 1.0 : 0.7,
            }))

          if (gapSuggestions.length > 0) {
            suggestionsList.push({
              type: 'gap',
              title: 'Content Gaps Identified',
              description: 'Missing or incomplete content areas',
              items: gapSuggestions,
              priority: 'high',
            })
          }
        }
      } catch (err) {
        console.warn('Failed to load knowledge gaps:', err)
      }

      // Load smart collections
      try {
        const collectionsResponse =
          await smartOrganizerService.generateOrganizationSuggestions(workspaceId)
        const collections = collectionsResponse.data || []

        if (collections && collections.length > 0) {
          suggestionsList.push({
            type: 'collection',
            title: 'Suggested Collections',
            description: 'Auto-generated document groupings',
            items: collections.slice(0, 3).map((suggestion: OrganizationSuggestion) => ({
              id: `collection-${suggestion.id}`,
              path: '',
              title: suggestion.type || 'Organization Suggestion',
              metadata: suggestion.description || 'Organization improvement',
              score: 0.8,
            })),
            priority: 'low',
          })
        }
      } catch (err) {
        console.warn('Failed to load smart collections:', err)
      }

      setSuggestions(suggestionsList)
    } catch (err) {
      console.error('Error loading content suggestions:', err)
      setError('Failed to load content suggestions')
    } finally {
      setLoading(false)
    }
  }, [currentDocumentId, workspaceId])

  useEffect(() => {
    loadContentSuggestions()
  }, [loadContentSuggestions])

  const filteredSuggestions = useMemo(() => {
    if (selectedType === 'all') {
      return suggestions
    }
    return suggestions.filter(s => s.type === selectedType)
  }, [suggestions, selectedType])

  const toggleExpanded = useCallback((index: number) => {
    setExpandedSuggestions(prev => {
      const next = new Set(prev)
      if (next.has(index)) {
        next.delete(index)
      } else {
        next.add(index)
      }
      return next
    })
  }, [])

  const handleOpenDocument = useCallback((path: string) => {
    // This would integrate with the document viewer
    console.log('Opening document:', path)
    // In a real implementation, this would use a router or state management
    // to navigate to the document
  }, [])

  const handleCreateCollection = useCallback((collectionName: string) => {
    console.log('Creating collection:', collectionName)
    // Integration with collection management
  }, [])

  const handleAddressGap = useCallback((gapId: string) => {
    console.log('Addressing gap:', gapId)
    // Integration with content generation or AI assistant
  }, [])

  const getPriorityColor = (priority: string): 'red' | 'yellow' | 'blue' => {
    switch (priority) {
      case 'high':
        return 'red'
      case 'medium':
        return 'yellow'
      default:
        return 'blue'
    }
  }

  const getTypeIcon = (type: string): string => {
    switch (type) {
      case 'similar':
        return '📄'
      case 'related':
        return '🔗'
      case 'gap':
        return '⚠️'
      case 'collection':
        return '📁'
      default:
        return '📌'
    }
  }

  if (loading) {
    return (
      <div className="p-6 space-y-4">
        <div className="animate-pulse space-y-4">
          {[1, 2, 3].map(i => (
            <Card key={i} className="p-4">
              <div className="h-4 bg-gray-700 rounded w-1/3 mb-3"></div>
              <div className="h-3 bg-gray-700 rounded w-2/3 mb-4"></div>
              <div className="space-y-2">
                {[1, 2].map(j => (
                  <div key={j} className="h-10 bg-gray-700 rounded"></div>
                ))}
              </div>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="p-6">
        <Card className="p-4 border-red-500/20 bg-red-500/5">
          <div className="flex items-start space-x-3">
            <span className="text-2xl">⚠️</span>
            <div className="flex-1">
              <h3 className="text-red-400 font-medium mb-1">Error Loading Suggestions</h3>
              <p className="text-sm text-gray-400">{error}</p>
              <Button variant="ghost" onClick={loadContentSuggestions} className="mt-3">
                Try Again
              </Button>
            </div>
          </div>
        </Card>
      </div>
    )
  }

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>Content Discovery</h2>
        <p style={styles.subtitle}>AI-powered recommendations and insights</p>
      </div>

      {/* Filter Tabs */}
      <div style={styles.filterTabs}>
        {['all', 'similar', 'related', 'gap', 'collection'].map(type => (
          <button
            key={type}
            onClick={() => setSelectedType(type as typeof selectedType)}
            className={`
              px-3 py-1.5 rounded-lg text-sm font-medium transition-colors whitespace-nowrap
              ${
                selectedType === type
                  ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30'
                  : 'text-gray-400 hover:text-gray-300 hover:bg-gray-800/50'
              }
            `}
          >
            {type === 'all' ? 'All' : type.charAt(0).toUpperCase() + type.slice(1)}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={styles.content}>
        {filteredSuggestions.length === 0 ? (
          <Card className="p-8 text-center">
            <div className="text-4xl mb-3">🔍</div>
            <h3 className="text-lg font-medium text-gray-300 mb-2">No Suggestions Available</h3>
            <p className="text-sm text-gray-500">
              {selectedType === 'all'
                ? 'Work with documents to receive AI-powered recommendations'
                : `No ${selectedType} suggestions at this time`}
            </p>
          </Card>
        ) : (
          filteredSuggestions.map((suggestion, index) => {
            const isExpanded = expandedSuggestions.has(index)
            const displayItems = isExpanded ? suggestion.items : suggestion.items.slice(0, 3)

            return (
              <Card key={index} className="overflow-hidden">
                {/* Suggestion Header */}
                <div className="p-4 border-b border-gray-800">
                  <div className="flex items-start justify-between">
                    <div className="flex items-start space-x-3 flex-1">
                      <span className="text-2xl">{getTypeIcon(suggestion.type)}</span>
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-1">
                          <h3 className="text-base font-semibold text-white">{suggestion.title}</h3>
                          <Badge color={getPriorityColor(suggestion.priority)} size="sm">
                            {suggestion.priority}
                          </Badge>
                        </div>
                        <p className="text-sm text-gray-400">{suggestion.description}</p>
                      </div>
                    </div>
                    <div className="text-xs text-gray-500 ml-2">
                      {suggestion.items.length} item{suggestion.items.length !== 1 ? 's' : ''}
                    </div>
                  </div>
                </div>

                {/* Suggestion Items */}
                <div className="divide-y divide-gray-800">
                  {displayItems.map(item => (
                    <div
                      key={item.id}
                      className="p-4 hover:bg-gray-800/30 transition-colors cursor-pointer group"
                      onClick={() => item.path && handleOpenDocument(item.path)}
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center space-x-2 mb-1">
                            <h4 className="text-sm font-medium text-gray-200 truncate group-hover:text-cyan-400 transition-colors">
                              {item.title}
                            </h4>
                            {item.score !== undefined && (
                              <span className="text-xs text-gray-500">
                                {Math.round(item.score * 100)}%
                              </span>
                            )}
                          </div>
                          {item.metadata && (
                            <p className="text-xs text-gray-500 truncate">{item.metadata}</p>
                          )}
                          {item.path && (
                            <p className="text-xs text-gray-600 truncate mt-1">{item.path}</p>
                          )}
                        </div>

                        {/* Action Buttons */}
                        <div className="ml-3 opacity-0 group-hover:opacity-100 transition-opacity flex items-center space-x-2">
                          {suggestion.type === 'gap' && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={e => {
                                e.stopPropagation()
                                handleAddressGap(item.id)
                              }}
                              className="text-xs"
                            >
                              Address
                            </Button>
                          )}
                          {suggestion.type === 'collection' && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={e => {
                                e.stopPropagation()
                                handleCreateCollection(item.title)
                              }}
                              className="text-xs"
                            >
                              Create
                            </Button>
                          )}
                          {(suggestion.type === 'similar' || suggestion.type === 'related') &&
                            item.path && (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={e => {
                                  e.stopPropagation()
                                  handleOpenDocument(item.path)
                                }}
                                className="text-xs"
                              >
                                Open
                              </Button>
                            )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Show More/Less Button */}
                {suggestion.items.length > 3 && (
                  <div className="p-3 border-t border-gray-800 text-center">
                    <button
                      onClick={() => toggleExpanded(index)}
                      className="text-sm text-cyan-400 hover:text-cyan-300 transition-colors"
                    >
                      {isExpanded ? 'Show Less' : `Show ${suggestion.items.length - 3} More`}
                    </button>
                  </div>
                )}
              </Card>
            )
          })
        )}
      </div>

      {/* Footer Actions */}
      {suggestions.length > 0 && (
        <div style={styles.footer}>
          <p style={styles.footerText}>
            {suggestions.reduce((acc, s) => acc + s.items.length, 0)} total suggestions
          </p>
          <Button variant="ghost" size="sm" onClick={loadContentSuggestions}>
            Refresh
          </Button>
        </div>
      )}
    </div>
  )
}

// Styles matching SearchInterface
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: colors.surface.primary,
    overflow: 'hidden',
  },
  header: {
    padding: spacing[6],
    borderBottom: `1px solid ${colors.border.subtle}`,
  },
  title: {
    fontSize: typography.fontSize['2xl'],
    fontWeight: typography.fontWeight.semibold,
    color: colors.text.primary,
    margin: 0,
    marginBottom: spacing[1],
  },
  subtitle: {
    fontSize: typography.fontSize.sm,
    color: colors.text.secondary,
    margin: 0,
  },
  filterTabs: {
    display: 'flex',
    gap: spacing[2],
    padding: `${spacing[3]} ${spacing[6]}`,
    borderBottom: `1px solid ${colors.border.subtle}`,
    overflowX: 'auto',
    backgroundColor: colors.surface.primary,
  },
  content: {
    flex: 1,
    overflowY: 'auto',
    padding: spacing[6],
    backgroundColor: colors.surface.primary,
  },
  footer: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: spacing[4],
    borderTop: `1px solid ${colors.border.subtle}`,
    backgroundColor: colors.surface.primary,
  },
  footerText: {
    fontSize: typography.fontSize.xs,
    color: colors.text.tertiary,
  },
}
