import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import { Card, Button, Icon, Badge, Tooltip, Dropdown } from '../ui'
import {
  DocumentSkeleton,
  LongOperationProgress,
  OperationProgressTracker,
  type OperationProgress,
} from '../ui/LoadingStates'
import {
  documentService,
  documentEditingService,
  structureService,
  contentClassificationService,
} from '../../services'
import { designTokens } from '../../styles/tokens'
import { Document, DocumentStructure, ContentClassification } from '../../types'
import DocumentRenderer from './DocumentRenderer'
import DocumentEditor from '../editor/DocumentEditor'
import VersionHistory from '../editor/VersionHistory'
import { ActiveUsers } from '../collaboration/ActiveUsers'
import { UserPresence } from '../collaboration/UserPresence'
import { LiveCursors } from '../collaboration/LiveCursors'
import { ConflictResolution } from '../collaboration/ConflictResolution'
import { OfflineIndicator } from '../ui/OfflineIndicator'
import { useDocumentState } from '../../hooks/useDocumentState'
import { useAutoSave } from '../../hooks/useAutoSave'
import { useCollaboration } from '../../context/useCollaboration'
import { useTypingIndicator } from '../../hooks/useTypingIndicator'
import { useConflictResolution } from '../../hooks/useConflictResolution'
import { useOfflineSync } from '../../hooks/useOfflineSync'
import '../../styles/editor.css'

interface DocumentViewerProps {
  documentId: string
  _workspaceId?: string
  onClose?: () => void
  onAISuggestionSelect?: (suggestion: AISuggestion) => void
}

interface AISuggestion {
  id: string
  type: 'improvement' | 'clarification' | 'expansion' | 'simplification'
  position: { start: number; end: number }
  content: string
  confidence: number
  reasoning: string
}

// Note: DocumentSection interface reserved for future use
// interface DocumentSection {
//   id: string
//   title: string
//   content: string
//   level: number
//   position: { start: number; end: number }
//   type: 'heading' | 'paragraph' | 'list' | 'table' | 'image'
// }

// Note: SemanticHighlight temporarily unused - will be re-enabled with React-based overlays
// eslint-disable-next-line @typescript-eslint/no-unused-vars
interface SemanticHighlight {
  id: string
  start: number
  end: number
  type: 'concept' | 'procedure' | 'definition' | 'reference'
  confidence: number
  metadata: Record<string, unknown>
}

