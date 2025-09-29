import React, { forwardRef } from 'react';
import { designTokens } from '../../styles/tokens';

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'glass' | 'elevated';
  padding?: 'none' | 'sm' | 'md' | 'lg';
  hoverable?: boolean;
  clickable?: boolean;
  borderless?: boolean;
}

const Card = forwardRef<HTMLDivElement, CardProps>(
  (
    {
      variant = 'default',
      padding = 'md',
      hoverable = false,
      clickable = false,
      borderless = false,
      children,
      className = '',
      style,
      ...props
    },
    ref
  ) => {
    const baseStyles = {
      borderRadius: designTokens.borderRadius.lg,
      transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
      position: 'relative' as const,
      overflow: 'hidden' as const,
      cursor: clickable ? 'pointer' : 'default',
    };

    // Padding variants
    const paddingStyles = {
      none: { padding: '0' },
      sm: { padding: designTokens.spacing[3] },
      md: { padding: designTokens.spacing[4] },
      lg: { padding: designTokens.spacing[6] },
    };

    // Variant styles
    const variantStyles = {
      default: {
        backgroundColor: designTokens.variants.card.default.background,
        border: borderless ? 'none' : `1px solid ${designTokens.variants.card.default.border}`,
        boxShadow: designTokens.variants.card.default.shadow,
      },
      glass: {
        backgroundColor: designTokens.variants.card.glass.background,
        backdropFilter: designTokens.variants.card.glass.backdropFilter,
        border: borderless ? 'none' : `1px solid ${designTokens.variants.card.glass.border}`,
        boxShadow: designTokens.variants.card.glass.shadow,
      },
      elevated: {
        backgroundColor: designTokens.variants.card.elevated.background,
        border: borderless ? 'none' : `1px solid ${designTokens.variants.card.elevated.border}`,
        boxShadow: designTokens.variants.card.elevated.shadow,
      },
    };

    const combinedStyles = {
      ...baseStyles,
      ...paddingStyles[padding],
      ...variantStyles[variant],
      ...style,
    };

    const hoverStyles = hoverable || clickable ? {
      transform: 'translateY(-2px)',
      boxShadow: variant === 'glass' ? designTokens.shadows.glassStrong : designTokens.shadows.lg,
    } : {};

    const activeStyles = clickable ? {
      transform: 'translateY(0px)',
      boxShadow: variant === 'glass' ? designTokens.shadows.glassMedium : designTokens.shadows.md,
    } : {};

    return (
      <>
        <style>
          {`
            .proxemic-card {
              ${(hoverable || clickable) ? `
                &:hover {
                  transform: translateY(-2px);
                  box-shadow: ${variant === 'glass' ? designTokens.shadows.glassStrong : designTokens.shadows.lg};
                }
              ` : ''}

              ${clickable ? `
                &:active {
                  transform: translateY(0px);
                  box-shadow: ${variant === 'glass' ? designTokens.shadows.glassMedium : designTokens.shadows.md};
                }

                &:focus {
                  outline: none;
                  box-shadow: ${variant === 'glass' ? designTokens.shadows.glassStrong : designTokens.shadows.lg},
                             0 0 0 3px ${designTokens.colors.state.focus}40;
                }
              ` : ''}
            }

            .proxemic-card-hoverable:hover {
              transform: translateY(-2px);
              box-shadow: ${variant === 'glass' ? designTokens.shadows.glassStrong : designTokens.shadows.lg};
            }

            .proxemic-card-clickable:hover {
              transform: translateY(-2px);
              box-shadow: ${variant === 'glass' ? designTokens.shadows.glassStrong : designTokens.shadows.lg};
            }

            .proxemic-card-clickable:active {
              transform: translateY(0px);
              box-shadow: ${variant === 'glass' ? designTokens.shadows.glassMedium : designTokens.shadows.md};
            }

            .proxemic-card-clickable:focus {
              outline: none;
              box-shadow: ${variant === 'glass' ? designTokens.shadows.glassStrong : designTokens.shadows.lg},
                         0 0 0 3px ${designTokens.colors.state.focus}40;
            }
          `}
        </style>
        <div
          ref={ref}
          className={`proxemic-card ${hoverable ? 'proxemic-card-hoverable' : ''} ${clickable ? 'proxemic-card-clickable' : ''} ${className}`}
          style={combinedStyles}
          tabIndex={clickable ? 0 : undefined}
          role={clickable ? 'button' : undefined}
          {...props}
        >
          {children}
        </div>
      </>
    );
  }
);

Card.displayName = 'Card';

// Card compound components for better composition
export const CardHeader = forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ children, className = '', style, ...props }, ref) => (
    <div
      ref={ref}
      className={`proxemic-card-header ${className}`}
      style={{
        padding: `${designTokens.spacing[4]} ${designTokens.spacing[4]} 0 ${designTokens.spacing[4]}`,
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  )
);

CardHeader.displayName = 'CardHeader';

export const CardBody = forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ children, className = '', style, ...props }, ref) => (
    <div
      ref={ref}
      className={`proxemic-card-body ${className}`}
      style={{
        padding: designTokens.spacing[4],
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  )
);

CardBody.displayName = 'CardBody';

export const CardFooter = forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ children, className = '', style, ...props }, ref) => (
    <div
      ref={ref}
      className={`proxemic-card-footer ${className}`}
      style={{
        padding: `0 ${designTokens.spacing[4]} ${designTokens.spacing[4]} ${designTokens.spacing[4]}`,
        marginTop: 'auto',
        borderTop: `1px solid ${designTokens.colors.border.subtle}`,
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  )
);

CardFooter.displayName = 'CardFooter';

export default Card;