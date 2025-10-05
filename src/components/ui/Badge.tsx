import React from 'react'
import { designTokens } from '../../styles/tokens'

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'default' | 'confidence' | 'status' | 'health' | 'ai' | 'success' | 'warning' | 'error'
  size?: 'sm' | 'md' | 'lg'
  shape?: 'rounded' | 'pill' | 'square'
  dot?: boolean
  outlined?: boolean
  pulsing?: boolean
  children?: React.ReactNode
}

const Badge: React.FC<BadgeProps> = ({
  variant = 'default',
  size = 'md',
  shape = 'rounded',
  dot = false,
  outlined = false,
  pulsing = false,
  children,
  className = '',
  style,
  ...props
}) => {
  // Size variants
  const sizeStyles = {
    sm: {
      height: dot ? '8px' : '20px',
      padding: dot ? '0' : `0 ${designTokens.spacing[2]}`,
      fontSize: designTokens.typography.fontSize.xs,
      fontWeight: designTokens.typography.fontWeight.medium,
      lineHeight: '1',
      minWidth: dot ? '8px' : '20px',
    },
    md: {
      height: dot ? '10px' : '24px',
      padding: dot ? '0' : `0 ${designTokens.spacing[3]}`,
      fontSize: designTokens.typography.fontSize.sm,
      fontWeight: designTokens.typography.fontWeight.medium,
      lineHeight: '1',
      minWidth: dot ? '10px' : '24px',
    },
    lg: {
      height: dot ? '12px' : '28px',
      padding: dot ? '0' : `0 ${designTokens.spacing[4]}`,
      fontSize: designTokens.typography.fontSize.base,
      fontWeight: designTokens.typography.fontWeight.medium,
      lineHeight: '1',
      minWidth: dot ? '12px' : '28px',
    },
  }

  // Shape variants
  const shapeStyles = {
    rounded: { borderRadius: designTokens.borderRadius.md },
    pill: { borderRadius: designTokens.borderRadius.full },
    square: { borderRadius: designTokens.borderRadius.sm },
  }

  // Variant colors
  const getVariantStyles = () => {
    const variants = {
      default: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.surface.tertiary,
        color: designTokens.colors.text.primary,
        border: `1px solid ${designTokens.colors.border.medium}`,
      },
      confidence: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.confidence.medium,
        color: outlined
          ? designTokens.colors.confidence.medium
          : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.confidence.medium}`,
      },
      status: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.accent.info,
        color: outlined ? designTokens.colors.accent.info : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.accent.info}`,
      },
      health: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.confidence.high,
        color: outlined ? designTokens.colors.confidence.high : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.confidence.high}`,
      },
      ai: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.accent.ai,
        color: outlined ? designTokens.colors.accent.ai : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.accent.ai}`,
      },
      success: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.confidence.high,
        color: outlined ? designTokens.colors.confidence.high : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.confidence.high}`,
      },
      warning: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.confidence.low,
        color: outlined ? designTokens.colors.confidence.low : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.confidence.low}`,
      },
      error: {
        backgroundColor: outlined ? 'transparent' : designTokens.colors.confidence.critical,
        color: outlined
          ? designTokens.colors.confidence.critical
          : designTokens.colors.surface.primary,
        border: `1px solid ${designTokens.colors.confidence.critical}`,
      },
    }

    return variants[variant]
  }

  const baseStyles = {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontFamily: designTokens.typography.fonts.sans.join(', '),
    textAlign: 'center' as const,
    whiteSpace: 'nowrap' as const,
    verticalAlign: 'baseline',
    userSelect: 'none' as const,
    position: 'relative' as const,
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    width: dot ? sizeStyles[size].minWidth : 'auto',
  }

  const combinedStyles = {
    ...baseStyles,
    ...sizeStyles[size],
    ...shapeStyles[shape],
    ...getVariantStyles(),
    ...style,
  }

  // Pulsing animation
  const pulsingStyles = pulsing
    ? {
        animation: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      }
    : {}

  return (
    <>
      {pulsing && (
        <style>
          {`
            @keyframes pulse {
              0%, 100% {
                opacity: 1;
              }
              50% {
                opacity: 0.5;
              }
            }
          `}
        </style>
      )}

      <span
        className={`fiovana-badge ${className}`}
        style={{ ...combinedStyles, ...pulsingStyles }}
        {...props}
      >
        {!dot && children}
      </span>
    </>
  )
}

// Specialized badge components for common use cases
export const StatusBadge: React.FC<
  Omit<BadgeProps, 'variant'> & { status: 'online' | 'offline' | 'away' | 'busy' }
> = ({ status, ...props }) => {
  const statusVariants = {
    online: { variant: 'success' as const, children: 'Online' },
    offline: { variant: 'default' as const, children: 'Offline' },
    away: { variant: 'warning' as const, children: 'Away' },
    busy: { variant: 'error' as const, children: 'Busy' },
  }

  return <Badge {...props} {...statusVariants[status]} />
}

export const ConfidenceBadge: React.FC<
  Omit<BadgeProps, 'variant' | 'children'> & { confidence: number }
> = ({ confidence, ...props }) => {
  const getConfidenceVariant = () => {
    if (confidence >= 80) return 'success'
    if (confidence >= 60) return 'confidence'
    if (confidence >= 40) return 'warning'
    return 'error'
  }

  return (
    <Badge {...props} variant={getConfidenceVariant() as BadgeProps['variant']}>
      {Math.round(confidence)}%
    </Badge>
  )
}

export const AIStatusBadge: React.FC<
  Omit<BadgeProps, 'variant' | 'children'> & { thinking?: boolean }
> = ({ thinking = false, ...props }) => (
  <Badge {...props} variant="ai" pulsing={thinking} dot size="sm" />
)

export default Badge
