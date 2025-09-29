import React, { useState, createContext, useContext } from 'react'
import { designTokens } from '../../styles/tokens'

interface TabsContextType {
  activeTab: string
  setActiveTab: (tab: string) => void
  variant: 'default' | 'pills' | 'minimal'
  size: 'sm' | 'md' | 'lg'
}

const TabsContext = createContext<TabsContextType | undefined>(undefined)

const useTabs = () => {
  const context = useContext(TabsContext)
  if (!context) {
    throw new Error('Tab components must be used within a Tabs component')
  }
  return context
}

export interface TabsProps {
  defaultValue?: string
  value?: string
  onChange?: (value: string) => void
  variant?: 'default' | 'pills' | 'minimal'
  size?: 'sm' | 'md' | 'lg'
  fullWidth?: boolean
  className?: string
  children: React.ReactNode
}

export interface TabListProps {
  className?: string
  children: React.ReactNode
}

export interface TabProps {
  value: string
  disabled?: boolean
  icon?: React.ReactNode
  badge?: React.ReactNode
  className?: string
  children: React.ReactNode
}

export interface TabPanelsProps {
  className?: string
  children: React.ReactNode
}

export interface TabPanelProps {
  value: string
  className?: string
  children: React.ReactNode
}

const Tabs: React.FC<TabsProps> = ({
  defaultValue,
  value: controlledValue,
  onChange,
  variant = 'default',
  size = 'md',
  fullWidth = false,
  className = '',
  children,
}) => {
  const [internalValue, setInternalValue] = useState(defaultValue || '')
  const activeTab = controlledValue !== undefined ? controlledValue : internalValue

  const setActiveTab = (tab: string) => {
    if (controlledValue === undefined) {
      setInternalValue(tab)
    }
    onChange?.(tab)
  }

  const containerStyles = {
    width: fullWidth ? '100%' : 'auto',
  }

  return (
    <TabsContext.Provider value={{ activeTab, setActiveTab, variant, size }}>
      <div className={`proxemic-tabs ${className}`} style={containerStyles}>
        {children}
      </div>
    </TabsContext.Provider>
  )
}

const TabList: React.FC<TabListProps> = ({ className = '', children }) => {
  const { variant, size } = useTabs()

  // Size variants
  const sizeStyles = {
    sm: {
      gap: variant === 'pills' ? designTokens.spacing[1] : '0',
      fontSize: designTokens.typography.fontSize.sm,
    },
    md: {
      gap: variant === 'pills' ? designTokens.spacing[2] : '0',
      fontSize: designTokens.typography.fontSize.base,
    },
    lg: {
      gap: variant === 'pills' ? designTokens.spacing[3] : '0',
      fontSize: designTokens.typography.fontSize.lg,
    },
  }

  const listStyles = {
    display: 'flex',
    alignItems: 'center',
    borderBottom: variant !== 'minimal' ? `1px solid ${designTokens.colors.border.subtle}` : 'none',
    marginBottom: designTokens.spacing[4],
    position: 'relative' as const,
    ...sizeStyles[size],
  }

  return (
    <div className={`proxemic-tab-list ${className}`} style={listStyles} role="tablist">
      {children}
    </div>
  )
}

