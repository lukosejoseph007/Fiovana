import React, { useState, useCallback, useRef, useEffect } from 'react'
import { colors, spacing, shadows, animation } from '../../styles/tokens'
import Icon from './Icon'
import Tooltip from './Tooltip'

export interface FloatingAction {
  id: string
  label: string
  icon: string
  description?: string
  badge?: number
  onClick: () => void
  disabled?: boolean
  variant?: 'primary' | 'secondary' | 'danger'
}

interface FloatingActionButtonProps {
  actions: FloatingAction[]
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left'
  mainIcon?: string
  mainLabel?: string
  size?: 'sm' | 'md' | 'lg'
  hideWhenEmpty?: boolean
}

export const FloatingActionButton: React.FC<FloatingActionButtonProps> = ({
  actions,
  position = 'bottom-right',
  mainIcon = 'Zap',
  mainLabel = 'Actions',
  size = 'md',
  hideWhenEmpty = false,
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)
  const buttonRef = useRef<HTMLButtonElement>(null)

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        menuRef.current &&
        !menuRef.current.contains(event.target as Node) &&
        buttonRef.current &&
        !buttonRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside)
      return () => document.removeEventListener('mousedown', handleClickOutside)
    }
    return undefined
  }, [isOpen])

  const toggleMenu = useCallback(() => {
    setIsOpen(prev => !prev)
  }, [])

  const handleActionClick = useCallback((action: FloatingAction) => {
    if (!action.disabled) {
      action.onClick()
      setIsOpen(false)
    }
  }, [])

  // Don't render if no actions and hideWhenEmpty is true
  if (hideWhenEmpty && actions.length === 0) {
    return null
  }

  // Position styles
  const positionStyles = {
    'bottom-right': { bottom: spacing[6], right: spacing[6] },
    'bottom-left': { bottom: spacing[6], left: spacing[6] },
    'top-right': { top: spacing[6], right: spacing[6] },
    'top-left': { top: spacing[6], left: spacing[6] },
  }

  // Size configurations
  const sizeConfig = {
    sm: { button: 48, icon: 20, menu: 200 },
    md: { button: 56, icon: 24, menu: 240 },
    lg: { button: 64, icon: 28, menu: 280 },
  }

  const config = sizeConfig[size]

  // Menu position based on FAB position
  const menuPosition = position.includes('bottom')
    ? { bottom: '100%', marginBottom: spacing[2] }
    : { top: '100%', marginTop: spacing[2] }

  const menuAlign = position.includes('right') ? { right: 0 } : { left: 0 }

  return (
    <div
      style={{
        position: 'fixed',
        ...positionStyles[position],
        zIndex: 1000,
      }}
    >
      {/* Actions Menu */}
      {isOpen && actions.length > 0 && (
        <div
          ref={menuRef}
          style={{
            position: 'absolute',
            ...menuPosition,
            ...menuAlign,
            width: `${config.menu}px`,
            backgroundColor: colors.surface.primary,
            borderRadius: '12px',
            border: `1px solid ${colors.border.medium}`,
            boxShadow: shadows['2xl'],
            overflow: 'hidden',
            animation: `${animation.keyframes.fadeIn} ${animation.duration.fast} ${animation.easing.easeOut}`,
          }}
        >
          {actions.map((action, index) => (
            <button
              key={action.id}
              onClick={() => handleActionClick(action)}
              disabled={action.disabled}
              style={{
                width: '100%',
                display: 'flex',
                alignItems: 'center',
                gap: spacing[2],
                padding: `${spacing[2]} ${spacing[4]}`,
                border: 'none',
                borderTop: index > 0 ? `1px solid ${colors.border.subtle}` : 'none',
                backgroundColor: 'transparent',
                color: action.disabled ? colors.text.muted : colors.text.primary,
                cursor: action.disabled ? 'not-allowed' : 'pointer',
                textAlign: 'left',
                transition: `background-color ${animation.duration.fast}`,
                fontSize: '14px',
                fontFamily: 'inherit',
                opacity: action.disabled ? 0.5 : 1,
              }}
              onMouseEnter={e => {
                if (!action.disabled) {
                  e.currentTarget.style.backgroundColor = colors.state.hover
                }
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = 'transparent'
              }}
            >
              <Icon
                name={action.icon as never}
                size={18}
                color={
                  action.variant === 'danger'
                    ? colors.accent.alert
                    : action.variant === 'primary'
                      ? colors.accent.ai
                      : colors.text.secondary
                }
              />
              <div style={{ flex: 1 }}>
                <div
                  style={{
                    fontWeight: 500,
                    color:
                      action.variant === 'danger'
                        ? colors.accent.alert
                        : action.disabled
                          ? colors.text.muted
                          : colors.text.primary,
                  }}
                >
                  {action.label}
                </div>
                {action.description && (
                  <div
                    style={{
                      fontSize: '12px',
                      color: colors.text.secondary,
                      marginTop: '2px',
                    }}
                  >
                    {action.description}
                  </div>
                )}
              </div>
              {action.badge !== undefined && action.badge > 0 && (
                <div
                  style={{
                    backgroundColor: colors.accent.ai,
                    color: colors.surface.primary,
                    borderRadius: '12px',
                    padding: '2px 8px',
                    fontSize: '11px',
                    fontWeight: 600,
                    minWidth: '20px',
                    textAlign: 'center',
                  }}
                >
                  {action.badge > 99 ? '99+' : action.badge}
                </div>
              )}
            </button>
          ))}
        </div>
      )}

      {/* Main FAB Button */}
      <Tooltip content={mainLabel} placement="left">
        <button
          ref={buttonRef}
          onClick={toggleMenu}
          style={{
            width: `${config.button}px`,
            height: `${config.button}px`,
            borderRadius: '50%',
            backgroundColor: colors.accent.ai,
            border: 'none',
            color: colors.surface.primary,
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: shadows.lg,
            transition: `all ${animation.duration.normal} ${animation.easing.easeInOut}`,
            transform: isOpen ? 'rotate(45deg) scale(1.1)' : 'rotate(0deg) scale(1)',
          }}
          onMouseEnter={e => {
            if (!isOpen) {
              e.currentTarget.style.transform = 'scale(1.05)'
              e.currentTarget.style.boxShadow = shadows.xl
            }
          }}
          onMouseLeave={e => {
            if (!isOpen) {
              e.currentTarget.style.transform = 'scale(1)'
              e.currentTarget.style.boxShadow = shadows.lg
            }
          }}
        >
          <Icon
            name={isOpen ? 'X' : (mainIcon as never)}
            size={config.icon}
            color={colors.surface.primary}
          />
        </button>
      </Tooltip>

      {/* Badge for action count */}
      {!isOpen && actions.length > 0 && (
        <div
          style={{
            position: 'absolute',
            top: '-4px',
            right: '-4px',
            backgroundColor: colors.accent.alert,
            color: colors.surface.primary,
            borderRadius: '50%',
            width: '20px',
            height: '20px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '11px',
            fontWeight: 600,
            border: `2px solid ${colors.surface.primary}`,
            boxShadow: shadows.md,
          }}
        >
          {actions.length > 9 ? '9+' : actions.length}
        </div>
      )}
    </div>
  )
}

export default FloatingActionButton
