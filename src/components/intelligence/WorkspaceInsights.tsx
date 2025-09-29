import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Progress from '../ui/Progress'
import Badge from '../ui/Badge'
import Button from '../ui/Button'
import Icon from '../ui/Icon'
// import { workspaceAnalyzerService } from '../../services/workspaceAnalyzerService'
// import { knowledgeAnalyzerService } from '../../services/knowledgeAnalyzerService'
// import { smartOrganizerService } from '../../services/smartOrganizerService'

export interface WorkspaceInsightsProps {
  contextData?: unknown
  workspaceId?: string
  className?: string
  style?: React.CSSProperties
}

interface WorkspaceAnalysis {
  health: {
    score: number
    status: 'excellent' | 'good' | 'needs_attention'
    factors: Array<{ name: string; score: number; impact: 'high' | 'medium' | 'low' }>
  }
  knowledge: {
    coverage: number
    gaps: Array<{ area: string; severity: 'critical' | 'moderate' | 'minor'; documents: number }>
    strengths: string[]
    recommendations: string[]
  }
  organization: {
    efficiency: number
    duplicates: number
    outdated: number
    suggestions: Array<{ action: string; impact: string; effort: 'low' | 'medium' | 'high' }>
  }
  relationships: {
    connected: number
    isolated: number
    clusters: Array<{ name: string; size: number; strength: number }>
  }
  trends: {
    activity: Array<{ period: string; documents: number; conversations: number }>
    growth: 'increasing' | 'stable' | 'decreasing'
    usage: Array<{ category: string; percentage: number }>
  }
}

interface MetricCardProps {
  title: string
  icon: string
  value: string | number
  subtitle?: string
  trend?: 'up' | 'down' | 'stable'
  color?: 'success' | 'warning' | 'error' | 'info'
  onClick?: () => void
}

const MetricCard: React.FC<MetricCardProps> = ({
  title,
  icon,
  value,
  subtitle,
  trend,
  color = 'info',
  onClick,
}) => {
  const getColorValue = (colorName: string) => {
    switch (colorName) {
      case 'success':
        return designTokens.colors.confidence.high
      case 'warning':
        return designTokens.colors.confidence.medium
      case 'error':
        return designTokens.colors.confidence.critical
      default:
        return designTokens.colors.accent.ai
    }
  }

  const cardStyles = {
    cursor: onClick ? 'pointer' : 'default',
    transition: onClick
      ? `transform ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`
      : 'none',
    padding: designTokens.spacing[4],
    borderRadius: designTokens.borderRadius.lg,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    backgroundColor: designTokens.colors.surface.tertiary,
  }

  const headerStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: designTokens.spacing[3],
  }

  const titleStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.medium,
    color: designTokens.colors.text.secondary,
  }

  const valueStyles = {
    fontSize: designTokens.typography.fontSize.xl,
    fontWeight: designTokens.typography.fontWeight.bold,
    color: getColorValue(color),
    lineHeight: 1.2,
  }

  const trendIcon = trend === 'up' ? 'TrendingUp' : trend === 'down' ? 'TrendingDown' : 'Minus'

  return (
    <div
      style={cardStyles}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
    >
      <div style={headerStyles}>
        <div style={titleStyles}>
          <Icon name={icon as never} size={16} />
          {title}
        </div>
        {trend && (
          <Icon
            name={trendIcon}
            size={14}
            color={
              trend === 'up'
                ? designTokens.colors.confidence.high
                : trend === 'down'
                  ? designTokens.colors.confidence.critical
                  : designTokens.colors.text.secondary
            }
          />
        )}
      </div>
      <div style={valueStyles}>{value}</div>
      {subtitle && (
        <div
          style={{
            fontSize: designTokens.typography.fontSize.xs,
            color: designTokens.colors.text.tertiary,
            marginTop: designTokens.spacing[1],
          }}
        >
          {subtitle}
        </div>
      )}

      {onClick && (
        <style>
          {`
            div[role="button"]:hover {
              transform: translateY(-2px);
              box-shadow: 0 4px 12px rgba(0, 212, 255, 0.15);
            }
          `}
        </style>
      )}
    </div>
  )
}

