import React, { useRef, useEffect, useCallback, useState } from 'react'
import {
  Send,
  Bot,
  User,
  Loader2,
  AlertCircle,
  CheckCircle,
  XCircle,
  RefreshCw,
  Copy,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { AISettings, AIStatus, ChatResponse } from '../../types/ai'
import { useChatContext } from '../../hooks/useChatContext'
import type { ChatMessage } from '../../context/chatTypes'

interface SessionChatInterfaceProps {
  className?: string
}

const SessionChatInterface: React.FC<SessionChatInterfaceProps> = ({ className = '' }) => {
  const { state, addMessage, getActiveSession, dispatch, addResponse, setActiveResponse } =
    useChatContext()
  const { isLoading, aiStatus, currentProvider, currentModel } = state
  const [input, setInput] = React.useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  const activeSession = getActiveSession()
  const messages = React.useMemo(() => activeSession?.messages || [], [activeSession?.messages])
  const [retryingMessageId, setRetryingMessageId] = useState<string | null>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const loadAISettings = useCallback(async () => {
    try {
      // Try to use Tauri API first
      const settings = (await invoke('get_ai_settings')) as AISettings
      console.log('Loaded AI settings from backend:', settings)
      dispatch({ type: 'CHAT_SET_PROVIDER', payload: settings.provider || 'local' })
      dispatch({ type: 'CHAT_SET_MODEL', payload: settings.selectedModel || '' })
      return
    } catch (error) {
      console.error('Failed to load AI settings from backend:', error)
      // Fallback to localStorage
      try {
        const stored = localStorage.getItem('ai_settings')
        if (stored) {
          const settings = JSON.parse(stored)
          console.log('Loaded AI settings from localStorage:', settings)
          dispatch({ type: 'CHAT_SET_PROVIDER', payload: settings.provider || 'local' })
          dispatch({ type: 'CHAT_SET_MODEL', payload: settings.selectedModel || '' })
        } else {
          console.log('No AI settings found in localStorage')
        }
      } catch (localError) {
        console.error('Failed to load AI settings from localStorage:', localError)
      }
    }
  }, [dispatch])

  const checkAIStatus = useCallback(async () => {
    try {
      const status = (await invoke('get_ai_status')) as AIStatus
      console.log('AI Status check result:', status)
      dispatch({
        type: 'CHAT_SET_AI_STATUS',
        payload: status.available ? 'available' : 'unavailable',
      })
    } catch (error) {
      console.error('Failed to check AI status:', error)
      dispatch({ type: 'CHAT_SET_AI_STATUS', payload: 'unavailable' })
    }
  }, [dispatch])

  const initializeAI = useCallback(async () => {
    try {
      console.log('Initializing AI system...')
      // Initialize AI system with current settings
      const initialized = await invoke('init_ai_system')
      console.log('AI initialization result:', initialized)

      if (initialized) {
        // Double-check status after initialization
        const status = (await invoke('get_ai_status')) as AIStatus
        console.log('AI status after initialization:', status)
        dispatch({
          type: 'CHAT_SET_AI_STATUS',
          payload: status.available ? 'available' : 'unavailable',
        })
      } else {
        console.log('AI initialization failed')
        dispatch({ type: 'CHAT_SET_AI_STATUS', payload: 'unavailable' })
      }
    } catch (error) {
      console.error('Failed to initialize AI:', error)
      dispatch({ type: 'CHAT_SET_AI_STATUS', payload: 'unavailable' })
    }
  }, [dispatch])

  useEffect(() => {
    // Load AI settings and check status on component mount
    loadAISettings()
    checkAIStatus()
    // Initialize AI system
    initializeAI()

    // Initialize document indexer for AI to access
    const initDocumentIndexer = async () => {
      try {
        await invoke('init_document_indexer', { indexDir: null })
        console.log('Document indexer initialized for AI chat')
      } catch (error) {
        console.error('Failed to initialize document indexer for AI chat:', error)
      }
    }
    initDocumentIndexer()

    // Set up storage listener for settings changes
    const handleStorageChange = (e: StorageEvent) => {
      if (e.key === 'ai_settings') {
        console.log('AI settings changed, reloading...')
        loadAISettings()
        // Reinitialize AI system with new settings
        setTimeout(initializeAI, 100) // Small delay to ensure settings are loaded
      }
    }

    window.addEventListener('storage', handleStorageChange)

    return () => {
      window.removeEventListener('storage', handleStorageChange)
    }
  }, [loadAISettings, checkAIStatus, initializeAI])

  const sendMessage = async (retryMessageId?: string) => {
    if ((!input.trim() && !retryMessageId) || (isLoading && !retryMessageId)) return

    if (!activeSession) {
      console.error('No active session to add message to')
      return
    }

    let userMessage: ChatMessage
    let isRetry = false

    if (retryMessageId) {
      // Find the user message for retry
      const messageIndex = messages.findIndex(m => m.id === retryMessageId)
      if (messageIndex === -1) return

      const userMsgIndex = messageIndex - 1
      if (userMsgIndex < 0 || messages[userMsgIndex]?.type !== 'user') return

      const foundUserMessage = messages[userMsgIndex]
      if (!foundUserMessage) return

      userMessage = foundUserMessage
      isRetry = true
      setRetryingMessageId(retryMessageId)
    } else {
      userMessage = {
        id: Date.now().toString(),
        type: 'user',
        content: input.trim(),
        timestamp: new Date(),
      }
      addMessage(activeSession.id, userMessage)
      setInput('')
    }

    dispatch({ type: 'CHAT_SET_LOADING', payload: true })

    try {
      const response = (await invoke('chat_with_ai', {
        request: {
          message: userMessage.content,
          context: null,
        },
      })) as ChatResponse

      const assistantMessage: ChatMessage = {
        id: Date.now().toString(),
        type: 'assistant',
        content: response.success
          ? response.response?.content || 'No response content'
          : response.error || 'Sorry, I encountered an error processing your request.',
        timestamp: new Date(),
        intent: response.response?.intent,
        confidence: response.response?.confidence,
        error: response.success ? undefined : response.error,
        parentMessageId: isRetry ? retryMessageId : undefined,
      }

      if (isRetry && retryMessageId) {
        addResponse(activeSession.id, retryMessageId, assistantMessage)
      } else {
        addMessage(activeSession.id, assistantMessage)
      }
    } catch (error) {
      console.error('Chat error:', error)
      const errorMessage: ChatMessage = {
        id: Date.now().toString(),
        type: 'assistant',
        content: 'Sorry, I encountered a technical error. Please try again.',
        timestamp: new Date(),
        error: String(error),
        parentMessageId: isRetry ? retryMessageId : undefined,
      }

      if (isRetry && retryMessageId) {
        addResponse(activeSession.id, retryMessageId, errorMessage)
      } else {
        addMessage(activeSession.id, errorMessage)
      }
    } finally {
      dispatch({ type: 'CHAT_SET_LOADING', payload: false })
      setRetryingMessageId(null)
      if (!isRetry) {
        inputRef.current?.focus()
      }
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }

  const formatTime = (timestamp: Date) => {
    return timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
    } catch (error) {
      console.error('Failed to copy to clipboard:', error)
      // Fallback for older browsers
      const textArea = document.createElement('textarea')
      textArea.value = text
      document.body.appendChild(textArea)
      textArea.focus()
      textArea.select()
      try {
        document.execCommand('copy')
      } catch (fallbackError) {
        console.error('Fallback copy failed:', fallbackError)
      }
      document.body.removeChild(textArea)
    }
  }

  const getCurrentResponse = (message: ChatMessage) => {
    if (!message.responses || message.responses.length === 0) {
      return message
    }
    const activeIndex = message.activeResponseIndex ?? 0
    return message.responses[activeIndex] || message
  }

  const handleRetry = (messageId: string) => {
    sendMessage(messageId)
  }

  const handleSwitchResponse = (messageId: string, direction: 'prev' | 'next') => {
    if (!activeSession) return

    const message = messages.find(m => m.id === messageId)
    if (!message || !message.responses || message.responses.length <= 1) return

    const currentIndex = message.activeResponseIndex ?? 0
    let newIndex: number

    if (direction === 'prev') {
      newIndex = currentIndex > 0 ? currentIndex - 1 : message.responses.length - 1
    } else {
      newIndex = currentIndex < message.responses.length - 1 ? currentIndex + 1 : 0
    }

    setActiveResponse(activeSession.id, messageId, newIndex)
  }

  const getStatusIcon = () => {
    switch (aiStatus) {
      case 'available':
        return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'unavailable':
        return <XCircle className="h-4 w-4 text-red-500" />
      default:
        return <Loader2 className="h-4 w-4 text-yellow-500 animate-spin" />
    }
  }

  const getStatusText = () => {
    switch (aiStatus) {
      case 'available':
        if (currentModel) {
          const providerDisplay =
            currentProvider === 'openrouter'
              ? 'OpenRouter'
              : currentProvider === 'anthropic'
                ? 'Anthropic'
                : 'Local'
          return `${providerDisplay}: ${currentModel}`
        }
        return 'AI Assistant Online'
      case 'unavailable':
        return currentProvider === 'local'
          ? 'Ollama not connected'
          : `${currentProvider} not configured`
      default:
        return 'Checking AI Status...'
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
            <div className="flex items-center space-x-2">
              {getStatusIcon()}
              <span className="text-sm text-gray-500 dark:text-gray-400">{getStatusText()}</span>
            </div>
          </div>
        </div>
        {aiStatus === 'unavailable' && (
          <button
            onClick={initializeAI}
            className="px-3 py-1 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Retry
          </button>
        )}
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="text-center text-gray-500 dark:text-gray-400 mt-8">
            <Bot className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg font-medium mb-2">Start a conversation</p>
            <p className="text-sm">
              I can help you process documents, compare content, and answer questions about your
              files.
            </p>
            <div className="mt-4 text-xs space-y-1">
              <p>Try asking:</p>
              <p>"Compare document A with document B"</p>
              <p>"Find all sections about [topic]"</p>
              <p>"Update the content based on these changes"</p>
            </div>
          </div>
        ) : (
          messages.map(message => {
            const currentResponse = getCurrentResponse(message)
            const hasMultipleResponses = message.responses && message.responses.length > 1
            const isRetrying = retryingMessageId === message.id

            return (
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
                    className={`inline-block max-w-3xl p-3 rounded-lg relative group ${
                      message.type === 'user'
                        ? 'bg-blue-600 text-white'
                        : currentResponse.error
                          ? 'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800'
                          : 'bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white'
                    }`}
                  >
                    {/* Copy button */}
                    <button
                      onClick={() => copyToClipboard(currentResponse.content)}
                      className={`absolute top-2 ${
                        message.type === 'user' ? 'left-2' : 'right-2'
                      } opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-black/10 dark:hover:bg-white/10`}
                      title="Copy message"
                    >
                      <Copy className="h-3 w-3" />
                    </button>

                    {/* Message content with markdown support for assistant messages */}
                    {message.type === 'assistant' ? (
                      <div className="text-sm prose prose-sm max-w-none dark:prose-invert">
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>
                          {currentResponse.content}
                        </ReactMarkdown>
                      </div>
                    ) : (
                      <p className="text-sm whitespace-pre-wrap">{currentResponse.content}</p>
                    )}

                    {currentResponse.error && (
                      <div className="mt-2 flex items-center space-x-1 text-xs text-red-600 dark:text-red-400">
                        <AlertCircle className="h-3 w-3" />
                        <span>Error occurred</span>
                      </div>
                    )}
                    {currentResponse.intent && (
                      <div className="mt-2 text-xs opacity-70">
                        Intent: {currentResponse.intent}
                        {currentResponse.confidence &&
                          ` (${Math.round(currentResponse.confidence * 100)}%)`}
                      </div>
                    )}
                  </div>

                  {/* Response controls for assistant messages */}
                  {message.type === 'assistant' && (
                    <div className="flex items-center space-x-2 mt-2">
                      {/* Retry button */}
                      <button
                        onClick={() => handleRetry(message.id)}
                        disabled={isLoading || isRetrying}
                        className="flex items-center space-x-1 text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 disabled:opacity-50 disabled:cursor-not-allowed"
                        title="Retry this response"
                      >
                        <RefreshCw className={`h-3 w-3 ${isRetrying ? 'animate-spin' : ''}`} />
                        <span>{isRetrying ? 'Retrying...' : 'Retry'}</span>
                      </button>

                      {/* Response navigation for multiple responses */}
                      {hasMultipleResponses && (
                        <div className="flex items-center space-x-1">
                          <button
                            onClick={() => handleSwitchResponse(message.id, 'prev')}
                            className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-600"
                            title="Previous response"
                          >
                            <ChevronLeft className="h-3 w-3" />
                          </button>
                          <span className="text-xs text-gray-500 dark:text-gray-400">
                            {(message.activeResponseIndex ?? 0) + 1} of {message.responses!.length}
                          </span>
                          <button
                            onClick={() => handleSwitchResponse(message.id, 'next')}
                            className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-600"
                            title="Next response"
                          >
                            <ChevronRight className="h-3 w-3" />
                          </button>
                        </div>
                      )}
                    </div>
                  )}

                  {/* User message copy button in controls area */}
                  {message.type === 'user' && (
                    <div className="flex justify-end mt-2">
                      <button
                        onClick={() => copyToClipboard(message.content)}
                        className="flex items-center space-x-1 text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                        title="Copy message"
                      >
                        <Copy className="h-3 w-3" />
                        <span>Copy</span>
                      </button>
                    </div>
                  )}

                  <div
                    className={`text-xs text-gray-500 dark:text-gray-400 mt-1 ${
                      message.type === 'user' ? 'text-right' : ''
                    }`}
                  >
                    {formatTime(currentResponse.timestamp)}
                  </div>
                </div>
              </div>
            )
          })
        )}
        {isLoading && (
          <div className="flex items-start space-x-3">
            <div className="p-2 rounded-lg bg-gray-50 dark:bg-gray-700">
              <Bot className="h-4 w-4 text-gray-600 dark:text-gray-400" />
            </div>
            <div className="flex-1">
              <div className="inline-block p-3 rounded-lg bg-gray-100 dark:bg-gray-700">
                <div className="flex items-center space-x-2">
                  <Loader2 className="h-4 w-4 animate-spin text-gray-600 dark:text-gray-400" />
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    AI is thinking...
                  </span>
                </div>
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="border-t border-gray-200 dark:border-gray-700 p-4">
        <div className="flex items-center space-x-3">
          <input
            ref={inputRef}
            type="text"
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyPress}
            placeholder={
              aiStatus === 'available'
                ? 'Ask me about your documents...'
                : 'AI assistant is not available'
            }
            disabled={aiStatus !== 'available' || isLoading}
            className="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                     bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                     placeholder-gray-500 dark:placeholder-gray-400
                     focus:ring-2 focus:ring-blue-500 focus:border-transparent
                     disabled:opacity-50 disabled:cursor-not-allowed"
          />
          <button
            onClick={() => sendMessage()}
            disabled={!input.trim() || aiStatus !== 'available' || isLoading}
            className="p-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700
                     disabled:opacity-50 disabled:cursor-not-allowed
                     transition-colors duration-200"
          >
            <Send className="h-4 w-4" />
          </button>
        </div>
        {aiStatus === 'unavailable' && (
          <div className="mt-2 text-sm text-red-600 dark:text-red-400 flex items-center space-x-1">
            <AlertCircle className="h-4 w-4" />
            <span>
              {currentProvider === 'local'
                ? 'AI assistant is not available. Make sure Ollama is running and try again.'
                : `AI assistant is not available. Please check your ${currentProvider} configuration in Settings.`}
            </span>
          </div>
        )}
      </div>
    </div>
  )
}

export default SessionChatInterface
