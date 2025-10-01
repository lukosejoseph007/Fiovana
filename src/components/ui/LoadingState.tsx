import React from 'react'
import { designTokens } from '../../styles/tokens'
import Icon from './Icon'
import Progress from './Progress'

export interface LoadingStateProps {
  variant?: 'spinner' | 'skeleton' | 'progress' | 'ai'
  message?: string
  progress?: number
  details?: string
  size?: 'sm' | 'md' | 'lg'
  className?: string
  style?: React.CSSProperties
}

const LoadingState: React.FC<LoadingStateProps> = ({
  variant = 'spinner',
  message,
  progress,
  details,
  size = 'md',
  className = '',
  style,
}) => {
  const sizeStyles = {
    sm: {
      spinnerSize: 20,
      fontSize: designTokens.typography.fontSize.sm,
      spacing: designTokens.spacing[2],
    },
    md: {
      spinnerSize: 32,
      fontSize: designTokens.typography.fontSize.base,
      spacing: designTokens.spacing[4],
    },
    lg: {
      spinnerSize: 48,
      fontSize: designTokens.typography.fontSize.lg,
      spacing: designTokens.spacing[6],
    },
  }

  const currentSize = sizeStyles[size]

  const renderSpinner = () => (
    <div
      style={{
        width: `${currentSize.spinnerSize}px`,
        height: `${currentSize.spinnerSize}px`,
        border: `3px solid ${designTokens.colors.border.subtle}`,
        borderTop: `3px solid ${designTokens.colors.accent.ai}`,
        borderRadius: '50%',
        animation: 'spin 1s linear infinite',
      }}
    />
  )

  const renderAIThinking = () => (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: designTokens.spacing[2],
      }}
    >
      <Icon
        name="Cpu"
        size={currentSize.spinnerSize}
        color={designTokens.colors.accent.ai}
        style={{
          animation: 'pulse 2s ease-in-out infinite',
        }}
      />
      <div
        style={{
          display: 'flex',
          gap: designTokens.spacing[1],
        }}
      >
        {[0, 1, 2].map(i => (
          <div
            key={i}
            style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor: designTokens.colors.accent.ai,
              animation: `bounce 1.4s ease-in-out ${i * 0.16}s infinite`,
            }}
          />
        ))}
      </div>
    </div>
  )

  const renderSkeleton = () => (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: designTokens.spacing[3],
        width: '100%',
      }}
    >
      {[0, 1, 2].map(i => (
        <div
          key={i}
          style={{
            height: size === 'sm' ? '12px' : size === 'md' ? '16px' : '20px',
            background: `linear-gradient(90deg, ${designTokens.colors.surface.tertiary} 25%, ${designTokens.colors.surface.quaternary} 50%, ${designTokens.colors.surface.tertiary} 75%)`,
            backgroundSize: '200% 100%',
            borderRadius: designTokens.borderRadius.md,
            animation: 'shimmer 1.5s ease-in-out infinite',
            width: i === 2 ? '60%' : '100%',
          }}
        />
      ))}
    </div>
  )

  const renderProgress = () => (
    <div style={{ width: '100%', maxWidth: '400px' }}>
      <Progress value={progress || 0} animated={true} />
    </div>
  )

  const renderContent = () => {
    switch (variant) {
      case 'ai':
        return renderAIThinking()
      case 'skeleton':
        return renderSkeleton()
      case 'progress':
        return renderProgress()
      case 'spinner':
      default:
        return renderSpinner()
    }
  }

  return (
    <>
      <style>
        {`
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }

          @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
          }

          @keyframes bounce {
            0%, 80%, 100% { transform: translateY(0); }
            40% { transform: translateY(-10px); }
          }

          @keyframes shimmer {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
          }
        `}
      </style>

      <div
        className={`proxemic-loading-state ${className}`}
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: currentSize.spacing,
          padding: currentSize.spacing,
          ...style,
        }}
      >
        {renderContent()}

        {message && (
          <div
            style={{
              fontSize: currentSize.fontSize,
              fontWeight: designTokens.typography.fontWeight.medium,
              color: designTokens.colors.text.primary,
              textAlign: 'center',
            }}
          >
            {message}
          </div>
        )}

        {details && (
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
              textAlign: 'center',
              maxWidth: '300px',
            }}
          >
            {details}
          </div>
        )}

        {variant === 'progress' && progress !== undefined && (
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
            }}
          >
            {Math.round(progress)}% complete
          </div>
        )}
      </div>
    </>
  )
}

