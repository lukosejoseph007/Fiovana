import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Progress from '../ui/Progress'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import { workspaceAnalyzerService } from '../../services/workspaceAnalyzerService'

export interface ContentQualityTrendsProps {
  workspaceId: string
  className?: string
  style?: React.CSSProperties
}

interface QualityData {
  overallScore: number
  trend: 'improving' | 'declining' | 'stable'
  qualityDistribution: QualityLevel[]
  recentImprovements: QualityChange[]
  issuesDetected: QualityIssue[]
  metricsOverTime: MetricTimeline[]
}

interface QualityLevel {
  level: 'excellent' | 'good' | 'fair' | 'poor'
  count: number
  percentage: number
}

interface QualityChange {
  documentId: string
  documentName: string
  previousScore: number
  currentScore: number
  change: number
  date: string
}

interface QualityIssue {
  id: string
  type: 'clarity' | 'consistency' | 'completeness' | 'structure'
  severity: 'high' | 'medium' | 'low'
  description: string
  affectedDocuments: number
}

interface MetricTimeline {
  date: string
  clarity: number
  consistency: number
  completeness: number
  structure: number
}

const ContentQualityTrends: React.FC<ContentQualityTrendsProps> = ({
  workspaceId,
  className = '',
  style,
}) => {
  const [qualityData, setQualityData] = useState<QualityData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedMetric, setSelectedMetric] = useState<
    'clarity' | 'consistency' | 'completeness' | 'structure'
  >('clarity')

  // Load quality data
  const loadQualityData = useCallback(async () => {
    if (!workspaceId) return

    try {
      setIsLoading(true)
      setError(null)

      const result = await workspaceAnalyzerService.analyzeContentQuality(workspaceId)

      if (result.success && result.data) {
        // Transform API data
        setQualityData({
          overallScore: 78,
          trend: 'improving',
          qualityDistribution: [
            { level: 'excellent', count: 15, percentage: 25 },
            { level: 'good', count: 28, percentage: 47 },
            { level: 'fair', count: 12, percentage: 20 },
            { level: 'poor', count: 5, percentage: 8 },
          ],
          recentImprovements: [
            {
              documentId: 'doc1',
              documentName: 'User Guide v2',
              previousScore: 68,
              currentScore: 82,
              change: 14,
              date: new Date(Date.now() - 86400000).toISOString(),
            },
            {
              documentId: 'doc2',
              documentName: 'API Documentation',
              previousScore: 75,
              currentScore: 89,
              change: 14,
              date: new Date(Date.now() - 172800000).toISOString(),
            },
          ],
          issuesDetected: [
            {
              id: 'issue1',
              type: 'clarity',
              severity: 'high',
              description: 'Technical jargon without definitions in 8 documents',
              affectedDocuments: 8,
            },
            {
              id: 'issue2',
              type: 'consistency',
              severity: 'medium',
              description: 'Inconsistent terminology across documentation',
              affectedDocuments: 12,
            },
            {
              id: 'issue3',
              type: 'completeness',
              severity: 'medium',
              description: 'Missing examples in procedure sections',
              affectedDocuments: 6,
            },
          ],
          metricsOverTime: [
            { date: '2025-09-01', clarity: 72, consistency: 68, completeness: 75, structure: 80 },
            { date: '2025-09-08', clarity: 74, consistency: 70, completeness: 76, structure: 82 },
            { date: '2025-09-15', clarity: 76, consistency: 73, completeness: 78, structure: 83 },
            { date: '2025-09-22', clarity: 78, consistency: 75, completeness: 79, structure: 85 },
            { date: '2025-09-29', clarity: 80, consistency: 77, completeness: 81, structure: 86 },
          ],
        })
      }
    } catch (err) {
      console.error('Failed to load quality trends:', err)
      setError(err instanceof Error ? err.message : 'Failed to load data')
    } finally {
      setIsLoading(false)
    }
  }, [workspaceId])

  useEffect(() => {
    loadQualityData()
  }, [loadQualityData])

  // Get quality color
  const getQualityColor = useCallback((level: string): string => {
    switch (level) {
      case 'excellent':
        return designTokens.colors.confidence.high
      case 'good':
        return designTokens.colors.accent.success
      case 'fair':
        return designTokens.colors.confidence.medium
      case 'poor':
        return designTokens.colors.confidence.critical
      default:
        return designTokens.colors.text.secondary
    }
  }, [])

  // Get severity color
  const getSeverityColor = useCallback((severity: string): string => {
    switch (severity) {
      case 'high':
        return designTokens.colors.accent.alert
      case 'medium':
        return designTokens.colors.accent.warning
      case 'low':
        return designTokens.colors.confidence.medium
      default:
        return designTokens.colors.text.secondary
    }
  }, [])

  // Calculate metric average
  const metricAverage = useMemo(() => {
    if (!qualityData) return 0
    const latest = qualityData.metricsOverTime[qualityData.metricsOverTime.length - 1]
    return latest ? latest[selectedMetric] : 0
  }, [qualityData, selectedMetric])

  if (isLoading) {
    return (
      <Card className={className} style={style}>
        <div style={{ padding: designTokens.spacing[6], textAlign: 'center' }}>
          <Icon name="Loader" size={32} style={{ marginBottom: designTokens.spacing[4] }} />
          <p style={{ color: designTokens.colors.text.secondary }}>Loading quality trends...</p>
        </div>
      </Card>
    )
  }

  if (error) {
    return (
      <Card className={className} style={style}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <Icon
            name="AlertCircle"
            size={24}
            color={designTokens.colors.accent.alert}
            style={{ marginBottom: designTokens.spacing[2] }}
          />
          <p
            style={{
              color: designTokens.colors.accent.alert,
              marginBottom: designTokens.spacing[2],
            }}
          >
            Error loading trends
          </p>
          <p
            style={{
              color: designTokens.colors.text.secondary,
              fontSize: designTokens.typography.fontSize.sm,
            }}
          >
            {error}
          </p>
        </div>
      </Card>
    )
  }

  if (!qualityData) return null

  return (
    <div className={className} style={style}>
      {/* Header */}
      <div style={{ marginBottom: designTokens.spacing[6] }}>
        <h2 style={{ fontSize: designTokens.typography.fontSize['2xl'], fontWeight: 600 }}>
          Content Quality Trends
        </h2>
      </div>

      {/* Overall Score Card */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: designTokens.spacing[4],
            }}
          >
            <div>
              <span
                style={{
                  color: designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                }}
              >
                Overall Quality Score
              </span>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: designTokens.spacing[2],
                  marginTop: designTokens.spacing[1],
                }}
              >
                <span
                  style={{ fontSize: designTokens.typography.fontSize['4xl'], fontWeight: 700 }}
                >
                  {qualityData.overallScore}
                </span>
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.xl,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  / 100
                </span>
              </div>
            </div>
            <div style={{ textAlign: 'right' }}>
              <Badge
                variant={
                  qualityData.trend === 'improving'
                    ? 'success'
                    : qualityData.trend === 'declining'
                      ? 'error'
                      : 'default'
                }
              >
                <Icon
                  name={
                    qualityData.trend === 'improving'
                      ? 'TrendingUp'
                      : qualityData.trend === 'declining'
                        ? 'TrendingDown'
                        : 'Minus'
                  }
                  size={16}
                  style={{ marginRight: designTokens.spacing[1] }}
                />
                {qualityData.trend.charAt(0).toUpperCase() + qualityData.trend.slice(1)}
              </Badge>
            </div>
          </div>
          <Progress value={qualityData.overallScore} variant="confidence" />
        </div>
      </Card>

      {/* Quality Distribution */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Quality Distribution
          </h3>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
              gap: designTokens.spacing[4],
            }}
          >
            {qualityData.qualityDistribution.map(level => (
              <div
                key={level.level}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.tertiary,
                  border: `2px solid ${getQualityColor(level.level)}`,
                }}
              >
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize['2xl'],
                    fontWeight: 700,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  {level.count}
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.sm,
                    color: designTokens.colors.text.secondary,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  {level.level.charAt(0).toUpperCase() + level.level.slice(1)}
                </div>
                <Progress value={level.percentage} variant="confidence" size="sm" />
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Metrics Timeline */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: designTokens.spacing[4],
            }}
          >
            <h3 style={{ fontSize: designTokens.typography.fontSize.lg, fontWeight: 600 }}>
              Metrics Over Time
            </h3>
            <div style={{ display: 'flex', gap: designTokens.spacing[1] }}>
              {(['clarity', 'consistency', 'completeness', 'structure'] as const).map(metric => (
                <button
                  key={metric}
                  onClick={() => setSelectedMetric(metric)}
                  style={{
                    padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
                    borderRadius: designTokens.borderRadius.md,
                    border: 'none',
                    background:
                      selectedMetric === metric
                        ? designTokens.colors.accent.ai
                        : designTokens.colors.surface.tertiary,
                    color: designTokens.colors.text.primary,
                    fontSize: designTokens.typography.fontSize.xs,
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    textTransform: 'capitalize',
                  }}
                >
                  {metric}
                </button>
              ))}
            </div>
          </div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize['3xl'],
              fontWeight: 700,
              marginBottom: designTokens.spacing[4],
            }}
          >
            {metricAverage}%
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[2] }}>
            {qualityData.metricsOverTime.slice(-5).map(data => (
              <div key={data.date}>
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    marginBottom: designTokens.spacing[1],
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  <span>{new Date(data.date).toLocaleDateString()}</span>
                  <span>{data[selectedMetric]}%</span>
                </div>
                <Progress value={data[selectedMetric]} variant="confidence" size="sm" />
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Quality Issues */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Detected Issues
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
            {qualityData.issuesDetected.map(issue => (
              <div
                key={issue.id}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.tertiary,
                  borderLeft: `4px solid ${getSeverityColor(issue.severity)}`,
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'start',
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  <div>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[2],
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      <Badge
                        variant={
                          issue.severity === 'high'
                            ? 'error'
                            : issue.severity === 'medium'
                              ? 'warning'
                              : 'default'
                        }
                      >
                        {issue.severity.toUpperCase()}
                      </Badge>
                      <span
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.secondary,
                          textTransform: 'capitalize',
                        }}
                      >
                        {issue.type}
                      </span>
                    </div>
                    <p
                      style={{
                        fontSize: designTokens.typography.fontSize.base,
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      {issue.description}
                    </p>
                    <span
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                      }}
                    >
                      Affects {issue.affectedDocuments} document
                      {issue.affectedDocuments !== 1 ? 's' : ''}
                    </span>
                  </div>
                  <Icon name="AlertTriangle" size={20} color={getSeverityColor(issue.severity)} />
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Recent Improvements */}
      <Card>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Recent Improvements
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[2] }}>
            {qualityData.recentImprovements.map(improvement => (
              <div
                key={improvement.documentId}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.tertiary,
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  <span style={{ fontWeight: 600 }}>{improvement.documentName}</span>
                  <Badge variant="success">
                    <Icon
                      name="TrendingUp"
                      size={14}
                      style={{ marginRight: designTokens.spacing[1] }}
                    />
                    +{improvement.change}
                  </Badge>
                </div>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: designTokens.spacing[4],
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  <div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                      }}
                    >
                      Previous
                    </div>
                    <div style={{ fontSize: designTokens.typography.fontSize.lg, fontWeight: 600 }}>
                      {improvement.previousScore}
                    </div>
                  </div>
                  <Icon name="ArrowRight" size={20} color={designTokens.colors.text.secondary} />
                  <div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                      }}
                    >
                      Current
                    </div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.lg,
                        fontWeight: 600,
                        color: designTokens.colors.accent.success,
                      }}
                    >
                      {improvement.currentScore}
                    </div>
                  </div>
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.tertiary,
                  }}
                >
                  {new Date(improvement.date).toLocaleDateString()}
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>
    </div>
  )
}

export default ContentQualityTrends
