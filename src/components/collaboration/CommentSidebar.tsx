import React, { useState, useMemo, useCallback } from 'react'
import { Button, Icon, Badge, Tooltip } from '../ui'
import { designTokens } from '../../styles/tokens'
import { CommentThread, type CommentThreadData } from './CommentThread'

interface CommentSidebarProps {
  threads: CommentThreadData[]
  onAddComment: (threadId: string, content: string, mentions: string[]) => void
  onEditComment: (commentId: string, content: string) => void
  onDeleteComment: (commentId: string) => void
  onDeleteThread: (threadId: string) => void
  onResolveThread: (threadId: string) => void
  onUnresolveThread: (threadId: string) => void
  onReaction: (commentId: string, emoji: string) => void
  onThreadClick: (threadId: string) => void
  onClose: () => void
  isOpen: boolean
}

type FilterType = 'all' | 'active' | 'resolved' | 'mine'

export const CommentSidebar: React.FC<CommentSidebarProps> = ({
  threads,
  onAddComment,
  onEditComment,
  onDeleteComment,
  onDeleteThread,
  onResolveThread,
  onUnresolveThread,
  onReaction,
  onThreadClick,
  onClose,
  isOpen,
}) => {
  const [filter, setFilter] = useState<FilterType>('active')
  const [expandedThreadId, setExpandedThreadId] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')

  // Filter and search threads
  const filteredThreads = useMemo(() => {
    let filtered = threads

    // Apply filter
    switch (filter) {
      case 'active':
        filtered = filtered.filter(t => t.status === 'active')
        break
      case 'resolved':
        filtered = filtered.filter(t => t.status === 'resolved')
        break
      case 'mine':
        // TODO: Filter by current user's comments
        break
      default:
        break
    }

    // Apply search
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(
        thread =>
          thread.selection.text.toLowerCase().includes(query) ||
          thread.comments.some(comment => comment.content.toLowerCase().includes(query))
      )
    }

    // Sort by creation date (newest first)
    return filtered.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
  }, [threads, filter, searchQuery])

  const stats = useMemo(() => {
    const activeCount = threads.filter(t => t.status === 'active').length
    const resolvedCount = threads.filter(t => t.status === 'resolved').length
    const totalComments = threads.reduce((sum, t) => sum + t.comments.length, 0)

    return {
      total: threads.length,
      active: activeCount,
      resolved: resolvedCount,
      comments: totalComments,
    }
  }, [threads])

  const handleThreadClick = useCallback(
    (threadId: string) => {
      if (expandedThreadId === threadId) {
        setExpandedThreadId(null)
      } else {
        setExpandedThreadId(threadId)
        onThreadClick(threadId)
      }
    },
    [expandedThreadId, onThreadClick]
  )

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        right: 0,
        width: '400px',
        height: '100vh',
        backgroundColor: designTokens.colors.background.paper,
        borderLeft: `1px solid ${designTokens.colors.border.subtle}`,
        boxShadow: designTokens.shadows.lg,
        display: 'flex',
        flexDirection: 'column',
        zIndex: 1000,
        fontFamily: designTokens.typography.fonts.sans.join(', '),
        transform: isOpen ? 'translateX(0)' : 'translateX(100%)',
        transition: 'transform 0.2s ease-out',
        pointerEvents: isOpen ? 'auto' : 'none',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: designTokens.spacing[4],
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
          backgroundColor: designTokens.colors.background.canvas,
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: designTokens.spacing[2],
          }}
        >
          <h2
            style={{
              margin: 0,
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: 600,
              color: designTokens.colors.text.primary,
            }}
          >
            Comments
          </h2>
          <Tooltip content="Close sidebar">
            <Button variant="ghost" size="sm" onClick={onClose} aria-label="Close sidebar">
              <Icon name="X" size={18} />
            </Button>
          </Tooltip>
        </div>

        {/* Stats */}
        <div
          style={{
            display: 'flex',
            gap: designTokens.spacing[2],
            marginBottom: designTokens.spacing[2],
          }}
        >
          <Badge variant="default" size="md">
            {stats.total} threads
          </Badge>
          <Badge variant="warning" size="md">
            {stats.active} active
          </Badge>
          <Badge variant="success" size="md">
            {stats.resolved} resolved
          </Badge>
          <Badge variant="ai" size="md">
            {stats.comments} comments
          </Badge>
        </div>

        {/* Search */}
        <div style={{ position: 'relative', marginBottom: designTokens.spacing[2] }}>
          <input
            type="text"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            placeholder="Search comments..."
            style={{
              width: '100%',
              padding: `${designTokens.spacing[2]} ${designTokens.spacing[2]} ${designTokens.spacing[2]} 36px`,
              border: `1px solid ${designTokens.colors.border.subtle}`,
              borderRadius: designTokens.borderRadius.md,
              fontFamily: designTokens.typography.fonts.sans.join(', '),
              fontSize: designTokens.typography.fontSize.sm,
              backgroundColor: designTokens.colors.surface.tertiary,
              color: designTokens.colors.text.primary,
            }}
          />
          <Icon
            name="Search"
            size={16}
            style={{
              position: 'absolute',
              left: '12px',
              top: '50%',
              transform: 'translateY(-50%)',
              color: designTokens.colors.text.secondary,
            }}
          />
        </div>

        {/* Filters */}
        <div style={{ display: 'flex', gap: designTokens.spacing[1] }}>
          <Button
            size="sm"
            variant={filter === 'all' ? 'primary' : 'secondary'}
            onClick={() => setFilter('all')}
          >
            All
          </Button>
          <Button
            size="sm"
            variant={filter === 'active' ? 'primary' : 'secondary'}
            onClick={() => setFilter('active')}
          >
            Active
          </Button>
          <Button
            size="sm"
            variant={filter === 'resolved' ? 'primary' : 'secondary'}
            onClick={() => setFilter('resolved')}
          >
            Resolved
          </Button>
          <Button
            size="sm"
            variant={filter === 'mine' ? 'primary' : 'secondary'}
            onClick={() => setFilter('mine')}
          >
            Mine
          </Button>
        </div>
      </div>

      {/* Thread List */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: designTokens.spacing[4],
        }}
      >
        {filteredThreads.length === 0 ? (
          <div
            style={{
              textAlign: 'center',
              padding: designTokens.spacing[20],
              color: designTokens.colors.text.secondary,
            }}
          >
            <Icon
              name="MessageCircle"
              size={48}
              style={{ marginBottom: designTokens.spacing[4], opacity: 0.5 }}
            />
            <div
              style={{
                fontSize: designTokens.typography.fontSize.base,
                marginBottom: designTokens.spacing[1],
              }}
            >
              No comments yet
            </div>
            <div style={{ fontSize: designTokens.typography.fontSize.sm }}>
              Select text in the document to add a comment
            </div>
          </div>
        ) : (
          filteredThreads.map(thread => (
            <div
              key={thread.id}
              style={{
                marginBottom: designTokens.spacing[4],
              }}
            >
              {/* Thread Preview */}
              <div
                onClick={() => handleThreadClick(thread.id)}
                style={{
                  padding: designTokens.spacing[2],
                  backgroundColor: designTokens.colors.background.canvas,
                  border: `1px solid ${
                    expandedThreadId === thread.id
                      ? designTokens.colors.accent.ai
                      : designTokens.colors.border.subtle
                  }`,
                  borderRadius: designTokens.borderRadius.md,
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={e => {
                  e.currentTarget.style.borderColor = designTokens.colors.accent.ai
                  e.currentTarget.style.boxShadow = designTokens.shadows.sm
                }}
                onMouseLeave={e => {
                  e.currentTarget.style.borderColor =
                    expandedThreadId === thread.id
                      ? designTokens.colors.accent.ai
                      : designTokens.colors.border.subtle
                  e.currentTarget.style.boxShadow = 'none'
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'flex-start',
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.sm,
                        color: designTokens.colors.text.primary,
                        fontStyle: 'italic',
                        marginBottom: designTokens.spacing[1],
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        display: '-webkit-box',
                        WebkitLineClamp: 2,
                        WebkitBoxOrient: 'vertical',
                        wordBreak: 'break-word',
                      }}
                    >
                      "{thread.selection.text}"
                    </div>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.secondary,
                      }}
                    >
                      {thread.comments.length} comment{thread.comments.length !== 1 ? 's' : ''} ·{' '}
                      {formatDate(thread.createdAt)}
                    </div>
                  </div>
                  <div
                    style={{
                      display: 'flex',
                      gap: designTokens.spacing[1],
                      marginLeft: designTokens.spacing[2],
                    }}
                  >
                    {thread.status === 'resolved' ? (
                      <Badge variant="success" size="sm">
                        <Icon name="AlertCircle" size={12} style={{ marginRight: '4px' }} />
                        Resolved
                      </Badge>
                    ) : (
                      <Badge variant="warning" size="sm">
                        Active
                      </Badge>
                    )}
                  </div>
                </div>
              </div>

              {/* Expanded Thread */}
              {expandedThreadId === thread.id && (
                <div style={{ marginTop: designTokens.spacing[2] }}>
                  <CommentThread
                    thread={thread}
                    onAddComment={onAddComment}
                    onEditComment={onEditComment}
                    onDeleteComment={onDeleteComment}
                    onDeleteThread={onDeleteThread}
                    onResolveThread={onResolveThread}
                    onUnresolveThread={onUnresolveThread}
                    onReaction={onReaction}
                    compact={false}
                  />
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  )
}

// Helper function
function formatDate(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / 86400000)

  if (days === 0) return 'Today'
  if (days === 1) return 'Yesterday'
  if (days < 7) return `${days} days ago`
  return date.toLocaleDateString()
}
