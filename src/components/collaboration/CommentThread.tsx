import React, { useState, useCallback } from 'react'
import { Button, Icon, Badge, Tooltip } from '../ui'
import { designTokens } from '../../styles/tokens'
import { useCollaboration } from '../../context/useCollaboration'

export interface Comment {
  id: string
  threadId: string
  author: {
    id: string
    name: string
    color: string
    avatar?: string
  }
  content: string
  timestamp: Date
  edited?: boolean
  editedAt?: Date
  mentions: string[] // User IDs mentioned in the comment
  reactions: Record<string, string[]> // emoji -> user IDs
  isResolved?: boolean
}

export interface CommentThreadData {
  id: string
  documentId: string
  selection: {
    start: number
    end: number
    text: string
  }
  comments: Comment[]
  createdAt: Date
  resolvedAt?: Date
  resolvedBy?: string
  status: 'active' | 'resolved'
  position?: {
    x: number
    y: number
  }
}

interface CommentThreadProps {
  thread: CommentThreadData
  onAddComment: (threadId: string, content: string, mentions: string[]) => void
  onEditComment: (commentId: string, content: string) => void
  onDeleteComment: (commentId: string) => void
  onDeleteThread?: (threadId: string) => void
  onResolveThread: (threadId: string) => void
  onUnresolveThread: (threadId: string) => void
  onReaction: (commentId: string, emoji: string) => void
  onClose?: () => void
  compact?: boolean
}

