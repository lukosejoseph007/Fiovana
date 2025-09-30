import React, { useState, useRef, useEffect, useCallback } from 'react'
import { colors, spacing, shadows, animation, typography } from '../../styles/tokens'
import Icon from './Icon'
import Badge from './Badge'

export interface ActionItem {
  id: string
  label: string
  icon: string
  description?: string
  badge?: number
  shortcut?: string
  disabled?: boolean
  variant?: 'default' | 'primary' | 'danger'
  onClick: () => void
}

export interface ActionCategory {
  id: string
  label: string
  actions: ActionItem[]
}

interface ActionsDropdownProps {
  categories: ActionCategory[]
  buttonLabel?: string
  buttonIcon?: string
  availableCount?: number
  placement?: 'left' | 'right'
}

export const ActionsDropdown: React.FC<ActionsDropdownProps> = ({
  categories,
  buttonLabel = 'Actions',
  buttonIcon = 'Zap',
  availableCount,
  placement = 'right',
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)
  const buttonRef = useRef<HTMLButtonElement>(null)

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
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

  // Close on Escape key
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('keydown', handleEscape)
      return () => document.removeEventListener('keydown', handleEscape)
    }
    return undefined
  }, [isOpen])

  const toggleDropdown = useCallback(() => {
    setIsOpen(prev => !prev)
  }, [])

  const handleActionClick = useCallback((action: ActionItem) => {
    console.log('Action clicked:', action.id, action.label)
    if (!action.disabled) {
      action.onClick()
      setIsOpen(false)
    }
  }, [])

  // Count total available actions
  const totalActions = categories.reduce(
    (sum, category) => sum + category.actions.filter(a => !a.disabled).length,
    0
  )

  return (
    <div style={{ position: 'relative' }}>
      {/* Dropdown Button */}
      <button
        ref={buttonRef}
        onClick={toggleDropdown}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: spacing[2],
          padding: `${spacing[2]} ${spacing[3]}`,
          backgroundColor: isOpen ? colors.state.hover : 'transparent',
          border: `1px solid ${isOpen ? colors.border.medium : 'transparent'}`,
          borderRadius: '6px',
          color: colors.text.primary,
          cursor: 'pointer',
          fontSize: typography.fontSize.base,
          fontFamily: 'inherit',
          fontWeight: typography.fontWeight.medium,
          transition: `all ${animation.duration.fast}`,
          position: 'relative',
        }}
        onMouseEnter={e => {
          if (!isOpen) {
            e.currentTarget.style.backgroundColor = colors.state.hover
          }
        }}
        onMouseLeave={e => {
          if (!isOpen) {
            e.currentTarget.style.backgroundColor = 'transparent'
          }
        }}
      >
        <Icon name={buttonIcon as never} size={18} />
        <span>{buttonLabel}</span>
        <Icon
          name="ChevronDown"
          size={14}
          style={{
            transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: `transform ${animation.duration.fast}`,
          }}
        />
        {(availableCount || totalActions) > 0 && (
          <Badge
            variant="ai"
            style={{
              position: 'absolute',
              top: '-6px',
              right: '-6px',
              minWidth: '18px',
              height: '18px',
              padding: '2px 4px',
              fontSize: '10px',
            }}
          >
            {availableCount || totalActions}
          </Badge>
        )}
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <div
          ref={dropdownRef}
          style={{
            position: 'absolute',
            top: 'calc(100% + 4px)',
            [placement]: 0,
            width: '320px',
            maxHeight: '480px',
            overflowY: 'auto',
            backgroundColor: colors.surface.primary,
            borderRadius: '8px',
            border: `1px solid ${colors.border.medium}`,
            boxShadow: shadows.xl,
            zIndex: 9999,
            animation: `${animation.keyframes.fadeIn} ${animation.duration.fast} ${animation.easing.easeOut}`,
          }}
        >
          {categories.map((category, catIndex) => (
            <div key={category.id}>
              {/* Category Header */}
              {category.label && (
                <div
                  style={{
                    padding: `${spacing[3]} ${spacing[4]}`,
                    fontSize: typography.fontSize.sm,
                    fontWeight: typography.fontWeight.semibold,
                    color: colors.text.secondary,
                    textTransform: 'uppercase',
                    letterSpacing: '0.05em',
                    backgroundColor: colors.surface.secondary,
                    borderTop: catIndex > 0 ? `1px solid ${colors.border.subtle}` : 'none',
                  }}
                >
                  {category.label}
                </div>
              )}

              {/* Category Actions */}
              {category.actions.map(action => (
                <button
                  key={action.id}
                  onClick={() => handleActionClick(action)}
                  disabled={action.disabled}
                  style={{
                    width: '100%',
                    display: 'flex',
                    alignItems: 'center',
                    gap: spacing[3],
                    padding: `${spacing[3]} ${spacing[4]}`,
                    border: 'none',
                    backgroundColor: 'transparent',
                    color: action.disabled ? colors.text.muted : colors.text.primary,
                    cursor: action.disabled ? 'not-allowed' : 'pointer',
                    textAlign: 'left',
                    transition: `background-color ${animation.duration.fast}`,
                    fontSize: typography.fontSize.base,
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
                  {/* Icon */}
                  <div
                    style={{
                      width: '32px',
                      height: '32px',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      backgroundColor:
                        action.variant === 'danger'
                          ? `${colors.accent.alert}20`
                          : action.variant === 'primary'
                            ? `${colors.accent.ai}20`
                            : colors.surface.secondary,
                      borderRadius: '6px',
                    }}
                  >
                    <Icon
                      name={action.icon as never}
                      size={16}
                      color={
                        action.variant === 'danger'
                          ? colors.accent.alert
                          : action.variant === 'primary'
                            ? colors.accent.ai
                            : colors.text.secondary
                      }
                    />
                  </div>

                  {/* Label & Description */}
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div
                      style={{
                        fontWeight: typography.fontWeight.medium,
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
                          fontSize: typography.fontSize.sm,
                          color: colors.text.secondary,
                          marginTop: '2px',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                        }}
                      >
                        {action.description}
                      </div>
                    )}
                  </div>

                  {/* Badge or Shortcut */}
                  {action.badge !== undefined && action.badge > 0 ? (
                    <Badge variant="ai" size="sm">
                      {action.badge > 99 ? '99+' : action.badge}
                    </Badge>
                  ) : action.shortcut ? (
                    <kbd
                      style={{
                        padding: '2px 6px',
                        backgroundColor: colors.surface.secondary,
                        border: `1px solid ${colors.border.subtle}`,
                        borderRadius: '4px',
                        fontSize: '11px',
                        fontFamily: 'monospace',
                        color: colors.text.secondary,
                      }}
                    >
                      {action.shortcut}
                    </kbd>
                  ) : null}
                </button>
              ))}
            </div>
          ))}

          {/* Empty State */}
          {categories.every(cat => cat.actions.length === 0) && (
            <div
              style={{
                padding: `${spacing[6]} ${spacing[4]}`,
                textAlign: 'center',
                color: colors.text.secondary,
                fontSize: typography.fontSize.sm,
              }}
            >
              <Icon
                name="Info"
                size={24}
                color={colors.text.tertiary}
                style={{ marginBottom: spacing[3] }}
              />
              <div>No actions available</div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default ActionsDropdown
