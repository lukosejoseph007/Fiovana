import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Progress from '../ui/Progress'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import Tooltip from '../ui/Tooltip'
import { styleAnalysisService } from '../../services/styleAnalysisService'

export interface StyleConsistencyAnalysisProps {
  workspaceId: string
  className?: string
  style?: React.CSSProperties
}

interface StyleConsistencyData {
  overallConsistency: number
  vocabularyConsistency: number
  toneConsistency: number
  formattingConsistency: number
  structureConsistency: number
  inconsistencies: StyleInconsistency[]
  styleProfiles: StyleProfileSummary[]
  recommendations: StyleRecommendation[]
}

interface StyleInconsistency {
  id: string
  category: 'vocabulary' | 'tone' | 'formatting' | 'structure'
  severity: 'high' | 'medium' | 'low'
  description: string
  examples: string[]
  affectedDocuments: number
  suggestedFix: string
}

interface StyleProfileSummary {
  id: string
  name: string
  documentsUsing: number
  consistency: number
  lastUpdated: string
}

interface StyleRecommendation {
  id: string
  type: 'unify' | 'establish' | 'improve'
  priority: 'high' | 'medium' | 'low'
  title: string
  description: string
  impact: string
}

const StyleConsistencyAnalysis: React.FC<StyleConsistencyAnalysisProps> = ({
  workspaceId,
  className = '',
  style,
}) => {
  const [consistencyData, setConsistencyData] = useState<StyleConsistencyData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedCategory, setSelectedCategory] = useState<
    'all' | 'vocabulary' | 'tone' | 'formatting' | 'structure'
  >('all')

  // Load consistency data
  const loadConsistencyData = useCallback(async () => {
    if (!workspaceId) return

    try {
      setIsLoading(true)
      setError(null)

      // Load style profiles
      const profilesResult = await styleAnalysisService.listStyleProfiles()

      if (profilesResult.success) {
        // Mock consistency data (would come from backend analysis)
        setConsistencyData({
          overallConsistency: 72,
          vocabularyConsistency: 78,
          toneConsistency: 68,
          formattingConsistency: 85,
          structureConsistency: 65,
          inconsistencies: [
            {
              id: 'inc1',
              category: 'vocabulary',
              severity: 'high',
              description: 'Mixed terminology for same concepts',
              examples: ['"user" vs "customer" vs "client"', '"delete" vs "remove" vs "discard"'],
              affectedDocuments: 15,
              suggestedFix: 'Standardize on "user" and "delete" across all documents',
            },
            {
              id: 'inc2',
              category: 'tone',
              severity: 'medium',
              description: 'Inconsistent formality levels',
              examples: ['Technical docs use casual tone', 'User guides mix formal and informal'],
              affectedDocuments: 8,
              suggestedFix: 'Establish tone guidelines per document type',
            },
            {
              id: 'inc3',
              category: 'formatting',
              severity: 'low',
              description: 'Varied heading styles',
              examples: ['Some use Title Case', 'Others use Sentence case'],
              affectedDocuments: 12,
              suggestedFix: 'Apply consistent heading capitalization',
            },
            {
              id: 'inc4',
              category: 'structure',
              severity: 'medium',
              description: 'Inconsistent section ordering',
              examples: ['Prerequisites placed differently', 'Examples section varies in position'],
              affectedDocuments: 10,
              suggestedFix: 'Create document templates with standard sections',
            },
          ],
          styleProfiles: [
            {
              id: 'profile1',
              name: 'Technical Documentation',
              documentsUsing: 22,
              consistency: 85,
              lastUpdated: new Date(Date.now() - 604800000).toISOString(),
            },
            {
              id: 'profile2',
              name: 'User Guides',
              documentsUsing: 18,
              consistency: 72,
              lastUpdated: new Date(Date.now() - 259200000).toISOString(),
            },
            {
              id: 'profile3',
              name: 'API Reference',
              documentsUsing: 12,
              consistency: 90,
              lastUpdated: new Date(Date.now() - 86400000).toISOString(),
            },
          ],
          recommendations: [
            {
              id: 'rec1',
              type: 'unify',
              priority: 'high',
              title: 'Unify Terminology',
              description: 'Create and apply a unified terminology glossary',
              impact: 'Improves consistency by estimated 15%',
            },
            {
              id: 'rec2',
              type: 'establish',
              priority: 'high',
              title: 'Establish Tone Guidelines',
              description: 'Define tone guidelines for each document category',
              impact: 'Reduces tone inconsistencies by 40%',
            },
            {
              id: 'rec3',
              type: 'improve',
              priority: 'medium',
              title: 'Template Standardization',
              description: 'Create standard templates for common document types',
              impact: 'Improves structural consistency by 20%',
            },
          ],
        })
      }
    } catch (err) {
      console.error('Failed to load consistency data:', err)
      setError(err instanceof Error ? err.message : 'Failed to load data')
    } finally {
      setIsLoading(false)
    }
  }, [workspaceId])

  useEffect(() => {
    loadConsistencyData()
  }, [loadConsistencyData])

  // Filter inconsistencies by category
  const filteredInconsistencies = useMemo(() => {
    if (!consistencyData) return []
    if (selectedCategory === 'all') return consistencyData.inconsistencies
    return consistencyData.inconsistencies.filter(inc => inc.category === selectedCategory)
  }, [consistencyData, selectedCategory])

  // Get consistency color
  const getConsistencyColor = useCallback((value: number): string => {
    if (value >= 85) return designTokens.colors.confidence.high
    if (value >= 70) return designTokens.colors.accent.success
    if (value >= 55) return designTokens.colors.confidence.medium
    return designTokens.colors.confidence.critical
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

  // Get priority color
  const getPriorityColor = useCallback((priority: string): string => {
    switch (priority) {
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

  if (isLoading) {
    return (
      <Card className={className} style={style}>
        <div style={{ padding: designTokens.spacing[6], textAlign: 'center' }}>
          <Icon name="Loader" size={32} style={{ marginBottom: designTokens.spacing[4] }} />
          <p style={{ color: designTokens.colors.text.secondary }}>
            Loading consistency analysis...
          </p>
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
            Error loading analysis
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

  if (!consistencyData) return null

  return (
    <div className={className} style={style}>
      {/* Header */}
      <div style={{ marginBottom: designTokens.spacing[6] }}>
        <h2 style={{ fontSize: designTokens.typography.fontSize['2xl'], fontWeight: 600 }}>
          Style Consistency Analysis
        </h2>
      </div>

      {/* Overall Consistency Score */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <div style={{ marginBottom: designTokens.spacing[4] }}>
            <span
              style={{
                color: designTokens.colors.text.secondary,
                fontSize: designTokens.typography.fontSize.sm,
              }}
            >
              Overall Style Consistency
            </span>
            <div
              style={{
                display: 'flex',
                alignItems: 'baseline',
                gap: designTokens.spacing[2],
                marginTop: designTokens.spacing[1],
              }}
            >
              <span style={{ fontSize: designTokens.typography.fontSize['4xl'], fontWeight: 700 }}>
                {consistencyData.overallConsistency}%
              </span>
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.base,
                  color: designTokens.colors.text.secondary,
                }}
              >
                consistency score
              </span>
            </div>
          </div>
          <Progress value={consistencyData.overallConsistency} variant="confidence" />
        </div>
      </Card>

      {/* Consistency Breakdown */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Consistency Breakdown
          </h3>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
              gap: designTokens.spacing[4],
            }}
          >
            {[
              {
                label: 'Vocabulary',
                value: consistencyData.vocabularyConsistency,
                icon: 'BookOpen',
              },
              { label: 'Tone', value: consistencyData.toneConsistency, icon: 'MessageCircle' },
              {
                label: 'Formatting',
                value: consistencyData.formattingConsistency,
                icon: 'Layers',
              },
              { label: 'Structure', value: consistencyData.structureConsistency, icon: 'Layout' },
            ].map(metric => (
              <div
                key={metric.label}
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
                  <span style={{ fontSize: designTokens.typography.fontSize.sm }}>
                    {metric.label}
                  </span>
                  <Icon
                    name={metric.icon as 'BookOpen' | 'MessageCircle' | 'Layers' | 'Layout'}
                    size={18}
                    color={getConsistencyColor(metric.value)}
                  />
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize['2xl'],
                    fontWeight: 700,
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  {metric.value}%
                </div>
                <Progress value={metric.value} variant="confidence" size="sm" />
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Inconsistencies */}
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
              Detected Inconsistencies
            </h3>
            <div style={{ display: 'flex', gap: designTokens.spacing[1] }}>
              {(['all', 'vocabulary', 'tone', 'formatting', 'structure'] as const).map(category => (
                <button
                  key={category}
                  onClick={() => setSelectedCategory(category)}
                  style={{
                    padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
                    borderRadius: designTokens.borderRadius.md,
                    border: 'none',
                    background:
                      selectedCategory === category
                        ? designTokens.colors.accent.ai
                        : designTokens.colors.surface.tertiary,
                    color: designTokens.colors.text.primary,
                    fontSize: designTokens.typography.fontSize.xs,
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    textTransform: 'capitalize',
                  }}
                >
                  {category}
                </button>
              ))}
            </div>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
            {filteredInconsistencies.map(inc => (
              <div
                key={inc.id}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.secondary,
                  borderLeft: `4px solid ${getSeverityColor(inc.severity)}`,
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
                  <div style={{ flex: 1 }}>
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
                          inc.severity === 'high'
                            ? 'error'
                            : inc.severity === 'medium'
                              ? 'warning'
                              : 'default'
                        }
                      >
                        {inc.severity.toUpperCase()}
                      </Badge>
                      <span
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.secondary,
                          textTransform: 'capitalize',
                        }}
                      >
                        {inc.category}
                      </span>
                    </div>
                    <p
                      style={{
                        fontSize: designTokens.typography.fontSize.base,
                        fontWeight: 600,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      {inc.description}
                    </p>
                    <div
                      style={{
                        padding: designTokens.spacing[2],
                        borderRadius: designTokens.borderRadius.sm,
                        background: designTokens.colors.surface.primary,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.xs,
                          color: designTokens.colors.text.tertiary,
                          marginBottom: designTokens.spacing[1],
                        }}
                      >
                        Examples:
                      </div>
                      {inc.examples.map((example, idx) => (
                        <div
                          key={idx}
                          style={{
                            fontSize: designTokens.typography.fontSize.sm,
                            color: designTokens.colors.text.secondary,
                            fontFamily: designTokens.typography.fonts.mono.join(', '),
                            marginBottom: designTokens.spacing[1],
                          }}
                        >
                          • {example}
                        </div>
                      ))}
                    </div>
                    <div
                      style={{
                        padding: designTokens.spacing[2],
                        borderRadius: designTokens.borderRadius.sm,
                        background: designTokens.colors.surface.tertiary,
                        borderLeft: `2px solid ${designTokens.colors.accent.success}`,
                      }}
                    >
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.xs,
                          color: designTokens.colors.text.tertiary,
                          marginBottom: designTokens.spacing[1],
                        }}
                      >
                        Suggested Fix:
                      </div>
                      <div style={{ fontSize: designTokens.typography.fontSize.sm }}>
                        {inc.suggestedFix}
                      </div>
                    </div>
                  </div>
                  <Tooltip content={`Affects ${inc.affectedDocuments} documents`}>
                    <Badge variant="default">{inc.affectedDocuments}</Badge>
                  </Tooltip>
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Style Profiles */}
      <Card style={{ marginBottom: designTokens.spacing[6] }}>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Active Style Profiles
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[2] }}>
            {consistencyData.styleProfiles.map(profile => (
              <div
                key={profile.id}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.tertiary,
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div>
                  <div style={{ fontWeight: 600, marginBottom: designTokens.spacing[1] }}>
                    {profile.name}
                  </div>
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      color: designTokens.colors.text.secondary,
                    }}
                  >
                    Used by {profile.documentsUsing} documents • Updated{' '}
                    {new Date(profile.lastUpdated).toLocaleDateString()}
                  </div>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.xl,
                      fontWeight: 700,
                      color: getConsistencyColor(profile.consistency),
                    }}
                  >
                    {profile.consistency}%
                  </div>
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      color: designTokens.colors.text.tertiary,
                    }}
                  >
                    consistency
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Recommendations */}
      <Card>
        <div style={{ padding: designTokens.spacing[6] }}>
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              marginBottom: designTokens.spacing[4],
            }}
          >
            Recommendations
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
            {consistencyData.recommendations.map(rec => (
              <div
                key={rec.id}
                style={{
                  padding: designTokens.spacing[4],
                  borderRadius: designTokens.borderRadius.md,
                  background: designTokens.colors.surface.tertiary,
                  borderLeft: `4px solid ${getPriorityColor(rec.priority)}`,
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
                          rec.priority === 'high'
                            ? 'error'
                            : rec.priority === 'medium'
                              ? 'warning'
                              : 'default'
                        }
                      >
                        {rec.priority.toUpperCase()}
                      </Badge>
                      <span
                        style={{
                          fontSize: designTokens.typography.fontSize.xs,
                          color: designTokens.colors.text.tertiary,
                          textTransform: 'uppercase',
                        }}
                      >
                        {rec.type}
                      </span>
                    </div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.base,
                        fontWeight: 600,
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      {rec.title}
                    </div>
                    <p
                      style={{
                        fontSize: designTokens.typography.fontSize.sm,
                        color: designTokens.colors.text.secondary,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      {rec.description}
                    </p>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.accent.success,
                      }}
                    >
                      <Icon
                        name="TrendingUp"
                        size={12}
                        style={{ marginRight: designTokens.spacing[1] }}
                      />
                      {rec.impact}
                    </div>
                  </div>
                  <Icon name="LightBulb" size={24} color={designTokens.colors.accent.semantic} />
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>
    </div>
  )
}

export default StyleConsistencyAnalysis
