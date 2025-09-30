import React, { useState, useCallback, useMemo, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import Button from '../ui/Button'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import Tooltip from '../ui/Tooltip'
import GenerationModal from '../generation/GenerationModal'
import { documentService } from '../../services/documentService'
import type { DocumentGeneration } from '../../types'

export interface OperationsToolbarProps {
  className?: string
  style?: React.CSSProperties
  documentId?: string
  selectedText?: string
  workspaceContext?: Record<string, unknown>
  onOperationStart?: (operation: OperationInfo) => void
  onOperationComplete?: (operation: OperationInfo, result: unknown) => void
  onOperationError?: (operation: OperationInfo, error: Error) => void
}

export interface OperationInfo {
  id: string
  type: OperationType
  label: string
  description: string
  icon: string
  requiresConfirmation: boolean
  estimatedTime?: string
  status?: 'idle' | 'running' | 'completed' | 'error'
  progress?: number
}

export type OperationType = 'analyze' | 'compare' | 'generate' | 'update' | 'search' | 'organize'

// Define available operations
const OPERATIONS: Record<OperationType, OperationInfo> = {
  analyze: {
    id: 'analyze',
    type: 'analyze',
    label: 'Analyze',
    description: 'Analyze document structure, content, and style',
    icon: 'search',
    requiresConfirmation: false,
    estimatedTime: '5-10s',
  },
  compare: {
    id: 'compare',
    type: 'compare',
    label: 'Compare',
    description: 'Compare this document with another',
    icon: 'git-compare',
    requiresConfirmation: true,
    estimatedTime: '10-15s',
  },
  generate: {
    id: 'generate',
    type: 'generate',
    label: 'Generate',
    description: 'Generate new content or documents',
    icon: 'file-plus',
    requiresConfirmation: true,
    estimatedTime: '30-60s',
  },
  update: {
    id: 'update',
    type: 'update',
    label: 'Update',
    description: 'Update document based on changes',
    icon: 'edit',
    requiresConfirmation: true,
    estimatedTime: '20-30s',
  },
  search: {
    id: 'search',
    type: 'search',
    label: 'Search',
    description: 'Search across documents',
    icon: 'search',
    requiresConfirmation: false,
    estimatedTime: '2-5s',
  },
  organize: {
    id: 'organize',
    type: 'organize',
    label: 'Organize',
    description: 'Organize and categorize content',
    icon: 'folder',
    requiresConfirmation: false,
    estimatedTime: '5-10s',
  },
}

const OperationsToolbar: React.FC<OperationsToolbarProps> = ({
  className = '',
  style,
  documentId,
  selectedText,
  workspaceContext,
  onOperationStart,
  onOperationComplete,
  onOperationError,
}) => {
  const [activeOperation, setActiveOperation] = useState<OperationInfo | null>(null)
  const [operationProgress, setOperationProgress] = useState(0)
  const [suggestedOperations, setSuggestedOperations] = useState<OperationType[]>([])
  const [contextualHints, setContextualHints] = useState<string[]>([])
  const [isGenerationModalOpen, setIsGenerationModalOpen] = useState(false)

  // Get context-aware operation suggestions
  const getOperationSuggestions = useCallback(async () => {
    try {
      // If we have a document, analyze it to suggest relevant operations
      if (documentId) {
        const docResult = await documentService.getDocument(documentId)

        if (docResult.success && docResult.data) {
          const suggestions: OperationType[] = []
          const hints: string[] = []

          // Always suggest analyze for new documents
          const isAnalyzed = docResult.data.metadata?.customFields?.analyzed as boolean | undefined
          if (!isAnalyzed) {
            suggestions.push('analyze')
            hints.push('Document not yet analyzed')
          }

          // Suggest compare if there are multiple documents
          if (workspaceContext?.documentCount && (workspaceContext.documentCount as number) > 1) {
            suggestions.push('compare')
            hints.push('Multiple documents available for comparison')
          }

          // Suggest generate for processed documents
          if (isAnalyzed) {
            suggestions.push('generate')
            hints.push('Generate outputs from this document')
          }

          setSuggestedOperations(suggestions)
          setContextualHints(hints)
        }
      } else {
        // No document context - suggest search and organize
        setSuggestedOperations(['search', 'organize'])
        setContextualHints(['Explore workspace content'])
      }
    } catch (error) {
      console.error('Failed to get operation suggestions:', error)
    }
  }, [documentId, workspaceContext])

  // Update suggestions when context changes
  useEffect(() => {
    getOperationSuggestions()
  }, [getOperationSuggestions])

  // Handle operation execution
  const executeOperation = useCallback(
    async (operation: OperationInfo) => {
      setActiveOperation({ ...operation, status: 'running', progress: 0 })
      setOperationProgress(0)
      onOperationStart?.(operation)

      try {
        let result: unknown = null

        // Simulate progress
        const progressInterval = setInterval(() => {
          setOperationProgress(prev => Math.min(prev + 10, 90))
        }, 500)

        switch (operation.type) {
          case 'analyze':
            if (documentId) {
              const analyzeResult = await documentService.analyzeDocument(documentId)
              result = analyzeResult.data
            }
            break

          case 'compare':
            // Compare operation requires document selection - emit event for UI handling
            result = { requiresDocumentSelection: true }
            break

          case 'generate':
            // Open generation modal instead of emitting event
            setIsGenerationModalOpen(true)
            result = { modalOpened: true }
            break

          case 'update':
            if (documentId) {
              // Update operation requires changes specification
              result = { requiresChangeSpecification: true }
            }
            break

          case 'search':
            // Search operation requires query - emit event for UI handling
            result = { requiresSearchQuery: true }
            break

          case 'organize':
            // Organization operation - trigger workspace organization
            result = { requiresOrganizationStrategy: true }
            break

          default:
            throw new Error(`Unknown operation type: ${operation.type}`)
        }

        clearInterval(progressInterval)
        setOperationProgress(100)

        setTimeout(() => {
          setActiveOperation({ ...operation, status: 'completed' })
          onOperationComplete?.(operation, result)

          // Clear active operation after animation
          setTimeout(() => {
            setActiveOperation(null)
            setOperationProgress(0)
          }, 1000)
        }, 300)
      } catch (error) {
        setActiveOperation({ ...operation, status: 'error' })
        onOperationError?.(operation, error as Error)

        // Clear after showing error
        setTimeout(() => {
          setActiveOperation(null)
          setOperationProgress(0)
        }, 3000)
      }
    },
    [documentId, onOperationStart, onOperationComplete, onOperationError]
  )

  // Handle operation button click
  const handleOperationClick = useCallback(
    (operationType: OperationType) => {
      const operation = OPERATIONS[operationType]

      if (operation.requiresConfirmation) {
        // For operations requiring confirmation, emit event for UI to show modal
        onOperationStart?.({ ...operation, status: 'idle' })
      } else {
        // Execute immediately
        executeOperation(operation)
      }
    },
    [executeOperation, onOperationStart]
  )

  // Cancel active operation
  const handleCancelOperation = useCallback(() => {
    setActiveOperation(null)
    setOperationProgress(0)
  }, [])

  // Handle generation modal close
  const handleGenerationModalClose = useCallback(() => {
    setIsGenerationModalOpen(false)
  }, [])

  // Handle generation complete
  const handleGenerationComplete = useCallback(
    (generation: DocumentGeneration) => {
      console.log('Generation complete:', generation)
      setIsGenerationModalOpen(false)
      // Notify parent component
      onOperationComplete?.(OPERATIONS.generate, generation)
    },
    [onOperationComplete]
  )

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      position: 'fixed' as const,
      bottom: 0,
      left: 0,
      right: 0,
      height: '64px',
      backgroundColor: designTokens.colors.surface.primary,
      borderTop: `1px solid ${designTokens.colors.border.medium}`,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      padding: `0 ${designTokens.spacing[4]}`,
      zIndex: designTokens.zIndex.docked,
      backdropFilter: 'blur(8px)',
      boxShadow: designTokens.shadows.lg,
      ...style,
    }),
    [style]
  )

  const operationsGroupStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
  }

  const progressContainerStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    padding: `${designTokens.spacing[2]} ${designTokens.spacing[4]}`,
    backgroundColor: designTokens.colors.surface.secondary,
    borderRadius: designTokens.borderRadius.md,
    border: `1px solid ${designTokens.colors.border.medium}`,
  }

  const progressBarContainerStyles = {
    width: '200px',
    height: '4px',
    backgroundColor: designTokens.colors.surface.tertiary,
    borderRadius: designTokens.borderRadius.full,
    overflow: 'hidden' as const,
  }

  const progressBarStyles = {
    height: '100%',
    backgroundColor:
      activeOperation?.status === 'error'
        ? designTokens.colors.accent.alert
        : activeOperation?.status === 'completed'
          ? designTokens.colors.confidence.high
          : designTokens.colors.accent.ai,
    transition: `width ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
    width: `${operationProgress}%`,
  }

  const hintStyles = {
    fontSize: designTokens.typography.fontSize.xs,
    color: designTokens.colors.text.tertiary,
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[1],
  }

  // Get operation button variant based on suggestion
  const getOperationVariant = useCallback(
    (operationType: OperationType): 'primary' | 'secondary' | 'ghost' => {
      if (suggestedOperations.includes(operationType)) {
        return 'primary'
      }
      return 'ghost'
    },
    [suggestedOperations]
  )

  return (
    <div className={`proxemic-operations-toolbar ${className}`} style={containerStyles}>
      {/* Left Section - Quick Actions */}
      <div style={operationsGroupStyles}>
        {(Object.keys(OPERATIONS) as OperationType[]).map(operationType => {
          const operation = OPERATIONS[operationType]
          const isSuggested = suggestedOperations.includes(operationType)

          return (
            <Tooltip key={operation.id} content={operation.description} placement="top">
              <div style={{ position: 'relative' }}>
                <Button
                  variant={getOperationVariant(operationType)}
                  size="sm"
                  onClick={() => handleOperationClick(operationType)}
                  disabled={!!activeOperation}
                  leftIcon={<Icon name={operation.icon as never} size={16} />}
                  style={{
                    position: 'relative',
                  }}
                >
                  {operation.label}
                </Button>
                {isSuggested && (
                  <div
                    style={{
                      position: 'absolute',
                      top: '-4px',
                      right: '-4px',
                      width: '8px',
                      height: '8px',
                      backgroundColor: designTokens.colors.accent.ai,
                      borderRadius: '50%',
                      border: `2px solid ${designTokens.colors.surface.primary}`,
                    }}
                  />
                )}
              </div>
            </Tooltip>
          )
        })}
      </div>

      {/* Center Section - Active Operation Progress */}
      {activeOperation && (
        <div style={progressContainerStyles}>
          <Icon
            name={
              activeOperation.status === 'error'
                ? 'AlertCircle'
                : activeOperation.status === 'completed'
                  ? 'Health'
                  : (activeOperation.icon as never)
            }
            size={16}
            color={
              activeOperation.status === 'error'
                ? designTokens.colors.accent.alert
                : activeOperation.status === 'completed'
                  ? designTokens.colors.confidence.high
                  : designTokens.colors.text.secondary
            }
          />
          <div>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                fontWeight: designTokens.typography.fontWeight.medium,
                color: designTokens.colors.text.primary,
              }}
            >
              {activeOperation.status === 'running'
                ? `${activeOperation.label}ing...`
                : activeOperation.status === 'error'
                  ? 'Operation failed'
                  : 'Operation completed'}
            </div>
            {activeOperation.status === 'running' && activeOperation.estimatedTime && (
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.tertiary,
                }}
              >
                Est. {activeOperation.estimatedTime}
              </div>
            )}
          </div>
          <div style={progressBarContainerStyles}>
            <div style={progressBarStyles} />
          </div>
          {activeOperation.status === 'running' && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCancelOperation}
              style={{ minWidth: 'auto', padding: designTokens.spacing[1] }}
            >
              <Icon name="X" size={14} />
            </Button>
          )}
        </div>
      )}

      {/* Right Section - Contextual Hints */}
      {!activeOperation && contextualHints.length > 0 && (
        <div style={hintStyles}>
          <Icon name="LightBulb" size={14} />
          <span>{contextualHints[0]}</span>
        </div>
      )}

      {/* Context indicator when text is selected */}
      {selectedText && !activeOperation && (
        <Badge variant="default" size="sm">
          <Icon
            name="FileText"
            size={12}
            style={{ marginRight: designTokens.spacing[1], display: 'inline-block' }}
          />
          {selectedText.length > 30 ? `${selectedText.substring(0, 30)}...` : selectedText}
        </Badge>
      )}

      {/* Generation Modal */}
      <GenerationModal
        isOpen={isGenerationModalOpen}
        onClose={handleGenerationModalClose}
        sourceDocumentId={documentId}
        onGenerationComplete={handleGenerationComplete}
      />
    </div>
  )
}

export default React.memo(OperationsToolbar)
