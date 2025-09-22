import React from 'react'
import { ChatProvider } from '../context/ChatContext'
import { ChatSidebar } from '../components/chat/ChatSidebar'
import SessionChatInterface from '../components/chat/SessionChatInterface'
import { useChatContext } from '../hooks/useChatContext'
import { ChevronLeftIcon, ChevronRightIcon } from '@heroicons/react/24/outline'
import ErrorBoundary from '../components/ErrorBoundary'

const ChatLayout: React.FC = () => {
  const { state, dispatch } = useChatContext()
  const { sidebarCollapsed } = state

  const toggleSidebar = () => {
    dispatch({ type: 'CHAT_TOGGLE_SIDEBAR' })
  }

  return (
    <div className="h-[calc(100vh-12rem)] flex bg-gray-50 dark:bg-gray-900 rounded-lg overflow-hidden">
      {/* Sidebar */}
      <div
        className={`${sidebarCollapsed ? 'w-16' : 'w-80'} flex-shrink-0 transition-all duration-300`}
      >
        <ChatSidebar className="h-full" />
      </div>

      {/* Toggle Button */}
      <div className="flex-shrink-0">
        <button
          onClick={toggleSidebar}
          className="h-full w-6 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors flex items-center justify-center border-l border-r border-gray-300 dark:border-gray-600"
          title={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {sidebarCollapsed ? (
            <ChevronRightIcon className="h-4 w-4 text-gray-600 dark:text-gray-400" />
          ) : (
            <ChevronLeftIcon className="h-4 w-4 text-gray-600 dark:text-gray-400" />
          )}
        </button>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 min-w-0">
        <SessionChatInterface className="h-full" />
      </div>
    </div>
  )
}

const Chat: React.FC = () => {
  return (
    <ErrorBoundary>
      <ChatProvider>
        <ChatLayout />
      </ChatProvider>
    </ErrorBoundary>
  )
}

export default Chat
