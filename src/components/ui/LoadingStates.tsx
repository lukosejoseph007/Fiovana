/**
 * LoadingStates Component
 *
 * Comprehensive loading state system matching the Fiovana design system
 * Features:
 * - Skeleton screens matching content structure
 * - Progress lines at top of panels (no spinners)
 * - AI thinking indicators (gentle pulse)
 * - Long operation progress tracking
 * - Cancellation capabilities for user control
 */

import React, { useCallback, useEffect, useState } from 'react'
import { designTokens } from '../../styles/tokens'
import Icon from './Icon'
import Progress from './Progress'

// ============================================================================
// TOP PANEL PROGRESS LINE
// ============================================================================

export interface TopProgressLineProps {
  progress?: number
  isIndeterminate?: boolean
  color?: string
  height?: number
  className?: string
}

export const TopProgressLine: React.FC<TopProgressLineProps> = ({
  progress = 0,
  isIndeterminate = false,
  color = designTokens.colors.accent.ai,
  height = 2,
  className = '',
}) => {
  return (
    <>
      <style>
        {`
          @keyframes indeterminateProgress {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(100%); }
          }
        `}
      </style>

      <div
        className={`top-progress-line ${className}`}
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          height: `${height}px`,
          backgroundColor: designTokens.colors.surface.tertiary,
          overflow: 'hidden',
          zIndex: designTokens.zIndex.sticky,
        }}
      >
        <div
          style={{
            height: '100%',
            backgroundColor: color,
            width: isIndeterminate ? '30%' : `${progress}%`,
            transition: isIndeterminate
              ? 'none'
              : `width ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
            animation: isIndeterminate ? 'indeterminateProgress 1.5s ease-in-out infinite' : 'none',
          }}
        />
      </div>
    </>
  )
}

// ============================================================================
// AI THINKING INDICATOR
// ============================================================================

export interface AIThinkingIndicatorProps {
  message?: string
  size?: 'sm' | 'md' | 'lg'
  showDots?: boolean
  className?: string
  style?: React.CSSProperties
}

export const AIThinkingIndicator: React.FC<AIThinkingIndicatorProps> = ({
  message = 'AI is thinking...',
  size = 'md',
  showDots = true,
  className = '',
  style,
}) => {
  const sizeMap = {
    sm: { icon: 16, fontSize: designTokens.typography.fontSize.sm, dotSize: 6 },
    md: { icon: 20, fontSize: designTokens.typography.fontSize.base, dotSize: 8 },
    lg: { icon: 24, fontSize: designTokens.typography.fontSize.lg, dotSize: 10 },
  }

  const currentSize = sizeMap[size]

  return (
    <>
      <style>
        {`
          @keyframes gentlePulse {
            0%, 100% { opacity: 1; transform: scale(1); }
            50% { opacity: 0.6; transform: scale(0.95); }
          }

          @keyframes dotBounce {
            0%, 80%, 100% { transform: translateY(0); opacity: 0.7; }
            40% { transform: translateY(-8px); opacity: 1; }
          }
        `}
      </style>

      <div
        className={`ai-thinking-indicator ${className}`}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: designTokens.spacing[3],
          ...style,
        }}
      >
        <Icon
          name="Cpu"
          size={currentSize.icon}
          color={designTokens.colors.accent.ai}
          style={{
            animation: 'gentlePulse 2s ease-in-out infinite',
          }}
        />

        <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[1] }}>
          <div
            style={{
              fontSize: currentSize.fontSize,
              fontWeight: designTokens.typography.fontWeight.medium,
              color: designTokens.colors.text.primary,
            }}
          >
            {message}
          </div>

          {showDots && (
            <div style={{ display: 'flex', gap: designTokens.spacing[1] }}>
              {[0, 1, 2].map(i => (
                <div
                  key={i}
                  style={{
                    width: `${currentSize.dotSize}px`,
                    height: `${currentSize.dotSize}px`,
                    borderRadius: '50%',
                    backgroundColor: designTokens.colors.accent.ai,
                    animation: `dotBounce 1.4s ease-in-out ${i * 0.16}s infinite`,
                  }}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  )
}

// ============================================================================
// SKELETON SCREENS
// ============================================================================

export interface SkeletonProps {
  variant?: 'text' | 'rect' | 'circle' | 'avatar' | 'button'
  width?: string | number
  height?: string | number
  count?: number
  className?: string
  style?: React.CSSProperties
}

export const Skeleton: React.FC<SkeletonProps> = ({
  variant = 'text',
  width = '100%',
  height,
  count = 1,
  className = '',
  style,
}) => {
  const getVariantStyles = (): React.CSSProperties => {
    switch (variant) {
      case 'circle':
      case 'avatar':
        return {
          width: width || '40px',
          height: height || '40px',
          borderRadius: '50%',
        }
      case 'button':
        return {
          width: width || '120px',
          height: height || '36px',
          borderRadius: designTokens.borderRadius.md,
        }
      case 'rect':
        return {
          width: width || '100%',
          height: height || '100px',
          borderRadius: designTokens.borderRadius.md,
        }
      case 'text':
      default:
        return {
          width,
          height: height || '16px',
          borderRadius: designTokens.borderRadius.sm,
        }
    }
  }

  const variantStyles = getVariantStyles()

  return (
    <>
      <style>
        {`
          @keyframes shimmer {
            0% { background-position: -200% 0; }
            100% { background-position: 200% 0; }
          }
        `}
      </style>

      <div className={className} style={style}>
        {Array.from({ length: count }).map((_, index) => (
          <div
            key={index}
            style={{
              ...variantStyles,
              background: `linear-gradient(90deg, ${designTokens.colors.surface.tertiary} 0%, ${designTokens.colors.surface.quaternary} 50%, ${designTokens.colors.surface.tertiary} 100%)`,
              backgroundSize: '200% 100%',
              animation: 'shimmer 1.5s ease-in-out infinite',
              marginBottom: index < count - 1 ? designTokens.spacing[2] : 0,
            }}
          />
        ))}
      </div>
    </>
  )
}

// Document-specific skeleton layouts
export const DocumentSkeleton: React.FC<{ className?: string }> = ({ className }) => (
  <div className={className} style={{ padding: designTokens.spacing[6] }}>
    {/* Header */}
    <div style={{ marginBottom: designTokens.spacing[6] }}>
      <Skeleton variant="text" height="32px" width="70%" />
      <div style={{ marginTop: designTokens.spacing[2] }}>
        <Skeleton variant="text" height="16px" width="40%" />
      </div>
    </div>

    {/* Paragraphs */}
    {[0, 1, 2].map(i => (
      <div key={i} style={{ marginBottom: designTokens.spacing[4] }}>
        <Skeleton variant="text" count={4} />
      </div>
    ))}
  </div>
)

export const ChatSkeleton: React.FC<{ count?: number; className?: string }> = ({
  count = 3,
  className,
}) => (
  <div className={className} style={{ padding: designTokens.spacing[4] }}>
    {Array.from({ length: count }).map((_, i) => (
      <div
        key={i}
        style={{
          marginBottom: designTokens.spacing[4],
          display: 'flex',
          gap: designTokens.spacing[3],
        }}
      >
        <Skeleton variant="avatar" width="32px" height="32px" />
        <div style={{ flex: 1 }}>
          <Skeleton
            variant="text"
            height="14px"
            width="120px"
            style={{ marginBottom: designTokens.spacing[2] }}
          />
          <Skeleton variant="text" count={2} />
        </div>
      </div>
    ))}
  </div>
)

export const CardSkeleton: React.FC<{
  count?: number
  className?: string
  style?: React.CSSProperties
}> = ({ count = 3, className, style }) => (
  <div
    className={className}
    style={{
      display: 'grid',
      gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
      gap: designTokens.spacing[4],
      ...style,
    }}
  >
    {Array.from({ length: count }).map((_, i) => (
      <div
        key={i}
        style={{
          padding: designTokens.spacing[4],
          background: designTokens.colors.surface.secondary,
          borderRadius: designTokens.borderRadius.lg,
          border: `1px solid ${designTokens.colors.border.subtle}`,
        }}
      >
        <Skeleton variant="rect" height="120px" style={{ marginBottom: designTokens.spacing[3] }} />
        <Skeleton
          variant="text"
          height="20px"
          width="80%"
          style={{ marginBottom: designTokens.spacing[2] }}
        />
        <Skeleton variant="text" count={2} />
      </div>
    ))}
  </div>
)

export const ListSkeleton: React.FC<{
  count?: number
  className?: string
  style?: React.CSSProperties
}> = ({ count = 5, className, style }) => (
  <div className={className} style={style}>
    {Array.from({ length: count }).map((_, i) => (
      <div
        key={i}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: designTokens.spacing[3],
          padding: designTokens.spacing[3],
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
        }}
      >
        <Skeleton variant="circle" width="40px" height="40px" />
        <div style={{ flex: 1 }}>
          <Skeleton
            variant="text"
            height="16px"
            width="60%"
            style={{ marginBottom: designTokens.spacing[1] }}
          />
          <Skeleton variant="text" height="14px" width="40%" />
        </div>
      </div>
    ))}
  </div>
)

// ============================================================================
// OPERATION PROGRESS TRACKER
// ============================================================================

export interface OperationProgress {
  id: string
  operation: string
  progress?: number
  status: 'pending' | 'in-progress' | 'completed' | 'failed' | 'cancelled'
  details?: string
  startTime?: number
  endTime?: number
  canCancel?: boolean
}

export interface OperationProgressTrackerProps {
  operations: OperationProgress[]
  onCancel?: (operationId: string) => void
  maxVisible?: number
  className?: string
  style?: React.CSSProperties
}

export const OperationProgressTracker: React.FC<OperationProgressTrackerProps> = ({
  operations,
  onCancel,
  maxVisible = 5,
  className = '',
  style,
}) => {
  const visibleOperations = operations.slice(0, maxVisible)

  const getStatusColor = (status: OperationProgress['status']) => {
    switch (status) {
      case 'completed':
        return designTokens.colors.accent.success
      case 'failed':
        return designTokens.colors.accent.alert
      case 'cancelled':
        return designTokens.colors.text.tertiary
      case 'in-progress':
      case 'pending':
      default:
        return designTokens.colors.accent.ai
    }
  }

  const getStatusIcon = (status: OperationProgress['status']) => {
    switch (status) {
      case 'completed':
        return 'AlertCircle'
      case 'failed':
        return 'AlertTriangle'
      case 'cancelled':
        return 'X'
      case 'in-progress':
        return 'Loader'
      case 'pending':
      default:
        return 'Loader'
    }
  }

  return (
    <div
      className={`operation-progress-tracker ${className}`}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: designTokens.spacing[2],
        ...style,
      }}
    >
      {visibleOperations.map(op => (
        <div
          key={op.id}
          style={{
            padding: designTokens.spacing[3],
            background: designTokens.colors.surface.secondary,
            border: `1px solid ${designTokens.colors.border.subtle}`,
            borderRadius: designTokens.borderRadius.md,
            transition: `all ${designTokens.animation.duration.normal}`,
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              marginBottom: designTokens.spacing[2],
            }}
          >
            <Icon
              name={getStatusIcon(op.status)}
              size={16}
              color={getStatusColor(op.status)}
              style={{
                animation: op.status === 'in-progress' ? 'spin 1s linear infinite' : 'none',
              }}
            />

            <div style={{ flex: 1 }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  color: designTokens.colors.text.primary,
                }}
              >
                {op.operation}
              </div>
              {op.details && (
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                    marginTop: designTokens.spacing[0.5],
                  }}
                >
                  {op.details}
                </div>
              )}
            </div>

            {op.canCancel && op.status === 'in-progress' && onCancel && (
              <button
                onClick={() => onCancel(op.id)}
                style={{
                  padding: designTokens.spacing[1],
                  background: 'transparent',
                  border: 'none',
                  cursor: 'pointer',
                  color: designTokens.colors.text.tertiary,
                  borderRadius: designTokens.borderRadius.sm,
                  transition: `all ${designTokens.animation.duration.fast}`,
                }}
                title="Cancel operation"
              >
                <Icon name="X" size={14} />
              </button>
            )}
          </div>

          {op.progress !== undefined && op.status === 'in-progress' && (
            <div>
              <Progress value={op.progress} size="sm" variant="ai" />
              <div
                style={{
                  marginTop: designTokens.spacing[1],
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.tertiary,
                  textAlign: 'right',
                }}
              >
                {Math.round(op.progress)}%
              </div>
            </div>
          )}
        </div>
      ))}

      {operations.length > maxVisible && (
        <div
          style={{
            padding: designTokens.spacing[2],
            fontSize: designTokens.typography.fontSize.xs,
            color: designTokens.colors.text.tertiary,
            textAlign: 'center',
          }}
        >
          +{operations.length - maxVisible} more operation
          {operations.length - maxVisible !== 1 ? 's' : ''}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// LONG OPERATION PROGRESS
// ============================================================================

export interface LongOperationProgressProps {
  operation: string
  progress?: number
  estimatedTimeRemaining?: number
  details?: string
  onCancel?: () => void
  variant?: 'default' | 'ai'
  className?: string
  style?: React.CSSProperties
}

export const LongOperationProgress: React.FC<LongOperationProgressProps> = ({
  operation,
  progress,
  estimatedTimeRemaining,
  details,
  onCancel,
  variant = 'default',
  className = '',
  style,
}) => {
  const [elapsedTime, setElapsedTime] = useState(0)

  useEffect(() => {
    const interval = setInterval(() => {
      setElapsedTime(prev => prev + 1)
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  const formatTime = useCallback((seconds: number): string => {
    const minutes = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${minutes}:${secs.toString().padStart(2, '0')}`
  }, [])

  return (
    <div
      className={`long-operation-progress ${className}`}
      style={{
        padding: designTokens.spacing[4],
        background: designTokens.colors.surface.secondary,
        border: `1px solid ${designTokens.colors.border.subtle}`,
        borderRadius: designTokens.borderRadius.lg,
        boxShadow: designTokens.shadows.lg,
        minWidth: '320px',
        maxWidth: '480px',
        ...style,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'flex-start',
          gap: designTokens.spacing[3],
          marginBottom: designTokens.spacing[4],
        }}
      >
        {variant === 'ai' ? (
          <Icon
            name="Cpu"
            size={24}
            color={designTokens.colors.accent.ai}
            style={{ animation: 'gentlePulse 2s ease-in-out infinite', flexShrink: 0 }}
          />
        ) : (
          <Icon
            name="Loader"
            size={24}
            color={designTokens.colors.accent.ai}
            style={{ animation: 'spin 1s linear infinite', flexShrink: 0 }}
          />
        )}

        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: designTokens.typography.fontWeight.semibold,
              color: designTokens.colors.text.primary,
              marginBottom: designTokens.spacing[1],
            }}
          >
            {operation}
          </div>

          {details && (
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
              }}
            >
              {details}
            </div>
          )}
        </div>

        {onCancel && (
          <button
            onClick={onCancel}
            style={{
              padding: designTokens.spacing[1.5],
              background: 'transparent',
              border: `1px solid ${designTokens.colors.border.medium}`,
              cursor: 'pointer',
              color: designTokens.colors.text.secondary,
              borderRadius: designTokens.borderRadius.md,
              transition: `all ${designTokens.animation.duration.fast}`,
              flexShrink: 0,
            }}
            title="Cancel operation"
          >
            <Icon name="X" size={16} />
          </button>
        )}
      </div>

      {progress !== undefined && (
        <div style={{ marginBottom: designTokens.spacing[3] }}>
          <Progress value={progress} animated={true} />
        </div>
      )}

      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          fontSize: designTokens.typography.fontSize.xs,
          color: designTokens.colors.text.tertiary,
        }}
      >
        <div>Elapsed: {formatTime(elapsedTime)}</div>
        {estimatedTimeRemaining !== undefined && (
          <div>Est. remaining: {formatTime(estimatedTimeRemaining)}</div>
        )}
        {progress !== undefined && <div>{Math.round(progress)}% complete</div>}
      </div>
    </div>
  )
}

// ============================================================================
// EXPORTS
// ============================================================================

export default {
  TopProgressLine,
  AIThinkingIndicator,
  Skeleton,
  DocumentSkeleton,
  ChatSkeleton,
  CardSkeleton,
  ListSkeleton,
  OperationProgressTracker,
  LongOperationProgress,
}
