import React, { useCallback, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import { useLayout } from './useLayoutContext'
import { Dropdown, Icon, OfflineIndicator } from '../ui'
import ActionsDropdown from '../ui/ActionsDropdown'
import SettingsDropdown from '../ui/SettingsDropdown'
import type { ActionCategory } from '../ui/ActionsDropdown'

export interface ActiveOperation {
  id: string
  type: string
  label: string
  progress?: number
  status: 'running' | 'completed' | 'error'
}

export interface HeaderBarProps {
  className?: string
  style?: React.CSSProperties
  currentWorkspace?: {
    id: string
    name: string
  }
  workspaces?: Array<{
    id: string
    name: string
  }>
  user?: {
    name: string
    avatar?: string
  }
  collaborators?: Array<{
    id: string
    name: string
    avatar?: string
    isActive: boolean
  }>
  aiStatus?: {
    isConnected: boolean
    isProcessing: boolean
    provider?: string
  }
  documentContext?: {
    currentDocument?: string
    breadcrumbs?: Array<{
      label: string
      path: string
    }>
  }
  activeOperations?: ActiveOperation[]
  onWorkspaceChange?: (workspaceId: string) => void
  onSearch?: (query: string) => void
  onAISettingsClick?: () => void
  onWorkspaceSettingsClick?: () => void
  onUserPreferencesClick?: () => void
  onCommandPaletteOpen?: () => void
  onLogoClick?: () => void
  onOperationTrigger?: (operationType: string) => void
}

const HeaderBar: React.FC<HeaderBarProps> = ({
  className = '',
  style,
  currentWorkspace,
  workspaces = [],
  collaborators = [],
  aiStatus = { isConnected: false, isProcessing: false },
  documentContext,
  activeOperations = [],
  onWorkspaceChange,
  onSearch: _onSearch,
  onAISettingsClick,
  onWorkspaceSettingsClick,
  onUserPreferencesClick,
  onCommandPaletteOpen,
  onLogoClick,
  onOperationTrigger,
}) => {
  const { isMobile, toggleIntelligence, intelligenceCollapsed } = useLayout()

  // Keyboard shortcuts handler
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Cmd+K or Ctrl+K to open command palette
      if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
        event.preventDefault()
        onCommandPaletteOpen?.()
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [onCommandPaletteOpen])

  const handleSearchFocus = useCallback(() => {
    onCommandPaletteOpen?.()
  }, [onCommandPaletteOpen])

  const handleLogoClick = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()
      onLogoClick?.()
    },
    [onLogoClick]
  )

  // Define action categories for the dropdown
  const actionCategories: ActionCategory[] = [
    {
      id: 'document-ops',
      label: 'Document Operations',
      actions: [
        {
          id: 'analyze',
          label: 'Analyze',
          icon: 'Search',
          description: 'Analyze document structure and content',
          shortcut: 'A',
          onClick: () => onOperationTrigger?.('analyze'),
        },
        {
          id: 'compare',
          label: 'Compare',
          icon: 'GitCompare',
          description: 'Compare with another document',
          shortcut: 'C',
          onClick: () => onOperationTrigger?.('compare'),
        },
        {
          id: 'generate',
          label: 'Generate',
          icon: 'FilePlus',
          description: 'Generate new content or documents',
          shortcut: 'G',
          onClick: () => onOperationTrigger?.('generate'),
        },
        {
          id: 'update',
          label: 'Update',
          icon: 'Edit',
          description: 'Update document based on changes',
          shortcut: 'U',
          onClick: () => onOperationTrigger?.('update'),
        },
      ],
    },
    {
      id: 'workspace-ops',
      label: 'Workspace Operations',
      actions: [
        {
          id: 'search',
          label: 'Search',
          icon: 'Search',
          description: 'Search across documents',
          shortcut: '/',
          onClick: () => onCommandPaletteOpen?.(),
        },
        {
          id: 'organize',
          label: 'Organize',
          icon: 'Folder',
          description: 'Organize and categorize content',
          shortcut: 'O',
          onClick: () => onOperationTrigger?.('organize'),
        },
      ],
    },
    {
      id: 'advanced-ops',
      label: 'Advanced',
      actions: [
        {
          id: 'style-transfer',
          label: 'Style Transfer',
          icon: 'Palette',
          description: 'Apply or learn document styles',
          onClick: () => onOperationTrigger?.('styleTransfer'),
        },
        {
          id: 'batch',
          label: 'Batch Operations',
          icon: 'Layers',
          description: 'Manage multiple operations',
          onClick: () => onOperationTrigger?.('batch'),
        },
      ],
    },
  ]

  const headerContentStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    width: '100%',
    height: '100%',
  }

  const leftZoneStyles = {
    display: 'flex',
    alignItems: 'center',
    width: isMobile ? 'auto' : designTokens.layout.navigation.width,
    flexShrink: 0,
  }

  const centerZoneStyles = {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    maxWidth: '600px',
    margin: `0 ${designTokens.spacing[4]}`,
  }

  const rightZoneStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    flexShrink: 0,
  }

  const logoStyles = {
    fontSize: designTokens.typography.fontSize.xl,
    fontWeight: designTokens.typography.fontWeight.thin,
    color: designTokens.colors.text.primary,
    textDecoration: 'none',
    letterSpacing: designTokens.typography.letterSpacing.wide,
  }

  const searchBarStyles = {
    width: '100%',
    maxWidth: '400px',
    height: '36px',
    backgroundColor: designTokens.colors.surface.tertiary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.lg,
    padding: `0 ${designTokens.spacing[4]}`,
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.primary,
    outline: 'none',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
  }

  return (
    <>
      <style>
        {`
          .proxemic-search-bar:focus {
            border-color: ${designTokens.colors.accent.ai};
            box-shadow: 0 0 0 3px ${designTokens.colors.accent.ai}40;
            background-color: ${designTokens.colors.surface.secondary};
          }

          .proxemic-search-bar::placeholder {
            color: ${designTokens.colors.text.tertiary};
          }

          .proxemic-header-icon:hover {
            color: ${designTokens.colors.text.primary};
            background-color: ${designTokens.colors.state.hover};
          }

          .proxemic-avatar-status {
            position: relative;
          }

          .proxemic-avatar-status::after {
            content: '';
            position: absolute;
            bottom: 0;
            right: 0;
            width: 10px;
            height: 10px;
            background-color: ${designTokens.colors.confidence.high};
            border: 2px solid ${designTokens.colors.surface.primary};
            border-radius: 50%;
          }
        `}
      </style>

      <div
        className={`proxemic-header-bar ${className}`}
        style={{ ...headerContentStyles, ...style }}
      >
        {/* Left Zone - Logo and Workspace Selector */}
        <div style={leftZoneStyles}>
          <a
            href="/"
            style={{ ...logoStyles, cursor: 'pointer' }}
            onClick={handleLogoClick}
            aria-label="Go to home"
          >
            Proxemic
          </a>

          {!isMobile && (
            <div style={{ marginLeft: designTokens.spacing[4] }}>
              <WorkspaceSelector
                currentWorkspace={currentWorkspace}
                workspaces={workspaces}
                onWorkspaceChange={onWorkspaceChange}
              />
            </div>
          )}
        </div>

        {/* Center Zone - Search/Command Bar or Breadcrumbs */}
        <div style={centerZoneStyles}>
          {documentContext?.breadcrumbs?.length ? (
            <BreadcrumbNavigation breadcrumbs={documentContext.breadcrumbs} />
          ) : (
            <div style={{ width: '100%', position: 'relative' }}>
              <input
                className="proxemic-search-bar"
                style={searchBarStyles}
                placeholder="Search, ask, or type / for commands..."
                onFocus={handleSearchFocus}
              />
              <div
                style={{
                  position: 'absolute',
                  right: designTokens.spacing[3],
                  top: '50%',
                  transform: 'translateY(-50%)',
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.tertiary,
                  pointerEvents: 'none',
                }}
              >
                ⌘K
              </div>
            </div>
          )}
        </div>

        {/* Right Zone - Collaborators, AI Status, Panel Toggles, Settings */}
        <div style={rightZoneStyles}>
          <CollaboratorAvatars collaborators={collaborators} />
          <AIStatusIndicator aiStatus={aiStatus} />

          {/* Offline Status Indicator */}
          <OfflineIndicator position="inline" showDetails={true} />

          {/* Active Operations Badge */}
          {activeOperations.length > 0 && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: designTokens.spacing[2],
                padding: `${designTokens.spacing[1.5]} ${designTokens.spacing[3]}`,
                backgroundColor: designTokens.colors.surface.secondary,
                border: `1px solid ${designTokens.colors.border.subtle}`,
                borderRadius: designTokens.borderRadius.full,
              }}
              title={`${activeOperations.length} operation${activeOperations.length !== 1 ? 's' : ''} running`}
            >
              <div
                style={{
                  width: '8px',
                  height: '8px',
                  borderRadius: '50%',
                  backgroundColor: designTokens.colors.accent.ai,
                  animation: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
                }}
              />
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.primary,
                  fontWeight: designTokens.typography.fontWeight.medium,
                }}
              >
                {activeOperations.length} operation{activeOperations.length !== 1 ? 's' : ''}
              </span>
            </div>
          )}

          {/* Actions Dropdown */}
          {!isMobile && <ActionsDropdown categories={actionCategories} buttonIcon="Zap" />}

          {/* Intelligence Panel Toggle Button */}
          {!isMobile && (
            <button
              style={{
                background: 'none',
                border: 'none',
                color: intelligenceCollapsed
                  ? designTokens.colors.text.tertiary
                  : designTokens.colors.text.secondary,
                cursor: 'pointer',
                padding: designTokens.spacing[2],
                borderRadius: designTokens.borderRadius.md,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                transition: `all ${designTokens.animation.duration.fast}`,
              }}
              className="proxemic-header-icon"
              aria-label="Toggle Intelligence Panel"
              title="Toggle Intelligence Panel"
              onClick={toggleIntelligence}
            >
              <Icon name="Layout" size={18} />
            </button>
          )}

          <SettingsDropdown
            menuItems={[
              {
                id: 'ai-settings',
                label: 'AI Settings',
                icon: 'Cpu',
                onClick: () => onAISettingsClick?.(),
              },
              {
                id: 'workspace-settings',
                label: 'Workspace Settings',
                icon: 'Folder',
                onClick: () => onWorkspaceSettingsClick?.(),
              },
              {
                id: 'user-preferences',
                label: 'User Preferences',
                icon: 'User',
                onClick: () => onUserPreferencesClick?.(),
              },
            ]}
          />
        </div>
      </div>
    </>
  )
}