const DocumentViewer: React.FC<DocumentViewerProps> = ({
  documentId,
  _workspaceId,
  onClose,
  onAISuggestionSelect,
}) => {
  const [document, setDocument] = useState<Document | null>(null)
  const [structure, setStructure] = useState<DocumentStructure | null>(null)
  const [classification, setClassification] = useState<ContentClassification | null>(null)
  const [aiSuggestions, setAiSuggestions] = useState<AISuggestion[]>([])
  // Note: semanticHighlights temporarily disabled - will be re-implemented with React-based overlays
  // const [semanticHighlights, setSemanticHighlights] = useState<SemanticHighlight[]>([])
  const [selectedText, setSelectedText] = useState<string>('')
  const [selectionPosition, setSelectionPosition] = useState<{ x: number; y: number } | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [loadingOperations, setLoadingOperations] = useState<OperationProgress[]>([])
  // Note: hoveredHighlight temporarily disabled - will be re-implemented with React-based overlays
  // const [hoveredHighlight, setHoveredHighlight] = useState<string | null>(null)
  const [showIntelligenceBar, setShowIntelligenceBar] = useState(false)
  const [isEditMode, setIsEditMode] = useState(false)
  const [showUnsavedWarning, setShowUnsavedWarning] = useState(false)
  const [showVersionHistory, setShowVersionHistory] = useState(false)
  const [showPresencePanel, setShowPresencePanel] = useState(false)
  const contentRef = useRef<HTMLDivElement>(null)
  const intelligenceBarRef = useRef<HTMLDivElement>(null)

  // Collaboration context
  const collaboration = useCollaboration()

  // Typing indicator hook (will be integrated with DocumentEditor in future task)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { isTyping, handleTyping } = useTypingIndicator({
    onTypingChange: typing => {
      // Update typing state for current user in collaboration context
      collaboration?.updateUserTyping(collaboration.currentUserId, typing)
    },
  })

  // Conflict resolution hook for collaborative editing
  const {
    conflicts,
    resolveConflict,
    dismissConflict,
    refreshConflicts,
  } = useConflictResolution({
    ydoc: null, // Will be connected to Yjs document when real-time editing is enabled
    enabled: collaboration?.settings.enabled && isEditMode,
    onConflictDetected: conflict => {
      console.log('Conflict detected:', conflict)
    },
    onConflictResolved: (conflictId, resolution) => {
      console.log('Conflict resolved:', conflictId, resolution)
    },
  })

  // Offline sync hook for queuing operations when offline
  const {
    isSyncing,
    queuedOperations,
    syncProgress,
    triggerSync,
  } = useOfflineSync({
    ydoc: null, // Will be connected to Yjs document when real-time editing is enabled
    enabled: collaboration?.settings.enabled && isEditMode,
    autoSync: true,
    maxRetries: 3,
    onSyncStart: () => {
      console.log('Sync started')
    },
    onSyncComplete: (success, syncedCount) => {
      console.log('Sync completed:', success, 'Synced:', syncedCount)
      // After sync, check for conflicts
      if (success) {
        refreshConflicts()
      }
    },
    onOperationQueued: operation => {
      console.log('Operation queued:', operation)
    },
  })

  // Convert Map to Array for component props
  const collaborationUsers = useMemo(() => {
    return Array.from(collaboration?.users.values() || []).map(user => ({
      ...user,
      isActive: Date.now() - user.lastSeen < 30000, // Active if seen in last 30 seconds
    }))
  }, [collaboration?.users])

  // Get cursor positions for LiveCursors
  const cursorPositions = useMemo(() => {
    return collaborationUsers
      .filter(user => user.cursor && user.id !== collaboration?.currentUserId)
      .map(user => ({
        userId: user.id,
        userName: user.name,
        userColor: user.color,
        x: user.cursor!.x,
        y: user.cursor!.y,
        lastUpdate: user.lastSeen,
      }))
  }, [collaborationUsers, collaboration?.currentUserId])

  // Document state management with auto-save
  const {
    content: editedContent,
    isDirty,
    isSaving,
    lastSaved,
    error: saveError,
    updateContent,
    save: saveDocument,
    initialize: initializeDocumentState,
  } = useDocumentState(document?.content || '', {
    onSave: async (content: string) => {
      console.log('Saving document:', documentId, 'Content length:', content.length)

      try {
        // Determine document format from file extension or type
        const fileExtension = document?.path?.split('.').pop()?.toLowerCase()
        let format: 'markdown' | 'plainText' | 'html' = 'markdown'

        if (fileExtension === 'md' || fileExtension === 'markdown') {
          format = 'markdown'
        } else if (fileExtension === 'html' || fileExtension === 'htm') {
          format = 'html'
        } else {
          format = 'plainText'
        }

        // Save document using backend service
        const response = await documentEditingService.saveDocument(documentId, content, format)

        if (!response.success) {
          throw new Error(response.error || 'Failed to save document')
        }

        console.log('Document saved successfully:', response.data)

        // Create a version snapshot after successful save
        try {
          await documentEditingService.createDocumentVersion(documentId, content)
          console.log('Version snapshot created')
        } catch (versionError) {
          // Don't fail the save if version creation fails
          console.warn('Failed to create version snapshot:', versionError)
        }

        // Reload the document to get the latest saved content from backend
        // This ensures the viewer shows the latest version
        // Skip loading state to avoid showing loading UI during save
        await loadDocument(true)
      } catch (error) {
        console.error('Save failed:', error)
        throw error
      }
    },
    onDirtyChange: (dirty: boolean) => {
      setShowUnsavedWarning(dirty)
    },
  })

  // Auto-save functionality (5 second delay)
  useAutoSave(editedContent, isDirty, {
    enabled: isEditMode,
    delay: 5000,
    onAutoSave: saveDocument,
    onError: (error: Error) => {
      console.error('Auto-save failed:', error)
    },
  })

  // Generate AI suggestions for the document
  const generateAISuggestions = useCallback(async () => {
    if (!documentId) return

    try {
      // Mock AI suggestions - in reality, this would call the AI service
      const suggestions: AISuggestion[] = [
        {
          id: 'suggestion-1',
          type: 'improvement',
          position: { start: 120, end: 180 },
          content: 'Consider adding more specific examples to illustrate this concept.',
          confidence: 0.85,
          reasoning:
            'This section introduces a complex concept but lacks concrete examples that would help readers understand better.',
        },
        {
          id: 'suggestion-2',
          type: 'clarification',
          position: { start: 350, end: 420 },
          content: 'This technical term might benefit from a brief definition.',
          confidence: 0.72,
          reasoning: 'Technical jargon detected that may not be familiar to all audience levels.',
        },
        {
          id: 'suggestion-3',
          type: 'simplification',
          position: { start: 680, end: 750 },
          content: 'This sentence could be broken into smaller, more digestible parts.',
          confidence: 0.78,
          reasoning:
            'Sentence complexity analysis indicates this may be difficult to parse for some readers.',
        },
      ]

      setAiSuggestions(suggestions)
    } catch (error) {
      console.error('Failed to generate AI suggestions:', error)
    }
  }, [documentId])

  // Note: Semantic highlights temporarily disabled - will be re-implemented with React-based overlays
  // const generateSemanticHighlights = useCallback(async () => {
  //   if (!documentId) return
  //   try {
  //     const highlights: SemanticHighlight[] = [
  //       {
  //         id: 'highlight-1',
  //         start: 50,
  //         end: 120,
  //         type: 'concept',
  //         confidence: 0.9,
  //         metadata: { conceptType: 'definition', importance: 'high' },
  //       },
  //     ]
  //     setSemanticHighlights(highlights)
  //   } catch (error) {
  //     console.error('Failed to generate semantic highlights:', error)
  //   }
  // }, [documentId])

  // Helper to update operation status
  const updateOperation = useCallback((id: string, updates: Partial<OperationProgress>) => {
    setLoadingOperations(prev => prev.map(op => (op.id === id ? { ...op, ...updates } : op)))
  }, [])

  // Load document data
  const loadDocument = useCallback(async (skipLoadingState = false) => {
    if (!documentId) return

    // Only show loading state if not skipped (e.g., during save reload)
    if (!skipLoadingState) {
      setIsLoading(true)

      // Initialize operations
      const operations: OperationProgress[] = [
        {
          id: 'load-document',
          operation: 'Loading document metadata',
          status: 'pending',
          progress: 0,
        },
        {
          id: 'load-structure',
          operation: 'Analyzing document structure',
          status: 'pending',
          progress: 0,
        },
        {
          id: 'load-classification',
          operation: 'Classifying content type',
          status: 'pending',
          progress: 0,
        },
        {
          id: 'generate-suggestions',
          operation: 'Generating AI suggestions',
          status: 'pending',
          progress: 0,
        },
        {
          id: 'generate-highlights',
          operation: 'Analyzing semantic content',
          status: 'pending',
          progress: 0,
        },
      ]
      setLoadingOperations(operations)
    }

    try {
      // Load document content
      if (!skipLoadingState) {
        updateOperation('load-document', { status: 'in-progress', progress: 10 })
      }
      console.log('Loading document with ID:', documentId)
      const docResponse = await documentService.getDocument(documentId)
      console.log('Document response:', docResponse)

      if (docResponse.success && docResponse.data) {
        console.log('Document loaded successfully:', docResponse.data)
        setDocument(docResponse.data)
        if (!skipLoadingState) {
          updateOperation('load-document', { status: 'completed', progress: 100 })
        }
      } else {
        console.error('Failed to load document:', docResponse.error)
        if (!skipLoadingState) {
          updateOperation('load-document', {
            status: 'failed',
            details: docResponse.error || 'Failed to load document',
          })
        }
      }

      // Load document structure - use the file path from the loaded document
      if (docResponse.data?.path) {
        if (!skipLoadingState) {
          updateOperation('load-structure', { status: 'in-progress', progress: 20 })
        }
        const structureResponse = await structureService.analyzeDocumentStructure(
          docResponse.data.path
        )
        if (structureResponse.success && structureResponse.data) {
          setStructure(structureResponse.data)
          if (!skipLoadingState) {
            updateOperation('load-structure', { status: 'completed', progress: 100 })
          }
        } else {
          if (!skipLoadingState) {
            updateOperation('load-structure', { status: 'failed', details: 'Analysis failed' })
          }
        }

        // Load content classification - use the file path
        if (!skipLoadingState) {
          updateOperation('load-classification', { status: 'in-progress', progress: 30 })
        }
        const classificationResponse = await contentClassificationService.classifyContentType(
          docResponse.data.path
        )
        if (classificationResponse.success && classificationResponse.data) {
          setClassification(classificationResponse.data)
          if (!skipLoadingState) {
            updateOperation('load-classification', { status: 'completed', progress: 100 })
          }
        } else {
          if (!skipLoadingState) {
            updateOperation('load-classification', {
              status: 'failed',
              details: 'Classification failed',
            })
          }
        }
      }

      // Generate AI suggestions (mock for now)
      if (!skipLoadingState) {
        updateOperation('generate-suggestions', { status: 'in-progress', progress: 40 })
      }
      await generateAISuggestions()
      if (!skipLoadingState) {
        updateOperation('generate-suggestions', { status: 'completed', progress: 100 })
      }

      // Note: Semantic highlights temporarily disabled
      // updateOperation('generate-highlights', { status: 'in-progress', progress: 50 })
      // await generateSemanticHighlights()
      if (!skipLoadingState) {
        updateOperation('generate-highlights', { status: 'completed', progress: 100 })
      }
    } catch (error) {
      console.error('Failed to load document:', error)
      // Mark all in-progress operations as failed (only if not skipped)
      if (!skipLoadingState) {
        setLoadingOperations(prev =>
          prev.map(op =>
            op.status === 'in-progress' || op.status === 'pending'
              ? { ...op, status: 'failed', details: 'Operation failed' }
              : op
          )
        )
      }
    } finally {
      if (!skipLoadingState) {
        setIsLoading(false)
      }
    }
  }, [documentId, generateAISuggestions, updateOperation])

  // Handle text selection
  const handleTextSelection = useCallback(() => {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) {
      setSelectedText('')
      setSelectionPosition(null)
      setShowIntelligenceBar(false)
      return
    }

    const range = selection.getRangeAt(0)
    const text = range.toString().trim()

    if (text.length > 0) {
      setSelectedText(text)

      // Get selection position for floating intelligence bar
      const rect = range.getBoundingClientRect()
      setSelectionPosition({
        x: rect.left + rect.width / 2,
        y: rect.top - 10,
      })
      setShowIntelligenceBar(true)
    } else {
      setSelectedText('')
      setSelectionPosition(null)
      setShowIntelligenceBar(false)
    }
  }, [])

  // Handle AI suggestion click
  const handleAISuggestionClick = useCallback(
    (suggestion: AISuggestion) => {
      onAISuggestionSelect?.(suggestion)
    },
    [onAISuggestionSelect]
  )

  // Note: Highlight hover functionality temporarily disabled
  // Will be re-implemented with React-based overlay system
  // const handleHighlightHover = useCallback((highlightId: string | null) => {
  //   setHoveredHighlight(highlightId)
  // }, [])

  // const getHighlightColor = useCallback((type: string, isHovered: boolean) => {
  //   const opacity = isHovered ? '40' : '20'
  //   switch (type) {
  //     case 'concept':
  //       return `${designTokens.colors.accent.ai}${opacity}`
  //     case 'procedure':
  //       return `${designTokens.colors.accent.success}${opacity}`
  //     case 'definition':
  //       return `${designTokens.colors.accent.warning}${opacity}`
  //     case 'reference':
  //       return `${designTokens.colors.accent.info}${opacity}`
  //     default:
  //       return `${designTokens.colors.text.secondary}${opacity}`
  //   }
  // }, [])

  // Note: Semantic highlights and AI suggestions are temporarily disabled
  // in favor of clean markdown rendering. They will be re-implemented
  // using React-based overlays in a future update.

  // Calculate confidence score
  const confidenceScore = useMemo(() => {
    if (!document || !classification) return 0

    // Mock confidence calculation based on various factors
    const factors = [
      classification.confidence || 0.8,
      structure ? 0.9 : 0.6,
      document.metadata?.wordCount ? Math.min(document.metadata.wordCount / 1000, 1) : 0.5,
    ]

    return factors.reduce((acc, factor) => acc + factor, 0) / factors.length
  }, [document, classification, structure])

  // Load data on mount
  useEffect(() => {
    loadDocument()
  }, [loadDocument])

  // Initialize document state when document loads
  useEffect(() => {
    if (document?.content) {
      initializeDocumentState(document.content)
    }
  }, [document?.content, initializeDocumentState])

  // Add event listeners for text selection
  useEffect(() => {
    const doc = window.document
    doc.addEventListener('mouseup', handleTextSelection)
    doc.addEventListener('keyup', handleTextSelection)

    return () => {
      doc.removeEventListener('mouseup', handleTextSelection)
      doc.removeEventListener('keyup', handleTextSelection)
    }
  }, [handleTextSelection])

  // Note: Highlight and suggestion click handlers temporarily disabled
  // Will be re-implemented with React-based overlay system
  useEffect(() => {
    const handleContentClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement

      // Handle AI suggestion click (when re-enabled)
      const suggestionId = target.getAttribute('data-suggestion-id')
      if (suggestionId) {
        const suggestion = aiSuggestions.find(s => s.id === suggestionId)
        if (suggestion) {
          handleAISuggestionClick(suggestion)
        }
      }
    }

    const contentElement = contentRef.current
    if (contentElement) {
      contentElement.addEventListener('click', handleContentClick)

      return () => {
        contentElement.removeEventListener('click', handleContentClick)
      }
    }

    return undefined
  }, [aiSuggestions, handleAISuggestionClick])

  if (isLoading) {
    return (
      <div
        style={{
          height: '100%',
          background: designTokens.colors.background.canvas,
          padding: designTokens.spacing[6],
          overflowY: 'auto',
          display: 'flex',
          flexDirection: 'column',
          gap: designTokens.spacing[6],
        }}
      >
        {/* Loading progress tracker */}
        <div
          style={{
            maxWidth: '600px',
            margin: '0 auto',
            width: '100%',
          }}
        >
          <LongOperationProgress
            operation="Loading Document"
            details={
              loadingOperations.find(op => op.status === 'in-progress')?.operation ||
              'Preparing to load...'
            }
            variant="ai"
          />

          <div style={{ marginTop: designTokens.spacing[6] }}>
            <OperationProgressTracker operations={loadingOperations} maxVisible={5} />
          </div>
        </div>

        {/* Document skeleton (shown below progress) */}
        <DocumentSkeleton />
      </div>
    )
  }

  if (!document) {
    return (
      <div
        style={{
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: designTokens.colors.background.canvas,
        }}
      >
        <div
          style={{
            textAlign: 'center',
            color: designTokens.colors.text.secondary,
          }}
        >
          <div style={{ marginBottom: designTokens.spacing[4] }}>
            <Icon name="Document" size="3xl" />
          </div>
          <div style={{ fontSize: designTokens.typography.fontSize.lg }}>Document not found</div>
        </div>
      </div>
    )
  }

  return (
    <div
      style={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        background: designTokens.colors.background.canvas,
        position: 'relative',
      }}
    >
      {/* Document Header Strip */}
      <div
        style={{
          minHeight: '48px',
          padding: `${designTokens.spacing[3]} ${designTokens.spacing[6]}`,
          background: 'linear-gradient(to bottom, rgba(26, 26, 30, 0.95), rgba(22, 22, 26, 0.98))',
          backdropFilter: 'blur(10px)',
          borderBottom: `1px solid rgba(255, 255, 255, 0.08)`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          boxShadow: '0 1px 3px rgba(0, 0, 0, 0.3)',
        }}
      >
        {/* Document Info */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[3],
            flex: 1,
            minWidth: 0, // Allow flex item to shrink
          }}
        >
          <Icon
            name="Document"
            size={18}
            style={{ color: designTokens.colors.accent.ai, flexShrink: 0 }}
          />
          <span
            style={{
              fontSize: designTokens.typography.fontSize.base,
              fontWeight: designTokens.typography.fontWeight.semibold,
              color: designTokens.colors.text.primary,
              maxWidth: '400px',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {document.name}
          </span>

          <Badge
            variant="confidence"
            style={{
              background: `${designTokens.colors.accent.ai}15`,
              color: designTokens.colors.accent.ai,
              fontSize: designTokens.typography.fontSize.xs,
              padding: '4px 10px',
              borderRadius: '6px',
              border: `1px solid ${designTokens.colors.accent.ai}30`,
              flexShrink: 0,
            }}
          >
            {Math.round(confidenceScore * 100)}% confidence
          </Badge>

          {classification && (
            <Badge
              variant="default"
              style={{
                fontSize: designTokens.typography.fontSize.xs,
                padding: '4px 10px',
                borderRadius: '6px',
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                color: designTokens.colors.text.secondary,
                flexShrink: 0,
              }}
            >
              {classification.categories?.[0]?.name || 'Unknown'}
            </Badge>
          )}

          {/* Active Users Badge */}
          {collaboration?.settings.enabled && collaboration.settings.showPresence && (
            <ActiveUsers
              users={collaborationUsers}
              currentUserId={collaboration.currentUserId}
              size="small"
              onClick={() => setShowPresencePanel(!showPresencePanel)}
            />
          )}

          {/* Offline Indicator */}
          {collaboration?.settings.enabled && isEditMode && (
            <OfflineIndicator
              position="inline"
              showDetails={false}
              showSyncProgress={queuedOperations.length > 0}
              syncProgress={syncProgress}
              isSyncing={isSyncing}
              onManualSync={triggerSync}
            />
          )}
        </div>

        {/* Actions */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[2],
            flexShrink: 0,
          }}
        >
          {/* Save Button (shown in edit mode) */}
          {isEditMode && (
            <>
              <Tooltip
                content={
                  isDirty ? 'Save changes (auto-saves every 5 seconds)' : 'No unsaved changes'
                }
              >
                <Button
                  variant={isDirty ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={saveDocument}
                  disabled={!isDirty || isSaving}
                  style={{
                    fontWeight: 600,
                    padding: '8px 16px',
                    borderRadius: '8px',
                    ...(isDirty
                      ? {
                          background: 'linear-gradient(135deg, #51cf66 0%, #40c057 100%)',
                          boxShadow: '0 2px 8px rgba(64, 192, 87, 0.3)',
                        }
                      : {}),
                  }}
                >
                  <Icon name={isSaving ? 'Spinner' : 'Document'} size={16} />
                  {isSaving ? 'Saving...' : isDirty ? 'Save' : 'Saved'}
                </Button>
              </Tooltip>

              {/* Save Status Indicator */}
              {lastSaved && !isDirty && (
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                    whiteSpace: 'nowrap',
                  }}
                >
                  Saved {new Date(lastSaved).toLocaleTimeString()}
                </span>
              )}

              {/* Error Indicator */}
              {saveError && (
                <Tooltip content={saveError}>
                  <Badge
                    variant="error"
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      padding: '4px 8px',
                    }}
                  >
                    <Icon name="X" size={12} />
                    Save failed
                  </Badge>
                </Tooltip>
              )}

              <div
                style={{
                  width: '1px',
                  height: '20px',
                  background: 'rgba(255, 255, 255, 0.1)',
                  margin: `0 ${designTokens.spacing[1]}`,
                }}
              />
            </>
          )}

          <Tooltip content={isEditMode ? 'Switch to View Mode' : 'Switch to Edit Mode'}>
            <Button
              variant={isEditMode ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => {
                if (isEditMode && isDirty && showUnsavedWarning) {
                  // Warn user about unsaved changes
                  const confirmSwitch = window.confirm(
                    'You have unsaved changes. Switch to view mode anyway? (Changes will be lost)'
                  )
                  if (!confirmSwitch) return
                }
                setIsEditMode(!isEditMode)
              }}
              style={{
                fontWeight: 600,
                padding: '8px 16px',
                borderRadius: '8px',
                ...(isEditMode
                  ? {
                      background: 'linear-gradient(135deg, #4c6ef5 0%, #5c7cff 100%)',
                      boxShadow: '0 2px 8px rgba(76, 110, 245, 0.3)',
                    }
                  : {}),
              }}
            >
              <Icon name={isEditMode ? 'Document' : 'Generate'} size={16} />
              {isEditMode ? 'View' : 'Edit'}
            </Button>
          </Tooltip>

          <Tooltip content="Version History">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowVersionHistory(true)}
              style={{
                fontWeight: 600,
                padding: '8px 16px',
                borderRadius: '8px',
              }}
            >
              <Icon name="Document" size={16} />
              Versions
            </Button>
          </Tooltip>

          <div
            style={{
              width: '1px',
              height: '20px',
              background: 'rgba(255, 255, 255, 0.1)',
              margin: `0 ${designTokens.spacing[1]}`,
            }}
          />

          <Tooltip content="Analyze Document">
            <Button variant="ghost" size="sm" style={{ padding: '8px' }}>
              <Icon name="Search" size={16} />
            </Button>
          </Tooltip>

          <Tooltip content="Generate Content">
            <Button variant="ghost" size="sm" style={{ padding: '8px' }}>
              <Icon name="Generate" size={16} />
            </Button>
          </Tooltip>

          <Tooltip content="Compare">
            <Button variant="ghost" size="sm" style={{ padding: '8px' }}>
              <Icon name="Compare" size={16} />
            </Button>
          </Tooltip>

          {/* Document Actions Menu */}
          <Dropdown
            options={[
              {
                value: 'rename',
                label: 'Rename Document',
              },
              {
                value: 'download',
                label: 'Download',
              },
              {
                value: 'compare',
                label: 'Compare with...',
              },
              {
                value: 'analyze',
                label: 'Deep Analysis',
              },
              {
                value: 'divider',
                label: '---',
                disabled: true,
              },
              {
                value: 'delete',
                label: 'Delete',
              },
            ]}
            onChange={(value: string | undefined) => {
              if (!value || value === 'divider') return

              console.log('Document action:', value)
              // Handle actions through conversational interface
              switch (value) {
                case 'rename':
                  console.log('Rename document:', documentId)
                  break
                case 'download':
                  console.log('Download document:', documentId)
                  break
                case 'compare':
                  console.log('Compare document:', documentId)
                  break
                case 'analyze':
                  console.log('Analyze document:', documentId)
                  break
                case 'delete':
                  console.log('Delete document:', documentId)
                  break
              }
            }}
            placeholder="More Actions"
            size="sm"
          />

          {onClose && (
            <Tooltip content="Close">
              <Button variant="ghost" size="sm" onClick={onClose}>
                <Icon name="ChevronDown" size={14} />
              </Button>
            </Tooltip>
          )}
        </div>
      </div>

      {/* Document Content */}
      <div
        ref={contentRef}
        style={{
          flex: 1,
          padding: isEditMode ? 0 : designTokens.spacing[8],
          overflowY: 'auto',
          backgroundColor: designTokens.colors.background.canvas,
          maxWidth: isEditMode ? '100%' : '75ch', // Full width for editor, optimal reading for viewer
          margin: '0 auto',
          width: '100%',
        }}
      >
        {document?.content &&
          (isEditMode ? (
            <DocumentEditor
              initialContent={document.content}
              onChange={(content: string) => {
                // Update document state for dirty tracking and auto-save
                updateContent(content)
                console.log('Content changed, length:', content.length, 'isDirty:', isDirty)
              }}
              className="h-full"
            />
          ) : (
            <DocumentRenderer
              content={document.content}
              documentType={
                document.type ||
                (document.metadata?.customFields?.detected_mime_type as string) ||
                document.path?.split('.').pop() ||
                undefined
              }
              documentName={document.name || document.path}
              style={{
                fontSize: designTokens.typography.fontSize.base,
                lineHeight: designTokens.typography.lineHeight.relaxed,
                color: designTokens.colors.text.primary,
              }}
            />
          ))}

        {/* Conflict Resolution Component */}
        {collaboration?.settings.enabled && isEditMode && conflicts.length > 0 && (
          <ConflictResolution
            conflicts={conflicts}
            onResolveConflict={resolveConflict}
            onDismissConflict={dismissConflict}
            onRefreshConflicts={refreshConflicts}
            className="mt-4"
          />
        )}
      </div>

      {/* Floating Intelligence Bar */}
      {showIntelligenceBar && selectedText && selectionPosition && (
        <Card
          ref={intelligenceBarRef}
          variant="glass"
          style={{
            position: 'fixed',
            top: selectionPosition.y - 50,
            left: selectionPosition.x - 150,
            width: '300px',
            padding: designTokens.spacing[3],
            zIndex: designTokens.zIndex.popover,
            pointerEvents: 'auto',
          }}
        >
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
              marginBottom: designTokens.spacing[2],
            }}
          >
            Selected: "{selectedText.substring(0, 50)}
            {selectedText.length > 50 ? '...' : ''}"
          </div>

          <div
            style={{
              display: 'flex',
              gap: designTokens.spacing[2],
              flexWrap: 'wrap',
            }}
          >
            <Button variant="ghost" size="sm">
              <Icon name="Search" size="xs" />
              Define
            </Button>
            <Button variant="ghost" size="sm">
              <Icon name="Generate" size="xs" />
              Explain
            </Button>
            <Button variant="ghost" size="sm">
              <Icon name="Compare" size="xs" />
              Related
            </Button>
          </div>
        </Card>
      )}

      {/* Change Tracking Indicators */}
      <div
        style={{
          position: 'absolute',
          right: 0,
          top: '32px',
          bottom: 0,
          width: '4px',
          background: designTokens.colors.surface.secondary,
        }}
      >
        {/* Mock change indicators */}
        <div
          style={{
            position: 'absolute',
            top: '20%',
            left: 0,
            right: 0,
            height: '2px',
            background: designTokens.colors.accent.success,
          }}
        />
        <div
          style={{
            position: 'absolute',
            top: '45%',
            left: 0,
            right: 0,
            height: '2px',
            background: designTokens.colors.accent.warning,
          }}
        />
        <div
          style={{
            position: 'absolute',
            top: '70%',
            left: 0,
            right: 0,
            height: '2px',
            background: designTokens.colors.accent.ai,
          }}
        />
      </div>

      {/* Version History Modal */}
      {showVersionHistory && (
        <VersionHistory
          documentId={documentId}
          currentContent={document?.content || ''}
          onClose={() => setShowVersionHistory(false)}
          onRestore={content => {
            // Restore the document content
            initializeDocumentState(content)
            setShowVersionHistory(false)
            // Reload the document to update metadata
            loadDocument()
          }}
        />
      )}

      {/* Live Cursors Overlay */}
      {collaboration?.settings.enabled && collaboration.settings.showCursors && (
        <LiveCursors
          cursors={cursorPositions}
          containerRef={contentRef}
          showLabels={true}
          fadeTimeout={3000}
        />
      )}

      {/* User Presence Panel */}
      {showPresencePanel && collaboration?.settings.enabled && (
        <div
          style={{
            position: 'fixed',
            top: '80px',
            right: '24px',
            zIndex: 10000,
            animation: 'slideInRight 0.2s ease-out',
          }}
        >
          <UserPresence
            users={collaborationUsers.map(user => ({
              id: user.id,
              name: user.name,
              color: user.color,
              isActive: user.isActive,
              cursor: user.cursor,
              lastSeen: user.lastSeen,
            }))}
            currentUserId={collaboration.currentUserId}
            isConnected={collaboration.isConnected}
            onUserClick={userId => {
              console.log('User clicked:', userId)
              // Future: Focus on user's cursor position
            }}
          />
        </div>
      )}

      <style>{`
        @keyframes slideInRight {
          from {
            transform: translateX(100%);
            opacity: 0;
          }
          to {
            transform: translateX(0);
            opacity: 1;
          }
        }
      `}</style>
    </div>
  )
}

export default DocumentViewer
