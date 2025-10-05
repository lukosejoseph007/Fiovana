import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Input from './Input'
import Badge from './Badge'
import Icon from './Icon'

type IconName =
  | 'Document'
  | 'PDF'
  | 'Word'
  | 'PowerPoint'
  | 'AIStatus'
  | 'Health'
  | 'Confidence'
  | 'Compare'
  | 'Generate'
  | 'Analyze'
  | 'Search'
  | 'Settings'
  | 'Workspace'
  | 'Spinner'
  | 'Pulse'
  | 'User'
  | 'Collaboration'
  | 'ChevronDown'

export interface CommandItem {
  id: string
  title: string
  description: string
  category: 'analyze' | 'generate' | 'compare' | 'organize' | 'workspace' | 'settings' | 'search'
  icon: IconName
  keywords: string[]
  shortcut?: string
  recent?: boolean
  action: () => void | Promise<void>
}

export interface DocumentResult {
  id: string
  title: string
  path: string
  type: 'pdf' | 'word' | 'powerpoint' | 'document'
  lastModified: Date
  confidence: number
  snippet?: string
}

export interface ConversationResult {
  id: string
  title: string
  lastMessage: string
  timestamp: Date
  messageCount: number
}

export interface CommandPaletteProps {
  isOpen: boolean
  onClose: () => void
  onCommandExecute: (command: CommandItem) => void
  onDocumentOpen: (document: DocumentResult) => void
  onConversationOpen: (conversation: ConversationResult) => void
  recentCommands?: CommandItem[]
  className?: string
}

