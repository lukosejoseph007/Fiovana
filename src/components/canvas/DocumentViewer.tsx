import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import { Card, Button, Icon, Badge, Tooltip, Dropdown } from '../ui'
import {
  DocumentSkeleton,
  LongOperationProgress,
  OperationProgressTracker,
  type OperationProgress,
} from '../ui/LoadingStates'
import { documentService, structureService, contentClassificationService } from '../../services'
import { designTokens } from '../../styles/tokens'
import { Document, DocumentStructure, ContentClassification } from '../../types'

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
  const [semanticHighlights, setSemanticHighlights] = useState<SemanticHighlight[]>([])
  const [selectedText, setSelectedText] = useState<string>('')
  const [selectionPosition, setSelectionPosition] = useState<{ x: number; y: number } | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [loadingOperations, setLoadingOperations] = useState<OperationProgress[]>([])
  const [hoveredHighlight, setHoveredHighlight] = useState<string | null>(null)
  const [showIntelligenceBar, setShowIntelligenceBar] = useState(false)
  const contentRef = useRef<HTMLDivElement>(null)
  const intelligenceBarRef = useRef<HTMLDivElement>(null)

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

  // Generate semantic highlights
  const generateSemanticHighlights = useCallback(async () => {
    if (!documentId) return

    try {
      // Mock semantic highlights - in reality, this would use content classification
      const highlights: SemanticHighlight[] = [
        {
          id: 'highlight-1',
          start: 50,
          end: 120,
          type: 'concept',
          confidence: 0.9,
          metadata: { conceptType: 'definition', importance: 'high' },
        },
        {
          id: 'highlight-2',
          start: 200,
          end: 280,
          type: 'procedure',
          confidence: 0.85,
          metadata: { stepNumber: 1, complexity: 'medium' },
        },
        {
          id: 'highlight-3',
          start: 450,
          end: 520,
          type: 'reference',
          confidence: 0.75,
          metadata: { referenceType: 'external', source: 'document' },
        },
      ]

      setSemanticHighlights(highlights)
    } catch (error) {
      console.error('Failed to generate semantic highlights:', error)
    }
  }, [documentId])

  // Helper to update operation status
  const updateOperation = useCallback((id: string, updates: Partial<OperationProgress>) => {
    setLoadingOperations(prev => prev.map(op => (op.id === id ? { ...op, ...updates } : op)))
  }, [])

  // Load document data
  const loadDocument = useCallback(async () => {
    if (!documentId) return

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

    try {
      // Load document content
      updateOperation('load-document', { status: 'in-progress', progress: 10 })
      console.log('Loading document with ID:', documentId)
      const docResponse = await documentService.getDocument(documentId)
      console.log('Document response:', docResponse)

      if (docResponse.success && docResponse.data) {
        console.log('Document loaded successfully:', docResponse.data)
        setDocument(docResponse.data)
        updateOperation('load-document', { status: 'completed', progress: 100 })
      } else {
        console.error('Failed to load document:', docResponse.error)
        updateOperation('load-document', {
          status: 'failed',
          details: docResponse.error || 'Failed to load document',
        })
      }

      // Load document structure - use the file path from the loaded document
      if (docResponse.data?.path) {
        updateOperation('load-structure', { status: 'in-progress', progress: 20 })
        const structureResponse = await structureService.analyzeDocumentStructure(
          docResponse.data.path
        )
        if (structureResponse.success && structureResponse.data) {
          setStructure(structureResponse.data)
          updateOperation('load-structure', { status: 'completed', progress: 100 })
        } else {
          updateOperation('load-structure', { status: 'failed', details: 'Analysis failed' })
        }

        // Load content classification - use the file path
        updateOperation('load-classification', { status: 'in-progress', progress: 30 })
        const classificationResponse = await contentClassificationService.classifyContentType(
          docResponse.data.path
        )
        if (classificationResponse.success && classificationResponse.data) {
          setClassification(classificationResponse.data)
          updateOperation('load-classification', { status: 'completed', progress: 100 })
        } else {
          updateOperation('load-classification', {
            status: 'failed',
            details: 'Classification failed',
          })
        }
      }

      // Generate AI suggestions (mock for now)
      updateOperation('generate-suggestions', { status: 'in-progress', progress: 40 })
      await generateAISuggestions()
      updateOperation('generate-suggestions', { status: 'completed', progress: 100 })

      // Generate semantic highlights (mock for now)
      updateOperation('generate-highlights', { status: 'in-progress', progress: 50 })
      await generateSemanticHighlights()
      updateOperation('generate-highlights', { status: 'completed', progress: 100 })
    } catch (error) {
      console.error('Failed to load document:', error)
      // Mark all in-progress operations as failed
      setLoadingOperations(prev =>
        prev.map(op =>
          op.status === 'in-progress' || op.status === 'pending'
            ? { ...op, status: 'failed', details: 'Operation failed' }
            : op
        )
      )
    } finally {
      setIsLoading(false)
    }
  }, [documentId, generateAISuggestions, generateSemanticHighlights, updateOperation])

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

  // Handle semantic highlight hover
  const handleHighlightHover = useCallback((highlightId: string | null) => {
    setHoveredHighlight(highlightId)
  }, [])

  // Get highlight color based on type and hover state
  const getHighlightColor = useCallback((type: string, isHovered: boolean) => {
    const opacity = isHovered ? '40' : '20'
    switch (type) {
      case 'concept':
        return `${designTokens.colors.accent.ai}${opacity}`
      case 'procedure':
        return `${designTokens.colors.accent.success}${opacity}`
      case 'definition':
        return `${designTokens.colors.accent.warning}${opacity}`
      case 'reference':
        return `${designTokens.colors.accent.info}${opacity}`
      default:
        return `${designTokens.colors.text.secondary}${opacity}`
    }
  }, [])

  // Render document content with highlights and suggestions
  const renderDocumentContent = useMemo(() => {
    if (!document?.content) return null

    const content = document.content
    let renderedContent = content

    // Apply semantic highlights
    semanticHighlights.forEach(highlight => {
      const beforeText = content.substring(0, highlight.start)
      const highlightText = content.substring(highlight.start, highlight.end)
      const afterText = content.substring(highlight.end)

      const highlightClass = `semantic-highlight semantic-highlight-${highlight.type}`
      const isHovered = hoveredHighlight === highlight.id

      renderedContent =
        beforeText +
        `<span
          class="${highlightClass}"
          data-highlight-id="${highlight.id}"
          style="
            background: ${getHighlightColor(highlight.type, isHovered)};
            border-radius: ${designTokens.borderRadius.sm};
            padding: 0 2px;
            transition: all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut};
            cursor: pointer;
          "
        >${highlightText}</span>` +
        afterText
    })

    // Apply AI suggestion indicators
    aiSuggestions.forEach(suggestion => {
      const indicatorHtml = `<span
        class="ai-suggestion-indicator"
        data-suggestion-id="${suggestion.id}"
        style="
          display: inline-block;
          width: 8px;
          height: 8px;
          background: ${designTokens.colors.accent.ai};
          border-radius: 50%;
          margin-left: 4px;
          vertical-align: middle;
          cursor: pointer;
          opacity: 0.7;
          transition: opacity ${designTokens.animation.duration.fast} ease;
        "
        title="${suggestion.content}"
      ></span>`

      const beforeText = renderedContent.substring(0, suggestion.position.end)
      const afterText = renderedContent.substring(suggestion.position.end)
      renderedContent = beforeText + indicatorHtml + afterText
    })

    return renderedContent
  }, [document?.content, semanticHighlights, aiSuggestions, hoveredHighlight, getHighlightColor])

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

  // Handle clicks on semantic highlights and AI suggestions
  useEffect(() => {
    const handleContentClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement

      // Handle semantic highlight click
      const highlightId = target.getAttribute('data-highlight-id')
      if (highlightId) {
        const highlight = semanticHighlights.find(h => h.id === highlightId)
        if (highlight) {
          console.log('Semantic highlight clicked:', highlight)
        }
      }

      // Handle AI suggestion click
      const suggestionId = target.getAttribute('data-suggestion-id')
      if (suggestionId) {
        const suggestion = aiSuggestions.find(s => s.id === suggestionId)
        if (suggestion) {
          handleAISuggestionClick(suggestion)
        }
      }
    }

    const handleContentHover = (event: MouseEvent) => {
      const target = event.target as HTMLElement
      const highlightId = target.getAttribute('data-highlight-id')
      handleHighlightHover(highlightId)
    }

    const contentElement = contentRef.current
    if (contentElement) {
      contentElement.addEventListener('click', handleContentClick)
      contentElement.addEventListener('mouseover', handleContentHover)
      contentElement.addEventListener('mouseout', () => handleHighlightHover(null))

      return () => {
        contentElement.removeEventListener('click', handleContentClick)
        contentElement.removeEventListener('mouseover', handleContentHover)
        contentElement.removeEventListener('mouseout', () => handleHighlightHover(null))
      }
    }

    return undefined
  }, [semanticHighlights, aiSuggestions, handleAISuggestionClick, handleHighlightHover])

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
          height: '32px',
          padding: `0 ${designTokens.spacing[4]}`,
          background: designTokens.colors.surface.secondary,
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        {/* Document Info */}
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}>
          <span
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              fontWeight: designTokens.typography.fontWeight.medium,
              color: designTokens.colors.text.primary,
              maxWidth: '300px',
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
              background: `${designTokens.colors.accent.ai}20`,
              color: designTokens.colors.accent.ai,
              fontSize: designTokens.typography.fontSize.xs,
            }}
          >
            {Math.round(confidenceScore * 100)}% confidence
          </Badge>

          {classification && (
            <Badge
              variant="default"
              style={{
                fontSize: designTokens.typography.fontSize.xs,
              }}
            >
              {classification.categories?.[0]?.name || 'Unknown'}
            </Badge>
          )}
        </div>

        {/* Actions */}
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
          <Tooltip content="Analyze Document">
            <Button variant="ghost" size="sm">
              <Icon name="Search" size={14} />
            </Button>
          </Tooltip>

          <Tooltip content="Generate Content">
            <Button variant="ghost" size="sm">
              <Icon name="Generate" size={14} />
            </Button>
          </Tooltip>

          <Tooltip content="Compare">
            <Button variant="ghost" size="sm">
              <Icon name="Compare" size={14} />
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
          padding: designTokens.spacing[8],
          overflowY: 'auto',
          fontSize: designTokens.typography.fontSize.base,
          lineHeight: designTokens.typography.lineHeight.relaxed,
          color: designTokens.colors.text.primary,
          backgroundColor: designTokens.colors.background.canvas,
          fontFamily: designTokens.typography.fonts.sans.join(', '),
          maxWidth: '75ch', // Optimal reading line length
          margin: '0 auto',
          width: '100%',
        }}
        dangerouslySetInnerHTML={{ __html: renderDocumentContent || '' }}
      />

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
    </div>
  )
}

export default DocumentViewer
