import React, { useState, useCallback, useMemo, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import Tabs from '../ui/Tabs'
import Icon from '../ui/Icon'
import ConversationMode from './ConversationMode'
import DocumentIntelligence from './DocumentIntelligence'
import WorkspaceInsights from './WorkspaceInsights'

export interface IntelligencePanelProps {
  className?: string
  style?: React.CSSProperties
  collapsible?: boolean
  defaultMode?: 'conversation' | 'document' | 'workspace'
  onModeChange?: (mode: 'conversation' | 'document' | 'workspace') => void
}

export type IntelligenceMode = 'conversation' | 'document' | 'workspace'

interface IntelligenceModeConfig {
  id: IntelligenceMode
  label: string
  icon: string
  badge?: string
  description: string
}

const IntelligencePanel: React.FC<IntelligencePanelProps> = ({
  className = '',
  style,
  collapsible = true,
  defaultMode = 'conversation',
  onModeChange,
}) => {
  const [activeMode, setActiveMode] = useState<IntelligenceMode>(defaultMode)
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [contextData, setContextData] = useState<unknown>(null)

  // Mode configurations
  const modes = useMemo<IntelligenceModeConfig[]>(
    () => [
      {
        id: 'conversation',
        label: 'Conversation',
        icon: 'User',
        description: 'AI chat interface for natural language interactions',
      },
      {
        id: 'document',
        label: 'Document',
        icon: 'Document',
        description: 'Analysis metrics and insights for current document',
      },
      {
        id: 'workspace',
        label: 'Workspace',
        icon: 'Layers',
        description: 'Knowledge graph and workspace-wide recommendations',
      },
    ],
    []
  )

  // Handle mode changes with context persistence
  const handleModeChange = useCallback(
    (mode: string) => {
      const newMode = mode as IntelligenceMode
      setActiveMode(newMode)
      onModeChange?.(newMode)
    },
    [onModeChange]
  )

  // Handle collapse toggle
  const handleToggleCollapse = useCallback(() => {
    setIsCollapsed(!isCollapsed)
  }, [isCollapsed])

  // Update context data when mode changes
  useEffect(() => {
    // This would typically fetch relevant context data for the active mode
    // For now, we'll set a placeholder to demonstrate the pattern
    setContextData({ mode: activeMode, timestamp: Date.now() })
  }, [activeMode])

  const containerStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100%',
    backgroundColor: designTokens.colors.surface.secondary,
    borderLeft: `1px solid ${designTokens.colors.border.subtle}`,
    transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
    position: 'relative' as const,
    ...style,
  }

  const headerStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: `${designTokens.spacing[4]} ${designTokens.spacing[4]} ${designTokens.spacing[2]}`,
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    backgroundColor: designTokens.colors.surface.secondary,
    position: 'sticky' as const,
    top: 0,
    zIndex: 1,
  }

  const titleStyles = {
    fontSize: designTokens.typography.fontSize.lg,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    margin: 0,
  }

  const collapseButtonStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '32px',
    height: '32px',
    border: 'none',
    borderRadius: designTokens.borderRadius.md,
    backgroundColor: 'transparent',
    color: designTokens.colors.text.secondary,
    cursor: 'pointer',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
  }

  const contentStyles = {
    flex: 1,
    display: 'flex',
    flexDirection: 'column' as const,
    transition: `opacity ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    opacity: isCollapsed ? 0 : 1,
    overflow: 'hidden' as const,
  }

  const tabsContainerStyles = {
    padding: `${designTokens.spacing[2]} ${designTokens.spacing[4]}`,
    backgroundColor: designTokens.colors.surface.secondary,
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    flexShrink: 0,
  }

  const panelContentStyles = {
    flex: 1,
    display: 'flex',
    flexDirection: 'column' as const,
    overflow: 'hidden' as const,
    minHeight: 0,
  }

  if (isCollapsed && collapsible) {
    return (
      <div className={`proxemic-intelligence-panel collapsed ${className}`} style={containerStyles}>
        <div style={headerStyles}>
          <button
            onClick={handleToggleCollapse}
            style={collapseButtonStyles}
            title="Expand Intelligence Panel"
            aria-label="Expand Intelligence Panel"
          >
            <Icon name="ChevronDown" size={18} />
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className={`proxemic-intelligence-panel ${className}`} style={containerStyles}>
      {/* Panel Header */}
      <div style={headerStyles}>
        <h2 style={titleStyles}>Intelligence</h2>
        {collapsible && (
          <button
            onClick={handleToggleCollapse}
            style={collapseButtonStyles}
            title="Collapse Intelligence Panel"
            aria-label="Collapse Intelligence Panel"
          >
            <Icon name="ChevronDown" size={18} />
          </button>
        )}
      </div>

      {/* Tabs - Fixed at top */}
      <div style={tabsContainerStyles}>
        <Tabs value={activeMode} onChange={handleModeChange} variant="minimal" size="sm" fullWidth>
          <Tabs.List>
            {modes.map(mode => (
              <Tabs.Tab
                key={mode.id}
                value={mode.id}
                icon={<Icon name={mode.icon as never} size={16} />}
              >
                {mode.label}
              </Tabs.Tab>
            ))}
          </Tabs.List>
        </Tabs>
      </div>

      {/* Panel Content - Scrollable */}
      <div style={contentStyles}>
        {activeMode === 'conversation' && (
          <div className="panel-content" style={panelContentStyles}>
            <ConversationMode contextData={contextData} />
          </div>
        )}
        {activeMode === 'document' && (
          <div className="panel-content" style={panelContentStyles}>
            <DocumentIntelligence contextData={contextData} />
          </div>
        )}
        {activeMode === 'workspace' && (
          <div className="panel-content" style={panelContentStyles}>
            <WorkspaceInsights contextData={contextData} />
          </div>
        )}
      </div>

      {/* Inline Styles for Hover Effects */}
      <style>
        {`
          .proxemic-intelligence-panel button:hover {
            background-color: ${designTokens.colors.state.hover};
            color: ${designTokens.colors.text.primary};
          }

          .proxemic-intelligence-panel button:focus {
            outline: none;
            box-shadow: 0 0 0 2px ${designTokens.colors.state.focus}40;
          }

          .proxemic-intelligence-panel.collapsed {
            min-width: 48px;
            max-width: 48px;
          }

          /* Custom scrollbar for panel content */
          .proxemic-intelligence-panel .panel-content::-webkit-scrollbar {
            width: 6px;
          }

          .proxemic-intelligence-panel .panel-content::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .proxemic-intelligence-panel .panel-content::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }

          .proxemic-intelligence-panel .panel-content::-webkit-scrollbar-thumb:hover {
            background: ${designTokens.colors.border.strong};
          }
        `}
      </style>
    </div>
  )
}

export default React.memo(IntelligencePanel)
