import React, { forwardRef } from 'react'
import { designTokens } from '../../styles/tokens'

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'minimal' | 'success' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  isLoading?: boolean
  leftIcon?: React.ReactNode
  rightIcon?: React.ReactNode
  fullWidth?: boolean
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      isLoading = false,
      leftIcon,
      rightIcon,
      fullWidth = false,
      children,
      className = '',
      disabled,
      ...props
    },
    ref
  ) => {
    const baseStyles = {
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      gap: designTokens.spacing[2],
      fontFamily: designTokens.typography.fonts.sans.join(', '),
      fontWeight: designTokens.typography.fontWeight.medium,
      borderRadius: designTokens.borderRadius.md,
      border: 'none',
      cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
      transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
      outline: 'none',
      position: 'relative' as const,
      textDecoration: 'none',
      userSelect: 'none' as const,
      width: fullWidth ? '100%' : 'auto',
      opacity: disabled || isLoading ? 0.5 : 1,
    }

    // Size variants
    const sizeStyles = {
      sm: {
        height: '32px',
        padding: `0 ${designTokens.spacing[3]}`,
        fontSize: designTokens.typography.fontSize.sm,
        lineHeight: designTokens.typography.lineHeight.tight,
      },
      md: {
        height: '40px',
        padding: `0 ${designTokens.spacing[4]}`,
        fontSize: designTokens.typography.fontSize.base,
        lineHeight: designTokens.typography.lineHeight.normal,
      },
      lg: {
        height: '48px',
        padding: `0 ${designTokens.spacing[6]}`,
        fontSize: designTokens.typography.fontSize.lg,
        lineHeight: designTokens.typography.lineHeight.normal,
      },
    }

    // Variant styles
    const variantStyles = {
      primary: {
        backgroundColor: designTokens.variants.button.primary.background,
        color: designTokens.variants.button.primary.color,
        boxShadow: designTokens.shadows.base,
        '&:hover':
          !disabled && !isLoading
            ? {
                backgroundColor: designTokens.variants.button.primary.hover,
                transform: `translateY(-1px)`,
                boxShadow: designTokens.shadows.md,
              }
            : {},
        '&:focus': {
          boxShadow: `0 0 0 3px ${designTokens.colors.state.focus}40`,
        },
        '&:active':
          !disabled && !isLoading
            ? {
                transform: `translateY(0)`,
                backgroundColor: designTokens.variants.button.primary.hover,
              }
            : {},
      },
      secondary: {
        backgroundColor: designTokens.variants.button.secondary.background,
        color: designTokens.variants.button.secondary.color,
        border: `1px solid ${designTokens.colors.border.subtle}`,
        '&:hover':
          !disabled && !isLoading
            ? {
                backgroundColor: designTokens.variants.button.secondary.hover,
                border: `1px solid ${designTokens.colors.border.medium}`,
                transform: `translateY(-1px)`,
                boxShadow: designTokens.shadows.sm,
              }
            : {},
        '&:focus': {
          border: `1px solid ${designTokens.colors.state.focus}`,
          boxShadow: `0 0 0 3px ${designTokens.colors.state.focus}40`,
        },
      },
      ghost: {
        backgroundColor: designTokens.variants.button.ghost.background,
        color: designTokens.variants.button.ghost.color,
        '&:hover':
          !disabled && !isLoading
            ? {
                backgroundColor: designTokens.variants.button.ghost.hover,
                color: designTokens.colors.text.primary,
              }
            : {},
        '&:focus': {
          backgroundColor: designTokens.variants.button.ghost.hover,
          boxShadow: `0 0 0 3px ${designTokens.colors.state.focus}40`,
        },
      },
      minimal: {
        backgroundColor: designTokens.variants.button.minimal.background,
        color: designTokens.variants.button.minimal.color,
        padding: `0 ${designTokens.spacing[2]}`,
        '&:hover':
          !disabled && !isLoading
            ? {
                backgroundColor: designTokens.variants.button.minimal.hover,
              }
            : {},
        '&:focus': {
          backgroundColor: designTokens.variants.button.minimal.hover,
          boxShadow: `0 0 0 2px ${designTokens.colors.state.focus}40`,
        },
      },
      success: {
        backgroundColor: '#10b981',
        color: '#ffffff',
        boxShadow: designTokens.shadows.base,
        '&:hover':
          !disabled && !isLoading
            ? {
                backgroundColor: '#059669',
                transform: `translateY(-1px)`,
                boxShadow: designTokens.shadows.md,
              }
            : {},
        '&:focus': {
          boxShadow: `0 0 0 3px rgba(16, 185, 129, 0.4)`,
        },
        '&:active':
          !disabled && !isLoading
            ? {
                transform: `translateY(0)`,
                backgroundColor: '#047857',
              }
            : {},
      },
      danger: {
        backgroundColor: '#ef4444',
        color: '#ffffff',
        boxShadow: designTokens.shadows.base,
        '&:hover':
          !disabled && !isLoading
            ? {
                backgroundColor: '#dc2626',
                transform: `translateY(-1px)`,
                boxShadow: designTokens.shadows.md,
              }
            : {},
        '&:focus': {
          boxShadow: `0 0 0 3px rgba(239, 68, 68, 0.4)`,
        },
        '&:active':
          !disabled && !isLoading
            ? {
                transform: `translateY(0)`,
                backgroundColor: '#b91c1c',
              }
            : {},
      },
    }

    const combinedStyles = {
      ...baseStyles,
      ...sizeStyles[size],
      ...variantStyles[variant],
    }

    const LoadingSpinner = () => (
      <div
        style={{
          width: '16px',
          height: '16px',
          border: '2px solid transparent',
          borderTop: `2px solid currentColor`,
          borderRadius: '50%',
          animation: 'spin 1s linear infinite',
        }}
      />
    )

    return (
      <>
        <style>
          {`
            @keyframes spin {
              0% { transform: rotate(0deg); }
              100% { transform: rotate(360deg); }
            }

            .fiovana-button:hover {
              ${
                !disabled && !isLoading && variant === 'primary'
                  ? `
                background-color: ${designTokens.variants.button.primary.hover};
                transform: translateY(-1px);
                box-shadow: ${designTokens.shadows.md};
              `
                  : ''
              }
              ${
                !disabled && !isLoading && variant === 'secondary'
                  ? `
                background-color: ${designTokens.variants.button.secondary.hover};
                border-color: ${designTokens.colors.border.medium};
                transform: translateY(-1px);
                box-shadow: ${designTokens.shadows.sm};
              `
                  : ''
              }
              ${
                !disabled && !isLoading && variant === 'ghost'
                  ? `
                background-color: ${designTokens.variants.button.ghost.hover};
                color: ${designTokens.colors.text.primary};
              `
                  : ''
              }
              ${
                !disabled && !isLoading && variant === 'minimal'
                  ? `
                background-color: ${designTokens.variants.button.minimal.hover};
              `
                  : ''
              }
              ${
                !disabled && !isLoading && variant === 'success'
                  ? `
                background-color: #059669;
                transform: translateY(-1px);
                box-shadow: ${designTokens.shadows.md};
              `
                  : ''
              }
              ${
                !disabled && !isLoading && variant === 'danger'
                  ? `
                background-color: #dc2626;
                transform: translateY(-1px);
                box-shadow: ${designTokens.shadows.md};
              `
                  : ''
              }
            }

            .fiovana-button:focus {
              outline: none;
              ${
                variant === 'primary'
                  ? `
                box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
              `
                  : ''
              }
              ${
                variant === 'secondary'
                  ? `
                border-color: ${designTokens.colors.state.focus};
                box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
              `
                  : ''
              }
              ${
                variant === 'ghost'
                  ? `
                background-color: ${designTokens.variants.button.ghost.hover};
                box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
              `
                  : ''
              }
              ${
                variant === 'minimal'
                  ? `
                background-color: ${designTokens.variants.button.minimal.hover};
                box-shadow: 0 0 0 2px ${designTokens.colors.state.focus}40;
              `
                  : ''
              }
              ${
                variant === 'success'
                  ? `
                box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.4);
              `
                  : ''
              }
              ${
                variant === 'danger'
                  ? `
                box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.4);
              `
                  : ''
              }
            }

            .fiovana-button:active {
              ${
                !disabled && !isLoading && variant === 'primary'
                  ? `
                transform: translateY(0);
                background-color: ${designTokens.variants.button.primary.hover};
              `
                  : ''
              }
            }
          `}
        </style>
        <button
          ref={ref}
          className={`fiovana-button ${className}`}
          style={combinedStyles}
          disabled={disabled || isLoading}
          {...props}
        >
          {isLoading ? (
            <LoadingSpinner />
          ) : (
            <>
              {leftIcon && (
                <span style={{ display: 'flex', alignItems: 'center' }}>{leftIcon}</span>
              )}
              {children && <span>{children}</span>}
              {rightIcon && (
                <span style={{ display: 'flex', alignItems: 'center' }}>{rightIcon}</span>
              )}
            </>
          )}
        </button>
      </>
    )
  }
)

Button.displayName = 'Button'

export default Button
