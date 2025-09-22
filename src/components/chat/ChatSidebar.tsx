import React, { useState } from 'react'
import { PlusIcon, ChatBubbleLeftIcon, TrashIcon, PencilIcon } from '@heroicons/react/24/outline'
import { useChatContext } from '../../hooks/useChatContext'

interface ChatSidebarProps {
  className?: string
}

export const ChatSidebar: React.FC<ChatSidebarProps> = ({ className = '' }) => {
  const { state, createNewChat, switchToChat, deleteChat, updateChatTitle } = useChatContext()
  const [editingSessionId, setEditingSessionId] = useState<string | null>(null)
  const [editingTitle, setEditingTitle] = useState('')
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null)

  const { sessions, activeChatId, sidebarCollapsed } = state

  const startEditing = (sessionId: string, currentTitle: string) => {
    setEditingSessionId(sessionId)
    setEditingTitle(currentTitle)
  }

  const saveEdit = () => {
    if (editingSessionId && editingTitle.trim()) {
      updateChatTitle(editingSessionId, editingTitle.trim())
    }
    setEditingSessionId(null)
    setEditingTitle('')
  }

  const cancelEdit = () => {
    setEditingSessionId(null)
    setEditingTitle('')
  }

  const handleDeleteClick = (sessionId: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setShowDeleteConfirm(sessionId)
  }

  const confirmDelete = (sessionId: string) => {
    deleteChat(sessionId)
    setShowDeleteConfirm(null)
  }

  const formatDate = (date: Date): string => {
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))

    if (days === 0) return 'Today'
    if (days === 1) return 'Yesterday'
    if (days < 7) return `${days} days ago`
    if (days < 30) return `${Math.floor(days / 7)} weeks ago`
    return date.toLocaleDateString()
  }

  // Group sessions by date
  const groupedSessions = sessions.reduce((groups: Record<string, typeof sessions>, session) => {
    const dateKey = formatDate(session.updatedAt)
    if (!groups[dateKey]) groups[dateKey] = []
    groups[dateKey].push(session)
    return groups
  }, {})

  if (sidebarCollapsed) {
    return (
      <div
        className={`flex flex-col bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 ${className}`}
      >
        <div className="flex items-center justify-center p-4">
          <button
            onClick={() => createNewChat()}
            className="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
            title="New Chat"
          >
            <PlusIcon className="h-5 w-5" />
          </button>
        </div>
      </div>
    )
  }

  return (
    <div
      className={`flex flex-col bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 ${className}`}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Chats</h2>
        <button
          onClick={() => createNewChat()}
          className="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
          title="New Chat"
        >
          <PlusIcon className="h-5 w-5" />
        </button>
      </div>

      {/* Chat List */}
      <div className="flex-1 overflow-y-auto">
        {Object.keys(groupedSessions).length === 0 ? (
          <div className="p-4 text-center text-gray-500 dark:text-gray-400">
            <ChatBubbleLeftIcon className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p>No chat sessions yet</p>
            <button
              onClick={() => createNewChat()}
              className="mt-2 text-blue-600 dark:text-blue-400 hover:underline"
            >
              Start your first chat
            </button>
          </div>
        ) : (
          <div className="p-2">
            {Object.entries(groupedSessions).map(([dateGroup, groupSessions]) => (
              <div key={dateGroup} className="mb-4">
                <h3 className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider px-2 mb-2">
                  {dateGroup}
                </h3>
                <div className="space-y-1">
                  {groupSessions.map(session => (
                    <div
                      key={session.id}
                      className={`group relative flex items-center p-2 rounded-lg cursor-pointer transition-colors ${
                        session.id === activeChatId
                          ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-900 dark:text-blue-100'
                          : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                      }`}
                      onClick={() => switchToChat(session.id)}
                    >
                      <ChatBubbleLeftIcon className="h-4 w-4 mr-2 flex-shrink-0" />

                      {editingSessionId === session.id ? (
                        <input
                          type="text"
                          value={editingTitle}
                          onChange={e => setEditingTitle(e.target.value)}
                          onBlur={saveEdit}
                          onKeyDown={e => {
                            if (e.key === 'Enter') saveEdit()
                            if (e.key === 'Escape') cancelEdit()
                          }}
                          className="flex-1 bg-transparent border-none outline-none text-sm"
                          autoFocus
                          onClick={e => e.stopPropagation()}
                        />
                      ) : (
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium truncate">{session.title}</p>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            {session.messageCount} message{session.messageCount !== 1 ? 's' : ''}
                          </p>
                        </div>
                      )}

                      {/* Action buttons */}
                      <div className="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button
                          onClick={e => {
                            e.stopPropagation()
                            startEditing(session.id, session.title)
                          }}
                          className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded"
                          title="Edit title"
                        >
                          <PencilIcon className="h-3 w-3" />
                        </button>

                        <button
                          onClick={e => handleDeleteClick(session.id, e)}
                          className="p-1 text-gray-400 hover:text-red-600 dark:hover:text-red-400 rounded"
                          title="Delete chat"
                        >
                          <TrashIcon className="h-3 w-3" />
                        </button>
                      </div>

                      {/* Delete confirmation */}
                      {showDeleteConfirm === session.id && (
                        <div className="absolute inset-0 bg-white dark:bg-gray-800 rounded-lg border border-red-200 dark:border-red-800 flex items-center justify-center z-10">
                          <div className="flex items-center space-x-2">
                            <span className="text-xs text-red-600 dark:text-red-400">Delete?</span>
                            <button
                              onClick={e => {
                                e.stopPropagation()
                                confirmDelete(session.id)
                              }}
                              className="px-2 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700"
                            >
                              Yes
                            </button>
                            <button
                              onClick={e => {
                                e.stopPropagation()
                                setShowDeleteConfirm(null)
                              }}
                              className="px-2 py-1 text-xs bg-gray-300 dark:bg-gray-600 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-400 dark:hover:bg-gray-500"
                            >
                              No
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-gray-200 dark:border-gray-700">
        <div className="text-xs text-gray-500 dark:text-gray-400 text-center">
          {sessions.length} chat{sessions.length !== 1 ? 's' : ''} •{' '}
          {state.aiStatus === 'available' ? 'AI Online' : 'AI Offline'}
        </div>
      </div>
    </div>
  )
}
