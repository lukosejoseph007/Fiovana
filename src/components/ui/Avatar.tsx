import React from 'react'
import { designTokens } from '../../styles/tokens'

export interface AvatarProps {
  src?: string
  alt?: string
  name?: string
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl'
  variant?: 'circular' | 'rounded' | 'square'
  status?: 'online' | 'offline' | 'away' | 'busy'
  showStatus?: boolean
  fallbackIcon?: React.ReactNode
  className?: string
  style?: React.CSSProperties
  onClick?: () => void
}

const Avatar: React.FC<AvatarProps> = ({
  src,
  alt,
  name,
  size = 'md',
  variant = 'circular',
  status,
  showStatus = false,
  fallbackIcon,
  className = '',
  style,
  onClick,
}) => {
  // Size variants
  const sizeStyles = {
    xs: {
      width: '24px',
      height: '24px',
      fontSize: designTokens.typography.fontSize.xs,
    },
    sm: {
      width: '32px',
      height: '32px',
      fontSize: designTokens.typography.fontSize.sm,
    },
    md: {
      width: '40px',
      height: '40px',
      fontSize: designTokens.typography.fontSize.base,
    },
    lg: {
      width: '48px',
      height: '48px',
      fontSize: designTokens.typography.fontSize.lg,
    },
    xl: {
      width: '64px',
      height: '64px',
      fontSize: designTokens.typography.fontSize.xl,
    },
    '2xl': {
      width: '80px',
      height: '80px',
      fontSize: designTokens.typography.fontSize['2xl'],
    },
  }

  // Variant styles
  const variantStyles = {
    circular: { borderRadius: designTokens.borderRadius.full },
    rounded: { borderRadius: designTokens.borderRadius.lg },
    square: { borderRadius: designTokens.borderRadius.md },
  }

  // Status colors
  const statusColors = {
    online: designTokens.colors.confidence.high,
    offline: designTokens.colors.text.tertiary,
    away: designTokens.colors.confidence.medium,
    busy: designTokens.colors.confidence.critical,
  }

  // Generate initials from name
  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(word => word.charAt(0))
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  // Generate background color from name
  const getBackgroundColor = (name: string) => {
    if (!name) return designTokens.colors.surface.tertiary

    const colors = [
      designTokens.colors.accent.ai,
      designTokens.colors.accent.semantic,
      designTokens.colors.confidence.medium,
      designTokens.colors.accent.info,
      designTokens.colors.confidence.low,
    ]

    let hash = 0
    for (let i = 0; i < name.length; i++) {
      hash = name.charCodeAt(i) + ((hash << 5) - hash)
    }

    return colors[Math.abs(hash) % colors.length]
  }

  const baseStyles = {
    position: 'relative' as const,
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: src ? 'transparent' : getBackgroundColor(name || ''),
    color: designTokens.colors.surface.primary,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
    fontWeight: designTokens.typography.fontWeight.semibold,
    border: `2px solid ${designTokens.colors.surface.secondary}`,
    overflow: 'hidden' as const,
    cursor: onClick ? 'pointer' : 'default',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    userSelect: 'none' as const,
    flexShrink: 0,
    ...sizeStyles[size],
    ...variantStyles[variant],
    ...style,
  }

  const imageStyles = {
    width: '100%',
    height: '100%',
    objectFit: 'cover' as const,
    objectPosition: 'center',
  }

  const statusIndicatorSize = {
    xs: '8px',
    sm: '10px',
    md: '12px',
    lg: '14px',
    xl: '16px',
    '2xl': '20px',
  }

  const statusStyles = {
    position: 'absolute' as const,
    bottom: '0',
    right: '0',
    width: statusIndicatorSize[size],
    height: statusIndicatorSize[size],
    backgroundColor: status ? statusColors[status] : designTokens.colors.text.tertiary,
    border: `2px solid ${designTokens.colors.surface.secondary}`,
    borderRadius: designTokens.borderRadius.full,
    transform: 'translate(25%, 25%)',
  }

  const DefaultIcon = () => (
    <svg
      width="60%"
      height="60%"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
      <circle cx="12" cy="7" r="4" />
    </svg>
  )

  return (
    <>
      <style>
        {`
          .fiovana-avatar-clickable:hover {
            transform: scale(1.05);
            box-shadow: ${designTokens.shadows.md};
          }

          .fiovana-avatar-clickable:focus {
            outline: none;
            box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
          }

          .fiovana-avatar-clickable:active {
            transform: scale(0.95);
          }
        `}
      </style>

      <div
        className={`fiovana-avatar ${onClick ? 'fiovana-avatar-clickable' : ''} ${className}`}
        style={baseStyles}
        onClick={onClick}
        tabIndex={onClick ? 0 : undefined}
        role={onClick ? 'button' : undefined}
        aria-label={alt || (name ? `Avatar for ${name}` : 'Avatar')}
      >
        {src ? (
          <img
            src={src}
            alt={alt || (name ? `Avatar for ${name}` : 'Avatar')}
            style={imageStyles}
            onError={e => {
              // Hide image on error to show fallback
              ;(e.target as HTMLImageElement).style.display = 'none'
            }}
          />
        ) : name ? (
          getInitials(name)
        ) : fallbackIcon ? (
          fallbackIcon
        ) : (
          <DefaultIcon />
        )}

        {showStatus && status && <div style={statusStyles} aria-label={`Status: ${status}`} />}
      </div>
    </>
  )
}

// Avatar Group for displaying multiple avatars
export interface AvatarGroupProps {
  avatars: Array<Omit<AvatarProps, 'size'> & { id: string }>
  size?: AvatarProps['size']
  max?: number
  spacing?: 'tight' | 'normal' | 'loose'
  className?: string
}

export const AvatarGroup: React.FC<AvatarGroupProps> = ({
  avatars,
  size = 'md',
  max = 4,
  spacing = 'normal',
  className = '',
}) => {
  const visibleAvatars = avatars.slice(0, max)
  const remainingCount = Math.max(0, avatars.length - max)

  const spacingValues = {
    tight: '-8px',
    normal: '-4px',
    loose: '4px',
  }

  const groupStyles = {
    display: 'flex',
    alignItems: 'center',
  }

  const avatarWrapperStyles = {
    marginLeft: spacingValues[spacing],
    position: 'relative' as const,
    zIndex: 1,
  }

  const remainingStyles = {
    ...avatarWrapperStyles,
    zIndex: 0,
  }

  return (
    <div className={`fiovana-avatar-group ${className}`} style={groupStyles}>
      {visibleAvatars.map((avatar, index) => (
        <div
          key={avatar.id}
          style={{
            ...avatarWrapperStyles,
            zIndex: visibleAvatars.length - index,
            marginLeft: index === 0 ? '0' : spacingValues[spacing],
          }}
        >
          <Avatar {...avatar} size={size} />
        </div>
      ))}

      {remainingCount > 0 && (
        <div style={remainingStyles}>
          <Avatar
            name={`+${remainingCount}`}
            size={size}
            style={{
              backgroundColor: designTokens.colors.surface.tertiary,
              color: designTokens.colors.text.primary,
            }}
          />
        </div>
      )}
    </div>
  )
}

export default Avatar
