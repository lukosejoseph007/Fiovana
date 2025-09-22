import React, { useState, useRef, useEffect } from 'react'
import { Send, Bot, User, Loader2, AlertCircle, CheckCircle, XCircle } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

interface Message {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: Date
  intent?: string
  confidence?: number
  error?: string
}

interface ChatInterfaceProps {
  className?: string
}

const ChatInterface: React.FC<ChatInterfaceProps> = ({ className = '' }) => {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [aiStatus, setAiStatus] = useState<'unknown' | 'available' | 'unavailable'>('unknown')
  const [currentProvider, setCurrentProvider] = useState<string>('local')
  const [currentModel, setCurrentModel] = useState<string>('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  useEffect(() => {
    // Load AI settings and check status on component mount
    loadAISettings()
    checkAIStatus()
    // Initialize AI system
    initializeAI()

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
  }, [])

  const loadAISettings = async () => {
    try {
      // Try to use Tauri API first
      const settings = await invoke('get_ai_settings')
      console.log('Loaded AI settings from backend:', settings)
      setCurrentProvider(settings.provider || 'local')
      setCurrentModel(settings.selectedModel || '')
      return
    } catch (error) {
      console.error('Failed to load AI settings from backend:', error)
      // Fallback to localStorage
      try {
        const stored = localStorage.getItem('ai_settings')
        if (stored) {
          const settings = JSON.parse(stored)
          console.log('Loaded AI settings from localStorage:', settings)
          setCurrentProvider(settings.provider || 'local')
          setCurrentModel(settings.selectedModel || '')
        } else {
          console.log('No AI settings found in localStorage')
        }
      } catch (localError) {
        console.error('Failed to load AI settings from localStorage:', localError)
      }
    }
  }

  const checkAIStatus = async () => {
    try {
      const status = await invoke('get_ai_status')
      console.log('AI Status check result:', status)
      setAiStatus(status.available ? 'available' : 'unavailable')
    } catch (error) {
      console.error('Failed to check AI status:', error)
      setAiStatus('unavailable')
    }
  }

  const initializeAI = async () => {
    try {
      console.log('Initializing AI system...')
      // Initialize AI system with current settings
      const initialized = await invoke('init_ai_system')
      console.log('AI initialization result:', initialized)

      if (initialized) {
        // Double-check status after initialization
        const status = await invoke('get_ai_status')
        console.log('AI status after initialization:', status)
        setAiStatus(status.available ? 'available' : 'unavailable')
      } else {
        console.log('AI initialization failed')
        setAiStatus('unavailable')
      }
    } catch (error) {
      console.error('Failed to initialize AI:', error)
      setAiStatus('unavailable')
    }
  }

  const sendMessage = async () => {
    if (!input.trim() || isLoading) return

    const userMessage: Message = {
      id: Date.now().toString(),
      type: 'user',
      content: input.trim(),
      timestamp: new Date(),
    }

    setMessages(prev => [...prev, userMessage])
    setInput('')
    setIsLoading(true)

    try {
      const response = await invoke('chat_with_ai', {
        request: {
          message: userMessage.content,
          context: null,
        },
      })

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: response.success
          ? response.response.content
          : response.error || 'Sorry, I encountered an error processing your request.',
        timestamp: new Date(),
        intent: response.response?.intent,
        confidence: response.response?.confidence,
        error: response.success ? undefined : response.error,
      }

      setMessages(prev => [...prev, assistantMessage])
    } catch (error) {
      console.error('Chat error:', error)
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: 'Sorry, I encountered a technical error. Please try again.',
        timestamp: new Date(),
        error: String(error),
      }
      setMessages(prev => [...prev, errorMessage])
    } finally {
      setIsLoading(false)
      inputRef.current?.focus()
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
            <h3 className="font-semibold text-gray-900 dark:text-white">AI Assistant</h3>
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
            <p className="text-lg font-medium mb-2">Welcome to Proxemic AI Assistant</p>
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
                      : message.error
                        ? 'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800'
                        : 'bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white'
                  }`}
                >
                  <p className="text-sm whitespace-pre-wrap">{message.content}</p>
                  {message.error && (
                    <div className="mt-2 flex items-center space-x-1 text-xs text-red-600 dark:text-red-400">
                      <AlertCircle className="h-3 w-3" />
                      <span>Error occurred</span>
                    </div>
                  )}
                  {message.intent && (
                    <div className="mt-2 text-xs opacity-70">
                      Intent: {message.intent}
                      {message.confidence && ` (${Math.round(message.confidence * 100)}%)`}
                    </div>
                  )}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  {formatTime(message.timestamp)}
                </div>
              </div>
            </div>
          ))
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
            onKeyPress={handleKeyPress}
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
            onClick={sendMessage}
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

export default ChatInterface
