import React, { useState } from 'react'
import { designTokens } from '../../styles/tokens'
import DocumentUsageMetrics from './DocumentUsageMetrics'
import ContentQualityTrends from './ContentQualityTrends'
import StyleConsistencyAnalysis from './StyleConsistencyAnalysis'
import AIOperationEffectiveness from './AIOperationEffectiveness'
import Card from '../ui/Card'
import Icon from '../ui/Icon'

export interface AnalyticsDashboardProps {
  workspaceId: string
  className?: string
  style?: React.CSSProperties
}

type AnalyticsView = 'overview' | 'usage' | 'quality' | 'style' | 'ai'

const AnalyticsDashboard: React.FC<AnalyticsDashboardProps> = ({
  workspaceId,
  className = '',
  style,
}) => {
  const [activeView, setActiveView] = useState<AnalyticsView>('overview')

  const views = [
    { id: 'overview', label: 'Overview', icon: 'Layout' },
    { id: 'usage', label: 'Usage Metrics', icon: 'Zap' },
    { id: 'quality', label: 'Quality Trends', icon: 'TrendingUp' },
    { id: 'style', label: 'Style Consistency', icon: 'FileText' },
    { id: 'ai', label: 'AI Operations', icon: 'Zap' },
  ] as const

  return (
    <div
      className={className}
      style={{ ...style, height: '100%', display: 'flex', flexDirection: 'column' }}
    >
      {/* Header with Navigation */}
      <div
        style={{
          padding: designTokens.spacing[6],
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
          background: designTokens.colors.surface.primary,
        }}
      >
        <h1
          style={{
            fontSize: designTokens.typography.fontSize['3xl'],
            fontWeight: 700,
            marginBottom: designTokens.spacing[4],
          }}
        >
          Analytics Dashboard
        </h1>
        <div style={{ display: 'flex', gap: designTokens.spacing[2], flexWrap: 'wrap' }}>
          {views.map(view => (
            <button
              key={view.id}
              onClick={() => setActiveView(view.id as AnalyticsView)}
              style={{
                padding: `${designTokens.spacing[2]} ${designTokens.spacing[4]}`,
                borderRadius: designTokens.borderRadius.md,
                border: 'none',
                background:
                  activeView === view.id
                    ? designTokens.colors.accent.ai
                    : designTokens.colors.surface.tertiary,
                color: designTokens.colors.text.primary,
                fontSize: designTokens.typography.fontSize.sm,
                fontWeight: activeView === view.id ? 600 : 400,
                cursor: 'pointer',
                transition: 'all 0.2s ease',
                display: 'flex',
                alignItems: 'center',
                gap: designTokens.spacing[1],
              }}
              onMouseEnter={e => {
                if (activeView !== view.id) {
                  e.currentTarget.style.background = designTokens.colors.surface.quaternary
                }
              }}
              onMouseLeave={e => {
                if (activeView !== view.id) {
                  e.currentTarget.style.background = designTokens.colors.surface.tertiary
                }
              }}
            >
              <Icon name={view.icon} size={16} />
              {view.label}
            </button>
          ))}
        </div>
      </div>

      {/* Content Area */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: designTokens.spacing[6],
          background: designTokens.colors.surface.secondary,
        }}
      >
        {activeView === 'overview' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[8] }}>
            <Card>
              <div style={{ padding: designTokens.spacing[6] }}>
                <h2
                  style={{
                    fontSize: designTokens.typography.fontSize['2xl'],
                    fontWeight: 600,
                    marginBottom: designTokens.spacing[4],
                  }}
                >
                  Analytics Overview
                </h2>
                <p
                  style={{
                    color: designTokens.colors.text.secondary,
                    marginBottom: designTokens.spacing[6],
                  }}
                >
                  Select a category from the navigation above to view detailed analytics:
                </p>
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
                    gap: designTokens.spacing[4],
                  }}
                >
                  <div
                    style={{
                      padding: designTokens.spacing[6],
                      borderRadius: designTokens.borderRadius.lg,
                      background: designTokens.colors.surface.tertiary,
                      border: `1px solid ${designTokens.colors.border.subtle}`,
                    }}
                  >
                    <Icon
                      name="Zap"
                      size={32}
                      color={designTokens.colors.accent.info}
                      style={{ marginBottom: designTokens.spacing[4] }}
                    />
                    <h3
                      style={{
                        fontSize: designTokens.typography.fontSize.lg,
                        fontWeight: 600,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      Usage Metrics
                    </h3>
                    <p
                      style={{
                        color: designTokens.colors.text.secondary,
                        fontSize: designTokens.typography.fontSize.sm,
                      }}
                    >
                      Track document access patterns, peak usage times, and document type
                      distribution across your workspace.
                    </p>
                  </div>

                  <div
                    style={{
                      padding: designTokens.spacing[6],
                      borderRadius: designTokens.borderRadius.lg,
                      background: designTokens.colors.surface.tertiary,
                      border: `1px solid ${designTokens.colors.border.subtle}`,
                    }}
                  >
                    <Icon
                      name="TrendingUp"
                      size={32}
                      color={designTokens.colors.accent.success}
                      style={{ marginBottom: designTokens.spacing[4] }}
                    />
                    <h3
                      style={{
                        fontSize: designTokens.typography.fontSize.lg,
                        fontWeight: 600,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      Quality Trends
                    </h3>
                    <p
                      style={{
                        color: designTokens.colors.text.secondary,
                        fontSize: designTokens.typography.fontSize.sm,
                      }}
                    >
                      Monitor content quality scores over time, detect issues, and track
                      improvements in clarity, consistency, and completeness.
                    </p>
                  </div>

                  <div
                    style={{
                      padding: designTokens.spacing[6],
                      borderRadius: designTokens.borderRadius.lg,
                      background: designTokens.colors.surface.tertiary,
                      border: `1px solid ${designTokens.colors.border.subtle}`,
                    }}
                  >
                    <Icon
                      name="FileText"
                      size={32}
                      color={designTokens.colors.accent.semantic}
                      style={{ marginBottom: designTokens.spacing[4] }}
                    />
                    <h3
                      style={{
                        fontSize: designTokens.typography.fontSize.lg,
                        fontWeight: 600,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      Style Consistency
                    </h3>
                    <p
                      style={{
                        color: designTokens.colors.text.secondary,
                        fontSize: designTokens.typography.fontSize.sm,
                      }}
                    >
                      Analyze writing style consistency across documents, identify inconsistencies,
                      and get recommendations for improvement.
                    </p>
                  </div>

                  <div
                    style={{
                      padding: designTokens.spacing[6],
                      borderRadius: designTokens.borderRadius.lg,
                      background: designTokens.colors.surface.tertiary,
                      border: `1px solid ${designTokens.colors.border.subtle}`,
                    }}
                  >
                    <Icon
                      name="Zap"
                      size={32}
                      color={designTokens.colors.accent.ai}
                      style={{ marginBottom: designTokens.spacing[4] }}
                    />
                    <h3
                      style={{
                        fontSize: designTokens.typography.fontSize.lg,
                        fontWeight: 600,
                        marginBottom: designTokens.spacing[2],
                      }}
                    >
                      AI Operations
                    </h3>
                    <p
                      style={{
                        color: designTokens.colors.text.secondary,
                        fontSize: designTokens.typography.fontSize.sm,
                      }}
                    >
                      View AI operation effectiveness, success rates, confidence scores, and
                      provider performance metrics.
                    </p>
                  </div>
                </div>
              </div>
            </Card>
          </div>
        )}

        {activeView === 'usage' && <DocumentUsageMetrics workspaceId={workspaceId} />}

        {activeView === 'quality' && <ContentQualityTrends workspaceId={workspaceId} />}

        {activeView === 'style' && <StyleConsistencyAnalysis workspaceId={workspaceId} />}

        {activeView === 'ai' && <AIOperationEffectiveness workspaceId={workspaceId} />}
      </div>
    </div>
  )
}

export default AnalyticsDashboard
