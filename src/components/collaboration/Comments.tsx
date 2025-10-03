import React, { useState, useCallback, useEffect, useRef } from 'react'
import { Button, Icon } from '../ui'
import { designTokens } from '../../styles/tokens'
import { CommentThread, type CommentThreadData, type Comment } from './CommentThread'
import { CommentSidebar } from './CommentSidebar'
import { useCollaboration } from '../../context/useCollaboration'

interface CommentsProps {
  documentId: string
  contentRef?: React.RefObject<HTMLElement | null>
  isEditMode?: boolean
}

export const Comments: React.FC<CommentsProps> = ({
  documentId,
  contentRef: _contentRef,
  isEditMode = false,
}) => {
  const collaboration = useCollaboration()
  const [threads, setThreads] = useState<CommentThreadData[]>([])
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null)
  const [showSidebar, setShowSidebar] = useState(false)
  const [selection, setSelection] = useState<{
    text: string
    start: number
    end: number
  } | null>(null)
  const [floatingPosition, setFloatingPosition] = useState<{ x: number; y: number } | null>(null)
  const [isHoveringButton, setIsHoveringButton] = useState(false)

  const selectionTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  // Listen for toggle sidebar event from header button
  useEffect(() => {
    const handleToggle = () => {
      setShowSidebar(prev => !prev)
    }

    window.addEventListener('toggleCommentsSidebar', handleToggle)
    return () => window.removeEventListener('toggleCommentsSidebar', handleToggle)
  }, [])

  // Load comments from localStorage on mount
  useEffect(() => {
    const storageKey = `comments-${documentId}`
    const savedComments = localStorage.getItem(storageKey)

    if (savedComments) {
      try {
        const parsed = JSON.parse(savedComments)
        // Convert date strings back to Date objects
        const restored = parsed.map((thread: CommentThreadData) => ({
          ...thread,
          createdAt: new Date(thread.createdAt),
          resolvedAt: thread.resolvedAt ? new Date(thread.resolvedAt) : undefined,
          comments: thread.comments.map((comment: Comment) => ({
            ...comment,
            timestamp: new Date(comment.timestamp),
            editedAt: comment.editedAt ? new Date(comment.editedAt) : undefined,
          })),
        }))
        setThreads(restored)
      } catch (error) {
        console.error('Failed to restore comments from localStorage:', error)
      }
    }
  }, [documentId])

  // Save comments to localStorage whenever they change
  useEffect(() => {
    const storageKey = `comments-${documentId}`
    try {
      if (threads.length > 0) {
        localStorage.setItem(storageKey, JSON.stringify(threads))
      } else {
        // Remove from localStorage if no threads exist
        localStorage.removeItem(storageKey)
      }
    } catch (error) {
      console.error('Failed to save comments to localStorage:', error)
    }
  }, [threads, documentId])

  // Handle text selection
  useEffect(() => {
    const handleSelectionChange = () => {
      if (!isEditMode) return

      const sel = window.getSelection()
      if (!sel || sel.isCollapsed || !sel.rangeCount) {
        // Clear selection after a delay to allow clicking the comment button
        // Don't clear if user is hovering over the button
        if (selectionTimeoutRef.current) {
          clearTimeout(selectionTimeoutRef.current)
        }

        selectionTimeoutRef.current = setTimeout(() => {
          if (!isHoveringButton) {
            setSelection(null)
            setFloatingPosition(null)
          }
        }, 1500) // Increased timeout to 1.5 seconds to give users time to move mouse
        return
      }

      // Check if selection is within the document editor/viewer area
      const range = sel.getRangeAt(0)
      const container = range.commonAncestorContainer
      const element =
        container.nodeType === Node.TEXT_NODE ? container.parentElement : (container as Element)

      // Only allow selections within the document editor or renderer
      // Exclude selections from sidebar, header, and other UI elements
      const isInEditor = element?.closest(
        '.cm-editor, .cm-content, [data-document-editor], [data-document-renderer]'
      )
      const isInSidebar = element?.closest('[style*="position: fixed"]')
      const isInCommentSidebar = element?.closest('[data-comment-sidebar]')

      if (!isInEditor || isInSidebar || isInCommentSidebar) {
        setSelection(null)
        setFloatingPosition(null)
        return
      }

      const selectedText = sel.toString().trim()
      if (selectedText.length < 3) {
        setSelection(null)
        setFloatingPosition(null)
        return
      }

      // Get selection position
      const rect = range.getBoundingClientRect()

      // Calculate position for floating button
      const x = rect.left + rect.width / 2
      const y = rect.top - 40 // Position above selection

      setSelection({
        text: selectedText,
        start: range.startOffset,
        end: range.endOffset,
      })
      setFloatingPosition({ x, y })
    }

    document.addEventListener('selectionchange', handleSelectionChange)
    return () => {
      document.removeEventListener('selectionchange', handleSelectionChange)
      if (selectionTimeoutRef.current) {
        clearTimeout(selectionTimeoutRef.current)
      }
    }
  }, [isEditMode, isHoveringButton])

  // Create new comment thread
  const handleCreateThread = useCallback(() => {
    if (!selection) return

    const newThread: CommentThreadData = {
      id: generateId(),
      documentId,
      selection: {
        start: selection.start,
        end: selection.end,
        text: selection.text,
      },
      comments: [],
      createdAt: new Date(),
      status: 'active',
      position: floatingPosition || undefined,
    }

    setThreads(prev => [newThread, ...prev])
    setActiveThreadId(newThread.id)
    setSelection(null)
    setFloatingPosition(null)
    setIsHoveringButton(false)
    setShowSidebar(true)

    // Clear selection
    window.getSelection()?.removeAllRanges()
  }, [selection, documentId, floatingPosition])

  // Add comment to thread
  const handleAddComment = useCallback(
    (threadId: string, content: string, mentions: string[]) => {
      const currentUser = {
        id: collaboration.settings.username || 'Anonymous',
        name: collaboration.settings.username || 'Anonymous',
        color: collaboration.settings.userColor || '#3B82F6',
      }

      const newComment: Comment = {
        id: generateId(),
        threadId,
        author: currentUser,
        content,
        timestamp: new Date(),
        mentions,
        reactions: {},
      }

      setThreads(prev =>
        prev.map(thread => {
          if (thread.id === threadId) {
            return {
              ...thread,
              comments: [...thread.comments, newComment],
            }
          }
          return thread
        })
      )

      // TODO: Emit notification event for mentioned users
      if (mentions.length > 0) {
        console.log('Mentioned users:', mentions)
      }
    },
    [collaboration.settings]
  )

  // Edit comment
  const handleEditComment = useCallback((commentId: string, content: string) => {
    setThreads(prev =>
      prev.map(thread => ({
        ...thread,
        comments: thread.comments.map(comment =>
          comment.id === commentId
            ? {
                ...comment,
                content,
                edited: true,
                editedAt: new Date(),
              }
            : comment
        ),
      }))
    )
  }, [])

  // Delete comment
  const handleDeleteComment = useCallback((commentId: string) => {
    setThreads(prev =>
      prev.map(thread => ({
        ...thread,
        comments: thread.comments.filter(comment => comment.id !== commentId),
      }))
    )
  }, [])

  // Resolve thread
  const handleResolveThread = useCallback(
    (threadId: string) => {
      setThreads(prev =>
        prev.map(thread =>
          thread.id === threadId
            ? {
                ...thread,
                status: 'resolved' as const,
                resolvedAt: new Date(),
                resolvedBy: collaboration.settings.username || 'Anonymous',
              }
            : thread
        )
      )
    },
    [collaboration.settings.username]
  )

  // Unresolve thread
  const handleUnresolveThread = useCallback((threadId: string) => {
    setThreads(prev =>
      prev.map(thread =>
        thread.id === threadId
          ? {
              ...thread,
              status: 'active' as const,
              resolvedAt: undefined,
              resolvedBy: undefined,
            }
          : thread
      )
    )
  }, [])

  // Add reaction to comment
  const handleReaction = useCallback(
    (commentId: string, emoji: string) => {
      const currentUserId = collaboration.settings.username || 'Anonymous'

      setThreads(prev =>
        prev.map(thread => ({
          ...thread,
          comments: thread.comments.map(comment => {
            if (comment.id === commentId) {
              const reactions = { ...comment.reactions }
              const users = reactions[emoji] || []

              if (users.includes(currentUserId)) {
                // Remove reaction
                reactions[emoji] = users.filter(id => id !== currentUserId)
                if (reactions[emoji].length === 0) {
                  delete reactions[emoji]
                }
              } else {
                // Add reaction
                reactions[emoji] = [...users, currentUserId]
              }

              return {
                ...comment,
                reactions,
              }
            }
            return comment
          }),
        }))
      )
    },
    [collaboration.settings.username]
  )

  // Delete thread
  const handleDeleteThread = useCallback(
    (threadId: string) => {
      setThreads(prev => prev.filter(thread => thread.id !== threadId))
      if (activeThreadId === threadId) {
        setActiveThreadId(null)
      }
    },
    [activeThreadId]
  )

  // Handle thread click (scroll to selection in document)
  const handleThreadClick = useCallback((threadId: string) => {
    // TODO: Implement scroll to selection in document
    console.log('Thread clicked:', threadId)
  }, [])

  const activeThread = threads.find(t => t.id === activeThreadId)

  return (
    <>
      {/* Floating Comment Button (appears on text selection) */}
      {selection && floatingPosition && (
        <div
          style={{
            position: 'fixed',
            left: floatingPosition.x,
            top: floatingPosition.y,
            transform: 'translateX(-50%)',
            zIndex: 10000,
          }}
          onMouseEnter={() => setIsHoveringButton(true)}
          onMouseLeave={() => setIsHoveringButton(false)}
        >
          <Button variant="primary" size="sm" onClick={handleCreateThread} aria-label="Add comment">
            <Icon name="MessageCircle" size={16} style={{ marginRight: '4px' }} />
            Comment
          </Button>
        </div>
      )}

      {/* Floating Thread View */}
      {activeThread && !showSidebar && (
        <div
          style={{
            position: 'fixed',
            right: designTokens.spacing[4],
            top: '100px',
            zIndex: 999,
          }}
        >
          <CommentThread
            thread={activeThread}
            onAddComment={handleAddComment}
            onEditComment={handleEditComment}
            onDeleteComment={handleDeleteComment}
            onDeleteThread={handleDeleteThread}
            onResolveThread={handleResolveThread}
            onUnresolveThread={handleUnresolveThread}
            onReaction={handleReaction}
            onClose={() => setActiveThreadId(null)}
            compact={false}
          />
        </div>
      )}

      {/* Comment Sidebar */}
      <CommentSidebar
        threads={threads}
        onAddComment={handleAddComment}
        onEditComment={handleEditComment}
        onDeleteComment={handleDeleteComment}
        onDeleteThread={handleDeleteThread}
        onResolveThread={handleResolveThread}
        onUnresolveThread={handleUnresolveThread}
        onReaction={handleReaction}
        onThreadClick={handleThreadClick}
        onClose={() => setShowSidebar(false)}
        isOpen={showSidebar}
      />
    </>
  )
}

// Helper function to generate unique IDs
function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}
