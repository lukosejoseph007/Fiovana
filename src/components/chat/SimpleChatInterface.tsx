import React, { useState } from 'react'
import { Send, Bot, User } from 'lucide-react'
import { useChatContext } from '../../hooks/useChatContext'
import type { ChatMessage } from '../../context/chatTypes'

interface SimpleChatInterfaceProps {
  className?: string
}

const SimpleChatInterface: React.FC<SimpleChatInterfaceProps> = ({ className = '' }) => {
  const { addMessage, getActiveSession } = useChatContext()
  const [input, setInput] = useState('')

  const activeSession = getActiveSession()
  const messages = activeSession?.messages || []

  console.log('SimpleChatInterface render - activeSession:', activeSession)
  console.log('Messages:', messages)

  const sendMessage = () => {
    if (!input.trim() || !activeSession) return

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      type: 'user',
      content: input.trim(),
      timestamp: new Date(),
    }

    console.log('Sending message:', userMessage)
    addMessage(activeSession.id, userMessage)
    setInput('')

    // Simple AI response for testing
    setTimeout(() => {
      const aiMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: `You said: "${userMessage.content}". This is a test response.`,
        timestamp: new Date(),
      }
      addMessage(activeSession.id, aiMessage)
    }, 1000)
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }

  if (!activeSession) {
    return (
      <div
        className={`flex flex-col h-full bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 ${className}`}
      >
        <div className="flex-1 flex items-center justify-center text-gray-500 dark:text-gray-400">
          <div className="text-center">
            <Bot className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg font-medium mb-2">No Active Chat Session</p>
            <p className="text-sm">
              Select a chat from the sidebar or create a new one to start chatting.
            </p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div
      className={`flex flex-col h-full bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 ${className}`}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <Bot className="h-5 w-5 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <h3 className="font-semibold text-gray-900 dark:text-white">{activeSession.title}</h3>
            <span className="text-sm text-gray-500 dark:text-gray-400">Simple Chat Test</span>
          </div>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="text-center text-gray-500 dark:text-gray-400 mt-8">
            <Bot className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg font-medium mb-2">Start a conversation</p>
            <p className="text-sm">Type a message below to get started.</p>
          </div>
        ) : (
          messages.map(message => (
            <div
              key={message.id}
              className={`flex items-start space-x-3 ${
                message.type === 'user' ? 'flex-row-reverse space-x-reverse' : ''
              }`}
            >
              <div
                className={`p-2 rounded-lg ${
                  message.type === 'user'
                    ? 'bg-blue-50 dark:bg-blue-900/20'
                    : 'bg-gray-50 dark:bg-gray-700'
                }`}
              >
                {message.type === 'user' ? (
                  <User className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                ) : (
                  <Bot className="h-4 w-4 text-gray-600 dark:text-gray-400" />
                )}
              </div>
              <div className={`flex-1 ${message.type === 'user' ? 'text-right' : ''}`}>
                <div
                  className={`inline-block max-w-3xl p-3 rounded-lg ${
                    message.type === 'user'
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white'
                  }`}
                >
                  <p className="text-sm whitespace-pre-wrap">{message.content}</p>
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Input */}
      <div className="border-t border-gray-200 dark:border-gray-700 p-4">
        <div className="flex items-center space-x-3">
          <input
            type="text"
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Type a message..."
            className="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                     bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                     placeholder-gray-500 dark:placeholder-gray-400
                     focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <button
            onClick={sendMessage}
            disabled={!input.trim()}
            className="p-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700
                     disabled:opacity-50 disabled:cursor-not-allowed
                     transition-colors duration-200"
          >
            <Send className="h-4 w-4" />
          </button>
        </div>
      </div>
    </div>
  )
}

export default SimpleChatInterface