// Breadcrumb Navigation Component
interface BreadcrumbNavigationProps {
  breadcrumbs: Array<{
    label: string
    path: string
  }>
}

const BreadcrumbNavigation: React.FC<BreadcrumbNavigationProps> = ({ breadcrumbs }) => {
  const breadcrumbStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
  }

  const breadcrumbItemStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
  }

  const breadcrumbLinkStyles = {
    color: designTokens.colors.text.secondary,
    textDecoration: 'none',
    padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
    borderRadius: designTokens.borderRadius.sm,
    transition: `color ${designTokens.animation.duration.fast}`,
    cursor: 'pointer',
  }

  const breadcrumbSeparatorStyles = {
    color: designTokens.colors.text.tertiary,
    fontSize: designTokens.typography.fontSize.xs,
  }

  return (
    <div style={breadcrumbStyles}>
      {breadcrumbs.map((breadcrumb, index) => (
        <div key={breadcrumb.path} style={breadcrumbItemStyles}>
          <span
            style={breadcrumbLinkStyles}
            className="proxemic-header-icon"
            onClick={() => {
              // TODO: Navigate to breadcrumb path
              console.log('Navigate to:', breadcrumb.path)
            }}
          >
            {breadcrumb.label}
          </span>
          {index < breadcrumbs.length - 1 && <span style={breadcrumbSeparatorStyles}>/</span>}
        </div>
      ))}
    </div>
  )
}