// Skeleton Component for specific use cases
export interface SkeletonProps {
  variant?: 'text' | 'rect' | 'circle' | 'document'
  count?: number
  width?: string | number
  height?: string | number
  className?: string
  style?: React.CSSProperties
}

export const Skeleton: React.FC<SkeletonProps> = ({
  variant = 'text',
  count = 1,
  width,
  height,
  className = '',
  style,
}) => {
  const getVariantStyles = () => {
    switch (variant) {
      case 'circle':
        return {
          width: width || '40px',
          height: height || '40px',
          borderRadius: '50%',
        }
      case 'rect':
        return {
          width: width || '100%',
          height: height || '100px',
          borderRadius: designTokens.borderRadius.md,
        }
      case 'document':
        return {
          width: width || '100%',
          height: height || '200px',
          borderRadius: designTokens.borderRadius.lg,
        }
      case 'text':
      default:
        return {
          width: width || '100%',
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
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
          }
        `}
      </style>

      <div className={className} style={style}>
        {Array.from({ length: count }).map((_, index) => (
          <div
            key={index}
            style={{
              ...variantStyles,
              background: `linear-gradient(90deg, ${designTokens.colors.surface.tertiary} 25%, ${designTokens.colors.surface.quaternary} 50%, ${designTokens.colors.surface.tertiary} 75%)`,
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

// Progress Card for operations
export interface ProgressCardProps {
  operation: string
  progress?: number
  details?: string
  onCancel?: () => void
  variant?: 'default' | 'ai'
  className?: string
  style?: React.CSSProperties
}

export const ProgressCard: React.FC<ProgressCardProps> = ({
  operation,
  progress,
  details,
  onCancel,
  variant = 'default',
  className = '',
  style,
}) => {
  return (
    <div
      className={`proxemic-progress-card ${className}`}
      style={{
        padding: designTokens.spacing[4],
        background: designTokens.colors.surface.secondary,
        border: `1px solid ${designTokens.colors.border.subtle}`,
        borderRadius: designTokens.borderRadius.lg,
        boxShadow: designTokens.shadows.lg,
        ...style,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: designTokens.spacing[3],
          marginBottom: designTokens.spacing[3],
        }}
      >
        {variant === 'ai' ? (
          <Icon
            name="Cpu"
            size={20}
            color={designTokens.colors.accent.ai}
            style={{ animation: 'pulse 2s ease-in-out infinite' }}
          />
        ) : (
          <div
            style={{
              width: '20px',
              height: '20px',
              border: `2px solid ${designTokens.colors.border.subtle}`,
              borderTop: `2px solid ${designTokens.colors.accent.ai}`,
              borderRadius: '50%',
              animation: 'spin 1s linear infinite',
            }}
          />
        )}

        <div style={{ flex: 1 }}>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.base,
              fontWeight: designTokens.typography.fontWeight.semibold,
              color: designTokens.colors.text.primary,
              marginBottom: designTokens.spacing[0.5],
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
              padding: designTokens.spacing[1],
              background: 'transparent',
              border: 'none',
              cursor: 'pointer',
              color: designTokens.colors.text.tertiary,
              borderRadius: designTokens.borderRadius.sm,
              transition: `all ${designTokens.animation.duration.fast}`,
            }}
            title="Cancel"
          >
            <Icon name="X" size={16} />
          </button>
        )}
      </div>

      {progress !== undefined && (
        <div>
          <Progress value={progress} />
          <div
            style={{
              marginTop: designTokens.spacing[1],
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.tertiary,
              textAlign: 'right',
            }}
          >
            {Math.round(progress)}%
          </div>
        </div>
      )}

      <style>
        {`
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }

          @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
          }
        `}
      </style>
    </div>
  )
}

export default LoadingState
