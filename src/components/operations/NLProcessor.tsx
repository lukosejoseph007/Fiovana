import React, { useState, useCallback, useMemo, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Button from '../ui/Button'
import Input from '../ui/Input'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import { nlOperationsService } from '../../services/nlOperationsService'

export interface NLProcessorProps {
  className?: string
  style?: React.CSSProperties
  onCommandExecute?: (command: ParsedCommand) => void
  placeholder?: string
  autoFocus?: boolean
}

interface ParsedCommand {
  intent: string
  confidence: number
  parameters: Record<string, unknown>
  suggestions: string[]
  requiresConfirmation: boolean
}

interface OperationType {
  id: string
  name: string
  description: string
  keywords: string[]
  icon: string
  parameters: ParameterDefinition[]
  examples: string[]
}

interface ParameterDefinition {
  name: string
  type: 'string' | 'number' | 'boolean' | 'document' | 'selection'
  required: boolean
  description: string
  options?: string[]
}

// Define operation types (6 operation types as per PRD)
const OPERATION_TYPES: OperationType[] = [
  {
    id: 'analyze',
    name: 'Analyze',
    description: 'Analyze document structure, content, or style',
    keywords: ['analyze', 'check', 'review', 'examine', 'inspect'],
    icon: 'search',
    parameters: [
      {
        name: 'document',
        type: 'document',
        required: true,
        description: 'Document to analyze',
      },
      {
        name: 'type',
        type: 'selection',
        required: false,
        description: 'Analysis type',
        options: ['structure', 'content', 'style', 'readability', 'sentiment'],
      },
    ],
    examples: [
      'Analyze this document',
      'Check the structure of document.pdf',
      'Review the style of the current document',
    ],
  },
  {
    id: 'compare',
    name: 'Compare',
    description: 'Compare two or more documents',
    keywords: ['compare', 'diff', 'difference', 'contrast', 'versus'],
    icon: 'git-compare',
    parameters: [
      {
        name: 'documents',
        type: 'document',
        required: true,
        description: 'Documents to compare',
      },
      {
        name: 'type',
        type: 'selection',
        required: false,
        description: 'Comparison type',
        options: ['textual', 'structural', 'semantic'],
      },
    ],
    examples: [
      'Compare doc1.pdf and doc2.pdf',
      'Show differences between these documents',
      'What changed between version 1 and version 2?',
    ],
  },
  {
    id: 'generate',
    name: 'Generate',
    description: 'Generate new content or documents',
    keywords: ['generate', 'create', 'make', 'produce', 'build'],
    icon: 'file-plus',
    parameters: [
      {
        name: 'type',
        type: 'selection',
        required: true,
        description: 'Content type to generate',
        options: ['summary', 'outline', 'questions', 'slides', 'document'],
      },
      {
        name: 'source',
        type: 'document',
        required: false,
        description: 'Source document',
      },
      {
        name: 'audience',
        type: 'selection',
        required: false,
        description: 'Target audience',
        options: ['instructors', 'students', 'technical', 'general'],
      },
    ],
    examples: [
      'Generate a summary of this document',
      'Create presentation slides from doc.pdf',
      'Make a student workbook from this content',
    ],
  },
  {
    id: 'update',
    name: 'Update',
    description: 'Update existing content based on changes',
    keywords: ['update', 'modify', 'change', 'revise', 'edit'],
    icon: 'edit',
    parameters: [
      {
        name: 'document',
        type: 'document',
        required: true,
        description: 'Document to update',
      },
      {
        name: 'changes',
        type: 'string',
        required: true,
        description: 'Changes to apply',
      },
      {
        name: 'preserveStyle',
        type: 'boolean',
        required: false,
        description: 'Preserve original style',
      },
    ],
    examples: [
      'Update the introduction section',
      'Change all references to new version',
      'Revise the document with these changes',
    ],
  },
  {
    id: 'search',
    name: 'Search',
    description: 'Search for content across documents',
    keywords: ['search', 'find', 'look for', 'locate', 'discover'],
    icon: 'search',
    parameters: [
      {
        name: 'query',
        type: 'string',
        required: true,
        description: 'Search query',
      },
      {
        name: 'scope',
        type: 'selection',
        required: false,
        description: 'Search scope',
        options: ['current', 'workspace', 'all'],
      },
      {
        name: 'semantic',
        type: 'boolean',
        required: false,
        description: 'Use semantic search',
      },
    ],
    examples: [
      'Find all documents about machine learning',
      'Search for troubleshooting procedures',
      'Locate content similar to this section',
    ],
  },
  {
    id: 'organize',
    name: 'Organize',
    description: 'Organize and categorize content',
    keywords: ['organize', 'categorize', 'group', 'sort', 'arrange'],
    icon: 'folder',
    parameters: [
      {
        name: 'documents',
        type: 'document',
        required: true,
        description: 'Documents to organize',
      },
      {
        name: 'method',
        type: 'selection',
        required: false,
        description: 'Organization method',
        options: ['topic', 'date', 'type', 'similarity'],
      },
    ],
    examples: [
      'Organize these documents by topic',
      'Group similar documents together',
      'Categorize all PDF files',
    ],
  },
]

const NLProcessor: React.FC<NLProcessorProps> = ({
  className = '',
  style,
  onCommandExecute,
  placeholder = 'Type a command... (e.g., "analyze this document")',
  autoFocus = false,
}) => {
  const [inputValue, setInputValue] = useState('')
  const [isProcessing, setIsProcessing] = useState(false)
  const [parsedCommand, setParsedCommand] = useState<ParsedCommand | null>(null)
  const [suggestions, setSuggestions] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)
  const [recentCommands, setRecentCommands] = useState<string[]>([])

  // Load recent commands from localStorage
  useEffect(() => {
    const stored = localStorage.getItem('proxemic_recent_commands')
    if (stored) {
      try {
        setRecentCommands(JSON.parse(stored))
      } catch {
        // Ignore parse errors
      }
    }
  }, [])

  // Save recent commands to localStorage
  const saveRecentCommand = useCallback((command: string) => {
    setRecentCommands(prev => {
      const updated = [command, ...prev.filter(c => c !== command)].slice(0, 10)
      localStorage.setItem('proxemic_recent_commands', JSON.stringify(updated))
      return updated
    })
  }, [])

  // Parse natural language input
  const parseCommand = useCallback(async (input: string): Promise<ParsedCommand> => {
    const normalizedInput = input.toLowerCase().trim()

    // Find matching operation type
    let matchedOperation: OperationType | null = null
    let maxKeywordMatches = 0

    for (const op of OPERATION_TYPES) {
      const matches = op.keywords.filter(keyword => normalizedInput.includes(keyword)).length
      if (matches > maxKeywordMatches) {
        maxKeywordMatches = matches
        matchedOperation = op
      }
    }

    // If we found a match, classify intent more precisely
    if (matchedOperation) {
      try {
        const classification = await nlOperationsService.classifyIntent(
          input,
          OPERATION_TYPES.map(op => op.id)
        )

        if (classification.success && classification.data) {
          const intentData = classification.data as { intent: string; confidence: number }
          const confidence = intentData.confidence || 0.7

          // Extract parameters from input
          const parameters: Record<string, unknown> = {}

          // Simple parameter extraction logic
          if (matchedOperation.id === 'analyze') {
            if (normalizedInput.includes('structure')) parameters.type = 'structure'
            else if (normalizedInput.includes('style')) parameters.type = 'style'
            else if (normalizedInput.includes('readability')) parameters.type = 'readability'
          } else if (matchedOperation.id === 'generate') {
            if (normalizedInput.includes('summary')) parameters.type = 'summary'
            else if (normalizedInput.includes('slides')) parameters.type = 'slides'
            else if (normalizedInput.includes('outline')) parameters.type = 'outline'
          }

          // Generate suggestions
          const commandSuggestions = matchedOperation.examples.filter(
            ex => !ex.toLowerCase().includes(normalizedInput)
          )

          return {
            intent: matchedOperation.id,
            confidence,
            parameters,
            suggestions: commandSuggestions.slice(0, 3),
            requiresConfirmation: confidence < 0.8,
          }
        }
      } catch {
        // Fall through to default parsing
      }
    }

    // Default parsing if no match or API call failed
    return {
      intent: matchedOperation?.id || 'unknown',
      confidence: matchedOperation ? 0.6 : 0.3,
      parameters: {},
      suggestions: OPERATION_TYPES.slice(0, 3).flatMap(op => op.examples.slice(0, 1)),
      requiresConfirmation: true,
    }
  }, [])

  // Handle input change with auto-suggestions
  const handleInputChange = useCallback(async (value: string) => {
    setInputValue(value)
    setError(null)

    if (value.trim().length > 3) {
      // Generate suggestions as user types
      const normalizedValue = value.toLowerCase()
      const matchingSuggestions: string[] = []

      for (const op of OPERATION_TYPES) {
        for (const example of op.examples) {
          if (example.toLowerCase().includes(normalizedValue) && matchingSuggestions.length < 5) {
            matchingSuggestions.push(example)
          }
        }
      }

      setSuggestions(matchingSuggestions)
    } else {
      setSuggestions([])
    }
  }, [])

  // Handle command submission
  const handleSubmit = useCallback(
    async (e?: React.FormEvent) => {
      e?.preventDefault()

      if (!inputValue.trim()) return

      setIsProcessing(true)
      setError(null)

      try {
        const parsed = await parseCommand(inputValue)
        setParsedCommand(parsed)
        saveRecentCommand(inputValue)

        // Auto-execute high-confidence commands
        if (parsed.confidence >= 0.8 && !parsed.requiresConfirmation) {
          onCommandExecute?.(parsed)
          setInputValue('')
          setParsedCommand(null)
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to parse command')
      } finally {
        setIsProcessing(false)
      }
    },
    [inputValue, parseCommand, saveRecentCommand, onCommandExecute]
  )

  // Handle command confirmation
  const handleConfirm = useCallback(() => {
    if (parsedCommand) {
      onCommandExecute?.(parsedCommand)
      setInputValue('')
      setParsedCommand(null)
      setSuggestions([])
    }
  }, [parsedCommand, onCommandExecute])

  // Handle command cancellation
  const handleCancel = useCallback(() => {
    setParsedCommand(null)
  }, [])

  // Handle suggestion click
  const handleSuggestionClick = useCallback((suggestion: string) => {
    setInputValue(suggestion)
    setSuggestions([])
  }, [])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column' as const,
      gap: designTokens.spacing[3],
      ...style,
    }),
    [style]
  )

  const inputContainerStyles = {
    position: 'relative' as const,
  }

  const suggestionsContainerStyles = {
    position: 'absolute' as const,
    top: '100%',
    left: 0,
    right: 0,
    marginTop: designTokens.spacing[1],
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.medium}`,
    borderRadius: designTokens.borderRadius.md,
    boxShadow: designTokens.shadows.lg,
    maxHeight: '240px',
    overflowY: 'auto' as const,
    zIndex: designTokens.zIndex.dropdown,
  }

  const suggestionItemStyles = {
    padding: `${designTokens.spacing[2]} ${designTokens.spacing[3]}`,
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    cursor: 'pointer',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
  }

  const confirmationCardStyles = {
    padding: designTokens.spacing[4],
    marginTop: designTokens.spacing[2],
  }

  const getOperationIcon = (intent: string): string => {
    const operation = OPERATION_TYPES.find(op => op.id === intent)
    return operation?.icon || 'help-circle'
  }

  const getConfidenceColor = (confidence: number): string => {
    if (confidence >= 0.8) return designTokens.colors.confidence.high
    if (confidence >= 0.6) return designTokens.colors.confidence.medium
    return designTokens.colors.confidence.low
  }

  return (
    <div className={`proxemic-nl-processor ${className}`} style={containerStyles}>
      {/* Input Form */}
      <form onSubmit={handleSubmit}>
        <div style={inputContainerStyles}>
          <Input
            value={inputValue}
            onChange={e => handleInputChange(e.target.value)}
            placeholder={placeholder}
            disabled={isProcessing}
            autoFocus={autoFocus}
            fullWidth
            rightIcon={
              <button
                type="submit"
                style={{
                  background: 'none',
                  border: 'none',
                  cursor: isProcessing ? 'default' : 'pointer',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  color: 'inherit',
                }}
                disabled={isProcessing}
              >
                {isProcessing ? (
                  <Icon name="Loader" size={16} className="animate-spin" />
                ) : (
                  <Icon name="Send" size={16} />
                )}
              </button>
            }
          />

          {/* Auto-complete Suggestions */}
          {suggestions.length > 0 && !parsedCommand && (
            <div style={suggestionsContainerStyles}>
              {suggestions.map((suggestion, index) => (
                <div
                  key={index}
                  style={suggestionItemStyles}
                  onClick={() => handleSuggestionClick(suggestion)}
                  className="suggestion-item"
                >
                  <Icon name="Zap" size={14} style={{ marginRight: designTokens.spacing[2] }} />
                  {suggestion}
                </div>
              ))}
            </div>
          )}
        </div>
      </form>

      {/* Command Validation/Confirmation */}
      {parsedCommand && (
        <Card variant="elevated" style={confirmationCardStyles}>
          <div
            style={{
              display: 'flex',
              alignItems: 'flex-start',
              justifyContent: 'space-between',
              marginBottom: designTokens.spacing[3],
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
              <Icon name={getOperationIcon(parsedCommand.intent) as never} size={20} />
              <div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.base,
                    fontWeight: designTokens.typography.fontWeight.semibold,
                    color: designTokens.colors.text.primary,
                    textTransform: 'capitalize',
                  }}
                >
                  {parsedCommand.intent}
                </div>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  {OPERATION_TYPES.find(op => op.id === parsedCommand.intent)?.description ||
                    'Operation'}
                </div>
              </div>
            </div>
            <Badge
              variant="default"
              size="sm"
              style={{
                color: getConfidenceColor(parsedCommand.confidence),
                borderColor: getConfidenceColor(parsedCommand.confidence),
              }}
            >
              {Math.round(parsedCommand.confidence * 100)}% confident
            </Badge>
          </div>

          {/* Parameters */}
          {Object.keys(parsedCommand.parameters).length > 0 && (
            <div style={{ marginBottom: designTokens.spacing[3] }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  color: designTokens.colors.text.primary,
                  marginBottom: designTokens.spacing[2],
                }}
              >
                Parameters:
              </div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: designTokens.spacing[2] }}>
                {Object.entries(parsedCommand.parameters).map(([key, value]) => (
                  <Badge key={key} variant="default" size="sm">
                    {key}: {String(value)}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Action Buttons */}
          <div
            style={{ display: 'flex', gap: designTokens.spacing[2], justifyContent: 'flex-end' }}
          >
            <Button variant="secondary" size="sm" onClick={handleCancel}>
              Cancel
            </Button>
            <Button variant="primary" size="sm" onClick={handleConfirm}>
              {parsedCommand.requiresConfirmation ? 'Confirm & Execute' : 'Execute'}
            </Button>
          </div>
        </Card>
      )}

      {/* Error Display */}
      {error && (
        <Card variant="elevated" style={{ padding: designTokens.spacing[3] }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
            <Icon name="AlertCircle" size={20} color={designTokens.colors.accent.alert} />
            <span
              style={{
                color: designTokens.colors.accent.alert,
                fontSize: designTokens.typography.fontSize.sm,
              }}
            >
              {error}
            </span>
          </div>
        </Card>
      )}

      {/* Recent Commands */}
      {recentCommands.length > 0 && !inputValue && (
        <div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              fontWeight: designTokens.typography.fontWeight.medium,
              color: designTokens.colors.text.secondary,
              marginBottom: designTokens.spacing[2],
            }}
          >
            Recent Commands
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: designTokens.spacing[2] }}>
            {recentCommands.slice(0, 3).map((cmd, index) => (
              <Button
                key={index}
                variant="ghost"
                size="sm"
                onClick={() => setInputValue(cmd)}
                style={{ fontSize: designTokens.typography.fontSize.xs }}
              >
                {cmd}
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* Inline Styles */}
      <style>
        {`
          .animate-spin {
            animation: spin 1s linear infinite;
          }

          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }

          .suggestion-item:hover {
            background-color: ${designTokens.colors.state.hover};
            color: ${designTokens.colors.text.primary};
          }

          .suggestion-item:last-child {
            border-bottom: none;
          }
        `}
      </style>
    </div>
  )
}

export default React.memo(NLProcessor)