const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  onCommandExecute,
  onDocumentOpen,
  onConversationOpen,
  recentCommands = [],
  className = '',
}) => {
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [activeTab, setActiveTab] = useState<'all' | 'commands' | 'documents' | 'conversations'>(
    'all'
  )
  const [isLoading, setIsLoading] = useState(false)

  const inputRef = useRef<HTMLInputElement>(null)
  const resultRefs = useRef<(HTMLDivElement | null)[]>([])
  const overlayRef = useRef<HTMLDivElement>(null)

  // Mock command data - In real implementation, this would come from the 35 backend command modules
  const availableCommands: CommandItem[] = useMemo(
    () => [
      // Document Analysis Commands
      {
        id: 'analyze-structure',
        title: 'Analyze Document Structure',
        description: 'Analyze headings, sections, and document organization',
        category: 'analyze',
        icon: 'Analyze',
        keywords: ['analyze', 'structure', 'headings', 'organization'],
        shortcut: 'Cmd+A+S',
        action: () => console.log('Analyzing structure'),
      },
      {
        id: 'analyze-content',
        title: 'Classify Content',
        description: 'Identify content types, procedures, and concepts',
        category: 'analyze',
        icon: 'Analyze',
        keywords: ['classify', 'content', 'procedures', 'concepts'],
        action: () => console.log('Classifying content'),
      },
      {
        id: 'analyze-style',
        title: 'Analyze Writing Style',
        description: 'Examine tone, complexity, and writing patterns',
        category: 'analyze',
        icon: 'Analyze',
        keywords: ['style', 'tone', 'writing', 'patterns'],
        action: () => console.log('Analyzing style'),
      },

      // Generation Commands
      {
        id: 'generate-summary',
        title: 'Generate Summary',
        description: 'Create executive summary with key points',
        category: 'generate',
        icon: 'Generate',
        keywords: ['generate', 'summary', 'key points', 'executive'],
        shortcut: 'Cmd+G+S',
        action: () => console.log('Generating summary'),
      },
      {
        id: 'generate-presentation',
        title: 'Create Presentation',
        description: 'Generate PowerPoint from document content',
        category: 'generate',
        icon: 'PowerPoint',
        keywords: ['presentation', 'powerpoint', 'slides'],
        action: () => console.log('Creating presentation'),
      },
      {
        id: 'generate-questions',
        title: 'Generate Discussion Questions',
        description: 'Create thought-provoking questions for engagement',
        category: 'generate',
        icon: 'Generate',
        keywords: ['questions', 'discussion', 'engagement'],
        action: () => console.log('Generating questions'),
      },

      // Comparison Commands
      {
        id: 'compare-documents',
        title: 'Compare Documents',
        description: 'Side-by-side document comparison with differences',
        category: 'compare',
        icon: 'Compare',
        keywords: ['compare', 'differences', 'side by side'],
        shortcut: 'Cmd+C+D',
        action: () => console.log('Comparing documents'),
      },
      {
        id: 'compare-versions',
        title: 'Compare Versions',
        description: 'Track changes between document versions',
        category: 'compare',
        icon: 'Compare',
        keywords: ['versions', 'changes', 'track'],
        action: () => console.log('Comparing versions'),
      },

      // Organization Commands
      {
        id: 'organize-smart-collections',
        title: 'Create Smart Collections',
        description: 'Auto-organize documents by content similarity',
        category: 'organize',
        icon: 'Workspace',
        keywords: ['organize', 'collections', 'similarity', 'group'],
        action: () => console.log('Creating smart collections'),
      },
      {
        id: 'organize-tag-documents',
        title: 'Tag Documents',
        description: 'Automatically tag documents by content type',
        category: 'organize',
        icon: 'Workspace',
        keywords: ['tag', 'labels', 'categories'],
        action: () => console.log('Tagging documents'),
      },

      // Workspace Commands
      {
        id: 'workspace-health',
        title: 'Check Workspace Health',
        description: 'Analyze workspace organization and identify gaps',
        category: 'workspace',
        icon: 'Health',
        keywords: ['health', 'gaps', 'organization'],
        shortcut: 'Cmd+W+H',
        action: () => console.log('Checking workspace health'),
      },
      {
        id: 'workspace-insights',
        title: 'View Workspace Insights',
        description: 'Analytics and trends for your workspace',
        category: 'workspace',
        icon: 'Workspace',
        keywords: ['insights', 'analytics', 'trends'],
        action: () => console.log('Viewing insights'),
      },

      // Settings Commands
      {
        id: 'settings-ai',
        title: 'Configure AI Providers',
        description: 'Manage Ollama, OpenRouter, and Anthropic settings',
        category: 'settings',
        icon: 'Settings',
        keywords: ['ai', 'providers', 'ollama', 'openrouter', 'anthropic'],
        action: () => console.log('Configuring AI'),
      },
      {
        id: 'settings-workspace',
        title: 'Workspace Settings',
        description: 'Configure workspace behavior and preferences',
        category: 'settings',
        icon: 'Settings',
        keywords: ['workspace', 'preferences', 'behavior'],
        action: () => console.log('Workspace settings'),
      },

      // Search Commands
      {
        id: 'search-semantic',
        title: 'Semantic Search',
        description: 'Search by meaning and context across documents',
        category: 'search',
        icon: 'Search',
        keywords: ['semantic', 'meaning', 'context'],
        shortcut: 'Cmd+/',
        action: () => console.log('Semantic search'),
      },
      {
        id: 'search-similar',
        title: 'Find Similar Documents',
        description: 'Discover documents with similar content',
        category: 'search',
        icon: 'Search',
        keywords: ['similar', 'discover', 'related'],
        action: () => console.log('Finding similar'),
      },
    ],
    []
  )

  // Filter and search logic
  const filteredResults = useMemo(() => {
    if (!query.trim()) {
      return {
        commands: recentCommands.slice(0, 5),
        documents: [],
        conversations: [],
      }
    }

    const lowerQuery = query.toLowerCase()

    // Filter commands
    const commands = availableCommands
      .filter(command => {
        return (
          command.title.toLowerCase().includes(lowerQuery) ||
          command.description.toLowerCase().includes(lowerQuery) ||
          command.keywords.some(keyword => keyword.toLowerCase().includes(lowerQuery)) ||
          command.category.toLowerCase().includes(lowerQuery)
        )
      })
      .slice(0, 8)

    // Mock document search results
    const documents: DocumentResult[] =
      query.length > 2
        ? [
            {
              id: 'doc-1',
              title: `Results for "${query}"`,
              path: '/workspace/documents/sample.pdf',
              type: 'pdf',
              lastModified: new Date(),
              confidence: 0.95,
              snippet: `Document containing relevant information about ${query}...`,
            },
          ]
        : []

    // Mock conversation search results
    const conversations: ConversationResult[] =
      query.length > 2
        ? [
            {
              id: 'conv-1',
              title: `Discussion about ${query}`,
              lastMessage: `Let me help you understand more about ${query}...`,
              timestamp: new Date(Date.now() - 3600000),
              messageCount: 12,
            },
          ]
        : []

    return { commands, documents, conversations }
  }, [query, availableCommands, recentCommands])

  // Get all results for keyboard navigation
  const allResults = useMemo(() => {
    const results: Array<{
      type: 'command' | 'document' | 'conversation'
      item: CommandItem | DocumentResult | ConversationResult
      index: number
    }> = []

    if (activeTab === 'all' || activeTab === 'commands') {
      filteredResults.commands.forEach(item => {
        results.push({ type: 'command', item, index: results.length })
      })
    }

    if (activeTab === 'all' || activeTab === 'documents') {
      filteredResults.documents.forEach(item => {
        results.push({ type: 'document', item, index: results.length })
      })
    }

    if (activeTab === 'all' || activeTab === 'conversations') {
      filteredResults.conversations.forEach(item => {
        results.push({ type: 'conversation', item, index: results.length })
      })
    }

    return results
  }, [filteredResults, activeTab])

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!isOpen) return

      switch (event.key) {
        case 'Escape':
          event.preventDefault()
          onClose()
          break

        case 'ArrowDown':
          event.preventDefault()
          setSelectedIndex(prev => Math.min(prev + 1, allResults.length - 1))
          break

        case 'ArrowUp':
          event.preventDefault()
          setSelectedIndex(prev => Math.max(prev - 1, 0))
          break

        case 'Enter':
          event.preventDefault()
          if (allResults[selectedIndex]) {
            const { type, item } = allResults[selectedIndex]
            if (type === 'command') {
              onCommandExecute(item as CommandItem)
            } else if (type === 'document') {
              onDocumentOpen(item as DocumentResult)
            } else if (type === 'conversation') {
              onConversationOpen(item as ConversationResult)
            }
            onClose()
          }
          break

        case 'Tab': {
          event.preventDefault()
          const tabs = ['all', 'commands', 'documents', 'conversations']
          const currentIndex = tabs.indexOf(activeTab)
          const nextIndex = (currentIndex + 1) % tabs.length
          setActiveTab(tabs[nextIndex] as typeof activeTab)
          setSelectedIndex(0)
          break
        }
        default:
          return
      }
    },
    [
      isOpen,
      selectedIndex,
      allResults,
      activeTab,
      onClose,
      onCommandExecute,
      onDocumentOpen,
      onConversationOpen,
    ]
  )

  // Effect for keyboard handling
  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  // Focus management
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus()
      setQuery('')
      setSelectedIndex(0)
      setActiveTab('all')
    }
  }, [isOpen])

  // Scroll selected item into view
  useEffect(() => {
    if (resultRefs.current[selectedIndex]) {
      resultRefs.current[selectedIndex]?.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
      })
    }
  }, [selectedIndex])

  // Mock search function (would be replaced with real API calls)
  useEffect(() => {
    if (query.trim()) {
      setIsLoading(true)
      const timeout = setTimeout(() => setIsLoading(false), 300)
      return () => clearTimeout(timeout)
    }
    return undefined
  }, [query])

  if (!isOpen) return null

  const categoryIcons: Record<string, IconName> = {
    analyze: 'Analyze',
    generate: 'Generate',
    compare: 'Compare',
    organize: 'Workspace',
    workspace: 'Workspace',
    settings: 'Settings',
    search: 'Search',
  }

  const categoryColors: Record<string, string> = {
    analyze: designTokens.colors.accent.info,
    generate: designTokens.colors.accent.success,
    compare: designTokens.colors.accent.warning,
    organize: designTokens.colors.accent.ai,
    workspace: designTokens.colors.accent.semantic,
    settings: designTokens.colors.accent.alert,
    search: designTokens.colors.accent.ai,
  }

  return (
    <>
      <style>
        {`
          @keyframes commandPaletteSlideIn {
            from {
              opacity: 0;
              transform: scale(0.98) translateY(-10px);
            }
            to {
              opacity: 1;
              transform: scale(1) translateY(0);
            }
          }

          .command-palette-result {
            transition: all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut};
          }

          .command-palette-result:hover,
          .command-palette-result.selected {
            background-color: ${designTokens.colors.state.hover};
            transform: translateX(2px);
          }

          .command-palette-result.selected {
            border-left: 2px solid ${designTokens.colors.accent.ai};
          }

          .command-palette-tabs button:hover {
            background-color: ${designTokens.colors.state.hover};
          }

          .command-palette-tabs button.active {
            background-color: ${designTokens.colors.accent.ai}20;
            color: ${designTokens.colors.accent.ai};
            border-bottom: 2px solid ${designTokens.colors.accent.ai};
          }
        `}
      </style>

      <div
        ref={overlayRef}
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: designTokens.colors.background.overlay,
          backdropFilter: 'blur(12px)',
          zIndex: designTokens.zIndex.modal + 1,
          display: 'flex',
          alignItems: 'flex-start',
          justifyContent: 'center',
          padding: `${designTokens.spacing[20]} ${designTokens.spacing[4]}`,
          animation: 'fadeIn 0.2s ease-out',
        }}
        onClick={e => {
          if (e.target === overlayRef.current) {
            onClose()
          }
        }}
      >
        <div
          className={`fiovana-command-palette ${className}`}
          style={{
            width: '100%',
            maxWidth: '640px',
            backgroundColor: designTokens.colors.surface.primary,
            borderRadius: designTokens.borderRadius.xl,
            border: `1px solid ${designTokens.colors.border.subtle}`,
            boxShadow: designTokens.shadows['2xl'],
            overflow: 'hidden',
            animation: 'commandPaletteSlideIn 0.3s ease-out',
          }}
        >
          {/* Search Input */}
          <div
            style={{
              padding: designTokens.spacing[4],
              borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
            }}
          >
            <Input
              ref={inputRef}
              variant="command"
              size="lg"
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder="Search commands, documents, conversations..."
              leftIcon={<Icon name="Search" size={20} />}
              rightIcon={
                isLoading ? (
                  <Icon name="Spinner" size={16} />
                ) : (
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: designTokens.spacing[2],
                      fontSize: designTokens.typography.fontSize.xs,
                      color: designTokens.colors.text.tertiary,
                    }}
                  >
                    <kbd
                      style={{
                        padding: '2px 6px',
                        borderRadius: '4px',
                        backgroundColor: designTokens.colors.surface.tertiary,
                      }}
                    >
                      ⌘K
                    </kbd>
                  </div>
                )
              }
              fullWidth
            />
          </div>

          {/* Tabs */}
          <div
            className="command-palette-tabs"
            style={{
              display: 'flex',
              borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
              backgroundColor: designTokens.colors.surface.secondary,
            }}
          >
            {(['all', 'commands', 'documents', 'conversations'] as const).map(tab => (
              <button
                key={tab}
                className={activeTab === tab ? 'active' : ''}
                onClick={() => {
                  setActiveTab(tab)
                  setSelectedIndex(0)
                }}
                style={{
                  flex: 1,
                  padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
                  border: 'none',
                  background: 'none',
                  color: designTokens.colors.text.secondary,
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  cursor: 'pointer',
                  textTransform: 'capitalize',
                  transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
                }}
              >
                {tab}
                {tab === 'commands' && filteredResults.commands.length > 0 && (
                  <Badge
                    variant="default"
                    size="sm"
                    style={{ marginLeft: designTokens.spacing[2] }}
                  >
                    {filteredResults.commands.length}
                  </Badge>
                )}
                {tab === 'documents' && filteredResults.documents.length > 0 && (
                  <Badge
                    variant="default"
                    size="sm"
                    style={{ marginLeft: designTokens.spacing[2] }}
                  >
                    {filteredResults.documents.length}
                  </Badge>
                )}
                {tab === 'conversations' && filteredResults.conversations.length > 0 && (
                  <Badge
                    variant="default"
                    size="sm"
                    style={{ marginLeft: designTokens.spacing[2] }}
                  >
                    {filteredResults.conversations.length}
                  </Badge>
                )}
              </button>
            ))}
          </div>

          {/* Results */}
          <div
            style={{
              maxHeight: '400px',
              overflowY: 'auto',
              padding: designTokens.spacing[2],
            }}
          >
            {allResults.length === 0 && !isLoading ? (
              <div
                style={{
                  padding: `${designTokens.spacing[8]} ${designTokens.spacing[4]}`,
                  textAlign: 'center',
                  color: designTokens.colors.text.secondary,
                }}
              >
                {query.trim() ? (
                  <>
                    <div style={{ marginBottom: designTokens.spacing[4], opacity: 0.5 }}>
                      <Icon name="Search" size={32} />
                    </div>
                    <p>No results found for "{query}"</p>
                    <p
                      style={{
                        fontSize: designTokens.typography.fontSize.sm,
                        marginTop: designTokens.spacing[2],
                      }}
                    >
                      Try different keywords or browse commands
                    </p>
                  </>
                ) : (
                  <>
                    <div style={{ marginBottom: designTokens.spacing[4], opacity: 0.5 }}>
                      <Icon name="Workspace" size={32} />
                    </div>
                    <p>Start typing to search commands</p>
                    <p
                      style={{
                        fontSize: designTokens.typography.fontSize.sm,
                        marginTop: designTokens.spacing[2],
                      }}
                    >
                      Use Tab to switch between categories
                    </p>
                  </>
                )}
              </div>
            ) : (
              <div>
                {/* Commands */}
                {(activeTab === 'all' || activeTab === 'commands') &&
                  filteredResults.commands.map(command => {
                    const globalIndex = allResults.findIndex(
                      r => r.type === 'command' && r.item.id === command.id
                    )
                    return (
                      <div
                        key={command.id}
                        ref={el => {
                          resultRefs.current[globalIndex] = el
                        }}
                        className={`command-palette-result ${selectedIndex === globalIndex ? 'selected' : ''}`}
                        onClick={() => {
                          onCommandExecute(command)
                          onClose()
                        }}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: designTokens.spacing[3],
                          padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
                          borderRadius: designTokens.borderRadius.md,
                          cursor: 'pointer',
                          marginBottom: designTokens.spacing[1],
                        }}
                      >
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            width: '32px',
                            height: '32px',
                            borderRadius: designTokens.borderRadius.md,
                            backgroundColor: `${categoryColors[command.category]}20`,
                          }}
                        >
                          <Icon
                            name={categoryIcons[command.category] || 'Settings'}
                            size={16}
                            color={categoryColors[command.category]}
                          />
                        </div>
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: designTokens.spacing[2],
                              marginBottom: designTokens.spacing[1],
                            }}
                          >
                            <span
                              style={{
                                fontSize: designTokens.typography.fontSize.sm,
                                fontWeight: designTokens.typography.fontWeight.medium,
                                color: designTokens.colors.text.primary,
                              }}
                            >
                              {command.title}
                            </span>
                            <Badge variant="default" size="sm">
                              {command.category}
                            </Badge>
                            {command.recent && (
                              <Badge variant="success" size="sm">
                                Recent
                              </Badge>
                            )}
                          </div>
                          <p
                            style={{
                              fontSize: designTokens.typography.fontSize.xs,
                              color: designTokens.colors.text.secondary,
                              margin: 0,
                              overflow: 'hidden',
                              textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap',
                            }}
                          >
                            {command.description}
                          </p>
                        </div>
                        {command.shortcut && (
                          <div
                            style={{
                              fontSize: designTokens.typography.fontSize.xs,
                              color: designTokens.colors.text.tertiary,
                              fontFamily: designTokens.typography.fonts.mono.join(', '),
                            }}
                          >
                            <kbd
                              style={{
                                padding: '2px 6px',
                                borderRadius: '4px',
                                backgroundColor: designTokens.colors.surface.tertiary,
                              }}
                            >
                              {command.shortcut}
                            </kbd>
                          </div>
                        )}
                      </div>
                    )
                  })}

                {/* Documents */}
                {(activeTab === 'all' || activeTab === 'documents') &&
                  filteredResults.documents.map(document => {
                    const globalIndex = allResults.findIndex(
                      r => r.type === 'document' && r.item.id === document.id
                    )
                    const typeIcons: Record<string, IconName> = {
                      pdf: 'PDF',
                      word: 'Word',
                      powerpoint: 'PowerPoint',
                      document: 'Document',
                    }
                    return (
                      <div
                        key={document.id}
                        ref={el => {
                          resultRefs.current[globalIndex] = el
                        }}
                        className={`command-palette-result ${selectedIndex === globalIndex ? 'selected' : ''}`}
                        onClick={() => {
                          onDocumentOpen(document)
                          onClose()
                        }}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: designTokens.spacing[3],
                          padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
                          borderRadius: designTokens.borderRadius.md,
                          cursor: 'pointer',
                          marginBottom: designTokens.spacing[1],
                        }}
                      >
                        <Icon name={typeIcons[document.type] || 'Document'} size={24} />
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: designTokens.spacing[2],
                              marginBottom: designTokens.spacing[1],
                            }}
                          >
                            <span
                              style={{
                                fontSize: designTokens.typography.fontSize.sm,
                                fontWeight: designTokens.typography.fontWeight.medium,
                                color: designTokens.colors.text.primary,
                              }}
                            >
                              {document.title}
                            </span>
                            <Badge variant="confidence" size="sm">
                              {Math.round(document.confidence * 100)}%
                            </Badge>
                          </div>
                          <p
                            style={{
                              fontSize: designTokens.typography.fontSize.xs,
                              color: designTokens.colors.text.secondary,
                              margin: 0,
                              marginBottom: designTokens.spacing[1],
                            }}
                          >
                            {document.path}
                          </p>
                          {document.snippet && (
                            <p
                              style={{
                                fontSize: designTokens.typography.fontSize.xs,
                                color: designTokens.colors.text.tertiary,
                                margin: 0,
                                overflow: 'hidden',
                                textOverflow: 'ellipsis',
                                whiteSpace: 'nowrap',
                              }}
                            >
                              {document.snippet}
                            </p>
                          )}
                        </div>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.tertiary,
                          }}
                        >
                          {document.lastModified.toLocaleDateString()}
                        </div>
                      </div>
                    )
                  })}

                {/* Conversations */}
                {(activeTab === 'all' || activeTab === 'conversations') &&
                  filteredResults.conversations.map(conversation => {
                    const globalIndex = allResults.findIndex(
                      r => r.type === 'conversation' && r.item.id === conversation.id
                    )
                    return (
                      <div
                        key={conversation.id}
                        ref={el => {
                          resultRefs.current[globalIndex] = el
                        }}
                        className={`command-palette-result ${selectedIndex === globalIndex ? 'selected' : ''}`}
                        onClick={() => {
                          onConversationOpen(conversation)
                          onClose()
                        }}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: designTokens.spacing[3],
                          padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
                          borderRadius: designTokens.borderRadius.md,
                          cursor: 'pointer',
                          marginBottom: designTokens.spacing[1],
                        }}
                      >
                        <Icon name="Collaboration" size={24} />
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: designTokens.spacing[2],
                              marginBottom: designTokens.spacing[1],
                            }}
                          >
                            <span
                              style={{
                                fontSize: designTokens.typography.fontSize.sm,
                                fontWeight: designTokens.typography.fontWeight.medium,
                                color: designTokens.colors.text.primary,
                              }}
                            >
                              {conversation.title}
                            </span>
                            <Badge variant="default" size="sm">
                              {conversation.messageCount} messages
                            </Badge>
                          </div>
                          <p
                            style={{
                              fontSize: designTokens.typography.fontSize.xs,
                              color: designTokens.colors.text.secondary,
                              margin: 0,
                              overflow: 'hidden',
                              textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap',
                            }}
                          >
                            {conversation.lastMessage}
                          </p>
                        </div>
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.tertiary,
                          }}
                        >
                          {conversation.timestamp.toLocaleDateString()}
                        </div>
                      </div>
                    )
                  })}
              </div>
            )}
          </div>

          {/* Footer with hints */}
          <div
            style={{
              padding: `${designTokens.spacing[2]} ${designTokens.spacing[4]}`,
              borderTop: `1px solid ${designTokens.colors.border.subtle}`,
              backgroundColor: designTokens.colors.surface.secondary,
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.tertiary,
            }}
          >
            <div style={{ display: 'flex', gap: designTokens.spacing[4] }}>
              <span>
                <kbd
                  style={{
                    padding: '2px 4px',
                    borderRadius: '2px',
                    backgroundColor: designTokens.colors.surface.tertiary,
                  }}
                >
                  ↑↓
                </kbd>{' '}
                Navigate
              </span>
              <span>
                <kbd
                  style={{
                    padding: '2px 4px',
                    borderRadius: '2px',
                    backgroundColor: designTokens.colors.surface.tertiary,
                  }}
                >
                  Enter
                </kbd>{' '}
                Select
              </span>
              <span>
                <kbd
                  style={{
                    padding: '2px 4px',
                    borderRadius: '2px',
                    backgroundColor: designTokens.colors.surface.tertiary,
                  }}
                >
                  Tab
                </kbd>{' '}
                Switch tabs
              </span>
            </div>
            <span>
              <kbd
                style={{
                  padding: '2px 4px',
                  borderRadius: '2px',
                  backgroundColor: designTokens.colors.surface.tertiary,
                }}
              >
                Esc
              </kbd>{' '}
              Close
            </span>
          </div>
        </div>
      </div>
    </>
  )
}

export default CommandPalette