export const CommentThread: React.FC<CommentThreadProps> = ({
  thread,
  onAddComment,
  onEditComment,
  onDeleteComment,
  onDeleteThread,
  onResolveThread,
  onUnresolveThread,
  onReaction,
  onClose,
  compact = false,
}) => {
  const collaboration = useCollaboration()
  const currentUserId = collaboration.settings.username || 'Anonymous'

  const [newCommentContent, setNewCommentContent] = useState('')
  const [editingCommentId, setEditingCommentId] = useState<string | null>(null)
  const [editContent, setEditContent] = useState('')
  const [showEmojiPicker, setShowEmojiPicker] = useState<string | null>(null)

  const handleAddComment = useCallback(() => {
    if (newCommentContent.trim()) {
      // Extract mentions from content (@username)
      const mentions = extractMentions(newCommentContent)
      onAddComment(thread.id, newCommentContent, mentions)
      setNewCommentContent('')
    }
  }, [newCommentContent, onAddComment, thread.id])

  const handleEditComment = useCallback(
    (commentId: string) => {
      if (editContent.trim()) {
        onEditComment(commentId, editContent)
        setEditingCommentId(null)
        setEditContent('')
      }
    },
    [editContent, onEditComment]
  )

  const startEditing = useCallback((comment: Comment) => {
    setEditingCommentId(comment.id)
    setEditContent(comment.content)
  }, [])

  const cancelEditing = useCallback(() => {
    setEditingCommentId(null)
    setEditContent('')
  }, [])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, action: () => void) => {
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault()
        action()
      } else if (e.key === 'Escape') {
        e.preventDefault()
        if (editingCommentId) {
          cancelEditing()
        }
      }
    },
    [editingCommentId, cancelEditing]
  )

  const emojis = ['👍', '❤️', '😄', '🎉', '👏', '🤔']

  return (
    <div
      style={{
        backgroundColor: designTokens.colors.background.paper,
        border: `1px solid ${designTokens.colors.border.subtle}`,
        borderRadius: designTokens.borderRadius.md,
        padding: compact ? designTokens.spacing[2] : designTokens.spacing[4],
        boxShadow: designTokens.shadows.md,
        maxWidth: compact ? '300px' : '400px',
        fontFamily: designTokens.typography.fonts.sans.join(', '),
      }}
    >
      {/* Thread Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: designTokens.spacing[2],
          paddingBottom: designTokens.spacing[2],
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
        }}
      >
        <div style={{ flex: 1 }}>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
              marginBottom: '4px',
            }}
          >
            Selected text:
          </div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.primary,
              fontStyle: 'italic',
              maxHeight: '60px',
              overflow: 'auto',
              padding: '4px 8px',
              backgroundColor: designTokens.colors.background.canvas,
              borderRadius: designTokens.borderRadius.sm,
              borderLeft: `3px solid ${designTokens.colors.accent.ai}`,
            }}
          >
            "{thread.selection.text}"
          </div>
        </div>
        <div
          style={{
            display: 'flex',
            gap: designTokens.spacing[1],
            marginLeft: designTokens.spacing[2],
          }}
        >
          {thread.status === 'active' ? (
            <Tooltip content="Resolve thread">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onResolveThread(thread.id)}
                aria-label="Resolve thread"
              >
                <Icon name="AlertCircle" size={16} />
              </Button>
            </Tooltip>
          ) : (
            <Badge variant="success" size="sm">
              Resolved
            </Badge>
          )}
          {onDeleteThread && thread.comments.length === 0 && (
            <Tooltip content="Delete thread">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  if (window.confirm('Delete this thread?')) {
                    onDeleteThread(thread.id)
                  }
                }}
                aria-label="Delete thread"
                style={{ color: designTokens.colors.accent.alert }}
              >
                <Icon name="X" size={16} />
              </Button>
            </Tooltip>
          )}
          {onClose && (
            <Tooltip content="Close">
              <Button variant="ghost" size="sm" onClick={onClose} aria-label="Close thread">
                <Icon name="X" size={16} />
              </Button>
            </Tooltip>
          )}
        </div>
      </div>

      {/* Comments List */}
      <div
        style={{
          maxHeight: compact ? '200px' : '400px',
          overflowY: 'auto',
          marginBottom: designTokens.spacing[2],
        }}
      >
        {thread.comments.map(comment => (
          <div
            key={comment.id}
            style={{
              padding: designTokens.spacing[2],
              marginBottom: designTokens.spacing[2],
              backgroundColor: designTokens.colors.background.canvas,
              borderRadius: designTokens.borderRadius.sm,
              position: 'relative',
            }}
          >
            {/* Comment Header */}
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: designTokens.spacing[1],
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[1] }}>
                {/* Avatar */}
                <div
                  style={{
                    width: '24px',
                    height: '24px',
                    borderRadius: '50%',
                    backgroundColor: comment.author.color,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    color: 'white',
                    fontSize: '12px',
                    fontWeight: 600,
                  }}
                >
                  {getInitials(comment.author.name)}
                </div>
                <span style={{ fontSize: designTokens.typography.fontSize.sm, fontWeight: 600 }}>
                  {comment.author.name}
                </span>
                {comment.author.id === currentUserId && (
                  <Badge variant="ai" size="sm">
                    You
                  </Badge>
                )}
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[1] }}>
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  {formatTimestamp(comment.timestamp)}
                  {comment.edited && ' (edited)'}
                </span>
                {comment.author.id === currentUserId && (
                  <>
                    <Tooltip content="Edit">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => startEditing(comment)}
                        aria-label="Edit comment"
                      >
                        <Icon name="Edit" size={12} />
                      </Button>
                    </Tooltip>
                    <Tooltip content="Delete">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onDeleteComment(comment.id)}
                        aria-label="Delete comment"
                      >
                        <Icon name="Minus" size={12} />
                      </Button>
                    </Tooltip>
                  </>
                )}
              </div>
            </div>

            {/* Comment Content */}
            {editingCommentId === comment.id ? (
              <div>
                <textarea
                  value={editContent}
                  onChange={e => setEditContent(e.target.value)}
                  onKeyDown={e => handleKeyDown(e, () => handleEditComment(comment.id))}
                  style={{
                    width: '100%',
                    minHeight: '60px',
                    padding: designTokens.spacing[2],
                    border: `1px solid ${designTokens.colors.border.subtle}`,
                    borderRadius: designTokens.borderRadius.sm,
                    fontFamily: designTokens.typography.fonts.sans.join(', '),
                    fontSize: designTokens.typography.fontSize.sm,
                    resize: 'vertical',
                    backgroundColor: designTokens.colors.surface.tertiary,
                    color: designTokens.colors.text.primary,
                  }}
                  autoFocus
                />
                <div
                  style={{
                    display: 'flex',
                    gap: designTokens.spacing[1],
                    marginTop: designTokens.spacing[1],
                  }}
                >
                  <Button size="sm" onClick={() => handleEditComment(comment.id)}>
                    Save
                  </Button>
                  <Button size="sm" variant="secondary" onClick={cancelEditing}>
                    Cancel
                  </Button>
                </div>
              </div>
            ) : (
              <>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.sm,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[1],
                    whiteSpace: 'pre-wrap',
                  }}
                >
                  {highlightMentions(comment.content)}
                </div>

                {/* Reactions */}
                <div
                  style={{
                    display: 'flex',
                    gap: designTokens.spacing[1],
                    marginTop: designTokens.spacing[1],
                  }}
                >
                  {Object.entries(comment.reactions || {}).map(([emoji, users]) =>
                    users.length > 0 ? (
                      <Tooltip key={emoji} content={users.join(', ')}>
                        <button
                          onClick={() => onReaction(comment.id, emoji)}
                          style={{
                            border: `1px solid ${
                              users.includes(currentUserId)
                                ? designTokens.colors.accent.ai
                                : designTokens.colors.border.subtle
                            }`,
                            borderRadius: designTokens.borderRadius.sm,
                            backgroundColor: users.includes(currentUserId)
                              ? designTokens.colors.accent.ai + '20'
                              : 'transparent',
                            padding: '2px 6px',
                            cursor: 'pointer',
                            fontSize: '12px',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '4px',
                          }}
                        >
                          {emoji} {users.length}
                        </button>
                      </Tooltip>
                    ) : null
                  )}
                  <div style={{ position: 'relative' }}>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        setShowEmojiPicker(showEmojiPicker === comment.id ? null : comment.id)
                      }
                      aria-label="Add reaction"
                    >
                      <Icon name="Heart" size={14} />
                    </Button>
                    {showEmojiPicker === comment.id && (
                      <div
                        style={{
                          position: 'absolute',
                          bottom: '100%',
                          left: 0,
                          backgroundColor: designTokens.colors.background.paper,
                          border: `1px solid ${designTokens.colors.border.subtle}`,
                          borderRadius: designTokens.borderRadius.sm,
                          padding: designTokens.spacing[1],
                          display: 'flex',
                          gap: '4px',
                          boxShadow: designTokens.shadows.md,
                          zIndex: 1000,
                        }}
                      >
                        {emojis.map(emoji => (
                          <button
                            key={emoji}
                            onClick={() => {
                              onReaction(comment.id, emoji)
                              setShowEmojiPicker(null)
                            }}
                            style={{
                              border: 'none',
                              background: 'none',
                              cursor: 'pointer',
                              fontSize: '18px',
                              padding: '4px',
                            }}
                          >
                            {emoji}
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}
          </div>
        ))}
      </div>

      {/* Add New Comment */}
      {thread.status === 'active' && (
        <div>
          <textarea
            value={newCommentContent}
            onChange={e => setNewCommentContent(e.target.value)}
            onKeyDown={e => handleKeyDown(e, handleAddComment)}
            placeholder="Add a comment... (@mention to notify)"
            style={{
              width: '100%',
              minHeight: '60px',
              padding: designTokens.spacing[2],
              border: `1px solid ${designTokens.colors.border.subtle}`,
              borderRadius: designTokens.borderRadius.sm,
              fontFamily: designTokens.typography.fonts.sans.join(', '),
              fontSize: designTokens.typography.fontSize.sm,
              resize: 'vertical',
              backgroundColor: designTokens.colors.surface.tertiary,
              color: designTokens.colors.text.primary,
            }}
          />
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginTop: designTokens.spacing[1],
            }}
          >
            <span
              style={{
                fontSize: designTokens.typography.fontSize.xs,
                color: designTokens.colors.text.secondary,
              }}
            >
              Ctrl+Enter to post
            </span>
            <Button size="sm" onClick={handleAddComment} disabled={!newCommentContent.trim()}>
              Comment
            </Button>
          </div>
        </div>
      )}

      {/* Resolved Thread Actions */}
      {thread.status === 'resolved' && (
        <div style={{ marginTop: designTokens.spacing[2], textAlign: 'center' }}>
          <Button size="sm" variant="secondary" onClick={() => onUnresolveThread(thread.id)}>
            Reopen Thread
          </Button>
        </div>
      )}
    </div>
  )
}

// Helper functions
function getInitials(name: string): string {
  return name
    .split(' ')
    .map(part => part[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

function formatTimestamp(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes}m ago`
  if (hours < 24) return `${hours}h ago`
  if (days < 7) return `${days}d ago`
  return date.toLocaleDateString()
}

function extractMentions(text: string): string[] {
  const mentionRegex = /@(\w+)/g
  const mentions: string[] = []
  let match
  while ((match = mentionRegex.exec(text)) !== null) {
    if (match[1]) {
      mentions.push(match[1])
    }
  }
  return mentions
}

function highlightMentions(text: string): React.ReactNode {
  const parts = text.split(/(@\w+)/g)
  return parts.map((part, index) => {
    if (part.startsWith('@')) {
      return (
        <span
          key={index}
          style={{
            color: designTokens.colors.accent.ai,
            fontWeight: 600,
          }}
        >
          {part}
        </span>
      )
    }
    return part
  })
}
