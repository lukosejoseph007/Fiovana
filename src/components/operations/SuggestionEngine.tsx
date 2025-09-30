import React, { useState, useCallback, useEffect, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Button from '../ui/Button'
import Icon from '../ui/Icon'
import Badge from '../ui/Badge'
import { workspaceAnalyzerService } from '../../services/workspaceAnalyzerService'
import { workspaceAiService } from '../../services/workspaceAiService'
import { smartOrganizerService } from '../../services/smartOrganizerService'
import { documentService } from '../../services/documentService'

export interface SuggestionEngineProps {
  className?: string
  style?: React.CSSProperties
  workspaceId: string
  documentId?: string
  onSuggestionAccept?: (suggestion: Suggestion) => void
  onSuggestionDismiss?: (suggestionId: string) => void
  maxSuggestions?: number
}

export interface Suggestion {
  id: string
  type: SuggestionType
  priority: SuggestionPriority
  title: string
  description: string
  actionLabel: string
  icon: string
  metadata?: Record<string, unknown>
  dismissible: boolean
  confidence: number
  estimatedTime?: string
  benefitScore?: number
  category: SuggestionCategory
}

export type SuggestionType =
  | 'analyze'
  | 'organize'
  | 'update'
  | 'generate'
  | 'compare'
  | 'review'
  | 'optimize'
  | 'cleanup'

export type SuggestionPriority = 'critical' | 'high' | 'medium' | 'low'

export type SuggestionCategory = 'quality' | 'organization' | 'productivity' | 'health' | 'learning'

const SuggestionEngine: React.FC<SuggestionEngineProps> = ({
  className = '',
  style,
  workspaceId,
  documentId,
  onSuggestionAccept,
  onSuggestionDismiss,
  maxSuggestions = 3,
}) => {
  const [suggestions, setSuggestions] = useState<Suggestion[]>([])
  const [loading, setLoading] = useState(false)
  const [dismissedSuggestionIds, setDismissedSuggestionIds] = useState<Set<string>>(new Set())

  // Generate suggestions based on workspace state
  const generateSuggestions = useCallback(async () => {
    if (!workspaceId) return

    setLoading(true)

    try {
      const allSuggestions: Suggestion[] = []

      // 1. Analyze workspace health
      const analysisResult = await workspaceAnalyzerService.analyzeWorkspace(workspaceId)
      if (analysisResult.success && analysisResult.data) {
        const healthScore = analysisResult.data.health?.score || 0

        // Suggest analysis if health score is low
        if (healthScore < 0.6) {
          allSuggestions.push({
            id: `health-analysis-${Date.now()}`,
            type: 'analyze',
            priority: 'high',
            title: 'Workspace Health Needs Attention',
            description: `Your workspace health score is ${Math.round(healthScore * 100)}%. Run a comprehensive analysis to identify issues.`,
            actionLabel: 'Analyze Workspace',
            icon: 'Health',
            dismissible: true,
            confidence: 0.9,
            estimatedTime: '2-3 min',
            benefitScore: 85,
            category: 'health',
            metadata: { healthScore },
          })
        }

        // Check for health issues (using insights as proxy for gaps)
        const healthIssues = analysisResult.data.health?.issues || []
        if (healthIssues.length > 0) {
          allSuggestions.push({
            id: `content-gaps-${Date.now()}`,
            type: 'generate',
            priority: 'medium',
            title: 'Address Workspace Issues',
            description: `Found ${healthIssues.length} workspace issues. Review and address them to improve health.`,
            actionLabel: 'Review Issues',
            icon: 'FileText',
            dismissible: true,
            confidence: 0.85,
            estimatedTime: '5-10 min',
            benefitScore: 75,
            category: 'quality',
            metadata: { issueCount: healthIssues.length },
          })
        }

        // Check for organization suggestions
        const orgSuggestions = analysisResult.data.organizationSuggestions || []
        if (orgSuggestions.length > 0) {
          allSuggestions.push({
            id: `org-suggestions-${Date.now()}`,
            type: 'organize',
            priority: 'medium',
            title: 'Improve Organization',
            description: `${orgSuggestions.length} organization improvements available.`,
            actionLabel: 'Review & Apply',
            icon: 'Folder',
            dismissible: true,
            confidence: 0.8,
            estimatedTime: '10-15 min',
            benefitScore: 70,
            category: 'organization',
            metadata: { suggestionCount: orgSuggestions.length },
          })
        }
      }

      // 2. Get AI-powered recommendations
      const aiRecommendations = await workspaceAiService.recommendContent(workspaceId)
      if (aiRecommendations.success && aiRecommendations.data) {
        const recommendations = (
          Array.isArray(aiRecommendations.data) ? aiRecommendations.data : []
        ) as Array<{
          type?: string
          title?: string
          description?: string
          confidence?: number
        }>

        recommendations.slice(0, 2).forEach((rec, idx) => {
          allSuggestions.push({
            id: `ai-recommendation-${idx}-${Date.now()}`,
            type: (rec.type as SuggestionType) || 'optimize',
            priority: 'medium',
            title: rec.title || 'AI Recommendation',
            description: rec.description || 'AI-suggested improvement for your workspace',
            actionLabel: 'Apply Suggestion',
            icon: 'Sparkles',
            dismissible: true,
            confidence: rec.confidence || 0.75,
            estimatedTime: '3-5 min',
            benefitScore: 65,
            category: 'learning',
            metadata: { source: 'ai' },
          })
        })
      }

      // 3. Check for organization opportunities
      const orgSuggestions =
        await smartOrganizerService.generateOrganizationSuggestions(workspaceId)
      if (orgSuggestions.success && orgSuggestions.data && orgSuggestions.data.length > 0) {
        const topOrgSuggestion = orgSuggestions.data[0] as {
          id?: string
          title?: string
          description?: string
          impact_score?: number
        }
        allSuggestions.push({
          id: `organization-${topOrgSuggestion.id || Date.now()}`,
          type: 'organize',
          priority: 'low',
          title: topOrgSuggestion.title || 'Improve Organization',
          description:
            topOrgSuggestion.description || 'Reorganize your workspace for better efficiency',
          actionLabel: 'Organize Now',
          icon: 'FolderTree',
          dismissible: true,
          confidence: 0.7,
          estimatedTime: '5-7 min',
          benefitScore: topOrgSuggestion.impact_score || 60,
          category: 'organization',
        })
      }

      // 4. Document-specific suggestions
      if (documentId) {
        const docResult = await documentService.getDocument(documentId)
        if (docResult.success && docResult.data) {
          const doc = docResult.data

          // Suggest analysis if not analyzed
          const isAnalyzed = doc.metadata?.customFields?.analyzed as boolean | undefined
          if (!isAnalyzed) {
            allSuggestions.push({
              id: `doc-analysis-${documentId}`,
              type: 'analyze',
              priority: 'high',
              title: 'Analyze This Document',
              description: 'Run AI analysis to extract structure, concepts, and insights',
              actionLabel: 'Analyze Document',
              icon: 'Search',
              dismissible: false,
              confidence: 0.95,
              estimatedTime: '30-45 sec',
              benefitScore: 90,
              category: 'quality',
              metadata: { documentId },
            })
          }

          // Suggest comparison if multiple docs exist
          const allDocsResult = await documentService.listDocuments(workspaceId)
          if (
            allDocsResult.success &&
            allDocsResult.data &&
            allDocsResult.data.length > 1 &&
            isAnalyzed
          ) {
            allSuggestions.push({
              id: `doc-comparison-${documentId}`,
              type: 'compare',
              priority: 'medium',
              title: 'Compare with Similar Documents',
              description: 'Find similarities and differences with related documents',
              actionLabel: 'Compare Documents',
              icon: 'GitCompare',
              dismissible: true,
              confidence: 0.8,
              estimatedTime: '1-2 min',
              benefitScore: 70,
              category: 'quality',
              metadata: { documentId },
            })
          }
        }
      }

      // 5. Analyze historical action patterns
      const productivityResult =
        await workspaceAnalyzerService.analyzeProductivityPatterns(workspaceId)
      if (productivityResult.success && productivityResult.data) {
        const patterns = productivityResult.data as Record<string, unknown>

        // Extract common actions
        if (patterns.common_actions) {
          const commonActions = patterns.common_actions as Array<{
            action?: string
            frequency?: number
          }>
          const actionCounts: Record<string, number> = {}

          commonActions.forEach(action => {
            if (action.action && action.frequency) {
              actionCounts[action.action] = action.frequency
            }
          })

          // Suggest based on patterns
          const mostCommon = Object.entries(actionCounts).sort((a, b) => b[1] - a[1])[0]
          if (mostCommon && mostCommon[1] > 5) {
            allSuggestions.push({
              id: `pattern-based-${Date.now()}`,
              type: 'optimize',
              priority: 'low',
              title: 'Automate Frequent Action',
              description: `You frequently perform "${mostCommon[0]}". Consider creating an automation rule.`,
              actionLabel: 'Create Automation',
              icon: 'Zap',
              dismissible: true,
              confidence: 0.75,
              estimatedTime: '2-3 min',
              benefitScore: 65,
              category: 'productivity',
              metadata: { action: mostCommon[0], frequency: mostCommon[1] },
            })
          }
        }
      }

      // Sort suggestions by priority and benefit score
      const priorityOrder: Record<SuggestionPriority, number> = {
        critical: 4,
        high: 3,
        medium: 2,
        low: 1,
      }

      const sortedSuggestions = allSuggestions
        .filter(s => !dismissedSuggestionIds.has(s.id))
        .sort((a, b) => {
          // First by priority
          const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority]
          if (priorityDiff !== 0) return priorityDiff

          // Then by benefit score
          return (b.benefitScore || 0) - (a.benefitScore || 0)
        })
        .slice(0, maxSuggestions)

      setSuggestions(sortedSuggestions)
    } catch (error) {
      console.error('Failed to generate suggestions:', error)
    } finally {
      setLoading(false)
    }
  }, [workspaceId, documentId, dismissedSuggestionIds, maxSuggestions])

  // Refresh suggestions periodically
  useEffect(() => {
    generateSuggestions()

    const interval = setInterval(
      () => {
        generateSuggestions()
      },
      5 * 60 * 1000
    ) // Refresh every 5 minutes

    return () => clearInterval(interval)
  }, [generateSuggestions])

  // Handle suggestion acceptance
  const handleAccept = useCallback(
    (suggestion: Suggestion) => {
      onSuggestionAccept?.(suggestion)

      // Remove accepted suggestion
      setSuggestions(prev => prev.filter(s => s.id !== suggestion.id))
    },
    [onSuggestionAccept]
  )

  // Handle suggestion dismissal
  const handleDismiss = useCallback(
    (suggestionId: string) => {
      setDismissedSuggestionIds(prev => new Set([...prev, suggestionId]))
      setSuggestions(prev => prev.filter(s => s.id !== suggestionId))
      onSuggestionDismiss?.(suggestionId)
    },
    [onSuggestionDismiss]
  )

  // Get priority color
  const getPriorityColor = useCallback((priority: SuggestionPriority): string => {
    switch (priority) {
      case 'critical':
        return designTokens.colors.accent.alert
      case 'high':
        return designTokens.colors.accent.warning
      case 'medium':
        return designTokens.colors.accent.ai
      case 'low':
        return designTokens.colors.text.tertiary
    }
  }, [])

  // Get category color
  const getCategoryColor = useCallback((category: SuggestionCategory): string => {
    switch (category) {
      case 'quality':
        return designTokens.colors.confidence.high
      case 'organization':
        return designTokens.colors.accent.ai
      case 'productivity':
        return designTokens.colors.confidence.medium
      case 'health':
        return designTokens.colors.accent.alert
      case 'learning':
        return designTokens.colors.accent.info
    }
  }, [])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column' as const,
      gap: designTokens.spacing[3],
      ...style,
    }),
    [style]
  )

  const headerStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: designTokens.spacing[2],
  }

  const titleStyles = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
  }

  const suggestionCardStyles = {
    padding: designTokens.spacing[3],
    display: 'flex',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[2],
  }

  const suggestionHeaderStyles = {
    display: 'flex',
    alignItems: 'flex-start',
    justifyContent: 'space-between',
    gap: designTokens.spacing[2],
  }

  const suggestionContentStyles = {
    display: 'flex',
    gap: designTokens.spacing[3],
    flex: 1,
  }

  const suggestionTextStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[1],
    flex: 1,
  }

  const suggestionTitleStyles = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.medium,
    color: designTokens.colors.text.primary,
  }

  const suggestionDescriptionStyles = {
    fontSize: designTokens.typography.fontSize.xs,
    color: designTokens.colors.text.secondary,
    lineHeight: designTokens.typography.lineHeight.relaxed,
  }

  const suggestionMetaStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    flexWrap: 'wrap' as const,
  }

  const suggestionActionsStyles = {
    display: 'flex',
    gap: designTokens.spacing[2],
    marginTop: designTokens.spacing[1],
  }

  const emptyStateStyles = {
    textAlign: 'center' as const,
    padding: designTokens.spacing[6],
    color: designTokens.colors.text.tertiary,
    fontSize: designTokens.typography.fontSize.sm,
  }

  if (loading && suggestions.length === 0) {
    return (
      <div className={`proxemic-suggestion-engine ${className}`} style={containerStyles}>
        <div style={emptyStateStyles}>
          <Icon name="Loader" size={24} style={{ marginBottom: designTokens.spacing[2] }} />
          <div>Analyzing workspace for suggestions...</div>
        </div>
      </div>
    )
  }

  if (suggestions.length === 0) {
    return (
      <div className={`proxemic-suggestion-engine ${className}`} style={containerStyles}>
        <div style={emptyStateStyles}>
          <Icon name="Health" size={24} style={{ marginBottom: designTokens.spacing[2] }} />
          <div>All caught up! No suggestions at the moment.</div>
        </div>
      </div>
    )
  }

  return (
    <div className={`proxemic-suggestion-engine ${className}`} style={containerStyles}>
      {/* Header */}
      <div style={headerStyles}>
        <div style={titleStyles}>
          <Icon name="LightBulb" size={16} />
          <span>Smart Suggestions</span>
          {suggestions.length > 0 && (
            <Badge variant="default" size="sm">
              {suggestions.length}
            </Badge>
          )}
        </div>
        {loading && <Icon name="Loader" size={14} />}
      </div>

      {/* Suggestions List */}
      {suggestions.map(suggestion => (
        <Card key={suggestion.id} style={suggestionCardStyles} variant="default">
          <div style={suggestionHeaderStyles}>
            <div style={suggestionContentStyles}>
              <Icon
                name={suggestion.icon as never}
                size={20}
                color={getPriorityColor(suggestion.priority)}
              />
              <div style={suggestionTextStyles}>
                <div style={suggestionTitleStyles}>{suggestion.title}</div>
                <div style={suggestionDescriptionStyles}>{suggestion.description}</div>

                {/* Metadata */}
                <div style={suggestionMetaStyles}>
                  <Badge
                    variant="default"
                    size="sm"
                    style={{
                      backgroundColor: `${getCategoryColor(suggestion.category)}20`,
                      color: getCategoryColor(suggestion.category),
                      borderColor: getCategoryColor(suggestion.category),
                    }}
                  >
                    {suggestion.category}
                  </Badge>
                  {suggestion.estimatedTime && (
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[1],
                      }}
                    >
                      <Icon name="AlertCircle" size={12} />
                      {suggestion.estimatedTime}
                    </div>
                  )}
                  {suggestion.confidence && (
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[1],
                      }}
                    >
                      <Icon name="Target" size={12} />
                      {Math.round(suggestion.confidence * 100)}% confident
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Dismiss button */}
            {suggestion.dismissible && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleDismiss(suggestion.id)}
                style={{ minWidth: 'auto', padding: designTokens.spacing[1] }}
              >
                <Icon name="X" size={14} />
              </Button>
            )}
          </div>

          {/* Actions */}
          <div style={suggestionActionsStyles}>
            <Button
              variant="primary"
              size="sm"
              onClick={() => handleAccept(suggestion)}
              leftIcon={<Icon name={suggestion.icon as never} size={14} />}
            >
              {suggestion.actionLabel}
            </Button>
            {suggestion.dismissible && (
              <Button variant="ghost" size="sm" onClick={() => handleDismiss(suggestion.id)}>
                Not Now
              </Button>
            )}
          </div>
        </Card>
      ))}
    </div>
  )
}

export default React.memo(SuggestionEngine)
