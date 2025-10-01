import React, { useState, useRef, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import Icon from './Icon'
import type { Icons } from '../../assets/icons/utils'

type IconName = Exclude<keyof typeof Icons, 'getDocumentTypeIcon'>

export interface SettingsMenuItem {
  id: string
  label: string
  icon: IconName
  onClick: () => void
}

export interface SettingsDropdownProps {
  menuItems: SettingsMenuItem[]
}

const SettingsDropdown: React.FC<SettingsDropdownProps> = ({ menuItems }) => {
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [isOpen])

  const buttonStyles = {
    background: 'none',
    border: 'none',
    color: designTokens.colors.text.secondary,
    cursor: 'pointer',
    padding: designTokens.spacing[2],
    borderRadius: designTokens.borderRadius.md,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: `all ${designTokens.animation.duration.fast}`,
  }

  const dropdownMenuStyles = {
    position: 'absolute' as const,
    right: 0,
    top: 'calc(100% + 8px)',
    minWidth: '220px',
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.medium}`,
    borderRadius: designTokens.borderRadius.lg,
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.4)',
    overflow: 'hidden',
    zIndex: designTokens.zIndex.dropdown,
    animation: 'fadeIn 0.15s ease-out',
  }

  const menuItemStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.primary,
    cursor: 'pointer',
    transition: `all ${designTokens.animation.duration.fast}`,
    backgroundColor: 'transparent',
    border: 'none',
    width: '100%',
    textAlign: 'left' as const,
  }

  return (
    <>
      <style>
        {`
          @keyframes fadeIn {
            from {
              opacity: 0;
              transform: translateY(-4px);
            }
            to {
              opacity: 1;
              transform: translateY(0);
            }
          }

          .settings-menu-item:hover {
            background-color: ${designTokens.colors.state.hover};
          }

          .settings-gear-button:hover {
            color: ${designTokens.colors.text.primary};
            background-color: ${designTokens.colors.state.hover};
          }
        `}
      </style>

      <div ref={dropdownRef} style={{ position: 'relative' }}>
        <button
          style={buttonStyles}
          className="settings-gear-button"
          aria-label="Settings"
          onClick={() => setIsOpen(!isOpen)}
        >
          <Icon name="Settings" size={18} />
        </button>

        {isOpen && (
          <div style={dropdownMenuStyles}>
            {menuItems.map((item, index) => (
              <button
                key={item.id}
                style={{
                  ...menuItemStyles,
                  borderBottom:
                    index < menuItems.length - 1
                      ? `1px solid ${designTokens.colors.border.subtle}`
                      : 'none',
                }}
                className="settings-menu-item"
                onClick={() => {
                  item.onClick()
                  setIsOpen(false)
                }}
              >
                <Icon name={item.icon} size={16} />
                <span>{item.label}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </>
  )
}

export default SettingsDropdown
