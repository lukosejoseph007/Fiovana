import React, { useState, useCallback, useEffect, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Button from '../ui/Button'
import Input from '../ui/Input'
import Icon from '../ui/Icon'
import Badge from '../ui/Badge'
import { aiService, ChatMessage, ChatRequest } from '../../services/aiService'
// import { conversationIntelligenceService } from '../../services/conversationIntelligenceService'

export interface ConversationModeProps {
  contextData?: unknown
  className?: string
  style?: React.CSSProperties
}

interface ConversationState {
  messages: ChatMessage[]
  isLoading: boolean
  error: string | null
  sessionId: string | null
  suggestedActions: string[]
  followupQuestions: string[]
}

interface MessageDisplayProps {
  message: ChatMessage
  isLatest?: boolean
}

const MessageDisplay: React.FC<MessageDisplayProps> = ({ message, isLatest = false }) => {
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
  })
  const [inputValue, setInputValue] = useState('')
  const [isInputFocused, setIsInputFocused] = useState(false)

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

          // Get conversation suggestions for follow-up
          const suggestionsResponse = await aiService.getConversationSuggestions(
            conversationState.sessionId || 'current'
          )

          setConversationState(prev => ({
            ...prev,
            messages: [...prev.messages, assistantMessage],
            isLoading: false,
            followupQuestions: suggestionsResponse.success ? suggestionsResponse.data || [] : [],
          }))
        } else {
          throw new Error(response.error || 'Failed to get AI response')
        }
      } catch (error) {
        setConversationState(prev => ({
          ...prev,
          isLoading: false,
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
      {/* Messages Container */}
      <div style={messagesContainerStyles}>
        {conversationState.messages.map((message, index) => (
          <MessageDisplay
            key={`${message.role}-${index}`}
            message={message}
            isLatest={index === conversationState.messages.length - 1}
          />
        ))}

        {/* Loading Indicator */}
        {conversationState.isLoading && (
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

      {/* Custom styles for animations */}
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
        `}
      </style>
    </div>
  )
}

export default React.memo(ConversationMode)
