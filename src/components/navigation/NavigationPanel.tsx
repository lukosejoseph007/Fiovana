/**
 * Navigation Panel - Smart File Tree
 *
 * Context-aware navigator (not traditional folders) with workspace intelligence integration.
 * Features sections for workspace health, active documents, conversations, and smart collections.
 */

import React, { useState, useEffect, useCallback } from 'react'
import Badge from '../ui/Badge'
import { Icon } from '../ui/Icon'
import Progress from '../ui/Progress'
import Tooltip from '../ui/Tooltip'
import { useLayout } from '../layout/useLayoutContext'
import {
  workspaceAnalyzerService,
  documentService,
  conversationIntelligenceService,
  smartOrganizerService,
} from '../../services'
import { WorkspaceAnalysis, Document, SmartOrganization, ApiResponse } from '../../types'
import { colors, spacing, typography, animation } from '../../styles/tokens'

type IconName =
  | 'Document'
  | 'PDF'
  | 'Word'
  | 'PowerPoint'
  | 'AIStatus'
  | 'Health'
  | 'Confidence'
  | 'Compare'
  | 'Generate'
  | 'Analyze'
  | 'Search'
  | 'Settings'
  | 'Workspace'
  | 'Spinner'
  | 'Pulse'
  | 'User'
  | 'Collaboration'
  | 'ChevronDown'

interface NavigationItem {
  id: string
  label: string
  icon: IconName
  count?: number
  status?: 'active' | 'warning' | 'critical' | 'info'
  confidence?: number
  lastUpdated?: Date
  children?: NavigationItem[]
}

interface NavigationSection {
  id: string
  title: string
  collapsed: boolean
  items: NavigationItem[]
  loading: boolean
  error?: string
}

interface NavigationPanelProps {
  workspaceId: string
  onItemSelect?: (item: NavigationItem) => void
  className?: string
  collapsed?: boolean
}

