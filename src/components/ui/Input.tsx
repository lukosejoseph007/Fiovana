import React, { forwardRef } from 'react';
import { designTokens } from '../../styles/tokens';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  variant?: 'default' | 'command' | 'search';
  size?: 'sm' | 'md' | 'lg';
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  leftAddon?: React.ReactNode;
  rightAddon?: React.ReactNode;
  error?: string;
  isInvalid?: boolean;
  fullWidth?: boolean;
}

const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      variant = 'default',
      size = 'md',
      leftIcon,
      rightIcon,
      leftAddon,
      rightAddon,
      error,
      isInvalid = false,
      fullWidth = false,
      className = '',
      disabled,
      ...props
    },
    ref
  ) => {
    const hasError = isInvalid || !!error;

    const containerStyles = {
      display: 'inline-flex',
      alignItems: 'center',
      position: 'relative' as const,
      width: fullWidth ? '100%' : 'auto',
    };

    const baseInputStyles = {
      fontFamily: designTokens.typography.fonts.sans.join(', '),
      border: `1px solid ${hasError ? designTokens.colors.accent.alert : designTokens.colors.border.subtle}`,
      borderRadius: designTokens.borderRadius.md,
      backgroundColor: variant === 'command' ? designTokens.colors.surface.primary : designTokens.colors.surface.secondary,
      color: designTokens.colors.text.primary,
      outline: 'none',
      transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
      width: '100%',
      opacity: disabled ? 0.5 : 1,
      cursor: disabled ? 'not-allowed' : 'text',
    };

    // Size variants
    const sizeStyles = {
      sm: {
        height: '32px',
        padding: leftIcon || leftAddon ? `0 ${designTokens.spacing[3]} 0 ${designTokens.spacing[8]}` :
                 rightIcon || rightAddon ? `0 ${designTokens.spacing[8]} 0 ${designTokens.spacing[3]}` :
                 `0 ${designTokens.spacing[3]}`,
        fontSize: designTokens.typography.fontSize.sm,
      },
      md: {
        height: '40px',
        padding: leftIcon || leftAddon ? `0 ${designTokens.spacing[4]} 0 ${designTokens.spacing[10]}` :
                 rightIcon || rightAddon ? `0 ${designTokens.spacing[10]} 0 ${designTokens.spacing[4]}` :
                 `0 ${designTokens.spacing[4]}`,
        fontSize: designTokens.typography.fontSize.base,
      },
      lg: {
        height: '48px',
        padding: leftIcon || leftAddon ? `0 ${designTokens.spacing[5]} 0 ${designTokens.spacing[12]}` :
                 rightIcon || rightAddon ? `0 ${designTokens.spacing[12]} 0 ${designTokens.spacing[5]}` :
                 `0 ${designTokens.spacing[5]}`,
        fontSize: designTokens.typography.fontSize.lg,
      },
    };

    const inputStyles = {
      ...baseInputStyles,
      ...sizeStyles[size],
    };

    // Apply variant-specific styles for command palette and search
    if (variant === 'command') {
      inputStyles.backgroundColor = designTokens.colors.surface.primary;
      inputStyles.border = `1px solid ${designTokens.colors.border.medium}`;
      inputStyles.boxShadow = designTokens.shadows.glassSubtle;
    } else if (variant === 'search') {
      inputStyles.borderRadius = designTokens.borderRadius.xl;
      inputStyles.backgroundColor = designTokens.colors.surface.tertiary;
    }

    const iconStyles = {
      position: 'absolute' as const,
      top: '50%',
      transform: 'translateY(-50%)',
      color: designTokens.colors.text.secondary,
      pointerEvents: 'none' as const,
      zIndex: 1,
    };

    const leftIconStyles = {
      ...iconStyles,
      left: size === 'sm' ? designTokens.spacing[2] :
           size === 'md' ? designTokens.spacing[3] :
           designTokens.spacing[4],
    };

    const rightIconStyles = {
      ...iconStyles,
      right: size === 'sm' ? designTokens.spacing[2] :
            size === 'md' ? designTokens.spacing[3] :
            designTokens.spacing[4],
    };

    const addonStyles = {
      display: 'flex',
      alignItems: 'center',
      padding: `0 ${designTokens.spacing[3]}`,
      backgroundColor: designTokens.colors.surface.tertiary,
      border: `1px solid ${designTokens.colors.border.subtle}`,
      color: designTokens.colors.text.secondary,
      fontSize: designTokens.typography.fontSize.sm,
      whiteSpace: 'nowrap' as const,
    };

    const leftAddonStyles = {
      ...addonStyles,
      borderRight: 'none',
      borderTopLeftRadius: designTokens.borderRadius.md,
      borderBottomLeftRadius: designTokens.borderRadius.md,
    };

    const rightAddonStyles = {
      ...addonStyles,
      borderLeft: 'none',
      borderTopRightRadius: designTokens.borderRadius.md,
      borderBottomRightRadius: designTokens.borderRadius.md,
    };

    // Adjust input border radius when addons are present
    if (leftAddon) {
      inputStyles.borderTopLeftRadius = 0;
      inputStyles.borderBottomLeftRadius = 0;
    }
    if (rightAddon) {
      inputStyles.borderTopRightRadius = 0;
      inputStyles.borderBottomRightRadius = 0;
    }

    return (
      <div style={{ width: fullWidth ? '100%' : 'auto' }}>
        <div style={containerStyles}>
          {leftAddon && <div style={leftAddonStyles}>{leftAddon}</div>}
          {leftIcon && <div style={leftIconStyles}>{leftIcon}</div>}

          <style>
            {`
              .proxemic-input:hover {
                ${!disabled && variant === 'default' ? `
                  border-color: ${designTokens.colors.border.medium};
                ` : ''}
                ${!disabled && variant === 'command' ? `
                  border-color: ${designTokens.colors.accent.ai};
                  box-shadow: ${designTokens.shadows.glassMedium};
                ` : ''}
                ${!disabled && variant === 'search' ? `
                  background-color: ${designTokens.colors.surface.quaternary};
                  border-color: ${designTokens.colors.border.medium};
                ` : ''}
              }

              .proxemic-input:focus {
                outline: none;
                ${variant === 'default' ? `
                  border-color: ${designTokens.colors.state.focus};
                  box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
                ` : ''}
                ${variant === 'command' ? `
                  border-color: ${designTokens.colors.accent.ai};
                  box-shadow: 0 0 0 3px ${designTokens.colors.accent.ai}40;
                  background-color: ${designTokens.colors.surface.secondary};
                ` : ''}
                ${variant === 'search' ? `
                  border-color: ${designTokens.colors.state.focus};
                  background-color: ${designTokens.colors.surface.secondary};
                  box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
                ` : ''}
              }

              .proxemic-input::placeholder {
                color: ${designTokens.colors.text.tertiary};
                opacity: 1;
              }
            `}
          </style>

          <input
            ref={ref}
            className={`proxemic-input ${className}`}
            style={inputStyles}
            disabled={disabled}
            {...props}
          />

          {rightIcon && <div style={rightIconStyles}>{rightIcon}</div>}
          {rightAddon && <div style={rightAddonStyles}>{rightAddon}</div>}
        </div>

        {error && (
          <div
            style={{
              marginTop: designTokens.spacing[1],
              color: designTokens.colors.accent.alert,
              fontSize: designTokens.typography.fontSize.sm,
              lineHeight: designTokens.typography.lineHeight.tight,
            }}
          >
            {error}
          </div>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

export default Input;