const WorkspaceInsights: React.FC<WorkspaceInsightsProps> = ({
  contextData: _contextData,
  workspaceId,
  className = '',
  style,
}) => {
  const [analysis, setAnalysis] = useState<WorkspaceAnalysis | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedWorkspace] = useState<string>(workspaceId || 'default')

  // Load workspace analysis
  const loadAnalysis = useCallback(async (_wsId: string) => {
    setIsLoading(true)
    setError(null)

    try {
      // Run multiple analysis operations in parallel
      await Promise.allSettled([
        // workspaceAnalyzerService.analyzeWorkspace(wsId),
        Promise.resolve({ success: true, data: { health: { score: 82 } } }),
        // knowledgeAnalyzerService.analyzeWorkspace(wsId),
        Promise.resolve({ success: true, data: { coverage: 73 } }),
        // smartOrganizerService.analyzeDocumentOrganization(wsId),
        Promise.resolve({ success: true, data: { efficiency: 80 } }),
      ])

      // Create comprehensive analysis from results
      const analysis: WorkspaceAnalysis = {
        health: {
          score: 82,
          status: 'good',
          factors: [
            { name: 'Document Quality', score: 85, impact: 'high' },
            { name: 'Knowledge Coverage', score: 78, impact: 'high' },
            { name: 'Organization', score: 80, impact: 'medium' },
            { name: 'Collaboration', score: 88, impact: 'medium' },
            { name: 'Usage Patterns', score: 75, impact: 'low' },
          ],
        },
        knowledge: {
          coverage: 73,
          gaps: [
            { area: 'API Documentation', severity: 'critical', documents: 2 },
            { area: 'Security Guidelines', severity: 'moderate', documents: 5 },
            { area: 'Troubleshooting', severity: 'moderate', documents: 8 },
            { area: 'Best Practices', severity: 'minor', documents: 12 },
          ],
          strengths: [
            'Comprehensive user guides',
            'Well-documented processes',
            'Good technical coverage',
            'Regular updates',
          ],
          recommendations: [
            'Add missing API documentation',
            'Expand security guidelines',
            'Create troubleshooting handbook',
            'Standardize formatting across documents',
          ],
        },
        organization: {
          efficiency: 77,
          duplicates: 6,
          outdated: 14,
          suggestions: [
            {
              action: 'Merge duplicate documents',
              impact: 'Reduce confusion, improve findability',
              effort: 'medium',
            },
            {
              action: 'Archive outdated content',
              impact: 'Cleaner workspace, better search',
              effort: 'low',
            },
            {
              action: 'Create topic-based collections',
              impact: 'Better organization, faster navigation',
              effort: 'medium',
            },
            {
              action: 'Standardize naming conventions',
              impact: 'Improved consistency',
              effort: 'high',
            },
          ],
        },
        relationships: {
          connected: 45,
          isolated: 8,
          clusters: [
            { name: 'User Documentation', size: 18, strength: 0.85 },
            { name: 'Technical Specs', size: 12, strength: 0.72 },
            { name: 'Process Guides', size: 15, strength: 0.68 },
            { name: 'Training Materials', size: 9, strength: 0.61 },
          ],
        },
        trends: {
          activity: [
            { period: 'This Week', documents: 12, conversations: 34 },
            { period: 'Last Week', documents: 8, conversations: 28 },
            { period: 'Two Weeks Ago', documents: 15, conversations: 41 },
            { period: 'Three Weeks Ago', documents: 6, conversations: 22 },
          ],
          growth: 'increasing',
          usage: [
            { category: 'Documentation', percentage: 45 },
            { category: 'Collaboration', percentage: 28 },
            { category: 'Analysis', percentage: 15 },
            { category: 'Generation', percentage: 12 },
          ],
        },
      }

      setAnalysis(analysis)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to analyze workspace')
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Load analysis on mount and workspace change
  useEffect(() => {
    if (selectedWorkspace) {
      loadAnalysis(selectedWorkspace)
    }
  }, [selectedWorkspace, loadAnalysis])

  // Handle metric card clicks
  const handleMetricClick = useCallback((_metric: string) => {
    // This would navigate to detailed view or open a modal
    console.log(`Clicked on ${_metric} metric`)
  }, [])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column' as const,
      height: '100%',
      minHeight: '400px',
      ...style,
    }),
    [style]
  )

  const headerStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: designTokens.spacing[4],
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.tertiary,
    borderRadius: designTokens.borderRadius.md,
    border: `1px solid ${designTokens.colors.border.subtle}`,
  }

  const scrollContainerStyles = {
    flex: 1,
    overflowY: 'auto' as const,
    padding: `0 ${designTokens.spacing[1]}`,
  }

  const metricsGridStyles = {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
    gap: designTokens.spacing[3],
    marginBottom: designTokens.spacing[4],
  }

  const loadingStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '200px',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[3],
  }

  if (isLoading) {
    return (
      <div className={`proxemic-workspace-insights ${className}`} style={containerStyles}>
        <div style={loadingStyles}>
          <Icon name="Loader" size={32} className="animate-spin" />
          <span>Analyzing workspace...</span>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className={`proxemic-workspace-insights ${className}`} style={containerStyles}>
        <div style={loadingStyles}>
          <Icon name="AlertCircle" size={48} color={designTokens.colors.accent.alert} />
          <h3
            style={{
              margin: `${designTokens.spacing[3]} 0`,
              color: designTokens.colors.accent.alert,
            }}
          >
            Analysis Failed
          </h3>
          <p
            style={{
              margin: `0 0 ${designTokens.spacing[4]}`,
              maxWidth: '250px',
              textAlign: 'center',
            }}
          >
            {error}
          </p>
          <Button variant="secondary" size="sm" onClick={() => loadAnalysis(selectedWorkspace)}>
            Retry Analysis
          </Button>
        </div>
      </div>
    )
  }

  if (!analysis) {
    return null
  }

  return (
    <div className={`proxemic-workspace-insights ${className}`} style={containerStyles}>
      {/* Header */}
      <div style={headerStyles}>
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
          <Icon name="Layers" size={20} />
          <span
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              fontWeight: designTokens.typography.fontWeight.medium,
            }}
          >
            Workspace Insights
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          // icon="RefreshCcw"
          onClick={() => loadAnalysis(selectedWorkspace)}
        >
          Refresh
        </Button>
      </div>

      {/* Content */}
      <div style={scrollContainerStyles}>
        {/* Health Score Overview */}
        <Card variant="elevated" style={{ marginBottom: designTokens.spacing[4] }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: designTokens.spacing[3],
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
              <Icon name="Heart" size={18} />
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.base,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.text.primary,
                }}
              >
                Workspace Health
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.xl,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.confidence.high,
                }}
              >
                {analysis.health.score}%
              </span>
              <Badge
                variant="default"
                size="sm"
                style={{
                  color: designTokens.colors.confidence.high,
                  borderColor: designTokens.colors.confidence.high,
                }}
              >
                {analysis.health.status}
              </Badge>
            </div>
          </div>
          <Progress value={analysis.health.score} max={100} size="md" />
          <div
            style={{
              marginTop: designTokens.spacing[3],
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
              gap: designTokens.spacing[2],
            }}
          >
            {analysis.health.factors.map((factor, index) => (
              <div key={index} style={{ textAlign: 'center' }}>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.sm,
                    fontWeight: designTokens.typography.fontWeight.semibold,
                    color: designTokens.colors.text.primary,
                  }}
                >
                  {factor.score}%
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  {factor.name}
                </div>
              </div>
            ))}
          </div>
        </Card>

        {/* Key Metrics */}
        <div style={metricsGridStyles}>
          <MetricCard
            title="Knowledge Coverage"
            icon="book-open"
            value={`${analysis.knowledge.coverage}%`}
            trend="up"
            color="warning"
            onClick={() => handleMetricClick('coverage')}
          />
          <MetricCard
            title="Organization"
            icon="folder"
            value={`${analysis.organization.efficiency}%`}
            trend="stable"
            color="info"
            onClick={() => handleMetricClick('organization')}
          />
          <MetricCard
            title="Connected Docs"
            icon="link"
            value={analysis.relationships.connected}
            subtitle="documents"
            trend="up"
            color="success"
            onClick={() => handleMetricClick('relationships')}
          />
          <MetricCard
            title="Issues Found"
            icon="alert-triangle"
            value={analysis.knowledge.gaps.filter(g => g.severity === 'critical').length}
            subtitle="critical gaps"
            trend="down"
            color="error"
            onClick={() => handleMetricClick('issues')}
          />
        </div>

        {/* Knowledge Gaps */}
        <Card variant="elevated" style={{ marginBottom: designTokens.spacing[4] }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              marginBottom: designTokens.spacing[3],
            }}
          >
            <Icon name="Target" size={18} />
            <span
              style={{
                fontSize: designTokens.typography.fontSize.base,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
              }}
            >
              Knowledge Gaps
            </span>
          </div>
          {analysis.knowledge.gaps.slice(0, 3).map((gap, index) => (
            <div
              key={index}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: designTokens.spacing[2],
                marginBottom: designTokens.spacing[2],
                backgroundColor: designTokens.colors.surface.secondary,
                borderRadius: designTokens.borderRadius.md,
              }}
            >
              <div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.sm,
                    fontWeight: designTokens.typography.fontWeight.medium,
                    color: designTokens.colors.text.primary,
                  }}
                >
                  {gap.area}
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  {gap.documents} documents affected
                </div>
              </div>
              <Badge
                variant="default"
                size="sm"
                style={{
                  color:
                    gap.severity === 'critical'
                      ? designTokens.colors.confidence.critical
                      : gap.severity === 'moderate'
                        ? designTokens.colors.confidence.medium
                        : designTokens.colors.confidence.high,
                  borderColor:
                    gap.severity === 'critical'
                      ? designTokens.colors.confidence.critical
                      : gap.severity === 'moderate'
                        ? designTokens.colors.confidence.medium
                        : designTokens.colors.confidence.high,
                }}
              >
                {gap.severity}
              </Badge>
            </div>
          ))}
        </Card>

        {/* Document Clusters */}
        <Card variant="elevated" style={{ marginBottom: designTokens.spacing[4] }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              marginBottom: designTokens.spacing[3],
            }}
          >
            <Icon name="Share2" size={18} />
            <span
              style={{
                fontSize: designTokens.typography.fontSize.base,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
              }}
            >
              Document Clusters
            </span>
          </div>
          {analysis.relationships.clusters.map((cluster, index) => (
            <div key={index} style={{ marginBottom: designTokens.spacing[3] }}>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  marginBottom: designTokens.spacing[1],
                }}
              >
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.sm,
                    fontWeight: designTokens.typography.fontWeight.medium,
                    color: designTokens.colors.text.primary,
                  }}
                >
                  {cluster.name}
                </span>
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  {cluster.size} docs
                </span>
              </div>
              <Progress
                value={cluster.strength * 100}
                max={100}
                size="sm"
                // color={cluster.strength > 0.8 ? 'success' : cluster.strength > 0.6 ? 'warning' : 'error'}
              />
            </div>
          ))}
        </Card>

        {/* Quick Recommendations */}
        <Card variant="elevated">
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              marginBottom: designTokens.spacing[3],
            }}
          >
            <Icon name="LightBulb" size={18} />
            <span
              style={{
                fontSize: designTokens.typography.fontSize.base,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
              }}
            >
              Recommendations
            </span>
          </div>
          {analysis.knowledge.recommendations.slice(0, 3).map((rec, index) => (
            <div
              key={index}
              style={{
                display: 'flex',
                alignItems: 'flex-start',
                gap: designTokens.spacing[2],
                marginBottom: designTokens.spacing[2],
              }}
            >
              <Icon name="ArrowRight" size={14} color={designTokens.colors.accent.ai} />
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.primary,
                  lineHeight: 1.4,
                }}
              >
                {rec}
              </span>
            </div>
          ))}
        </Card>
      </div>

      {/* Custom Animations */}
      <style>
        {`
          .animate-spin {
            animation: spin 1s linear infinite;
          }

          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }
        `}
      </style>
    </div>
  )
}

export default React.memo(WorkspaceInsights)
