/**
 * Loading States Demo Page
 *
 * Demonstrates all loading state components from the LoadingStates system
 * This page showcases the various loading states available in Proxemic
 */

import React, { useState, useCallback } from 'react'
import { designTokens } from '../../styles/tokens'
import {
  TopProgressLine,
  AIThinkingIndicator,
  DocumentSkeleton,
  ChatSkeleton,
  CardSkeleton,
  ListSkeleton,
  OperationProgressTracker,
  LongOperationProgress,
  type OperationProgress,
} from '../ui/LoadingStates'
import Button from '../ui/Button'
import Card from '../ui/Card'

const LoadingStatesDemo: React.FC = () => {
  const [showTopProgress, setShowTopProgress] = useState(false)
  const [topProgress, setTopProgress] = useState(0)
  const [operations, setOperations] = useState<OperationProgress[]>([
    {
      id: '1',
      operation: 'Processing document batch',
      progress: 45,
      status: 'in-progress',
      details: '15 of 33 documents processed',
      canCancel: true,
    },
    {
      id: '2',
      operation: 'Analyzing workspace',
      progress: 78,
      status: 'in-progress',
      details: 'Computing knowledge gaps',
      canCancel: true,
    },
    {
      id: '3',
      operation: 'AI Style Transfer',
      status: 'completed',
      details: 'Successfully applied organizational style',
    },
  ])

  const handleCancelOperation = useCallback((operationId: string) => {
    setOperations(prev =>
      prev.map(op => (op.id === operationId ? { ...op, status: 'cancelled' as const } : op))
    )
  }, [])

  const handleTestTopProgress = useCallback(() => {
    setShowTopProgress(true)
    setTopProgress(0)

    const interval = setInterval(() => {
      setTopProgress(prev => {
        if (prev >= 100) {
          clearInterval(interval)
          setTimeout(() => setShowTopProgress(false), 500)
          return 100
        }
        return prev + 2
      })
    }, 50)
  }, [])

  return (
    <div
      style={{
        padding: designTokens.spacing[6],
        minHeight: '100vh',
        backgroundColor: designTokens.colors.background.canvas,
        position: 'relative',
      }}
    >
      {/* Top Progress Line Demo */}
      {showTopProgress && <TopProgressLine progress={topProgress} />}

      {/* Header */}
      <div style={{ marginBottom: designTokens.spacing[8] }}>
        <h1
          style={{
            fontSize: designTokens.typography.fontSize['3xl'],
            fontWeight: designTokens.typography.fontWeight.bold,
            color: designTokens.colors.text.primary,
            marginBottom: designTokens.spacing[2],
          }}
        >
          Loading States System
        </h1>
        <p
          style={{
            fontSize: designTokens.typography.fontSize.lg,
            color: designTokens.colors.text.secondary,
          }}
        >
          Comprehensive loading indicators and skeleton screens
        </p>
      </div>

      {/* Sections */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[8] }}>
        {/* Top Progress Line */}
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <h2
              style={{
                fontSize: designTokens.typography.fontSize.xl,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[3],
              }}
            >
              Top Progress Line
            </h2>
            <p
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Thin progress indicator at the top of panels, no spinners
            </p>

            <Button onClick={handleTestTopProgress}>Test Top Progress</Button>
          </div>
        </Card>

        {/* AI Thinking Indicator */}
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <h2
              style={{
                fontSize: designTokens.typography.fontSize.xl,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[3],
              }}
            >
              AI Thinking Indicator
            </h2>
            <p
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Gentle pulse animation for AI processing
            </p>

            <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
              <AIThinkingIndicator size="sm" message="Processing your request..." />
              <AIThinkingIndicator size="md" message="AI is analyzing documents..." />
              <AIThinkingIndicator
                size="lg"
                message="Generating comprehensive report..."
                showDots={false}
              />
            </div>
          </div>
        </Card>

        {/* Skeleton Screens */}
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <h2
              style={{
                fontSize: designTokens.typography.fontSize.xl,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[3],
              }}
            >
              Skeleton Screens
            </h2>
            <p
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Matching content structure for better perceived performance
            </p>

            <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[6] }}>
              <div>
                <h3
                  style={{
                    fontSize: designTokens.typography.fontSize.base,
                    fontWeight: designTokens.typography.fontWeight.medium,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  Document Skeleton
                </h3>
                <div
                  style={{
                    background: designTokens.colors.surface.secondary,
                    borderRadius: designTokens.borderRadius.lg,
                    border: `1px solid ${designTokens.colors.border.subtle}`,
                  }}
                >
                  <DocumentSkeleton />
                </div>
              </div>

              <div>
                <h3
                  style={{
                    fontSize: designTokens.typography.fontSize.base,
                    fontWeight: designTokens.typography.fontWeight.medium,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  Chat Skeleton
                </h3>
                <div
                  style={{
                    background: designTokens.colors.surface.secondary,
                    borderRadius: designTokens.borderRadius.lg,
                    border: `1px solid ${designTokens.colors.border.subtle}`,
                  }}
                >
                  <ChatSkeleton count={2} />
                </div>
              </div>

              <div>
                <h3
                  style={{
                    fontSize: designTokens.typography.fontSize.base,
                    fontWeight: designTokens.typography.fontWeight.medium,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  Card Skeleton
                </h3>
                <CardSkeleton count={2} />
              </div>

              <div>
                <h3
                  style={{
                    fontSize: designTokens.typography.fontSize.base,
                    fontWeight: designTokens.typography.fontWeight.medium,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[2],
                  }}
                >
                  List Skeleton
                </h3>
                <div
                  style={{
                    background: designTokens.colors.surface.secondary,
                    borderRadius: designTokens.borderRadius.lg,
                    border: `1px solid ${designTokens.colors.border.subtle}`,
                  }}
                >
                  <ListSkeleton count={3} />
                </div>
              </div>
            </div>
          </div>
        </Card>

        {/* Operation Progress Tracker */}
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <h2
              style={{
                fontSize: designTokens.typography.fontSize.xl,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[3],
              }}
            >
              Operation Progress Tracker
            </h2>
            <p
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Track multiple operations with cancellation support
            </p>

            <OperationProgressTracker operations={operations} onCancel={handleCancelOperation} />
          </div>
        </Card>

        {/* Long Operation Progress */}
        <Card>
          <div style={{ padding: designTokens.spacing[4] }}>
            <h2
              style={{
                fontSize: designTokens.typography.fontSize.xl,
                fontWeight: designTokens.typography.fontWeight.semibold,
                color: designTokens.colors.text.primary,
                marginBottom: designTokens.spacing[3],
              }}
            >
              Long Operation Progress
            </h2>
            <p
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[4],
              }}
            >
              Detailed progress for long-running operations with time estimates
            </p>

            <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[4] }}>
              <LongOperationProgress
                operation="Processing large document batch"
                progress={65}
                estimatedTimeRemaining={120}
                details="Analyzing 50 documents for knowledge gaps"
                onCancel={() => alert('Operation cancelled')}
                variant="default"
              />

              <LongOperationProgress
                operation="AI Style Transfer in Progress"
                progress={42}
                estimatedTimeRemaining={180}
                details="Applying organizational style patterns"
                onCancel={() => alert('Operation cancelled')}
                variant="ai"
              />
            </div>
          </div>
        </Card>
      </div>
    </div>
  )
}

export default LoadingStatesDemo
