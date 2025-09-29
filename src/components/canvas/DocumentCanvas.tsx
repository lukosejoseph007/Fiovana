import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { Card, Button, Icon } from '../ui'
import { aiService, workspaceService } from '../../services'
import { designTokens } from '../../styles/tokens'
import { Message, WorkspaceHealth } from '../../types'
import type { IconComponentProps } from '../ui/Icon'
import DocumentViewer from './DocumentViewer'

interface DocumentCanvasProps {
  workspaceId?: string
  documentId?: string | null
  _onDocumentSelect?: (documentId: string) => void
  onModeChange?: (mode: 'chat' | 'document') => void
}

interface SuggestedAction {
  id: string
  title: string
  description: string
  icon: IconComponentProps['name']
  action: () => void
}

interface ConversationSession {
  id: string
  title: string
  lastMessage?: string
  timestamp: Date
  documentIds: string[]
}

const DocumentCanvas: React.FC<DocumentCanvasProps> = ({
  workspaceId,
  documentId,
  _onDocumentSelect,
  onModeChange,
}) => {
  const [mode, setMode] = useState<'chat' | 'document'>('chat')
  const [messages, setMessages] = useState<Message[]>([])
  const [currentInput, setCurrentInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [suggestedActions, setSuggestedActions] = useState<SuggestedAction[]>([])
  const [recentSessions, setRecentSessions] = useState<ConversationSession[]>([])
  const [workspaceHealth, setWorkspaceHealth] = useState<WorkspaceHealth | null>(null)
  const [isDragOver, setIsDragOver] = useState(false)

  // Handle sending messages
  const handleSendMessage = useCallback(
    async (message: string) => {
      if (!message.trim() || isLoading) return

      setIsLoading(true)
      const userMessage: Message = {
        id: `msg-${Date.now()}`,
        conversationId: 'current',
        senderId: 'user',
        content: message,
        type: 'text',
        timestamp: new Date(),
        metadata: {},
      }

      setMessages(prev => [...prev, userMessage])
      setCurrentInput('')

      try {
        // Send to AI service with workspace context
        const chatRequest = {
          messages: [
            ...messages.map(m => ({
              role: m.senderId === 'user' ? ('user' as const) : ('assistant' as const),
              content: m.content,
            })),
            { role: 'user' as const, content: message },
          ],
          options: {
            workspace_id: workspaceId,
            document_id: documentId,
          },
        }

        const response = await aiService.chat(chatRequest)

        if (response.success && response.data) {
          const assistantMessage: Message = {
            id: `msg-${Date.now()}-assistant`,
            conversationId: 'current',
            senderId: 'assistant',
            content: response.data.message.content,
            type: 'text',
            timestamp: new Date(),
            metadata: {
              model: response.data.model,
              tokens: response.data.usage.totalTokens,
              confidence: 0.9, // Mock confidence score
            },
          }

          setMessages(prev => [...prev, assistantMessage])
        } else {
          throw new Error(response.error || 'Failed to get AI response')
        }
      } catch (error) {
        console.error('Failed to send message:', error)
        const errorMessage: Message = {
          id: `msg-${Date.now()}-error`,
          conversationId: 'current',
          senderId: 'system',
          content: 'Sorry, I encountered an error processing your request. Please try again.',
          type: 'text',
          timestamp: new Date(),
          metadata: {},
        }
        setMessages(prev => [...prev, errorMessage])
      } finally {
        setIsLoading(false)
      }
    },
    [messages, isLoading, workspaceId, documentId]
  )

  // Handle suggested action clicks
  const handleSuggestedAction = useCallback(
    (actionType: string) => {
      const actionMessages: Record<string, string> = {
        analyze: 'Analyze my workspace documents and provide insights',
        compare: 'Compare my documents and show me the differences',
        generate: 'Help me generate new content based on my existing documents',
        gaps: 'Identify knowledge gaps in my workspace',
      }

      const message = actionMessages[actionType]
      if (message) {
        handleSendMessage(message)
      }
    },
    [handleSendMessage]
  )

  // Load workspace intelligence recommendations
  const loadWorkspaceRecommendations = useCallback(async () => {
    if (!workspaceId) return

    try {
      // Get workspace health for contextual suggestions
      const healthResponse = await workspaceService.getWorkspaceHealth(workspaceId)
      if (healthResponse.success && healthResponse.data) {
        setWorkspaceHealth(healthResponse.data)
      }

      // Generate suggested actions based on workspace state
      const suggestions: SuggestedAction[] = [
        {
          id: 'analyze-documents',
          title: 'Analyze Documents',
          description: 'Get insights and recommendations for your documents',
          icon: 'Search',
          action: () => handleSuggestedAction('analyze'),
        },
        {
          id: 'compare-versions',
          title: 'Compare Versions',
          description: 'Compare different versions of your documents',
          icon: 'Compare',
          action: () => handleSuggestedAction('compare'),
        },
        {
          id: 'generate-content',
          title: 'Generate New Content',
          description: 'Create new documents based on existing content',
          icon: 'Generate',
          action: () => handleSuggestedAction('generate'),
        },
        {
          id: 'review-gaps',
          title: 'Review Knowledge Gaps',
          description: 'Identify and address gaps in your knowledge base',
          icon: 'Health',
          action: () => handleSuggestedAction('gaps'),
        },
      ]

      setSuggestedActions(suggestions)
    } catch (error) {
      console.error('Failed to load workspace recommendations:', error)
    }
  }, [workspaceId, handleSuggestedAction])

  // Load recent conversation sessions
  const loadRecentSessions = useCallback(async () => {
    if (!workspaceId) return

    try {
      // In a real implementation, this would load from conversation history
      const sessions: ConversationSession[] = [
        {
          id: 'session-1',
          title: 'Document Review Session',
          lastMessage: 'Can you help me analyze the style consistency?',
          timestamp: new Date(Date.now() - 3600000), // 1 hour ago
          documentIds: ['doc-1', 'doc-2'],
        },
        {
          id: 'session-2',
          title: 'Content Generation',
          lastMessage: 'Generate a training manual from this technical documentation',
          timestamp: new Date(Date.now() - 86400000), // 1 day ago
          documentIds: ['doc-3'],
        },
      ]

      setRecentSessions(sessions)
    } catch (error) {
      console.error('Failed to load recent sessions:', error)
    }
  }, [workspaceId])

  // Handle file drop
  const handleFileDrop = useCallback(
    async (files: FileList) => {
      setIsDragOver(false)

      if (files.length === 0) return

      // Handle document import - this would typically upload and process files
      const fileNames = Array.from(files).map(file => file.name)
      const message = `I've uploaded ${fileNames.length} file(s): ${fileNames.join(', ')}. Please analyze ${
        fileNames.length === 1 ? 'it' : 'them'
      } and let me know what you find.`

      await handleSendMessage(message)
    },
    [handleSendMessage]
  )

  // Handle drag and drop
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(false)
  }, [])

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault()
      const files = e.dataTransfer.files
      handleFileDrop(files)
    },
    [handleFileDrop]
  )

  // Continue conversation from session
  const handleContinueSession = useCallback(async (session: ConversationSession) => {
    // Load session messages and continue
    setMessages([
      {
        id: 'session-message',
        conversationId: session.id,
        senderId: 'assistant',
        content: `Continuing from "${session.title}". How can I help you further?`,
        type: 'text',
        timestamp: new Date(),
        metadata: {},
      },
    ])
  }, [])

  // Mode switching
  const handleModeChange = useCallback(
    (newMode: 'chat' | 'document') => {
      setMode(newMode)
      onModeChange?.(newMode)
    },
    [onModeChange]
  )

  // Load initial data
  useEffect(() => {
    loadWorkspaceRecommendations()
    loadRecentSessions()
  }, [loadWorkspaceRecommendations, loadRecentSessions])

  // Welcome message for empty state
  const welcomeMessage = useMemo(() => {
    if (workspaceHealth) {
      return `Welcome back! Your workspace has a health score of ${Math.round(
        workspaceHealth.score * 100
      )}%. How can I help you today?`
    }
    return 'Welcome to Proxemic! I can help you analyze documents, generate content, and manage your knowledge base. What would you like to work on?'
  }, [workspaceHealth])

  return (
    <div
      className="document-canvas"
      style={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        background: designTokens.colors.surface.primary,
        position: 'relative',
      }}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Drag Overlay */}
      {isDragOver && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: `${designTokens.colors.accent.ai}20`,
            border: `2px dashed ${designTokens.colors.accent.ai}`,
            borderRadius: designTokens.borderRadius.lg,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: designTokens.zIndex.overlay,
          }}
        >
          <div
            style={{
              textAlign: 'center',
              color: designTokens.colors.accent.ai,
              fontSize: designTokens.typography.fontSize.xl,
              fontWeight: designTokens.typography.fontWeight.medium,
            }}
          >
            <Icon name="Document" size="xl" />
            <div style={{ marginTop: designTokens.spacing[2] }}>Drop documents here to analyze</div>
          </div>
        </div>
      )}

      {/* Chat Interface */}
      {mode === 'chat' && (
        <>
          {/* Home Button - Only show when there are messages (in conversation) */}
          {messages.length > 0 && (
            <div
              style={{
                position: 'absolute',
                top: designTokens.spacing[4],
                left: designTokens.spacing[6],
                zIndex: designTokens.zIndex.sticky,
              }}
            >
              <button
                onClick={() => {
                  setMessages([])
                  setCurrentInput('')
                }}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: designTokens.spacing[2],
                  padding: `${designTokens.spacing[2]} ${designTokens.spacing[3]}`,
                  backgroundColor: designTokens.colors.surface.secondary,
                  border: `1px solid ${designTokens.colors.border.subtle}`,
                  borderRadius: designTokens.borderRadius.md,
                  color: designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  cursor: 'pointer',
                  transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
                }}
                onMouseEnter={e => {
                  e.currentTarget.style.backgroundColor = designTokens.colors.state.hover
                  e.currentTarget.style.borderColor = designTokens.colors.accent.ai
                  e.currentTarget.style.color = designTokens.colors.text.primary
                }}
                onMouseLeave={e => {
                  e.currentTarget.style.backgroundColor = designTokens.colors.surface.secondary
                  e.currentTarget.style.borderColor = designTokens.colors.border.subtle
                  e.currentTarget.style.color = designTokens.colors.text.secondary
                }}
                title="Return to home screen"
              >
                <Icon name="ChevronDown" size={16} style={{ transform: 'rotate(90deg)' }} />
                Home
              </button>
            </div>
          )}

          {/* Messages Area */}
          <div
            style={{
              flex: 1,
              padding: `${designTokens.spacing[4]} ${designTokens.spacing[6]}`,
              paddingTop: messages.length > 0 ? designTokens.spacing[12] : designTokens.spacing[4],
              overflowY: 'auto',
              display: 'flex',
              flexDirection: 'column',
            }}
          >
            {/* Welcome State */}
            {messages.length === 0 && (
              <div
                style={{
                  flex: 1,
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                }}
              >
                {/* Welcome Message */}
                <div
                  style={{
                    textAlign: 'center',
                    marginBottom: designTokens.spacing[8],
                  }}
                >
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize['2xl'],
                      fontWeight: designTokens.typography.fontWeight.medium,
                      color: designTokens.colors.text.primary,
                      marginBottom: designTokens.spacing[4],
                    }}
                  >
                    {welcomeMessage}
                  </div>
                </div>

                {/* Suggested Actions */}
                <div style={{ marginBottom: designTokens.spacing[8] }}>
                  <h3
                    style={{
                      fontSize: designTokens.typography.fontSize.lg,
                      fontWeight: designTokens.typography.fontWeight.medium,
                      color: designTokens.colors.text.primary,
                      marginBottom: designTokens.spacing[4],
                      textAlign: 'center',
                    }}
                  >
                    Suggested Actions
                  </h3>
                  <div
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))',
                      gap: designTokens.spacing[4],
                      maxWidth: '800px',
                      margin: '0 auto',
                    }}
                  >
                    {suggestedActions.map(action => (
                      <Card
                        key={action.id}
                        variant="glass"
                        style={{
                          padding: designTokens.spacing[4],
                          cursor: 'pointer',
                          transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
                        }}
                        onClick={action.action}
                      >
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'flex-start',
                            gap: designTokens.spacing[3],
                          }}
                        >
                          <Icon
                            name={action.icon}
                            size="md"
                            color={designTokens.colors.accent.ai}
                          />
                          <div style={{ flex: 1 }}>
                            <div
                              style={{
                                fontSize: designTokens.typography.fontSize.base,
                                fontWeight: designTokens.typography.fontWeight.medium,
                                color: designTokens.colors.text.primary,
                                marginBottom: designTokens.spacing[1],
                              }}
                            >
                              {action.title}
                            </div>
                            <div
                              style={{
                                fontSize: designTokens.typography.fontSize.sm,
                                color: designTokens.colors.text.muted,
                                lineHeight: designTokens.typography.lineHeight.normal,
                              }}
                            >
                              {action.description}
                            </div>
                          </div>
                        </div>
                      </Card>
                    ))}
                  </div>
                </div>

                {/* Recent Conversations */}
                {recentSessions.length > 0 && (
                  <div>
                    <h3
                      style={{
                        fontSize: designTokens.typography.fontSize.lg,
                        fontWeight: designTokens.typography.fontWeight.medium,
                        color: designTokens.colors.text.primary,
                        marginBottom: designTokens.spacing[4],
                        textAlign: 'center',
                      }}
                    >
                      Continue Recent Conversations
                    </h3>
                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: designTokens.spacing[3],
                        maxWidth: '600px',
                        margin: '0 auto',
                      }}
                    >
                      {recentSessions.map(session => (
                        <Card
                          key={session.id}
                          variant="default"
                          style={{
                            padding: designTokens.spacing[4],
                            cursor: 'pointer',
                            transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
                          }}
                          onClick={() => handleContinueSession(session)}
                        >
                          <div
                            style={{
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'flex-start',
                            }}
                          >
                            <div style={{ flex: 1 }}>
                              <div
                                style={{
                                  fontSize: designTokens.typography.fontSize.base,
                                  fontWeight: designTokens.typography.fontWeight.medium,
                                  color: designTokens.colors.text.primary,
                                  marginBottom: designTokens.spacing[1],
                                }}
                              >
                                {session.title}
                              </div>
                              {session.lastMessage && (
                                <div
                                  style={{
                                    fontSize: designTokens.typography.fontSize.sm,
                                    color: designTokens.colors.text.secondary,
                                    marginBottom: designTokens.spacing[2],
                                  }}
                                >
                                  {session.lastMessage}
                                </div>
                              )}
                              <div
                                style={{
                                  fontSize: designTokens.typography.fontSize.xs,
                                  color: designTokens.colors.text.tertiary,
                                }}
                              >
                                {session.timestamp.toLocaleDateString()} •{' '}
                                {session.documentIds.length} document
                                {session.documentIds.length !== 1 ? 's' : ''}
                              </div>
                            </div>
                            <Icon
                              name="ChevronDown"
                              size="sm"
                              color={designTokens.colors.text.tertiary}
                            />
                          </div>
                        </Card>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Messages */}
            {messages.map(message => (
              <div
                key={message.id}
                style={{
                  display: 'flex',
                  marginBottom: designTokens.spacing[4],
                  alignItems: 'flex-start',
                  justifyContent: message.senderId === 'user' ? 'flex-end' : 'flex-start',
                }}
              >
                <div
                  style={{
                    maxWidth: '70%',
                    padding: designTokens.spacing[3],
                    borderRadius: designTokens.borderRadius.lg,
                    background:
                      message.senderId === 'user'
                        ? designTokens.colors.accent.ai
                        : message.senderId === 'system'
                          ? designTokens.colors.accent.alert
                          : designTokens.colors.surface.secondary,
                    color:
                      message.senderId === 'user'
                        ? designTokens.colors.surface.primary
                        : designTokens.colors.text.primary,
                  }}
                >
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.base,
                      lineHeight: designTokens.typography.lineHeight.normal,
                      marginBottom:
                        message.metadata && Object.keys(message.metadata).length > 0
                          ? designTokens.spacing[2]
                          : 0,
                    }}
                  >
                    {message.content}
                  </div>
                  {message.metadata?.model && (
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        opacity: 0.7,
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[2],
                      }}
                    >
                      <span>{message.metadata.model}</span>
                      {message.metadata.tokens && <span>• {message.metadata.tokens} tokens</span>}
                      {message.metadata.confidence && (
                        <span>• {Math.round(message.metadata.confidence * 100)}% confident</span>
                      )}
                    </div>
                  )}
                </div>
              </div>
            ))}

            {/* Loading Indicator */}
            {isLoading && (
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: designTokens.spacing[2],
                  color: designTokens.colors.text.secondary,
                  marginBottom: designTokens.spacing[4],
                }}
              >
                <div
                  style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    background: designTokens.colors.accent.ai,
                    animation: `${designTokens.animation.keyframes.pulse} 1s ease-in-out infinite`,
                  }}
                />
                <span>AI is thinking...</span>
              </div>
            )}
          </div>

          {/* Input Area */}
          <div
            style={{
              padding: `${designTokens.spacing[4]} ${designTokens.spacing[6]}`,
              borderTop: `1px solid ${designTokens.colors.border.subtle}`,
              background: designTokens.colors.surface.secondary,
            }}
          >
            <div style={{ display: 'flex', gap: designTokens.spacing[2], alignItems: 'flex-end' }}>
              <textarea
                value={currentInput}
                onChange={e => setCurrentInput(e.target.value)}
                onKeyDown={e => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault()
                    handleSendMessage(currentInput)
                  }
                }}
                placeholder="Ask me anything about your documents..."
                disabled={isLoading}
                style={{
                  flex: 1,
                  minHeight: '40px',
                  maxHeight: '120px',
                  padding: designTokens.spacing[3],
                  borderRadius: designTokens.borderRadius.lg,
                  border: `1px solid ${designTokens.colors.border.subtle}`,
                  background: designTokens.colors.surface.primary,
                  color: designTokens.colors.text.primary,
                  fontSize: designTokens.typography.fontSize.base,
                  resize: 'none',
                  outline: 'none',
                  fontFamily: designTokens.typography.fonts.sans.join(', '),
                }}
              />
              <Button
                variant="primary"
                size="md"
                onClick={() => handleSendMessage(currentInput)}
                disabled={!currentInput.trim() || isLoading}
                style={{
                  padding: designTokens.spacing[3],
                  minWidth: '60px',
                }}
              >
                <Icon name="Generate" size="sm" />
              </Button>
            </div>
          </div>
        </>
      )}

      {/* Document Mode */}
      {mode === 'document' && documentId && (
        <DocumentViewer
          documentId={documentId}
          _workspaceId={workspaceId}
          onClose={() => handleModeChange('chat')}
          onAISuggestionSelect={suggestion => {
            console.log('AI suggestion selected:', suggestion)
            // This could trigger actions like opening a modal or adding content to chat
          }}
        />
      )}

      {/* Document Mode - No Document Selected */}
      {mode === 'document' && !documentId && (
        <div
          style={{
            flex: 1,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexDirection: 'column',
            gap: designTokens.spacing[4],
          }}
        >
          <Icon name="Document" size="xl" color={designTokens.colors.text.tertiary} />
          <div
            style={{
              textAlign: 'center',
              color: designTokens.colors.text.secondary,
              fontSize: designTokens.typography.fontSize.lg,
            }}
          >
            No document selected
          </div>
          <Button variant="secondary" onClick={() => handleModeChange('chat')}>
            Return to Chat
          </Button>
        </div>
      )}
    </div>
  )
}

export default DocumentCanvas