// Workspace Selector Component
interface WorkspaceSelectorProps {
  currentWorkspace?: {
    id: string
    name: string
  }
  workspaces?: Array<{
    id: string
    name: string
  }>
  onWorkspaceChange?: (workspaceId: string) => void
}

const WorkspaceSelector: React.FC<WorkspaceSelectorProps> = ({
  currentWorkspace,
  workspaces = [],
  onWorkspaceChange,
}) => {
  const workspaceOptions = workspaces.map(workspace => ({
    value: workspace.id,
    label: workspace.name,
  }))

  return (
    <div style={{ position: 'relative' }}>
      <Dropdown
        options={workspaceOptions}
        value={currentWorkspace?.id}
        onChange={onWorkspaceChange}
        placeholder="Select Workspace"
        size="sm"
      />
    </div>
  )
}

// Collaborator Avatars Component
interface CollaboratorAvatarsProps {
  collaborators?: Array<{
    id: string
    name: string
    avatar?: string
    isActive: boolean
  }>
}

const CollaboratorAvatars: React.FC<CollaboratorAvatarsProps> = ({ collaborators = [] }) => {
  const avatarGroupStyles = {
    display: 'flex',
    alignItems: 'center',
  }

  const avatarStyles = {
    width: '28px',
    height: '28px',
    borderRadius: '50%',
    backgroundColor: designTokens.colors.accent.ai,
    border: `2px solid ${designTokens.colors.surface.primary}`,
    marginLeft: '-8px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: designTokens.typography.fontSize.xs,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.surface.primary,
  }

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  return (
    <div style={avatarGroupStyles}>
      {collaborators.slice(0, 3).map((collaborator, index) => (
        <div
          key={collaborator.id}
          style={{
            ...avatarStyles,
            marginLeft: index > 0 ? '-8px' : '0',
            zIndex: collaborators.length - index,
            backgroundColor: collaborator.isActive
              ? designTokens.colors.accent.ai
              : designTokens.colors.text.tertiary,
          }}
          className={collaborator.isActive ? 'proxemic-avatar-status' : ''}
          title={`${collaborator.name} (${collaborator.isActive ? 'Active' : 'Offline'})`}
        >
          {collaborator.avatar ? (
            <img
              src={collaborator.avatar}
              alt={collaborator.name}
              style={{
                width: '100%',
                height: '100%',
                borderRadius: '50%',
                objectFit: 'cover',
              }}
            />
          ) : (
            getInitials(collaborator.name)
          )}
        </div>
      ))}
      {collaborators.length > 3 && (
        <div
          style={{
            ...avatarStyles,
            marginLeft: '-8px',
            backgroundColor: designTokens.colors.text.tertiary,
            fontSize: designTokens.typography.fontSize.xs,
          }}
          title={`+${collaborators.length - 3} more collaborators`}
        >
          +{collaborators.length - 3}
        </div>
      )}
    </div>
  )
}

