import React from 'react'
import { designTokens } from '../../styles/tokens'
import Icon from './Icon'
import Button from './Button'
import type { IconComponentProps } from './Icon'

export interface EmptyStateAction {
  label: string
  onClick: () => void
  variant?: 'primary' | 'secondary' | 'ghost'
}

export interface EmptyStateProps {
  icon?: IconComponentProps['name']
  title: string
  description?: string
  actions?: EmptyStateAction[]
  illustration?: React.ReactNode
  className?: string
  style?: React.CSSProperties
  size?: 'sm' | 'md' | 'lg'
}

const EmptyState: React.FC<EmptyStateProps> = ({
  icon,
  title,
  description,
  actions = [],
  illustration,
  className = '',
  style,
  size = 'md',
}) => {
  const sizeStyles = {
    sm: {
      iconSize: 32,
      titleSize: designTokens.typography.fontSize.base,
      descriptionSize: designTokens.typography.fontSize.sm,
      spacing: designTokens.spacing[4],
      padding: designTokens.spacing[6],
    },
    md: {
      iconSize: 48,
      titleSize: designTokens.typography.fontSize.xl,
      descriptionSize: designTokens.typography.fontSize.base,
      spacing: designTokens.spacing[6],
      padding: designTokens.spacing[8],
    },
    lg: {
      iconSize: 64,
      titleSize: designTokens.typography.fontSize['2xl'],
      descriptionSize: designTokens.typography.fontSize.lg,
      spacing: designTokens.spacing[8],
      padding: designTokens.spacing[12],
    },
  }

  const currentSize = sizeStyles[size]

  return (
    <div
      className={`proxemic-empty-state ${className}`}
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        textAlign: 'center',
        padding: currentSize.padding,
        color: designTokens.colors.text.secondary,
        ...style,
      }}
    >
      {/* Illustration or Icon */}
      {illustration ? (
        <div
          style={{
            marginBottom: currentSize.spacing,
          }}
        >
          {illustration}
        </div>
      ) : icon ? (
        <div
          style={{
            width: `${currentSize.iconSize * 1.5}px`,
            height: `${currentSize.iconSize * 1.5}px`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            marginBottom: currentSize.spacing,
            background: `linear-gradient(135deg, ${designTokens.colors.surface.tertiary}, ${designTokens.colors.surface.secondary})`,
            borderRadius: designTokens.borderRadius.full,
            border: `2px solid ${designTokens.colors.border.subtle}`,
          }}
        >
          <Icon name={icon} size={currentSize.iconSize} color={designTokens.colors.text.tertiary} />
        </div>
      ) : null}

      {/* Title */}
      <h3
        style={{
          fontSize: currentSize.titleSize,
          fontWeight: designTokens.typography.fontWeight.semibold,
          color: designTokens.colors.text.primary,
          marginBottom: description ? designTokens.spacing[2] : currentSize.spacing,
          lineHeight: designTokens.typography.lineHeight.tight,
        }}
      >
        {title}
      </h3>

      {/* Description */}
      {description && (
        <p
          style={{
            fontSize: currentSize.descriptionSize,
            color: designTokens.colors.text.secondary,
            lineHeight: designTokens.typography.lineHeight.relaxed,
            marginBottom: actions.length > 0 ? currentSize.spacing : 0,
            maxWidth: '400px',
          }}
        >
          {description}
        </p>
      )}

      {/* Actions */}
      {actions.length > 0 && (
        <div
          style={{
            display: 'flex',
            gap: designTokens.spacing[3],
            flexWrap: 'wrap',
            justifyContent: 'center',
          }}
        >
          {actions.map((action, index) => (
            <Button
              key={index}
              variant={action.variant || (index === 0 ? 'primary' : 'secondary')}
              onClick={action.onClick}
            >
              {action.label}
            </Button>
          ))}
        </div>
      )}
    </div>
  )
}

export default EmptyState