export const NavigationPanel: React.FC<NavigationPanelProps> = ({
  workspaceId,
  onItemSelect,
  className = '',
  collapsed = false,
}) => {
  const { toggleNavigation } = useLayout()
  const [sections, setSections] = useState<NavigationSection[]>([
    {
      id: 'workspace-intelligence',
      title: 'Workspace Intelligence',
      collapsed: false,
      items: [],
      loading: true,
    },
    {
      id: 'active-documents',
      title: 'Active Documents',
      collapsed: false,
      items: [],
      loading: true,
    },
    {
      id: 'recent-conversations',
      title: 'Recent Conversations',
      collapsed: false,
      items: [],
      loading: true,
    },
    {
      id: 'smart-collections',
      title: 'Smart Collections',
      collapsed: true,
      items: [],
      loading: true,
    },
  ])

  const [workspaceHealth, setWorkspaceHealth] = useState<WorkspaceAnalysis | null>(null)
  const [refreshing, setRefreshing] = useState(false)

  // Load workspace intelligence data
  const loadWorkspaceIntelligence = useCallback(async () => {
    try {
      const response: ApiResponse<WorkspaceAnalysis> =
        await workspaceAnalyzerService.analyzeWorkspace(workspaceId)

      if (response.success && response.data) {
        setWorkspaceHealth(response.data)

        const healthItems: NavigationItem[] = [
          {
            id: 'health-score',
            label: 'Health Score',
            icon: 'Health',
            status:
              response.data.health?.score > 80
                ? 'active'
                : response.data.health?.score > 60
                  ? 'warning'
                  : 'critical',
            confidence: response.data.health?.score,
          },
          {
            id: 'knowledge-gaps',
            label: 'Knowledge Gaps',
            icon: 'Analyze',
            count: response.data.insights?.length || 0,
            status: (response.data.insights?.length || 0) > 5 ? 'warning' : 'info',
          },
          {
            id: 'recommendations',
            label: 'Recommendations',
            icon: 'Generate',
            count: response.data.health?.recommendations?.length || 0,
            status: 'info',
          },
        ]

        setSections(prev =>
          prev.map(section =>
            section.id === 'workspace-intelligence'
              ? { ...section, items: healthItems, loading: false }
              : section
          )
        )
      }
    } catch (error) {
      console.error('Failed to load workspace intelligence:', error)
      setSections(prev =>
        prev.map(section =>
          section.id === 'workspace-intelligence'
            ? { ...section, loading: false, error: 'Failed to load workspace data' }
            : section
        )
      )
    }
  }, [workspaceId])

  // Load active documents
  const loadActiveDocuments = useCallback(async () => {
    try {
      const response: ApiResponse<Document[]> = await documentService.listDocuments(workspaceId)

      if (response.success && response.data) {
        const documentItems: NavigationItem[] = response.data
          .filter(
            doc =>
              (doc.metadata as unknown as Record<string, unknown>)?.status === 'active' ||
              (doc.metadata as unknown as Record<string, unknown>)?.recentlyEdited
          )
          .slice(0, 10) // Show most recent 10
          .map(doc => ({
            id: doc.id,
            label:
              (doc as unknown as { title?: string; name?: string }).title ||
              (doc as unknown as { title?: string; name?: string }).name ||
              'Untitled Document',
            icon: getDocumentIcon((doc as unknown as { type?: string }).type || 'document'),
            status: getDocumentStatus(doc),
            lastUpdated: (doc.metadata as unknown as { updatedAt?: string })?.updatedAt
              ? new Date((doc.metadata as unknown as { updatedAt: string }).updatedAt)
              : undefined,
          }))

        setSections(prev =>
          prev.map(section =>
            section.id === 'active-documents'
              ? { ...section, items: documentItems, loading: false }
              : section
          )
        )
      }
    } catch (error) {
      console.error('Failed to load active documents:', error)
      setSections(prev =>
        prev.map(section =>
          section.id === 'active-documents'
            ? { ...section, loading: false, error: 'Failed to load documents' }
            : section
        )
      )
    }
  }, [workspaceId])

  // Load recent conversations
  const loadRecentConversations = useCallback(async () => {
    try {
      const response: ApiResponse<unknown> =
        await conversationIntelligenceService.getConversationAnalytics(workspaceId, 'last_7_days')

      if (response.success && response.data) {
        // Mock conversation data - replace with actual conversation API
        const conversationItems: NavigationItem[] = [
          {
            id: 'conv-1',
            label: 'Document Analysis Chat',
            icon: 'AIStatus',
            status: 'active',
            lastUpdated: new Date(Date.now() - 1000 * 60 * 30), // 30 minutes ago
          },
          {
            id: 'conv-2',
            label: 'Style Learning Session',
            icon: 'AIStatus',
            status: 'info',
            lastUpdated: new Date(Date.now() - 1000 * 60 * 60 * 2), // 2 hours ago
          },
          {
            id: 'conv-3',
            label: 'Workspace Optimization',
            icon: 'AIStatus',
            status: 'info',
            lastUpdated: new Date(Date.now() - 1000 * 60 * 60 * 24), // 1 day ago
          },
        ]

        setSections(prev =>
          prev.map(section =>
            section.id === 'recent-conversations'
              ? { ...section, items: conversationItems, loading: false }
              : section
          )
        )
      }
    } catch (error) {
      console.error('Failed to load conversations:', error)
      setSections(prev =>
        prev.map(section =>
          section.id === 'recent-conversations'
            ? { ...section, loading: false, error: 'Failed to load conversations' }
            : section
        )
      )
    }
  }, [workspaceId])

  // Load smart collections
  const loadSmartCollections = useCallback(async () => {
    try {
      const response: ApiResponse<SmartOrganization> =
        await smartOrganizerService.getSmartOrganization(workspaceId)

      if (response.success && response.data) {
        const collectionItems: NavigationItem[] = [
          {
            id: 'needs-review',
            label: 'Needs Review',
            icon: 'Analyze',
            count: response.data.suggestions?.filter(s => s.type === 'categorization').length || 0,
            status: 'warning',
          },
          {
            id: 'outdated',
            label: 'Outdated Content',
            icon: 'Health',
            count: response.data.suggestions?.filter(s => s.type === 'cleanup').length || 0,
            status: 'critical',
          },
          {
            id: 'related',
            label: 'Related Documents',
            icon: 'Compare',
            count: response.data.suggestions?.filter(s => s.type === 'restructure').length || 0,
            status: 'info',
          },
          {
            id: 'duplicates',
            label: 'Potential Duplicates',
            icon: 'Document',
            count: response.data.suggestions?.filter(s => s.type === 'tagging').length || 0,
            status: 'warning',
          },
        ]

        setSections(prev =>
          prev.map(section =>
            section.id === 'smart-collections'
              ? { ...section, items: collectionItems, loading: false }
              : section
          )
        )
      }
    } catch (error) {
      console.error('Failed to load smart collections:', error)
      setSections(prev =>
        prev.map(section =>
          section.id === 'smart-collections'
            ? { ...section, loading: false, error: 'Failed to load collections' }
            : section
        )
      )
    }
  }, [workspaceId])

  // Initial data load
  useEffect(() => {
    if (workspaceId) {
      loadWorkspaceIntelligence()
      loadActiveDocuments()
      loadRecentConversations()
      loadSmartCollections()
    }
  }, [
    workspaceId,
    loadWorkspaceIntelligence,
    loadActiveDocuments,
    loadRecentConversations,
    loadSmartCollections,
  ])

  // Refresh all data
  const handleRefresh = useCallback(async () => {
    setRefreshing(true)
    try {
      await Promise.all([
        loadWorkspaceIntelligence(),
        loadActiveDocuments(),
        loadRecentConversations(),
        loadSmartCollections(),
      ])
    } finally {
      setRefreshing(false)
    }
  }, [
    loadWorkspaceIntelligence,
    loadActiveDocuments,
    loadRecentConversations,
    loadSmartCollections,
  ])

  // Toggle section collapsed state
  const toggleSection = useCallback((sectionId: string) => {
    setSections(prev =>
      prev.map(section =>
        section.id === sectionId ? { ...section, collapsed: !section.collapsed } : section
      )
    )
  }, [])

  // Handle item selection
  const handleItemSelect = useCallback(
    (item: NavigationItem) => {
      onItemSelect?.(item)
    },
    [onItemSelect]
  )

  // Render collapsed version with icons only
  if (collapsed) {
    return (
      <div
        className={`navigation-panel navigation-panel--collapsed ${className}`}
        style={{
          width: '100%',
          height: '100%',
          backgroundColor: colors.surface.primary,
          borderRight: `1px solid ${colors.border.subtle}`,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          padding: spacing[2],
        }}
      >
        {/* Collapsed Header with Expand Button */}
        <div
          style={{
            padding: spacing[3],
            borderBottom: `1px solid ${colors.border.subtle}`,
            width: '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: spacing[3],
            marginBottom: spacing[4],
          }}
        >
          <Tooltip content="Workspace Navigator">
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '6px',
                backgroundColor: colors.surface.secondary,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: colors.text.primary,
              }}
            >
              <Icon name="Workspace" size={18} />
            </div>
          </Tooltip>

          <Tooltip content="Expand navigation panel">
            <button
              onClick={toggleNavigation}
              style={{
                background: 'transparent',
                border: `1px solid ${colors.border.subtle}`,
                color: colors.text.secondary,
                cursor: 'pointer',
                padding: spacing[1],
                borderRadius: '4px',
                transition: `all ${animation.duration.fast} ${animation.easing.easeOut}`,
                width: '28px',
                height: '28px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = colors.state.hover
                e.currentTarget.style.borderColor = colors.accent.ai
                e.currentTarget.style.color = colors.text.primary
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = 'transparent'
                e.currentTarget.style.borderColor = colors.border.subtle
                e.currentTarget.style.color = colors.text.secondary
              }}
            >
              <Icon name="ChevronDown" size={12} style={{ transform: 'rotate(90deg)' }} />
            </button>
          </Tooltip>
        </div>

        {/* Collapsed Navigation Icons */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: spacing[3], width: '100%' }}>
          {/* Workspace Health Icon */}
          <Tooltip content={`Health Score: ${workspaceHealth?.health?.score || 'N/A'}%`}>
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '6px',
                backgroundColor: colors.surface.secondary,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: colors.accent.success,
                cursor: 'pointer',
                transition: `all ${animation.duration.fast}`,
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = colors.state.hover
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = colors.surface.secondary
              }}
            >
              <Icon name="Health" size={16} />
            </div>
          </Tooltip>

          {/* Active Documents Icon */}
          <Tooltip content="Active Documents">
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '6px',
                backgroundColor: colors.surface.secondary,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: colors.text.secondary,
                cursor: 'pointer',
                transition: `all ${animation.duration.fast}`,
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = colors.state.hover
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = colors.surface.secondary
              }}
            >
              <Icon name="Document" size={16} />
            </div>
          </Tooltip>

          {/* Conversations Icon */}
          <Tooltip content="Recent Conversations">
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '6px',
                backgroundColor: colors.surface.secondary,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: colors.accent.ai,
                cursor: 'pointer',
                transition: `all ${animation.duration.fast}`,
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = colors.state.hover
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = colors.surface.secondary
              }}
            >
              <Icon name="MessageCircle" size={16} />
            </div>
          </Tooltip>

          {/* Smart Collections Icon */}
          <Tooltip content="Smart Collections">
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '6px',
                backgroundColor: colors.surface.secondary,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: colors.text.secondary,
                cursor: 'pointer',
                transition: `all ${animation.duration.fast}`,
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = colors.state.hover
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = colors.surface.secondary
              }}
            >
              <Icon name="Layers" size={16} />
            </div>
          </Tooltip>
        </div>
      </div>
    )
  }

  // Render expanded version
  return (
    <div
      className={`navigation-panel ${className}`}
      style={{
        width: '100%',
        height: '100%',
        backgroundColor: colors.surface.primary,
        borderRight: `1px solid ${colors.border.subtle}`,
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: spacing[4],
          borderBottom: `1px solid ${colors.border.subtle}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: spacing[2] }}>
          <h2
            style={{
              fontSize: typography.fontSize.sm,
              fontWeight: typography.fontWeight.semibold,
              color: colors.text.primary,
              margin: 0,
            }}
          >
            Workspace Navigator
          </h2>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: spacing[1] }}>
          <Tooltip content="Refresh workspace data">
            <button
              onClick={handleRefresh}
              disabled={refreshing}
              style={{
                background: 'transparent',
                border: 'none',
                color: colors.text.secondary,
                cursor: refreshing ? 'wait' : 'pointer',
                padding: spacing[1],
                borderRadius: '4px',
                transition: `all ${animation.duration.fast} ${animation.easing.easeOut}`,
              }}
              onMouseEnter={e => {
                if (!refreshing) {
                  e.currentTarget.style.backgroundColor = colors.state.hover
                  e.currentTarget.style.color = colors.text.primary
                }
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = 'transparent'
                e.currentTarget.style.color = colors.text.secondary
              }}
            >
              <Icon name="Settings" size={14} />
            </button>
          </Tooltip>

          <Tooltip content="Collapse navigation panel">
            <button
              onClick={toggleNavigation}
              style={{
                background: 'transparent',
                border: 'none',
                color: colors.text.secondary,
                cursor: 'pointer',
                padding: spacing[1],
                borderRadius: '4px',
                transition: `all ${animation.duration.fast} ${animation.easing.easeOut}`,
              }}
              onMouseEnter={e => {
                e.currentTarget.style.backgroundColor = colors.state.hover
                e.currentTarget.style.color = colors.text.primary
              }}
              onMouseLeave={e => {
                e.currentTarget.style.backgroundColor = 'transparent'
                e.currentTarget.style.color = colors.text.secondary
              }}
            >
              <Icon name="ChevronDown" size={14} />
            </button>
          </Tooltip>
        </div>
      </div>

      {/* Workspace Health Summary */}
      {workspaceHealth && (
        <div
          style={{
            padding: spacing[4],
            borderBottom: `1px solid ${colors.border.subtle}`,
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: spacing[2],
            }}
          >
            <span
              style={{
                fontSize: typography.fontSize.xs,
                color: colors.text.secondary,
                textTransform: 'uppercase',
                letterSpacing: typography.letterSpacing.wide,
              }}
            >
              Health Score
            </span>
            <Badge
              variant={
                workspaceHealth.health?.score > 80
                  ? 'success'
                  : workspaceHealth.health?.score > 60
                    ? 'warning'
                    : 'error'
              }
              size="sm"
            >
              {workspaceHealth.health?.score}%
            </Badge>
          </div>
          <Progress value={workspaceHealth.health?.score || 0} variant="confidence" size="sm" />
        </div>
      )}

      {/* Navigation Sections */}
      <div
        className="navigation-content"
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: spacing[2],
        }}
      >
        {sections.map(section => (
          <NavigationSection
            key={section.id}
            section={section}
            onToggle={() => toggleSection(section.id)}
            onItemSelect={handleItemSelect}
          />
        ))}
      </div>

      {/* CSS for animations and scrollbars */}
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        /* Custom scrollbar for navigation content */
        .navigation-panel .navigation-content::-webkit-scrollbar {
          width: 6px;
        }

        .navigation-panel .navigation-content::-webkit-scrollbar-track {
          background: ${colors.surface.tertiary};
          border-radius: 3px;
        }

        .navigation-panel .navigation-content::-webkit-scrollbar-thumb {
          background: ${colors.border.medium};
          border-radius: 3px;
        }

        .navigation-panel .navigation-content::-webkit-scrollbar-thumb:hover {
          background: ${colors.border.strong};
        }
      `}</style>
    </div>
  )
}

