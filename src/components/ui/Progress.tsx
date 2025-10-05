import React from 'react'
import { designTokens } from '../../styles/tokens'

export interface ProgressProps {
  value: number
  max?: number
  variant?: 'default' | 'confidence' | 'health' | 'ai'
  size?: 'sm' | 'md' | 'lg'
  showLabel?: boolean
  label?: string
  showPercentage?: boolean
  animated?: boolean
  className?: string
  style?: React.CSSProperties
}

const Progress: React.FC<ProgressProps> = ({
  value,
  max = 100,
  variant = 'default',
  size = 'md',
  showLabel = false,
  label,
  showPercentage = false,
  animated = false,
  className = '',
  style,
}) => {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100)

  // Size variants
  const sizeStyles = {
    sm: {
      height: '4px',
      fontSize: designTokens.typography.fontSize.xs,
    },
    md: {
      height: '6px',
      fontSize: designTokens.typography.fontSize.sm,
    },
    lg: {
      height: '8px',
      fontSize: designTokens.typography.fontSize.base,
    },
  }

  // Variant colors
  const getVariantColor = () => {
    switch (variant) {
      case 'confidence':
        if (percentage >= 80) return designTokens.colors.confidence.high
        if (percentage >= 60) return designTokens.colors.confidence.medium
        if (percentage >= 40) return designTokens.colors.confidence.low
        return designTokens.colors.confidence.critical
      case 'health':
        if (percentage >= 90) return designTokens.colors.confidence.high
        if (percentage >= 70) return designTokens.colors.confidence.medium
        if (percentage >= 50) return designTokens.colors.confidence.low
        return designTokens.colors.confidence.critical
      case 'ai':
        return designTokens.colors.accent.ai
      default:
        return designTokens.colors.accent.semantic
    }
  }

  const containerStyles = {
    width: '100%',
    ...style,
  }

  const trackStyles = {
    width: '100%',
    height: sizeStyles[size].height,
    backgroundColor: designTokens.colors.surface.tertiary,
    borderRadius: designTokens.borderRadius.full,
    overflow: 'hidden' as const,
    position: 'relative' as const,
    border: `1px solid ${designTokens.colors.border.subtle}`,
  }

  const fillStyles = {
    height: '100%',
    width: `${percentage}%`,
    backgroundColor: getVariantColor(),
    borderRadius: 'inherit',
    transition: animated
      ? `width ${designTokens.animation.duration.slow} ${designTokens.animation.easing.easeOut}`
      : 'none',
    position: 'relative' as const,
    overflow: 'hidden' as const,
  }

  const labelContainerStyles = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: designTokens.spacing[1],
    fontSize: sizeStyles[size].fontSize,
    color: designTokens.colors.text.primary,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
  }

  const percentageStyles = {
    fontSize: sizeStyles[size].fontSize,
    color: designTokens.colors.text.secondary,
    fontWeight: designTokens.typography.fontWeight.medium,
    minWidth: '3ch',
    textAlign: 'right' as const,
  }

  // Animated shimmer effect for AI variant
  const shimmerStyles =
    variant === 'ai' && animated
      ? {
          position: 'absolute' as const,
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: `linear-gradient(
      90deg,
      transparent,
      rgba(255, 255, 255, 0.2),
      transparent
    )`,
          animation: 'shimmer 2s infinite',
          transform: 'translateX(-100%)',
        }
      : {}

  return (
    <>
      {variant === 'ai' && animated && (
        <style>
          {`
            @keyframes shimmer {
              0% {
                transform: translateX(-100%);
              }
              100% {
                transform: translateX(100%);
              }
            }
          `}
        </style>
      )}

      <div className={`fiovana-progress ${className}`} style={containerStyles}>
        {(showLabel || showPercentage) && (
          <div style={labelContainerStyles}>
            {showLabel && (
              <span style={{ fontWeight: designTokens.typography.fontWeight.medium }}>
                {label || `Progress`}
              </span>
            )}
            {showPercentage && <span style={percentageStyles}>{Math.round(percentage)}%</span>}
          </div>
        )}

        <div
          style={trackStyles}
          role="progressbar"
          aria-valuenow={value}
          aria-valuemin={0}
          aria-valuemax={max}
          aria-label={label || 'Progress'}
        >
          <div style={fillStyles}>
            {variant === 'ai' && animated && <div style={shimmerStyles} />}
          </div>
        </div>
      </div>
    </>
  )
}

// Specialized progress components for common use cases
export const ConfidenceProgress: React.FC<Omit<ProgressProps, 'variant'>> = props => (
  <Progress {...props} variant="confidence" />
)

export const HealthProgress: React.FC<Omit<ProgressProps, 'variant'>> = props => (
  <Progress {...props} variant="health" />
)

export const AIProgress: React.FC<Omit<ProgressProps, 'variant'>> = props => (
  <Progress {...props} variant="ai" animated />
)

export default Progress
