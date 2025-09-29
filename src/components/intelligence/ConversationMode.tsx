import React, { useState, useCallback, useEffect, useMemo, useRef } from 'react'
import { designTokens } from '../../styles/tokens'
import Button from '../ui/Button'
import Input from '../ui/Input'
import Icon from '../ui/Icon'
import Badge from '../ui/Badge'
import { aiService, ChatMessage, ChatRequest } from '../../services/aiService'

export interface ConversationModeProps {
  contextData?: unknown
  className?: string
  style?: React.CSSProperties
}

interface DocumentReference {
  id: string
  name: string
  excerpt?: string
  relevance?: number
}

interface ConversationState {
  messages: ChatMessage[]
  isLoading: boolean
  error: string | null
  sessionId: string | null
  suggestedActions: string[]
  followupQuestions: string[]
  documentReferences: DocumentReference[]
  isTyping: boolean
}

interface MessageDisplayProps {
  message: ChatMessage
  isLatest?: boolean
  documentReferences?: DocumentReference[]
}

const MessageDisplay: React.FC<MessageDisplayProps> = ({
  message,
  isLatest = false,
  documentReferences = [],
}) => {
  const isUser = message.role === 'user'
  const isSystem = message.role === 'system'

  const messageStyles = {
    display: 'flex',
    flexDirection: isUser ? ('row-reverse' as const) : ('row' as const),
    alignItems: 'flex-start',
    gap: designTokens.spacing[3],
    marginBottom: designTokens.spacing[4],
    padding: isSystem ? designTokens.spacing[2] : '0',
    backgroundColor: isSystem ? designTokens.colors.surface.tertiary : 'transparent',
    borderRadius: isSystem ? designTokens.borderRadius.md : '0',
    opacity: isSystem ? 0.7 : 1,
  }

  const avatarStyles = {
    width: '32px',
    height: '32px',
    borderRadius: designTokens.borderRadius.full,
    backgroundColor: isUser ? designTokens.colors.accent.semantic : designTokens.colors.accent.ai,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    color: designTokens.colors.surface.primary,
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
  }

  const bubbleStyles = {
    maxWidth: '85%',
    padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
    borderRadius: designTokens.borderRadius.lg,
    backgroundColor: isUser
      ? designTokens.colors.surface.quaternary
      : designTokens.colors.surface.tertiary,
    border: isUser ? 'none' : `1px solid ${designTokens.colors.border.subtle}`,
    position: 'relative' as const,
  }

  const contentStyles = {
    fontSize: designTokens.typography.fontSize.sm,
    lineHeight: designTokens.typography.lineHeight.relaxed,
    color: designTokens.colors.text.primary,
    margin: 0,
    whiteSpace: 'pre-wrap' as const,
  }

  if (isSystem) {
    return (
      <div style={messageStyles}>
        <div
          style={{
            fontSize: designTokens.typography.fontSize.xs,
            color: designTokens.colors.text.tertiary,
          }}
        >
          {message.content}
        </div>
      </div>
    )
  }

  return (
    <div style={messageStyles}>
      <div style={avatarStyles}>{isUser ? 'U' : <Icon name="Cpu" size={16} />}</div>
      <div style={bubbleStyles}>
        <p style={contentStyles}>{message.content}</p>

        {/* Document References */}
        {!isUser && documentReferences.length > 0 && (
          <div
            style={{
              marginTop: designTokens.spacing[3],
              paddingTop: designTokens.spacing[2],
              borderTop: `1px solid ${designTokens.colors.border.subtle}`,
            }}
          >
            <div
              style={{
                fontSize: designTokens.typography.fontSize.xs,
                color: designTokens.colors.text.secondary,
                fontWeight: designTokens.typography.fontWeight.semibold,
                marginBottom: designTokens.spacing[1],
                display: 'flex',
                alignItems: 'center',
                gap: designTokens.spacing[1],
              }}
            >
              <Icon name="FileText" size={12} />
              Referenced Documents
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[1] }}>
              {documentReferences.map(ref => (
                <div
                  key={ref.id}
                  style={{
                    padding: designTokens.spacing[2],
                    backgroundColor: designTokens.colors.surface.primary,
                    borderRadius: designTokens.borderRadius.sm,
                    border: `1px solid ${designTokens.colors.border.subtle}`,
                    cursor: 'pointer',
                    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
                  }}
                  className="document-reference-card"
                >
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      fontWeight: designTokens.typography.fontWeight.medium,
                      color: designTokens.colors.text.primary,
                      marginBottom: ref.excerpt ? designTokens.spacing[1] : 0,
                    }}
                  >
                    {ref.name}
                  </div>
                  {ref.excerpt && (
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                        lineHeight: designTokens.typography.lineHeight.snug,
                      }}
                    >
                      {ref.excerpt}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {isLatest && !isUser && (
          <div
            style={{
              marginTop: designTokens.spacing[2],
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.tertiary,
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[1],
            }}
          >
            <Icon name="Zap" size={12} />
            AI Response
          </div>
        )}
      </div>
    </div>
  )
}

const ConversationMode: React.FC<ConversationModeProps> = ({
  contextData,
  className = '',
  style,
}) => {
  const [conversationState, setConversationState] = useState<ConversationState>({
    messages: [],
    isLoading: false,
    error: null,
    sessionId: null,
    suggestedActions: [],
    followupQuestions: [],
    documentReferences: [],
    isTyping: false,
  })
  const [inputValue, setInputValue] = useState('')
  const [isInputFocused, setIsInputFocused] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Auto-scroll to latest message
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [conversationState.messages])

  // Initialize conversation with welcome message
  useEffect(() => {
    const welcomeMessage: ChatMessage = {
      role: 'system',
      content:
        'Welcome to Proxemic Intelligence. I can help you analyze documents, generate content, and optimize your workspace. How can I assist you today?',
    }

    setConversationState(prev => ({
      ...prev,
      messages: [welcomeMessage],
      suggestedActions: [
        'Analyze current document',
        'Compare documents',
        'Generate summary',
        'Find similar content',
        'Extract key insights',
      ],
    }))
  }, [])

  // Handle sending messages
  const handleSendMessage = useCallback(
    async (message: string) => {
      if (!message.trim() || conversationState.isLoading) return

      const userMessage: ChatMessage = {
        role: 'user',
        content: message.trim(),
      }

      setConversationState(prev => ({
        ...prev,
        messages: [...prev.messages, userMessage],
        isLoading: true,
        isTyping: true,
        error: null,
      }))

      setInputValue('')

      try {
        const chatRequest: ChatRequest = {
          messages: [...conversationState.messages, userMessage],
          options: {
            contextData: contextData,
            includeDocumentContext: true,
            includeWorkspaceContext: true,
          },
        }

        const response = await aiService.chat(chatRequest)

        if (response.success && response.data) {
          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: response.data.message.content,
            metadata: response.data.metadata,
          }

          // Extract document references from metadata if available
          const docRefs: DocumentReference[] =
            (response.data.metadata?.documentReferences as DocumentReference[]) || []

          // Get conversation suggestions for follow-up
          const suggestionsResponse = await aiService.getConversationSuggestions(
            conversationState.sessionId || 'current'
          )

          setConversationState(prev => ({
            ...prev,
            messages: [...prev.messages, assistantMessage],
            isLoading: false,
            isTyping: false,
            documentReferences: docRefs,
            followupQuestions: suggestionsResponse.success ? suggestionsResponse.data || [] : [],
          }))
        } else {
          throw new Error(response.error || 'Failed to get AI response')
        }
      } catch (error) {
        setConversationState(prev => ({
          ...prev,
          isLoading: false,
          isTyping: false,
          error: error instanceof Error ? error.message : 'An error occurred',
        }))
      }
    },
    [
      conversationState.messages,
      conversationState.sessionId,
      conversationState.isLoading,
      contextData,
    ]
  )

  // Export conversation to JSON
  const handleExportConversation = useCallback(() => {
    const exportData = {
      sessionId: conversationState.sessionId,
      timestamp: new Date().toISOString(),
      messages: conversationState.messages,
      metadata: {
        contextData,
      },
    }

    const dataStr = JSON.stringify(exportData, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)
    const link = document.createElement('a')
    link.href = url
    link.download = `proxemic-conversation-${Date.now()}.json`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  }, [conversationState.messages, conversationState.sessionId, contextData])

  // Import conversation from JSON
  const handleImportConversation = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

  // Handle file selection for import
  const handleFileSelect = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = e => {
      try {
        const importedData = JSON.parse(e.target?.result as string)
        if (importedData.messages && Array.isArray(importedData.messages)) {
          setConversationState(prev => ({
            ...prev,
            messages: importedData.messages,
            sessionId: importedData.sessionId || null,
          }))
        } else {
          throw new Error('Invalid conversation format')
        }
      } catch (error) {
        setConversationState(prev => ({
          ...prev,
          error:
            error instanceof Error
              ? `Import failed: ${error.message}`
              : 'Failed to import conversation',
        }))
      }
    }
    reader.readAsText(file)

    // Reset file input
    if (event.target) {
      event.target.value = ''
    }
  }, [])

  // Handle input submission
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault()
      handleSendMessage(inputValue)
    },
    [inputValue, handleSendMessage]
  )

  // Handle suggested action clicks
  const handleSuggestedAction = useCallback(
    (action: string) => {
      handleSendMessage(action)
    },
    [handleSendMessage]
  )

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column' as const,
      height: '100%',
      minHeight: '400px',
      ...style,
    }),
    [style]
  )

  const messagesContainerStyles = {
    flex: 1,
    overflowY: 'auto' as const,
    padding: designTokens.spacing[2],
    marginBottom: designTokens.spacing[4],
  }

  const inputContainerStyles = {
    padding: designTokens.spacing[3],
    borderTop: `1px solid ${designTokens.colors.border.subtle}`,
    backgroundColor: designTokens.colors.surface.secondary,
    position: 'sticky' as const,
    bottom: 0,
  }

  const suggestionsStyles = {
    display: 'flex',
    flexWrap: 'wrap' as const,
    gap: designTokens.spacing[2],
    marginBottom: designTokens.spacing[3],
  }

  const loadingIndicatorStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    backgroundColor: designTokens.colors.surface.tertiary,
    borderRadius: designTokens.borderRadius.md,
    margin: `0 ${designTokens.spacing[2]} ${designTokens.spacing[4]}`,
  }

  return (
    <div className={`proxemic-conversation-mode ${className}`} style={containerStyles}>
      {/* Conversation Header with Actions */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: `${designTokens.spacing[2]} ${designTokens.spacing[3]}`,
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
          backgroundColor: designTokens.colors.surface.tertiary,
        }}
      >
        <div
          style={{
            fontSize: designTokens.typography.fontSize.xs,
            color: designTokens.colors.text.secondary,
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[2],
          }}
        >
          <Icon name="MessageCircle" size={14} />
          {conversationState.messages.filter(m => m.role !== 'system').length} messages
        </div>
        <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
          <button
            onClick={handleExportConversation}
            disabled={conversationState.messages.length <= 1}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[1],
              padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.secondary,
              backgroundColor: 'transparent',
              border: `1px solid ${designTokens.colors.border.subtle}`,
              borderRadius: designTokens.borderRadius.sm,
              cursor: 'pointer',
              transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
            }}
            className="export-button"
            title="Export conversation"
          >
            <Icon name="Share2" size={12} />
            Export
          </button>
          <button
            onClick={handleImportConversation}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[1],
              padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.secondary,
              backgroundColor: 'transparent',
              border: `1px solid ${designTokens.colors.border.subtle}`,
              borderRadius: designTokens.borderRadius.sm,
              cursor: 'pointer',
              transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
            }}
            className="import-button"
            title="Import conversation"
          >
            <Icon name="ArrowRight" size={12} />
            Import
          </button>
        </div>
      </div>

      {/* Hidden file input for import */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".json"
        onChange={handleFileSelect}
        style={{ display: 'none' }}
      />

      {/* Messages Container */}
      <div className="messages-container" style={messagesContainerStyles}>
        {conversationState.messages.map((message, index) => (
          <MessageDisplay
            key={`${message.role}-${index}`}
            message={message}
            isLatest={index === conversationState.messages.length - 1}
            documentReferences={
              index === conversationState.messages.length - 1
                ? conversationState.documentReferences
                : []
            }
          />
        ))}

        {/* Typing Indicator */}
        {conversationState.isTyping && (
          <div style={loadingIndicatorStyles}>
            <Icon name="Loader" size={16} className="animate-spin" />
            AI is typing...
          </div>
        )}

        {/* Loading Indicator */}
        {conversationState.isLoading && !conversationState.isTyping && (
          <div style={loadingIndicatorStyles}>
            <Icon name="Loader" size={16} className="animate-spin" />
            AI is thinking...
          </div>
        )}

        {/* Error Display */}
        {conversationState.error && (
          <div
            style={{
              ...loadingIndicatorStyles,
              backgroundColor: `${designTokens.colors.accent.alert}20`,
              color: designTokens.colors.accent.alert,
              border: `1px solid ${designTokens.colors.accent.alert}40`,
            }}
          >
            <Icon name="AlertCircle" size={16} />
            {conversationState.error}
          </div>
        )}

        {/* Auto-scroll anchor */}
        <div ref={messagesEndRef} />
      </div>

      {/* Input Container */}
      <div style={inputContainerStyles}>
        {/* Suggested Actions */}
        {(conversationState.suggestedActions.length > 0 ||
          conversationState.followupQuestions.length > 0) && (
          <div style={suggestionsStyles}>
            {[...conversationState.suggestedActions, ...conversationState.followupQuestions]
              .slice(0, 3)
              .map((suggestion, index) => (
                <Badge
                  key={index}
                  variant="default"
                  size="sm"
                  style={{
                    cursor: 'pointer',
                    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
                  }}
                  onClick={() => handleSuggestedAction(suggestion)}
                >
                  {suggestion}
                </Badge>
              ))}
          </div>
        )}

        {/* Input Form */}
        <form onSubmit={handleSubmit}>
          <div style={{ display: 'flex', gap: designTokens.spacing[2], alignItems: 'flex-end' }}>
            <div style={{ flex: 1 }}>
              <Input
                value={inputValue}
                onChange={e => setInputValue(e.target.value)}
                placeholder="Ask about your documents, workspace, or request an operation..."
                disabled={conversationState.isLoading}
                onFocus={() => setIsInputFocused(true)}
                onBlur={() => setIsInputFocused(false)}
                style={{
                  backgroundColor: designTokens.colors.surface.primary,
                  border: `1px solid ${isInputFocused ? designTokens.colors.accent.ai : designTokens.colors.border.subtle}`,
                  transition: `border-color ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
                }}
              />
            </div>
            <Button
              type="submit"
              variant="primary"
              size="md"
              disabled={!inputValue.trim() || conversationState.isLoading}
              // icon={conversationState.isLoading ? 'Loader' : 'Send'}
            >
              {conversationState.isLoading ? 'Sending...' : 'Send'}
            </Button>
          </div>
        </form>
      </div>

      {/* Custom styles for animations and scrollbars */}
      <style>
        {`
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }

          .animate-spin {
            animation: spin 1s linear infinite;
          }

          .proxemic-conversation-mode .suggestion-badge:hover {
            background-color: ${designTokens.colors.state.hover};
            transform: translateY(-1px);
          }

          .proxemic-conversation-mode .export-button:hover,
          .proxemic-conversation-mode .import-button:hover {
            background-color: ${designTokens.colors.state.hover};
            color: ${designTokens.colors.text.primary};
            border-color: ${designTokens.colors.border.medium};
          }

          .proxemic-conversation-mode .export-button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }

          .proxemic-conversation-mode .document-reference-card:hover {
            background-color: ${designTokens.colors.state.hover};
            border-color: ${designTokens.colors.border.medium};
            transform: translateX(2px);
          }

          /* Custom scrollbar for messages container */
          .proxemic-conversation-mode .messages-container::-webkit-scrollbar {
            width: 6px;
          }

          .proxemic-conversation-mode .messages-container::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .proxemic-conversation-mode .messages-container::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }

          .proxemic-conversation-mode .messages-container::-webkit-scrollbar-thumb:hover {
            background: ${designTokens.colors.border.strong};
          }
        `}
      </style>
    </div>
  )
}

export default React.memo(ConversationMode)
