import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Progress from '../ui/Progress'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import Tooltip from '../ui/Tooltip'
import { CardSkeleton } from '../ui/LoadingStates'
// import { workspaceAnalyzerService } from '../../services/workspaceAnalyzerService'

export interface AIOperationEffectivenessProps {
  workspaceId: string
  className?: string
  style?: React.CSSProperties
}

interface AIEffectivenessData {
  totalOperations: number
  successRate: number
  averageConfidence: number
  operationBreakdown: OperationStats[]
  recentOperations: RecentOperation[]
  performanceMetrics: PerformanceMetric[]
  providerStats: ProviderPerformance[]
}

interface OperationStats {
  type: string
  count: number
  successRate: number
  avgConfidence: number
  avgDuration: number
}

interface RecentOperation {
  id: string
  type: string
  timestamp: string
  duration: number
  success: boolean
  confidence: number
  provider: string
  result: string
}

interface PerformanceMetric {
  metric: string
  value: number
  trend: 'up' | 'down' | 'stable'
  change: number
}

interface ProviderPerformance {
  provider: string
  operations: number
  successRate: number
  avgLatency: number
  avgConfidence: number
}

const AIOperationEffectiveness: React.FC<AIOperationEffectivenessProps> = ({
  workspaceId,
  className = '',
  style,
}) => {
  const [effectivenessData, setEffectivenessData] = useState<AIEffectivenessData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedType, setSelectedType] = useState<string>('all')

  // Load effectiveness data
  const loadEffectivenessData = useCallback(async () => {
    if (!workspaceId) return

    try {
      setIsLoading(true)
      setError(null)

      // Mock AI operation effectiveness data
      setEffectivenessData({
        totalOperations: 1247,
        successRate: 94.2,
        averageConfidence: 82.5,
        operationBreakdown: [
          {
            type: 'Document Analysis',
            count: 342,
            successRate: 96.5,
            avgConfidence: 87.2,
            avgDuration: 2.3,
          },
          {
            type: 'Content Generation',
            count: 278,
            successRate: 91.8,
            avgConfidence: 79.4,
            avgDuration: 4.7,
          },
          {
            type: 'Style Transfer',
            count: 189,
            successRate: 93.1,
            avgConfidence: 81.3,
            avgDuration: 3.2,
          },
          {
            type: 'Comparison',
            count: 165,
            successRate: 97.2,
            avgConfidence: 88.6,
            avgDuration: 1.8,
          },
          {
            type: 'Classification',
            count: 142,
            successRate: 95.8,
            avgConfidence: 85.1,
            avgDuration: 1.2,
          },
          {
            type: 'Summarization',
            count: 131,
            successRate: 92.4,
            avgConfidence: 78.9,
            avgDuration: 2.1,
          },
        ],
        recentOperations: [
          {
            id: 'op1',
            type: 'Document Analysis',
            timestamp: new Date(Date.now() - 300000).toISOString(),
            duration: 2.1,
            success: true,
            confidence: 92,
            provider: 'Ollama',
            result: 'Successfully analyzed document structure',
          },
          {
            id: 'op2',
            type: 'Content Generation',
            timestamp: new Date(Date.now() - 600000).toISOString(),
            duration: 5.3,
            success: true,
            confidence: 85,
            provider: 'OpenRouter',
            result: 'Generated instructor guide',
          },
          {
            id: 'op3',
            type: 'Style Transfer',
            timestamp: new Date(Date.now() - 900000).toISOString(),
            duration: 3.8,
            success: true,
            confidence: 78,
            provider: 'Anthropic',
            result: 'Applied organizational style',
          },
          {
            id: 'op4',
            type: 'Comparison',
            timestamp: new Date(Date.now() - 1200000).toISOString(),
            duration: 1.6,
            success: true,
            confidence: 94,
            provider: 'Ollama',
            result: 'Compared 2 documents',
          },
        ],
        performanceMetrics: [
          { metric: 'Success Rate', value: 94.2, trend: 'up', change: 2.1 },
          { metric: 'Avg Confidence', value: 82.5, trend: 'up', change: 1.8 },
          { metric: 'Avg Duration', value: 2.8, trend: 'down', change: -0.3 },
          { metric: 'Operations/Day', value: 187, trend: 'up', change: 15 },
        ],
        providerStats: [
          {
            provider: 'Ollama',
            operations: 623,
            successRate: 95.8,
            avgLatency: 1.8,
            avgConfidence: 84.2,
          },
          {
            provider: 'OpenRouter',
            operations: 412,
            successRate: 92.1,
            avgLatency: 3.2,
            avgConfidence: 81.5,
          },
          {
            provider: 'Anthropic',
            operations: 212,
            successRate: 94.8,
            avgLatency: 2.4,
            avgConfidence: 86.7,
          },
        ],
      })
    } catch (err) {
      console.error('Failed to load effectiveness data:', err)
      setError(err instanceof Error ? err.message : 'Failed to load data')
    } finally {
      setIsLoading(false)
    }
  }, [workspaceId])

  useEffect(() => {
    loadEffectivenessData()
  }, [loadEffectivenessData])

  // Filter operations by type
  const filteredOperations = useMemo(() => {
    if (!effectivenessData) return []
    if (selectedType === 'all') return effectivenessData.recentOperations
    return effectivenessData.recentOperations.filter(op => op.type === selectedType)
  }, [effectivenessData, selectedType])

  // Get confidence color
  const getConfidenceColor = useCallback((confidence: number): string => {
    if (confidence >= 85) return designTokens.colors.confidence.high
    if (confidence >= 70) return designTokens.colors.confidence.medium
    if (confidence >= 55) return designTokens.colors.confidence.low
    return designTokens.colors.confidence.critical
  }, [])

  // Format duration
  const formatDuration = useCallback((seconds: number): string => {
    return `${seconds.toFixed(1)}s`
  }, [])

  // Format time ago
  const formatTimeAgo = useCallback((timestamp: string): string => {
    const seconds = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000)
    if (seconds < 60) return `${seconds}s ago`
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
    return `${Math.floor(seconds / 86400)}d ago`
  }, [])

  if (isLoading) {
    return <CardSkeleton style={style} className={className} />
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
            Error loading effectiveness data
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

  if (!effectivenessData) return null

  return (
    <div className={className} style={style}>
      {/* Header */}
      <div style={{ marginBottom: designTokens.spacing[6] }}>
        <h2 style={{ fontSize: designTokens.typography.fontSize['2xl'], fontWeight: 600 }}>
          AI Operation Effectiveness
        </h2>
      </div>

      {/* Key Metrics */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))',
          gap: designTokens.spacing[4],
          marginBottom: designTokens.spacing[6],
        }}
      >
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                marginBottom: designTokens.spacing[2],
              }}
            >
              <span
                style={{
                  color: designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                }}
              >
                Total Operations
              </span>
              <Icon name="Zap" size={20} color={designTokens.colors.accent.ai} />
            </div>
            <div style={{ fontSize: designTokens.typography.fontSize['3xl'], fontWeight: 700 }}>
              {effectivenessData.totalOperations.toLocaleString()}
            </div>
          </div>
        </Card>

        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                marginBottom: designTokens.spacing[2],
              }}
            >
              <span
                style={{
                  color: designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                }}
              >
                Success Rate
              </span>
              <Icon name="Heart" size={20} color={designTokens.colors.accent.success} />
            </div>
            <div style={{ fontSize: designTokens.typography.fontSize['3xl'], fontWeight: 700 }}>
              {effectivenessData.successRate.toFixed(1)}%
            </div>
            <Progress
              value={effectivenessData.successRate}
              variant="confidence"
              style={{ marginTop: designTokens.spacing[2] }}
            />
          </div>
        </Card>

        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                marginBottom: designTokens.spacing[2],
              }}
            >
              <span
                style={{
                  color: designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                }}
              >
                Avg Confidence
              </span>
              <Icon name="Target" size={20} color={designTokens.colors.accent.semantic} />
            </div>
            <div style={{ fontSize: designTokens.typography.fontSize['3xl'], fontWeight: 700 }}>
              {effectivenessData.averageConfidence.toFixed(1)}%
            </div>
            <Progress
              value={effectivenessData.averageConfidence}
              variant="confidence"
              style={{ marginTop: designTokens.spacing[2] }}
            />
          </div>
        </Card>
      </div>

      {/* Performance Metrics */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Performance Trends
          </h3>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))',
              gap: designTokens.spacing[4],
            }}
          >
            {effectivenessData.performanceMetrics.map(metric => (
              <div
                key={metric.metric}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.tertiary,
                }}
              >
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  {metric.metric}
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize['2xl'],
                    fontWeight: 700,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  {metric.metric.includes('Duration') ? formatDuration(metric.value) : metric.value}
                </div>
                <Badge
                  variant={
                    metric.trend === 'up'
                      ? metric.metric.includes('Duration')
                        ? 'error'
                        : 'success'
                      : metric.trend === 'down'
                        ? metric.metric.includes('Duration')
                          ? 'success'
                          : 'error'
                        : 'default'
                  }
                >
                  <Icon
                    name={
                      metric.trend === 'up'
                        ? 'TrendingUp'
                        : metric.trend === 'down'
                          ? 'TrendingDown'
                          : 'Minus'
                    }
                    size={12}
                    style={{ marginRight: designTokens.spacing[1] }}
                  />
                  {metric.change > 0 ? '+' : ''}
                  {metric.change}
                  {metric.metric.includes('Duration')
                    ? 's'
                    : metric.metric.includes('Rate') || metric.metric.includes('Confidence')
                      ? '%'
                      : ''}
                </Badge>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Operation Breakdown */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Operations by Type
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
            {effectivenessData.operationBreakdown.map(op => (
              <div
                key={op.type}
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
                  <div>
                    <div style={{ fontWeight: 600, marginBottom: designTokens.spacing[1] }}>
                      {op.type}
                    </div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.secondary,
                      }}
                    >
                      {op.count} operations • Avg {formatDuration(op.avgDuration)}
                    </div>
                  </div>
                  <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
                    <Tooltip content="Success Rate">
                      <Badge variant="success">{op.successRate.toFixed(1)}%</Badge>
                    </Tooltip>
                    <Tooltip content="Avg Confidence">
                      <Badge variant="status">{op.avgConfidence.toFixed(0)}%</Badge>
                    </Tooltip>
                  </div>
                </div>
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: '1fr 1fr',
                    gap: designTokens.spacing[2],
                  }}
                >
                  <div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      Success Rate
                    </div>
                    <Progress value={op.successRate} variant="confidence" size="sm" />
                  </div>
                  <div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      Confidence
                    </div>
                    <Progress value={op.avgConfidence} variant="confidence" size="sm" />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Provider Performance */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Provider Performance
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
            {effectivenessData.providerStats.map(provider => (
              <div
                key={provider.provider}
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
                  <div>
                    <div style={{ fontWeight: 600, marginBottom: designTokens.spacing[1] }}>
                      {provider.provider}
                    </div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.secondary,
                      }}
                    >
                      {provider.operations} operations • {formatDuration(provider.avgLatency)}{' '}
                      latency
                    </div>
                  </div>
                  <Badge variant="status">{provider.successRate.toFixed(1)}%</Badge>
                </div>
                <Progress value={provider.successRate} variant="confidence" size="sm" />
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Recent Operations */}
      <Card>
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
              Recent Operations
            </h3>
            <select
              value={selectedType}
              onChange={e => setSelectedType(e.target.value)}
              style={{
                padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
                borderRadius: designTokens.borderRadius.md,
                border: `1px solid ${designTokens.colors.border.subtle}`,
                background: designTokens.colors.surface.tertiary,
                color: designTokens.colors.text.primary,
                fontSize: designTokens.typography.fontSize.sm,
                cursor: 'pointer',
              }}
            >
              <option value="all">All Types</option>
              {effectivenessData.operationBreakdown.map(op => (
                <option key={op.type} value={op.type}>
                  {op.type}
                </option>
              ))}
            </select>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[2] }}>
            {filteredOperations.map(op => (
              <div
                key={op.id}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.secondary,
                  borderLeft: `4px solid ${op.success ? designTokens.colors.accent.success : designTokens.colors.accent.alert}`,
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
                      <Badge variant={op.success ? 'success' : 'error'}>
                        <Icon
                          name={op.success ? 'Heart' : 'X'}
                          size={12}
                          style={{ marginRight: designTokens.spacing[1] }}
                        />
                        {op.success ? 'Success' : 'Failed'}
                      </Badge>
                      <span
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          fontWeight: 600,
                        }}
                      >
                        {op.type}
                      </span>
                    </div>
                    <p
                      style={{
                        fontSize: designTokens.typography.fontSize.sm,
                        color: designTokens.colors.text.secondary,
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      {op.result}
                    </p>
                    <div
                      style={{
                        display: 'flex',
                        gap: designTokens.spacing[4],
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                      }}
                    >
                      <span>{formatTimeAgo(op.timestamp)}</span>
                      <span>Duration: {formatDuration(op.duration)}</span>
                      <span>Provider: {op.provider}</span>
                    </div>
                  </div>
                  <Tooltip content={`Confidence: ${op.confidence}%`}>
                    <div
                      style={{
                        width: 48,
                        height: 48,
                        borderRadius: '50%',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: getConfidenceColor(op.confidence),
                        fontWeight: 700,
                        fontSize: designTokens.typography.fontSize.sm,
                      }}
                    >
                      {op.confidence}
                    </div>
                  </Tooltip>
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>
    </div>
  )
}

export default AIOperationEffectiveness