const Tab: React.FC<TabProps> = ({
  value,
  disabled = false,
  icon,
  badge,
  className = '',
  children,
}) => {
  const { activeTab, setActiveTab, variant, size } = useTabs()
  const isActive = activeTab === value

  const handleClick = () => {
    if (!disabled) {
      setActiveTab(value)
    }
  }

  // Size variants
  const sizeStyles = {
    sm: {
      height: '32px',
      padding: `0 ${designTokens.spacing[3]}`,
      fontSize: designTokens.typography.fontSize.sm,
    },
    md: {
      height: '40px',
      padding: `0 ${designTokens.spacing[4]}`,
      fontSize: designTokens.typography.fontSize.base,
    },
    lg: {
      height: '48px',
      padding: `0 ${designTokens.spacing[5]}`,
      fontSize: designTokens.typography.fontSize.lg,
    },
  }

  // Variant styles
  const getVariantStyles = () => {
    const base = {
      display: 'flex',
      alignItems: 'center',
      gap: designTokens.spacing[2],
      border: 'none',
      background: 'transparent',
      cursor: disabled ? 'not-allowed' : 'pointer',
      fontFamily: designTokens.typography.fonts.sans.join(', '),
      fontWeight: isActive
        ? designTokens.typography.fontWeight.semibold
        : designTokens.typography.fontWeight.medium,
      transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
      opacity: disabled ? 0.5 : 1,
      outline: 'none',
      position: 'relative' as const,
      ...sizeStyles[size],
    }

    switch (variant) {
      case 'pills':
        return {
          ...base,
          borderRadius: designTokens.borderRadius.full,
          backgroundColor: isActive ? designTokens.colors.accent.ai : 'transparent',
          color: isActive
            ? designTokens.colors.surface.primary
            : designTokens.colors.text.secondary,
        }
      case 'minimal':
        return {
          ...base,
          color: isActive ? designTokens.colors.accent.ai : designTokens.colors.text.secondary,
        }
      default:
        return {
          ...base,
          borderBottom: `2px solid ${isActive ? designTokens.colors.accent.ai : 'transparent'}`,
          color: isActive ? designTokens.colors.accent.ai : designTokens.colors.text.secondary,
          marginBottom: '-1px',
        }
    }
  }

  const tabStyles = getVariantStyles()

  return (
    <>
      <style>
        {`
          .proxemic-tab:hover:not(:disabled) {
            ${
              variant === 'pills'
                ? `
              background-color: ${isActive ? designTokens.colors.accent.ai : designTokens.colors.state.hover};
              color: ${isActive ? designTokens.colors.surface.primary : designTokens.colors.text.primary};
            `
                : variant === 'minimal'
                  ? `
              color: ${isActive ? designTokens.colors.accent.ai : designTokens.colors.text.primary};
            `
                  : `
              color: ${isActive ? designTokens.colors.accent.ai : designTokens.colors.text.primary};
              border-bottom-color: ${isActive ? designTokens.colors.accent.ai : designTokens.colors.border.medium};
            `
            }
          }

          .proxemic-tab:focus {
            outline: none;
            ${
              variant === 'pills'
                ? `
              box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
            `
                : `
              box-shadow: inset 0 0 0 2px ${designTokens.colors.state.focus}40;
            `
            }
          }
        `}
      </style>

      <button
        className={`proxemic-tab ${className}`}
        style={tabStyles}
        onClick={handleClick}
        disabled={disabled}
        role="tab"
        aria-selected={isActive}
        aria-controls={`panel-${value}`}
        id={`tab-${value}`}
        tabIndex={isActive ? 0 : -1}
      >
        {icon && <span style={{ display: 'flex', alignItems: 'center' }}>{icon}</span>}
        <span>{children}</span>
        {badge && <span style={{ display: 'flex', alignItems: 'center' }}>{badge}</span>}
      </button>
    </>
  )
}

const TabPanels: React.FC<TabPanelsProps> = ({ className = '', children }) => {
  return <div className={`proxemic-tab-panels ${className}`}>{children}</div>
}

const TabPanel: React.FC<TabPanelProps> = ({ value, className = '', children }) => {
  const { activeTab } = useTabs()
  const isActive = activeTab === value

  if (!isActive) return null

  const panelStyles = {
    animation: 'fadeIn 0.2s ease-out',
  }

  return (
    <>
      <style>
        {`
          @keyframes fadeIn {
            from { opacity: 0; transform: translateY(8px); }
            to { opacity: 1; transform: translateY(0); }
          }
        `}
      </style>

      <div
        className={`proxemic-tab-panel ${className}`}
        style={panelStyles}
        role="tabpanel"
        aria-labelledby={`tab-${value}`}
        id={`panel-${value}`}
        tabIndex={0}
      >
        {children}
      </div>
    </>
  )
}

// Compound component pattern with proper typing
interface TabsComponent extends React.FC<TabsProps> {
  List: typeof TabList
  Tab: typeof Tab
  Panels: typeof TabPanels
  Panel: typeof TabPanel
}

const TabsWithSubComponents = Tabs as TabsComponent
TabsWithSubComponents.List = TabList
TabsWithSubComponents.Tab = Tab
TabsWithSubComponents.Panels = TabPanels
TabsWithSubComponents.Panel = TabPanel

export { TabList, Tab, TabPanels, TabPanel }
export default TabsWithSubComponents