// AI Status Indicator Component
interface AIStatusIndicatorProps {
  aiStatus?: {
    isConnected: boolean
    isProcessing: boolean
    provider?: string
  }
}

const AIStatusIndicator: React.FC<AIStatusIndicatorProps> = ({
  aiStatus = { isConnected: false, isProcessing: false },
}) => {
  const getStatusColor = () => {
    if (!aiStatus.isConnected) return designTokens.colors.text.tertiary
    if (aiStatus.isProcessing) return designTokens.colors.accent.ai
    return designTokens.colors.confidence.high
  }

  const getStatusText = () => {
    if (!aiStatus.isConnected) return 'AI Disconnected'
    if (aiStatus.isProcessing) return 'AI Processing...'
    return `AI Ready${aiStatus.provider ? ` (${aiStatus.provider})` : ''}`
  }

  const indicatorStyles = {
    width: '12px',
    height: '12px',
    borderRadius: '50%',
    backgroundColor: getStatusColor(),
    animation: aiStatus.isProcessing ? 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite' : 'none',
    cursor: 'pointer',
  }

  return (
    <>
      <style>
        {`
          @keyframes pulse {
            0%, 100% {
              opacity: 1;
            }
            50% {
              opacity: 0.5;
            }
          }
        `}
      </style>

      <div style={indicatorStyles} title={getStatusText()} aria-label={getStatusText()} />
    </>
  )
}

export default HeaderBar
