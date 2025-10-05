import React, { useState, useEffect } from 'react'
import { Button, Icon, Badge, Tooltip, Dropdown } from '../ui'
import { designTokens } from '../../styles/tokens'
import { ActiveUsers } from '../collaboration/ActiveUsers'
import { OfflineIndicator } from '../ui/OfflineIndicator'

export type ToolbarTab = 'home' | 'review' | 'ai-tools' | 'more'

import type { Icons } from '../../assets/icons/utils'

type IconName = Exclude<keyof typeof Icons, 'getDocumentTypeIcon'>

const TOOLBAR_COLLAPSED_KEY = 'fiovana_toolbar_collapsed'

export interface ToolbarAction {
  id: string
  label: string
  icon: IconName
  onClick: () => void
  disabled?: boolean
  badge?: string | number
  tooltip?: string
  variant?: 'ghost' | 'primary' | 'secondary'
  showLabel?: boolean
}

export interface ToolbarGroup {
  id: string
  label: string
  actions: ToolbarAction[]
}

export interface DocumentToolbarProps {
  // Document info
  documentName: string
  documentIcon?: IconName
  confidence?: number
  documentType?: string

  // Collaboration
  activeUsers?: Array<{
    id: string
    name: string
    color: string
    isActive: boolean
    cursor?: { x: number; y: number }
    lastSeen: number
  }>
  currentUserId?: string
  showPresence?: boolean
  onTogglePresence?: () => void

  // Sync status
  isSyncing?: boolean
  queuedOperations?: number
  syncProgress?: number
  onManualSync?: () => void
  showOfflineIndicator?: boolean
  isEditMode?: boolean

  // Primary actions
  onSave?: () => void
  isSaving?: boolean
  isDirty?: boolean
  saveError?: string
  lastSaved?: Date | null

  // Tab groups
  homeActions?: ToolbarGroup[]
  reviewActions?: ToolbarGroup[]
  aiToolsActions?: ToolbarGroup[]
  moreActions?: ToolbarGroup[]

  // Dropdown actions
  dropdownActions?: Array<{
    value: string
    label: string
    disabled?: boolean
  }>
  onDropdownAction?: (value: string) => void

  // Callbacks
  onClose?: () => void
  onTabChange?: (tab: ToolbarTab) => void
  defaultTab?: ToolbarTab
  className?: string

  // Collapse functionality
  defaultCollapsed?: boolean
  onCollapseChange?: (collapsed: boolean) => void
}