// Navigation section component
interface NavigationSectionProps {
  section: NavigationSection
  onToggle: () => void
  onItemSelect: (item: NavigationItem) => void
}

const NavigationSection: React.FC<NavigationSectionProps> = ({
  section,
  onToggle,
  onItemSelect,
}) => {
  return (
    <div style={{ marginBottom: spacing[4] }}>
      {/* Section Header */}
      <button
        onClick={onToggle}
        style={{
          width: '100%',
          background: 'transparent',
          border: 'none',
          padding: `${spacing[2]} ${spacing[3]}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          cursor: 'pointer',
          borderRadius: '6px',
          transition: `all ${animation.duration.fast} ${animation.easing.easeOut}`,
        }}
        onMouseEnter={e => {
          e.currentTarget.style.backgroundColor = colors.state.hover
        }}
        onMouseLeave={e => {
          e.currentTarget.style.backgroundColor = 'transparent'
        }}
      >
        <span
          style={{
            fontSize: typography.fontSize.xs,
            fontWeight: typography.fontWeight.semibold,
            color: colors.text.primary,
            textTransform: 'uppercase',
            letterSpacing: typography.letterSpacing.wide,
          }}
        >
          {section.title}
        </span>

        <Icon name="ChevronDown" size={12} />
      </button>

      {/* Section Content */}
      {!section.collapsed && (
        <div
          style={{
            marginTop: spacing[1],
            animation: `slideDown ${animation.duration.normal} ${animation.easing.easeOut}`,
          }}
        >
          {section.loading && (
            <div
              style={{
                padding: spacing[3],
                display: 'flex',
                alignItems: 'center',
                gap: spacing[2],
              }}
            >
              <Icon name="Spinner" size={14} />
              <span
                style={{
                  fontSize: typography.fontSize.xs,
                  color: colors.text.secondary,
                }}
              >
                Loading...
              </span>
            </div>
          )}

          {section.error && (
            <div
              style={{
                padding: spacing[3],
                color: colors.accent.alert,
                fontSize: typography.fontSize.xs,
              }}
            >
              {section.error}
            </div>
          )}

          {!section.loading &&
            !section.error &&
            section.items.map(item => (
              <NavigationItem key={item.id} item={item} onSelect={() => onItemSelect(item)} />
            ))}
        </div>
      )}
    </div>
  )
}

// Navigation item component
interface NavigationItemProps {
  item: NavigationItem
  onSelect: () => void
}

const NavigationItem: React.FC<NavigationItemProps> = ({ item, onSelect }) => {
  const formatLastUpdated = (date?: Date) => {
    if (!date) return ''

    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / (1000 * 60))
    const hours = Math.floor(diff / (1000 * 60 * 60))
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))

    if (minutes < 60) return `${minutes}m ago`
    if (hours < 24) return `${hours}h ago`
    return `${days}d ago`
  }

  return (
    <button
      onClick={onSelect}
      style={{
        width: '100%',
        background: 'transparent',
        border: 'none',
        padding: `${spacing[2]} ${spacing[3]}`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        cursor: 'pointer',
        borderRadius: '4px',
        transition: `all ${animation.duration.fast} ${animation.easing.easeOut}`,
        marginBottom: spacing[0.5],
      }}
      onMouseEnter={e => {
        e.currentTarget.style.backgroundColor = colors.state.hover
      }}
      onMouseLeave={e => {
        e.currentTarget.style.backgroundColor = 'transparent'
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: spacing[2],
          flex: 1,
          minWidth: 0,
        }}
      >
        <Icon name={item.icon} size={14} />

        <span
          style={{
            fontSize: typography.fontSize.sm,
            color: colors.text.primary,
            textAlign: 'left',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {item.label}
        </span>
      </div>

      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: spacing[2],
          flexShrink: 0,
        }}
      >
        {item.confidence !== undefined && (
          <span
            style={{
              fontSize: typography.fontSize.xs,
              color: colors.text.tertiary,
            }}
          >
            {item.confidence}%
          </span>
        )}

        {item.count !== undefined && (
          <Badge variant="default" size="sm">
            {item.count}
          </Badge>
        )}

        {item.lastUpdated && (
          <span
            style={{
              fontSize: typography.fontSize.xs,
              color: colors.text.tertiary,
            }}
          >
            {formatLastUpdated(item.lastUpdated)}
          </span>
        )}
      </div>
    </button>
  )
}

// Helper functions
const getDocumentIcon = (type: string): IconName => {
  switch (type?.toLowerCase()) {
    case 'pdf':
      return 'PDF'
    case 'docx':
    case 'doc':
      return 'Word'
    case 'xlsx':
    case 'xls':
      return 'Document'
    case 'pptx':
    case 'ppt':
      return 'PowerPoint'
    case 'txt':
      return 'Document'
    default:
      return 'Document'
  }
}

const getDocumentStatus = (doc: Document): 'active' | 'warning' | 'critical' | 'info' => {
  const metadata = doc.metadata as unknown as Record<string, unknown>
  if (metadata?.recentlyEdited) return 'active'
  if (metadata?.needsReview) return 'warning'
  if (metadata?.hasErrors) return 'critical'
  return 'info'
}

export default NavigationPanel
