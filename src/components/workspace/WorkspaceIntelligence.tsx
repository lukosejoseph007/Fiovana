import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Button from '../ui/Button'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import Progress from '../ui/Progress'
import Tooltip from '../ui/Tooltip'
import { CardSkeleton } from '../ui/LoadingStates'
import { workspaceAnalyzerService } from '../../services/workspaceAnalyzerService'
import type { WorkspaceAnalysis } from '../../types'

export interface WorkspaceIntelligenceProps {
  workspaceId: string
  className?: string
  style?: React.CSSProperties
  onRefresh?: () => void
  onActionClick?: (action: string, data: unknown) => void
}

interface KnowledgeGap {
  id: string
  type: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  description: string
  impact: string
  recommendations: string[]
}

interface DocumentRelationship {
  sourceId: string
  targetId: string
  type: string
  strength: number
  description: string
}

interface UtilizationMetric {
  category: string
  value: number
  trend: 'up' | 'down' | 'stable'
  change: number
}

const WorkspaceIntelligence: React.FC<WorkspaceIntelligenceProps> = ({
  workspaceId,
  className = '',
  style,
  onRefresh,
  onActionClick,
}) => {
  const [analysis, setAnalysis] = useState<WorkspaceAnalysis | null>(null)
  const [knowledgeGaps, setKnowledgeGaps] = useState<KnowledgeGap[]>([])
  const [relationships, setRelationships] = useState<DocumentRelationship[]>([])
  const [utilization, setUtilization] = useState<UtilizationMetric[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'gaps' | 'relationships' | 'trends'>(
    'overview'
  )
  const [refreshing, setRefreshing] = useState(false)

  // Load workspace intelligence data
  const loadWorkspaceData = useCallback(async () => {
    if (!workspaceId) return

    try {
      setIsLoading(true)
      setError(null)

      // Load comprehensive workspace analysis
      const analysisResult = await workspaceAnalyzerService.analyzeWorkspace(workspaceId)

      if (analysisResult.success && analysisResult.data) {
        setAnalysis(analysisResult.data)
      }

      // Load knowledge gaps
      const gapsResult = await workspaceAnalyzerService.analyzeKnowledgeGaps(workspaceId)
      if (gapsResult.success && gapsResult.data) {
        // Handle both array and object responses
        const gapsData = Array.isArray(gapsResult.data) ? gapsResult.data : []
        const gaps = gapsData.map((gap: unknown, index) => ({
          id: `gap-${index}`,
          type: 'content',
          severity: 'medium' as const,
          description: typeof gap === 'string' ? gap : 'Unknown gap',
          impact: 'Moderate impact on documentation completeness',
          recommendations: ['Review and address this gap'],
        }))
        setKnowledgeGaps(gaps)
      }

      // Mock relationship data (would come from relationship analyzer)
      setRelationships([
        {
          sourceId: 'doc1',
          targetId: 'doc2',
          type: 'references',
          strength: 0.85,
          description: 'Strong content reference',
        },
        {
          sourceId: 'doc2',
          targetId: 'doc3',
          type: 'similar',
          strength: 0.72,
          description: 'Similar topic coverage',
        },
      ])

      // Mock utilization metrics (would come from lifecycle manager)
      setUtilization([
        { category: 'Documents', value: 85, trend: 'up', change: 12 },
        { category: 'Storage', value: 62, trend: 'stable', change: 0 },
        { category: 'Activity', value: 78, trend: 'up', change: 8 },
        { category: 'Collaboration', value: 54, trend: 'down', change: -5 },
      ])
    } catch (err) {
      console.error('Failed to load workspace intelligence:', err)
      setError(err instanceof Error ? err.message : 'Failed to load workspace data')
    } finally {
      setIsLoading(false)
    }
  }, [workspaceId])

  // Initial load
  useEffect(() => {
    loadWorkspaceData()
  }, [loadWorkspaceData])

  // Handle refresh
  const handleRefresh = useCallback(async () => {
    setRefreshing(true)
    await loadWorkspaceData()
    setRefreshing(false)
    onRefresh?.()
  }, [loadWorkspaceData, onRefresh])

  // Get health status color
  const getHealthColor = useCallback((score: number) => {
    if (score >= 90) return designTokens.colors.confidence.high
    if (score >= 70) return designTokens.colors.confidence.medium
    if (score >= 50) return designTokens.colors.confidence.low
    return designTokens.colors.confidence.critical
  }, [])

  // Get severity color
  const getSeverityColor = useCallback((severity: string) => {
    switch (severity) {
      case 'critical':
        return designTokens.colors.accent.alert
      case 'high':
        return designTokens.colors.accent.warning
      case 'medium':
        return designTokens.colors.accent.ai
      case 'low':
        return designTokens.colors.text.secondary
      default:
        return designTokens.colors.text.tertiary
    }
  }, [])

  // Get trend icon
  const getTrendIcon = useCallback((trend: 'up' | 'down' | 'stable') => {
    switch (trend) {
      case 'up':
        return 'TrendingUp'
      case 'down':
        return 'TrendingDown'
      default:
        return 'Minus'
    }
  }, [])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column' as const,
      gap: designTokens.spacing[4],
      backgroundColor: designTokens.colors.surface.primary,
      minHeight: '100vh',
      width: '100%',
      ...style,
    }),
    [style]
  )

  const headerStyles = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: designTokens.spacing[4],
  }

  const tabsStyles = {
    display: 'flex',
    gap: designTokens.spacing[2],
    marginBottom: designTokens.spacing[4],
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    paddingBottom: designTokens.spacing[2],
  }

  const metricsGridStyles = {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    gap: designTokens.spacing[4],
  }

  const issueListStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[2],
    maxHeight: '400px',
    overflowY: 'auto' as const,
  }

  if (isLoading && !analysis) {
    return (
      <div className={`fiovana-workspace-intelligence ${className}`} style={containerStyles}>
        <CardSkeleton count={3} />
      </div>
    )
  }

  if (error && !analysis) {
    return (
      <div className={`fiovana-workspace-intelligence ${className}`} style={containerStyles}>
        <Card>
          <div
            style={{
              padding: designTokens.spacing[8],
              textAlign: 'center',
            }}
          >
            <Icon
              name="AlertCircle"
              size={48}
              color={designTokens.colors.accent.alert}
              style={{ marginBottom: designTokens.spacing[4] }}
            />
            <div
              style={{
                fontSize: designTokens.typography.fontSize.lg,
                fontWeight: designTokens.typography.fontWeight.medium,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[2],
              }}
            >
              Failed to Load Intelligence Data
            </div>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              {error}
            </div>
            <Button variant="primary" onClick={handleRefresh}>
              Retry
            </Button>
          </div>
        </Card>
      </div>
    )
  }

  return (
    <div className={`fiovana-workspace-intelligence ${className}`} style={containerStyles}>
      {/* Header */}
      <div style={headerStyles}>
        <div>
          <h2
            style={{
              fontSize: designTokens.typography.fontSize['2xl'],
              fontWeight: designTokens.typography.fontWeight.bold,
              color: designTokens.colors.text.primary,
              marginBottom: designTokens.spacing[1],
            }}
          >
            Workspace Intelligence
          </h2>
          <p
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
            }}
          >
            Comprehensive analytics and insights for your workspace
          </p>
        </div>
        <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onActionClick?.('close', null)}
            leftIcon={<Icon name="ChevronDown" size={16} style={{ transform: 'rotate(90deg)' }} />}
          >
            Back to Documents
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefresh}
            disabled={refreshing}
            leftIcon={<Icon name="RefreshCcw" size={16} />}
          >
            {refreshing ? 'Refreshing...' : 'Refresh'}
          </Button>
        </div>
      </div>

      {/* Health Score Card */}
      {analysis?.health && (
        <Card variant="elevated" hoverable>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[6],
              padding: designTokens.spacing[6],
            }}
          >
            <div
              style={{
                position: 'relative',
                width: '120px',
                height: '120px',
              }}
            >
              {/* Circular progress */}
              <svg width="120" height="120" style={{ transform: 'rotate(-90deg)' }}>
                <circle
                  cx="60"
                  cy="60"
                  r="54"
                  stroke={designTokens.colors.surface.tertiary}
                  strokeWidth="8"
                  fill="none"
                />
                <circle
                  cx="60"
                  cy="60"
                  r="54"
                  stroke={getHealthColor(analysis.health.score)}
                  strokeWidth="8"
                  fill="none"
                  strokeDasharray={`${(analysis.health.score / 100) * 339.292} 339.292`}
                  strokeLinecap="round"
                  style={{
                    transition: `stroke-dasharray ${designTokens.animation.duration.slow} ${designTokens.animation.easing.easeOut}`,
                  }}
                />
              </svg>
              <div
                style={{
                  position: 'absolute',
                  top: '50%',
                  left: '50%',
                  transform: 'translate(-50%, -50%)',
                  textAlign: 'center',
                }}
              >
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize['3xl'],
                    fontWeight: designTokens.typography.fontWeight.bold,
                    color: getHealthColor(analysis.health.score),
                  }}
                >
                  {analysis.health.score}
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.tertiary,
                  }}
                >
                  HEALTH
                </div>
              </div>
            </div>

            <div style={{ flex: 1 }}>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: designTokens.spacing[2],
                  marginBottom: designTokens.spacing[3],
                }}
              >
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xl,
                    fontWeight: designTokens.typography.fontWeight.bold,
                    color: designTokens.colors.text.primary,
                  }}
                >
                  Workspace Health
                </div>
                <Badge
                  variant={
                    analysis.health.status === 'excellent'
                      ? 'success'
                      : analysis.health.status === 'good'
                        ? 'default'
                        : 'warning'
                  }
                >
                  {analysis.health.status.toUpperCase()}
                </Badge>
              </div>

              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.secondary,
                  marginBottom: designTokens.spacing[3],
                }}
              >
                {analysis.health.issues.length === 0
                  ? 'No issues detected. Your workspace is in excellent condition!'
                  : `${analysis.health.issues.length} issue${analysis.health.issues.length > 1 ? 's' : ''} detected that need attention.`}
              </div>

              {analysis.health.recommendations.length > 0 && (
                <div style={{ display: 'flex', gap: designTokens.spacing[2], flexWrap: 'wrap' }}>
                  {analysis.health.recommendations.slice(0, 3).map((rec, idx) => (
                    <Badge key={idx} variant="default" size="sm">
                      {rec}
                    </Badge>
                  ))}
                </div>
              )}
            </div>

            <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
              <Button
                variant="primary"
                size="sm"
                onClick={() => onActionClick?.('view_details', analysis.health)}
              >
                View Details
              </Button>
            </div>
          </div>
        </Card>
      )}

      {/* Tabs */}
      <div style={tabsStyles}>
        {(['overview', 'gaps', 'relationships', 'trends'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: `${designTokens.spacing[2]} ${designTokens.spacing[4]}`,
              backgroundColor: 'transparent',
              border: 'none',
              borderBottom:
                activeTab === tab
                  ? `2px solid ${designTokens.colors.accent.ai}`
                  : '2px solid transparent',
              color:
                activeTab === tab
                  ? designTokens.colors.accent.ai
                  : designTokens.colors.text.secondary,
              fontSize: designTokens.typography.fontSize.sm,
              fontWeight:
                activeTab === tab
                  ? designTokens.typography.fontWeight.semibold
                  : designTokens.typography.fontWeight.medium,
              cursor: 'pointer',
              transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
            }}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <>
          {/* Metrics Grid */}
          <div style={metricsGridStyles}>
            {analysis?.overview && (
              <>
                <Card hoverable>
                  <div
                    style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}
                  >
                    <Icon name="FileText" size={40} color={designTokens.colors.accent.ai} />
                    <div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize['2xl'],
                          fontWeight: designTokens.typography.fontWeight.bold,
                          color: designTokens.colors.text.primary,
                        }}
                      >
                        {analysis.overview.totalFiles}
                      </div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.secondary,
                        }}
                      >
                        Total Documents
                      </div>
                    </div>
                  </div>
                </Card>

                <Card hoverable>
                  <div
                    style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}
                  >
                    <Icon name="Folder" size={40} color={designTokens.colors.accent.semantic} />
                    <div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize['2xl'],
                          fontWeight: designTokens.typography.fontWeight.bold,
                          color: designTokens.colors.text.primary,
                        }}
                      >
                        {analysis.overview.activeProjects}
                      </div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.secondary,
                        }}
                      >
                        Active Projects
                      </div>
                    </div>
                  </div>
                </Card>

                <Card hoverable>
                  <div
                    style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}
                  >
                    <Icon name="Cpu" size={40} color={designTokens.colors.confidence.high} />
                    <div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize['2xl'],
                          fontWeight: designTokens.typography.fontWeight.bold,
                          color: designTokens.colors.text.primary,
                        }}
                      >
                        {Object.keys(analysis.overview.documentTypes).length}
                      </div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.secondary,
                        }}
                      >
                        Document Types
                      </div>
                    </div>
                  </div>
                </Card>

                <Card hoverable>
                  <div
                    style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}
                  >
                    <Icon name="Zap" size={40} color={designTokens.colors.accent.warning} />
                    <div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize['2xl'],
                          fontWeight: designTokens.typography.fontWeight.bold,
                          color: designTokens.colors.text.primary,
                        }}
                      >
                        {analysis.insights.length}
                      </div>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.secondary,
                        }}
                      >
                        Active Insights
                      </div>
                    </div>
                  </div>
                </Card>
              </>
            )}
          </div>

          {/* Issues List */}
          {analysis?.health?.issues &&
            Array.isArray(analysis.health.issues) &&
            analysis.health.issues.length > 0 && (
              <Card>
                <div style={{ padding: designTokens.spacing[4] }}>
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.lg,
                      fontWeight: designTokens.typography.fontWeight.semibold,
                      color: designTokens.colors.text.primary,
                      marginBottom: designTokens.spacing[4],
                    }}
                  >
                    Health Issues
                  </div>
                  <div style={issueListStyles}>
                    {analysis.health.issues.map(issue => (
                      <Card key={issue.id} variant="glass">
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'flex-start',
                            gap: designTokens.spacing[3],
                          }}
                        >
                          <Icon
                            name="AlertCircle"
                            size={20}
                            color={getSeverityColor(issue.severity)}
                          />
                          <div style={{ flex: 1 }}>
                            <div
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: designTokens.spacing[2],
                                marginBottom: designTokens.spacing[1],
                              }}
                            >
                              <div
                                style={{
                                  fontSize: designTokens.typography.fontSize.sm,
                                  fontWeight: designTokens.typography.fontWeight.medium,
                                  color: designTokens.colors.text.primary,
                                }}
                              >
                                {issue.description}
                              </div>
                              <Badge
                                variant={
                                  issue.severity === 'critical' || issue.severity === 'high'
                                    ? 'error'
                                    : 'warning'
                                }
                                size="sm"
                              >
                                {issue.severity}
                              </Badge>
                            </div>
                            {issue.resolution && (
                              <div
                                style={{
                                  fontSize: designTokens.typography.fontSize.xs,
                                  color: designTokens.colors.text.tertiary,
                                }}
                              >
                                Resolution: {issue.resolution}
                              </div>
                            )}
                          </div>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onActionClick?.('fix_issue', issue)}
                          >
                            Fix
                          </Button>
                        </div>
                      </Card>
                    ))}
                  </div>
                </div>
              </Card>
            )}

          {/* Insights */}
          {analysis?.insights &&
            Array.isArray(analysis.insights) &&
            analysis.insights.length > 0 && (
              <Card>
                <div style={{ padding: designTokens.spacing[4] }}>
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.lg,
                      fontWeight: designTokens.typography.fontWeight.semibold,
                      color: designTokens.colors.text.primary,
                      marginBottom: designTokens.spacing[4],
                    }}
                  >
                    Workspace Insights
                  </div>
                  <div style={issueListStyles}>
                    {analysis.insights.map(insight => (
                      <Card key={insight.id} variant="glass" hoverable>
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'flex-start',
                            gap: designTokens.spacing[3],
                          }}
                        >
                          <Icon name="LightBulb" size={20} color={designTokens.colors.accent.ai} />
                          <div style={{ flex: 1 }}>
                            <div
                              style={{
                                fontSize: designTokens.typography.fontSize.sm,
                                fontWeight: designTokens.typography.fontWeight.medium,
                                color: designTokens.colors.text.primary,
                                marginBottom: designTokens.spacing[1],
                              }}
                            >
                              {insight.title}
                            </div>
                            <div
                              style={{
                                fontSize: designTokens.typography.fontSize.xs,
                                color: designTokens.colors.text.secondary,
                                marginBottom: designTokens.spacing[2],
                              }}
                            >
                              {insight.description}
                            </div>
                            <div
                              style={{
                                display: 'flex',
                                gap: designTokens.spacing[2],
                                alignItems: 'center',
                              }}
                            >
                              <Progress value={insight.confidence * 100} size="sm" variant="ai" />
                              <span
                                style={{
                                  fontSize: designTokens.typography.fontSize.xs,
                                  color: designTokens.colors.text.tertiary,
                                }}
                              >
                                {Math.round(insight.confidence * 100)}% confidence
                              </span>
                            </div>
                          </div>
                          {insight.actionable && insight.suggestedActions && (
                            <Tooltip content="View suggested actions">
                              <Button
                                variant="secondary"
                                size="sm"
                                onClick={() => onActionClick?.('view_suggestions', insight)}
                              >
                                Actions
                              </Button>
                            </Tooltip>
                          )}
                        </div>
                      </Card>
                    ))}
                  </div>
                </div>
              </Card>
            )}
        </>
      )}

      {/* Knowledge Gaps Tab */}
      {activeTab === 'gaps' && (
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.lg,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Knowledge Gap Analysis
            </div>
            {knowledgeGaps.length === 0 ? (
              <div
                style={{
                  textAlign: 'center',
                  padding: designTokens.spacing[8],
                  color: designTokens.colors.text.secondary,
                }}
              >
                <Icon
                  name="Health"
                  size={48}
                  color={designTokens.colors.confidence.high}
                  style={{ marginBottom: designTokens.spacing[4] }}
                />
                <div>No knowledge gaps detected. Your documentation is comprehensive!</div>
              </div>
            ) : (
              <div style={issueListStyles}>
                {knowledgeGaps.map(gap => (
                  <Card key={gap.id} variant="glass">
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'flex-start',
                        gap: designTokens.spacing[3],
                      }}
                    >
                      <Icon name="AlertTriangle" size={20} color={getSeverityColor(gap.severity)} />
                      <div style={{ flex: 1 }}>
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: designTokens.spacing[2],
                            marginBottom: designTokens.spacing[1],
                          }}
                        >
                          <div
                            style={{
                              fontSize: designTokens.typography.fontSize.sm,
                              fontWeight: designTokens.typography.fontWeight.medium,
                              color: designTokens.colors.text.primary,
                            }}
                          >
                            {gap.description}
                          </div>
                          <Badge
                            variant={
                              gap.severity === 'high' || gap.severity === 'critical'
                                ? 'error'
                                : 'warning'
                            }
                            size="sm"
                          >
                            {gap.severity}
                          </Badge>
                        </div>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.tertiary,
                            marginBottom: designTokens.spacing[2],
                          }}
                        >
                          Impact: {gap.impact}
                        </div>
                        {gap.recommendations.length > 0 && (
                          <div
                            style={{
                              fontSize: designTokens.typography.fontSize.xs,
                              color: designTokens.colors.text.secondary,
                            }}
                          >
                            <strong>Recommendations:</strong> {gap.recommendations.join(', ')}
                          </div>
                        )}
                      </div>
                      <Button
                        variant="primary"
                        size="sm"
                        onClick={() => onActionClick?.('address_gap', gap)}
                      >
                        Address
                      </Button>
                    </div>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Relationships Tab */}
      {activeTab === 'relationships' && (
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.lg,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Document Relationship Mapping
            </div>
            {relationships.length === 0 ? (
              <div
                style={{
                  textAlign: 'center',
                  padding: designTokens.spacing[8],
                  color: designTokens.colors.text.secondary,
                }}
              >
                <Icon
                  name="Link"
                  size={48}
                  color={designTokens.colors.text.tertiary}
                  style={{ marginBottom: designTokens.spacing[4] }}
                />
                <div>No document relationships detected yet.</div>
              </div>
            ) : (
              <div style={issueListStyles}>
                {relationships.map((rel, idx) => (
                  <Card key={idx} variant="glass" hoverable>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[3],
                      }}
                    >
                      <Icon name="Link" size={20} color={designTokens.colors.accent.semantic} />
                      <div style={{ flex: 1 }}>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.sm,
                            fontWeight: designTokens.typography.fontWeight.medium,
                            color: designTokens.colors.text.primary,
                            marginBottom: designTokens.spacing[1],
                          }}
                        >
                          {rel.sourceId} → {rel.targetId}
                        </div>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.secondary,
                            marginBottom: designTokens.spacing[1],
                          }}
                        >
                          Type: {rel.type} • {rel.description}
                        </div>
                        <Progress value={rel.strength * 100} size="sm" variant="confidence" />
                      </div>
                      <Badge variant="default" size="sm">
                        {Math.round(rel.strength * 100)}% strength
                      </Badge>
                    </div>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Trends Tab */}
      {activeTab === 'trends' && (
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.lg,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Utilization Metrics & Trends
            </div>
            <div style={issueListStyles}>
              {utilization.map((metric, idx) => (
                <Card key={idx} variant="glass" hoverable>
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: designTokens.spacing[4],
                    }}
                  >
                    <div style={{ flex: 1 }}>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          fontWeight: designTokens.typography.fontWeight.medium,
                          color: designTokens.colors.text.primary,
                          marginBottom: designTokens.spacing[2],
                        }}
                      >
                        {metric.category}
                      </div>
                      <Progress value={metric.value} size="sm" variant="health" showPercentage />
                    </div>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[2],
                      }}
                    >
                      <Icon
                        name={getTrendIcon(metric.trend) as never}
                        size={20}
                        color={
                          metric.trend === 'up'
                            ? designTokens.colors.confidence.high
                            : metric.trend === 'down'
                              ? designTokens.colors.accent.alert
                              : designTokens.colors.text.tertiary
                        }
                      />
                      <span
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color:
                            metric.trend === 'up'
                              ? designTokens.colors.confidence.high
                              : metric.trend === 'down'
                                ? designTokens.colors.accent.alert
                                : designTokens.colors.text.tertiary,
                          fontWeight: designTokens.typography.fontWeight.medium,
                        }}
                      >
                        {metric.change > 0 ? '+' : ''}
                        {metric.change}%
                      </span>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          </div>
        </Card>
      )}

      {/* Organization Suggestions */}
      {analysis?.organizationSuggestions &&
        Array.isArray(analysis.organizationSuggestions) &&
        analysis.organizationSuggestions.length > 0 && (
          <Card>
            <div style={{ padding: designTokens.spacing[4] }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.lg,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.text.primary,
                  marginBottom: designTokens.spacing[4],
                }}
              >
                Organization Recommendations
              </div>
              <div style={issueListStyles}>
                {analysis.organizationSuggestions.map(suggestion => (
                  <Card key={suggestion.id} variant="glass" hoverable>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'flex-start',
                        gap: designTokens.spacing[3],
                      }}
                    >
                      <Icon name="Target" size={20} color={designTokens.colors.accent.semantic} />
                      <div style={{ flex: 1 }}>
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: designTokens.spacing[2],
                            marginBottom: designTokens.spacing[1],
                          }}
                        >
                          <div
                            style={{
                              fontSize: designTokens.typography.fontSize.sm,
                              fontWeight: designTokens.typography.fontWeight.medium,
                              color: designTokens.colors.text.primary,
                            }}
                          >
                            {suggestion.description}
                          </div>
                          <Badge
                            variant={
                              suggestion.priority === 'high'
                                ? 'error'
                                : suggestion.priority === 'medium'
                                  ? 'warning'
                                  : 'default'
                            }
                            size="sm"
                          >
                            {suggestion.priority}
                          </Badge>
                        </div>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.secondary,
                            marginBottom: designTokens.spacing[2],
                          }}
                        >
                          {suggestion.implementation}
                        </div>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.tertiary,
                          }}
                        >
                          Estimated Impact: {suggestion.estimatedImpact}%
                        </div>
                      </div>
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => onActionClick?.('apply_suggestion', suggestion)}
                      >
                        Apply
                      </Button>
                    </div>
                  </Card>
                ))}
              </div>
            </div>
          </Card>
        )}
    </div>
  )
}

export default React.memo(WorkspaceIntelligence)