const DocumentToolbar: React.FC<DocumentToolbarProps> = ({
  documentName,
  documentIcon = 'Document',
  confidence,
  documentType,
  activeUsers = [],
  currentUserId,
  showPresence = false,
  onTogglePresence,
  isSyncing = false,
  queuedOperations = 0,
  syncProgress,
  onManualSync,
  showOfflineIndicator = false,
  isEditMode = false,
  onSave,
  isSaving = false,
  isDirty = false,
  saveError,
  lastSaved,
  homeActions = [],
  reviewActions = [],
  aiToolsActions = [],
  moreActions = [],
  dropdownActions = [],
  onDropdownAction,
  onClose,
  onTabChange,
  defaultTab = 'home',
  className = '',
  defaultCollapsed = false,
  onCollapseChange,
}) => {
  const [activeTab, setActiveTab] = useState<ToolbarTab>(defaultTab)
  const [isCollapsed, setIsCollapsed] = useState<boolean>(() => {
    // Load from localStorage on mount
    const saved = localStorage.getItem(TOOLBAR_COLLAPSED_KEY)
    return saved !== null ? saved === 'true' : defaultCollapsed
  })

  // Persist collapse state to localStorage
  useEffect(() => {
    localStorage.setItem(TOOLBAR_COLLAPSED_KEY, isCollapsed.toString())
    onCollapseChange?.(isCollapsed)
  }, [isCollapsed, onCollapseChange])

  const handleTabChange = (tab: ToolbarTab) => {
    setActiveTab(tab)
    onTabChange?.(tab)
    // Auto-expand when switching tabs if collapsed
    if (isCollapsed) {
      setIsCollapsed(false)
    }
  }

  const toggleCollapse = () => {
    setIsCollapsed(!isCollapsed)
  }

  const renderToolbarGroup = (group: ToolbarGroup) => (
    <div
      key={group.id}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: designTokens.spacing[1],
        paddingRight: designTokens.spacing[3],
        borderRight: `1px solid ${designTokens.colors.border.subtle}`,
      }}
    >
      {group.actions.map(action => (
        <Tooltip key={action.id} content={action.tooltip || action.label}>
          <Button
            variant={action.variant || 'ghost'}
            size="sm"
            onClick={action.onClick}
            disabled={action.disabled}
            style={{
              fontWeight: 600,
              padding: action.showLabel ? '6px 12px' : '6px',
              borderRadius: '6px',
              position: 'relative',
            }}
          >
            <Icon name={action.icon} size={16} />
            {action.showLabel && <span>{action.label}</span>}
            {action.badge && (
              <Badge
                variant="default"
                style={{
                  position: 'absolute',
                  top: '-4px',
                  right: '-4px',
                  minWidth: '18px',
                  height: '18px',
                  padding: '2px 4px',
                  fontSize: '10px',
                  borderRadius: '9px',
                  background: designTokens.colors.accent.ai,
                  color: designTokens.colors.surface.primary,
                  border: `2px solid ${designTokens.colors.surface.secondary}`,
                }}
              >
                {action.badge}
              </Badge>
            )}
          </Button>
        </Tooltip>
      ))}
    </div>
  )

  const renderTabContent = () => {
    const groupsByTab = {
      home: homeActions,
      review: reviewActions,
      'ai-tools': aiToolsActions,
      more: moreActions,
    }

    const groups = groupsByTab[activeTab] || []

    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: designTokens.spacing[3],
          flex: 1,
          minWidth: 0,
          overflowX: 'auto',
          paddingBottom: '2px',
        }}
      >
        {groups.map(group => renderToolbarGroup(group))}
        {groups.length === 0 && (
          <div
            style={{
              color: designTokens.colors.text.tertiary,
              fontSize: designTokens.typography.fontSize.sm,
              padding: `0 ${designTokens.spacing[3]}`,
            }}
          >
            No actions available
          </div>
        )}
      </div>
    )
  }

  return (
    <div
      className={className}
      style={{
        display: 'flex',
        flexDirection: 'column',
        background: 'linear-gradient(to bottom, rgba(26, 26, 30, 0.95), rgba(22, 22, 26, 0.98))',
        backdropFilter: 'blur(10px)',
        borderBottom: `1px solid rgba(255, 255, 255, 0.08)`,
        boxShadow: '0 1px 3px rgba(0, 0, 0, 0.3)',
      }}
    >
      {/* Row 1: Document Info */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: `${designTokens.spacing[2]} ${designTokens.spacing[6]}`,
          minHeight: '44px',
        }}
      >
        {/* Left: Document Info */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[3],
            flex: 1,
            minWidth: 0,
          }}
        >
          <Icon
            name={documentIcon}
            size={18}
            style={{ color: designTokens.colors.accent.ai, flexShrink: 0 }}
          />
          <span
            style={{
              fontSize: designTokens.typography.fontSize.base,
              fontWeight: designTokens.typography.fontWeight.semibold,
              color: designTokens.colors.text.primary,
              maxWidth: '400px',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {documentName}
          </span>

          {confidence !== undefined && (
            <Badge
              variant="confidence"
              style={{
                background: `${designTokens.colors.accent.ai}15`,
                color: designTokens.colors.accent.ai,
                fontSize: designTokens.typography.fontSize.xs,
                padding: '3px 8px',
                borderRadius: '6px',
                border: `1px solid ${designTokens.colors.accent.ai}30`,
                flexShrink: 0,
              }}
            >
              {Math.round(confidence * 100)}% confidence
            </Badge>
          )}

          {documentType && (
            <Badge
              variant="default"
              style={{
                fontSize: designTokens.typography.fontSize.xs,
                padding: '3px 8px',
                borderRadius: '6px',
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                color: designTokens.colors.text.secondary,
                flexShrink: 0,
              }}
            >
              {documentType}
            </Badge>
          )}

          {/* Active Users */}
          {showPresence && activeUsers.length > 0 && (
            <ActiveUsers
              users={activeUsers}
              currentUserId={currentUserId || ''}
              size="small"
              onClick={onTogglePresence}
            />
          )}

          {/* Offline Indicator */}
          {showOfflineIndicator && isEditMode && (
            <OfflineIndicator
              position="inline"
              showDetails={false}
              showSyncProgress={queuedOperations > 0}
              syncProgress={syncProgress}
              isSyncing={isSyncing}
              onManualSync={onManualSync}
            />
          )}
        </div>

        {/* Right: Close Button */}
        {onClose && (
          <Tooltip content="Close">
            <Button
              variant="ghost"
              size="sm"
              onClick={onClose}
              style={{ padding: '6px', flexShrink: 0 }}
            >
              <Icon name="X" size={16} />
            </Button>
          </Tooltip>
        )}
      </div>

      {/* Row 2: Tabbed Toolbar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: `0 ${designTokens.spacing[6]} ${designTokens.spacing[2]}`,
          minHeight: '40px',
          gap: designTokens.spacing[4],
        }}
      >
        {/* Left: Tab Navigation */}
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[1] }}>
          {[
            { id: 'home' as ToolbarTab, label: 'Home', count: homeActions.length },
            { id: 'review' as ToolbarTab, label: 'Review', count: reviewActions.length },
            { id: 'ai-tools' as ToolbarTab, label: 'AI Tools', count: aiToolsActions.length },
            { id: 'more' as ToolbarTab, label: 'More', count: moreActions.length },
          ].map(tab => (
            <Tooltip
              key={tab.id}
              content={
                isCollapsed && activeTab === tab.id
                  ? `Click to expand ${tab.label} tools`
                  : undefined
              }
            >
              <button
                onClick={() => handleTabChange(tab.id)}
                style={{
                  padding: `${designTokens.spacing[1]} ${designTokens.spacing[3]}`,
                  background:
                    activeTab === tab.id
                      ? isCollapsed
                        ? 'rgba(0, 212, 255, 0.15)'
                        : 'rgba(255, 255, 255, 0.1)'
                      : 'transparent',
                  border: 'none',
                  borderBottom:
                    activeTab === tab.id
                      ? `2px solid ${designTokens.colors.accent.ai}`
                      : '2px solid transparent',
                  color:
                    activeTab === tab.id
                      ? designTokens.colors.text.primary
                      : designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  cursor: 'pointer',
                  transition: `all ${designTokens.animation.duration.fast}`,
                  fontFamily: designTokens.typography.fonts.sans.join(', '),
                  borderRadius: `${designTokens.borderRadius.md} ${designTokens.borderRadius.md} 0 0`,
                }}
                onMouseEnter={e => {
                  if (activeTab !== tab.id) {
                    ;(e.target as HTMLButtonElement).style.background = 'rgba(255, 255, 255, 0.05)'
                  }
                }}
                onMouseLeave={e => {
                  if (activeTab !== tab.id) {
                    ;(e.target as HTMLButtonElement).style.background = 'transparent'
                  }
                }}
              >
                {tab.label}
                {tab.count > 0 && (
                  <span
                    style={{
                      marginLeft: designTokens.spacing[1],
                      fontSize: designTokens.typography.fontSize.xs,
                      opacity: 0.6,
                    }}
                  >
                    ({tab.count})
                  </span>
                )}
              </button>
            </Tooltip>
          ))}
        </div>

        {/* Right: Primary Actions */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[2],
            flexShrink: 0,
          }}
        >
          {/* Collapse/Expand Button */}
          <Tooltip content={isCollapsed ? 'Expand Toolbar' : 'Collapse Toolbar'}>
            <Button
              variant="ghost"
              size="sm"
              onClick={toggleCollapse}
              style={{
                padding: '6px',
                flexShrink: 0,
                transition: `transform ${designTokens.animation.duration.fast}`,
              }}
            >
              <Icon
                name={isCollapsed ? 'ChevronDown' : 'ChevronDown'}
                size={14}
                style={{
                  transform: isCollapsed ? 'rotate(0deg)' : 'rotate(180deg)',
                  transition: `transform ${designTokens.animation.duration.fast}`,
                }}
              />
            </Button>
          </Tooltip>

          <div
            style={{
              width: '1px',
              height: '20px',
              background: 'rgba(255, 255, 255, 0.1)',
              margin: `0 ${designTokens.spacing[1]}`,
            }}
          />

          {/* Save Button */}
          {onSave && isEditMode && (
            <>
              <Tooltip
                content={
                  isDirty ? 'Save changes (auto-saves every 5 seconds)' : 'No unsaved changes'
                }
              >
                <Button
                  variant={isDirty ? 'success' : 'ghost'}
                  size="sm"
                  onClick={onSave}
                  disabled={!isDirty || isSaving}
                  style={{
                    fontWeight: 600,
                    padding: '6px 14px',
                    borderRadius: '6px',
                    ...(isDirty
                      ? {
                          background: 'linear-gradient(135deg, #51cf66 0%, #40c057 100%)',
                          boxShadow: '0 2px 8px rgba(64, 192, 87, 0.3)',
                        }
                      : {}),
                  }}
                >
                  <Icon name={isSaving ? 'Spinner' : 'Check'} size={14} />
                  <span>{isSaving ? 'Saving...' : isDirty ? 'Save' : 'Saved'}</span>
                </Button>
              </Tooltip>

              {saveError && (
                <Tooltip content={saveError}>
                  <Badge
                    variant="error"
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      padding: '4px 8px',
                    }}
                  >
                    <Icon name="X" size={12} />
                    Error
                  </Badge>
                </Tooltip>
              )}

              {lastSaved && !isDirty && (
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.tertiary,
                  }}
                >
                  Saved {new Date(lastSaved).toLocaleTimeString()}
                </span>
              )}

              <div
                style={{
                  width: '1px',
                  height: '20px',
                  background: 'rgba(255, 255, 255, 0.1)',
                  margin: `0 ${designTokens.spacing[1]}`,
                }}
              />
            </>
          )}

          {/* Dropdown Menu */}
          {dropdownActions.length > 0 && (
            <Dropdown
              options={dropdownActions}
              onChange={value => value && onDropdownAction?.(value)}
              placeholder="Actions"
              size="sm"
            />
          )}
        </div>
      </div>

      {/* Row 3: Tab Content (Contextual Tools) - Collapsible */}
      <div
        style={{
          display: isCollapsed ? 'none' : 'flex',
          alignItems: 'center',
          padding: `${designTokens.spacing[2]} ${designTokens.spacing[6]}`,
          borderTop: `1px solid rgba(255, 255, 255, 0.05)`,
          background: 'rgba(10, 10, 11, 0.4)',
          minHeight: '48px',
          opacity: isCollapsed ? 0 : 1,
          maxHeight: isCollapsed ? '0px' : '48px',
          overflow: 'hidden',
          transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
        }}
      >
        {renderTabContent()}
      </div>

      <style>{`
        /* Scrollbar styling for toolbar overflow */
        div::-webkit-scrollbar {
          height: 4px;
        }

        div::-webkit-scrollbar-track {
          background: transparent;
        }

        div::-webkit-scrollbar-thumb {
          background: ${designTokens.colors.border.medium};
          border-radius: 2px;
        }

        div::-webkit-scrollbar-thumb:hover {
          background: ${designTokens.colors.border.strong};
        }
      `}</style>
    </div>
  )
}

export default DocumentToolbar
