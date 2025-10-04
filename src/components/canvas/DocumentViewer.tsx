import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import { Icon } from '../ui'
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
import DocumentToolbar, { type ToolbarGroup } from './DocumentToolbar'
import VersionHistory from '../editor/VersionHistory'
import { UserPresence } from '../collaboration/UserPresence'
import { LiveCursors } from '../collaboration/LiveCursors'
import { ConflictResolution } from '../collaboration/ConflictResolution'
import { Comments } from '../collaboration/Comments'
import { useDocumentState } from '../../hooks/useDocumentState'
import { useAutoSave } from '../../hooks/useAutoSave'
import { useCollaboration } from '../../context/useCollaboration'
import { useTypingIndicator } from '../../hooks/useTypingIndicator'
import { useConflictResolution } from '../../hooks/useConflictResolution'
import { useOfflineSync } from '../../hooks/useOfflineSync'
import { useTrackChanges } from '../../hooks/useTrackChanges'
import { TrackChanges } from '../editor/TrackChanges'
import { ChangeReview } from '../editor/ChangeReview'
import { DocumentDiff } from '../editor/DocumentDiff'
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
  const [isLoading, setIsLoading] = useState(true)
  const [loadingOperations, setLoadingOperations] = useState<OperationProgress[]>([])
  // Note: hoveredHighlight temporarily disabled - will be re-implemented with React-based overlays
  // const [hoveredHighlight, setHoveredHighlight] = useState<string | null>(null)
  const [isEditMode, setIsEditMode] = useState(false)
  const [showUnsavedWarning, setShowUnsavedWarning] = useState(false)
  const [showVersionHistory, setShowVersionHistory] = useState(false)
  const [showPresencePanel, setShowPresencePanel] = useState(false)
  const [enableAutoComplete, setEnableAutoComplete] = useState(false)
  const [showSaveNotification, setShowSaveNotification] = useState(false)
  const [showTrackChangesPanel, setShowTrackChangesPanel] = useState(false)
  const [showChangeReview, setShowChangeReview] = useState(false)
  const [showDocumentDiff, setShowDocumentDiff] = useState(false)
  const [diffOldContent, setDiffOldContent] = useState<string>('')
  const [diffNewContent, setDiffNewContent] = useState<string>('')
  const contentRef = useRef<HTMLDivElement>(null)
  const saveNotificationTimeoutRef = useRef<NodeJS.Timeout | null>(null)

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
  const { conflicts, resolveConflict, dismissConflict, refreshConflicts } = useConflictResolution({
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
  const { isSyncing, queuedOperations, syncProgress, triggerSync } = useOfflineSync({
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

  // Track Changes hook for revision tracking
  const {
    changes,
    isTrackingEnabled,
    showChanges,
    addChange, // eslint-disable-line @typescript-eslint/no-unused-vars
    acceptChange,
    rejectChange,
    acceptAllChanges,
    rejectAllChanges,
    toggleTracking,
    toggleShowChanges,
  } = useTrackChanges({
    enabled: false, // Track changes is opt-in
    currentUser: collaboration?.settings.username || 'You',
    onChangeAccepted: change => {
      console.log('Change accepted:', change)
    },
    onChangeRejected: change => {
      console.log('Change rejected:', change)
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

        // Show save notification
        setShowSaveNotification(true)
        if (saveNotificationTimeoutRef.current) {
          clearTimeout(saveNotificationTimeoutRef.current)
        }
        saveNotificationTimeoutRef.current = setTimeout(() => {
          setShowSaveNotification(false)
        }, 3000)

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
  const loadDocument = useCallback(
    async (skipLoadingState = false) => {
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
    },
    [documentId, generateAISuggestions, updateOperation]
  )

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

  // Create toolbar action groups with proper typing
  const homeActions = useMemo<ToolbarGroup[]>(
    () => [
      {
        id: 'mode',
        label: 'Mode',
        actions: [
          {
            id: 'toggle-mode',
            label: isEditMode ? 'View' : 'Edit',
            icon: isEditMode ? ('Eye' as const) : ('Edit' as const),
            onClick: () => {
              if (isEditMode && isDirty && showUnsavedWarning) {
                const confirmSwitch = window.confirm(
                  'You have unsaved changes. Switch to view mode anyway? (Changes will be lost)'
                )
                if (!confirmSwitch) return
              }
              setIsEditMode(!isEditMode)
            },
            tooltip: isEditMode ? 'Switch to View Mode' : 'Switch to Edit Mode',
            showLabel: true,
            variant: isEditMode ? ('secondary' as const) : ('ghost' as const),
          },
          ...(isEditMode
            ? [
                {
                  id: 'ai-autocomplete',
                  label: enableAutoComplete ? 'AI On' : 'AI Off',
                  icon: 'Sparkles' as const,
                  onClick: () => setEnableAutoComplete(!enableAutoComplete),
                  tooltip: enableAutoComplete
                    ? 'Disable AI Auto-Complete'
                    : 'Enable AI Auto-Complete',
                  showLabel: true,
                  variant: enableAutoComplete ? ('secondary' as const) : ('ghost' as const),
                },
              ]
            : []),
        ],
      },
    ],
    [isEditMode, isDirty, showUnsavedWarning, enableAutoComplete]
  )

  const reviewActions = useMemo<ToolbarGroup[]>(
    () => [
      {
        id: 'versions',
        label: 'Versions',
        actions: [
          {
            id: 'version-history',
            label: 'Versions',
            icon: 'History' as const,
            onClick: () => setShowVersionHistory(true),
            tooltip: 'Version History',
            showLabel: true,
          },
          {
            id: 'compare',
            label: 'Compare',
            icon: 'GitCompare' as const,
            onClick: () => {
              if (document && editedContent) {
                setDiffOldContent(document.content || '')
                setDiffNewContent(editedContent)
                setShowDocumentDiff(true)
              }
            },
            tooltip: 'Compare Versions',
            showLabel: true,
          },
        ],
      },
      {
        id: 'collaboration',
        label: 'Collaboration',
        actions: [
          {
            id: 'comments',
            label: 'Comments',
            icon: 'MessageCircle' as const,
            onClick: () => window.dispatchEvent(new CustomEvent('toggleCommentsSidebar')),
            tooltip: 'Comments & Annotations',
            showLabel: true,
          },
          {
            id: 'track-changes',
            label: 'Track',
            icon: 'GitBranch' as const,
            onClick: () => setShowTrackChangesPanel(!showTrackChangesPanel),
            tooltip: 'Track Changes',
            showLabel: true,
            variant: showTrackChangesPanel ? ('secondary' as const) : ('ghost' as const),
            badge: changes.length > 0 ? changes.length : undefined,
          },
          {
            id: 'review-changes',
            label: 'Review',
            icon: 'CheckCircle' as const,
            onClick: () => setShowChangeReview(true),
            tooltip: 'Review Changes',
            showLabel: true,
          },
        ],
      },
    ],
    [document, editedContent, showTrackChangesPanel, changes.length]
  )

  const aiToolsActions = useMemo<ToolbarGroup[]>(
    () => [
      {
        id: 'ai-analysis',
        label: 'AI Analysis',
        actions: [
          {
            id: 'analyze',
            label: 'Analyze',
            icon: 'Search' as const,
            onClick: () => console.log('Analyze document'),
            tooltip: 'Analyze Document',
            showLabel: true,
          },
          {
            id: 'deep-analysis',
            label: 'Deep',
            icon: 'Zap' as const,
            onClick: () => console.log('Deep analysis'),
            tooltip: 'Deep Analysis',
            showLabel: true,
          },
        ],
      },
      {
        id: 'ai-generation',
        label: 'AI Generation',
        actions: [
          {
            id: 'generate',
            label: 'Generate',
            icon: 'Sparkles' as const,
            onClick: () => console.log('Generate content'),
            tooltip: 'Generate Content',
            showLabel: true,
          },
          {
            id: 'suggestions',
            label: 'Suggest',
            icon: 'Lightbulb' as const,
            onClick: () => generateAISuggestions(),
            tooltip: 'AI Suggestions',
            showLabel: true,
            badge: aiSuggestions.length > 0 ? aiSuggestions.length : undefined,
          },
        ],
      },
    ],
    [aiSuggestions.length, generateAISuggestions]
  )

  const moreActions = useMemo<ToolbarGroup[]>(
    () => [
      {
        id: 'document-actions',
        label: 'Document',
        actions: [
          {
            id: 'rename',
            label: 'Rename',
            icon: 'Edit' as const,
            onClick: () => console.log('Rename document'),
            tooltip: 'Rename Document',
            showLabel: true,
          },
          {
            id: 'download',
            label: 'Download',
            icon: 'Download' as const,
            onClick: () => console.log('Download document'),
            tooltip: 'Download',
            showLabel: true,
          },
        ],
      },
    ],
    []
  )

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
      {/* New Document Toolbar */}
      <DocumentToolbar
        documentName={document.name}
        documentIcon="FileText"
        confidence={confidenceScore}
        documentType={classification?.categories?.[0]?.name || 'Unknown'}
        activeUsers={collaborationUsers}
        currentUserId={collaboration?.currentUserId}
        showPresence={collaboration?.settings.enabled && collaboration.settings.showPresence}
        onTogglePresence={() => setShowPresencePanel(!showPresencePanel)}
        isSyncing={isSyncing}
        queuedOperations={queuedOperations.length}
        syncProgress={syncProgress}
        onManualSync={triggerSync}
        showOfflineIndicator={collaboration?.settings.enabled || false}
        isEditMode={isEditMode}
        onSave={saveDocument}
        isSaving={isSaving}
        isDirty={isDirty}
        saveError={saveError || undefined}
        lastSaved={lastSaved}
        homeActions={homeActions}
        reviewActions={reviewActions}
        aiToolsActions={aiToolsActions}
        moreActions={moreActions}
        dropdownActions={[
          { value: 'delete', label: 'Delete Document' },
          { value: 'duplicate', label: 'Duplicate' },
          { value: 'export', label: 'Export as...' },
        ]}
        onDropdownAction={value => console.log('Dropdown action:', value)}
        onClose={onClose}
        defaultTab="home"
      />

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
              documentId={documentId}
              documentTitle={document.name || 'Untitled Document'}
              enableAutoComplete={enableAutoComplete}
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

      {/* Comments & Annotations System */}
      <Comments documentId={documentId} contentRef={contentRef} isEditMode={isEditMode} />

      {/* Track Changes Panel */}
      {showTrackChangesPanel && (
        <div
          style={{
            position: 'fixed',
            top: '80px',
            right: 0,
            bottom: 0,
            zIndex: 9999,
            animation: 'slideInRight 0.2s ease-out',
          }}
        >
          <TrackChanges
            changes={changes}
            onAcceptChange={acceptChange}
            onRejectChange={rejectChange}
            onAcceptAll={acceptAllChanges}
            onRejectAll={rejectAllChanges}
            isTrackingEnabled={isTrackingEnabled}
            onToggleTracking={toggleTracking}
            showChanges={showChanges}
            onToggleShowChanges={toggleShowChanges}
            currentUser="You"
          />
        </div>
      )}

      {/* Change Review Modal */}
      {showChangeReview && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10001,
            padding: '24px',
          }}
          onClick={() => setShowChangeReview(false)}
        >
          <div onClick={e => e.stopPropagation()}>
            <ChangeReview
              changes={changes}
              documentTitle={document?.name || 'Untitled Document'}
              onAcceptChange={acceptChange}
              onRejectChange={rejectChange}
              onAcceptAll={acceptAllChanges}
              onRejectAll={rejectAllChanges}
              onClose={() => setShowChangeReview(false)}
              currentUser="You"
            />
          </div>
        </div>
      )}

      {/* Document Diff Modal */}
      {showDocumentDiff && (
        <DocumentDiff
          oldContent={diffOldContent}
          newContent={diffNewContent}
          oldTitle="Original Version"
          newTitle="Current Version"
          onClose={() => setShowDocumentDiff(false)}
          onMergeLeft={() => {
            // Merge left: use new version (current content)
            updateContent(diffNewContent)
            setShowDocumentDiff(false)
          }}
          onMergeRight={() => {
            // Merge right: revert to old version (original)
            updateContent(diffOldContent)
            setShowDocumentDiff(false)
          }}
        />
      )}

      {/* Save Notification Toast */}
      {showSaveNotification && lastSaved && (
        <div
          style={{
            position: 'fixed',
            bottom: '24px',
            left: '50%',
            transform: 'translateX(-50%)',
            zIndex: 10000,
            backgroundColor: designTokens.colors.accent.success,
            color: 'white',
            padding: '12px 24px',
            borderRadius: '8px',
            boxShadow: designTokens.shadows.lg,
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontFamily: designTokens.typography.fonts.sans.join(', '),
            fontSize: designTokens.typography.fontSize.sm,
            fontWeight: 500,
            animation: 'slideUpFade 0.3s ease-out',
          }}
        >
          <Icon name="AlertCircle" size={16} />
          Saved at {new Date(lastSaved).toLocaleTimeString()}
        </div>
      )}

      <style>{`
        @keyframes slideUpFade {
          from {
            opacity: 0;
            transform: translate(-50%, 20px);
          }
          to {
            opacity: 1;
            transform: translate(-50%, 0);
          }
        }
      `}</style>

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
