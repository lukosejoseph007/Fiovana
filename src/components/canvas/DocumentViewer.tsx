import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import { Card, Button, Icon, Badge, Tooltip } from '../ui'
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

  // Load document data
  const loadDocument = useCallback(async () => {
    if (!documentId) return

    setIsLoading(true)
    try {
      // Load document content
      const docResponse = await documentService.getDocument(documentId)
      if (docResponse.success && docResponse.data) {
        setDocument(docResponse.data)
      }

      // Load document structure
      const structureResponse = await structureService.analyzeDocumentStructure(documentId)
      if (structureResponse.success && structureResponse.data) {
        setStructure(structureResponse.data)
      }

      // Load content classification
      const classificationResponse =
        await contentClassificationService.classifyContentType(documentId)
      if (classificationResponse.success && classificationResponse.data) {
        setClassification(classificationResponse.data)
      }

      // Generate AI suggestions (mock for now)
      await generateAISuggestions()

      // Generate semantic highlights (mock for now)
      await generateSemanticHighlights()
    } catch (error) {
      console.error('Failed to load document:', error)
    } finally {
      setIsLoading(false)
    }
  }, [documentId, generateAISuggestions, generateSemanticHighlights])

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
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: designTokens.colors.background.canvas,
        }}
      >
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: designTokens.spacing[4],
          }}
        >
          <div
            style={{
              width: '32px',
              height: '32px',
              border: `3px solid ${designTokens.colors.border.subtle}`,
              borderTop: `3px solid ${designTokens.colors.accent.ai}`,
              borderRadius: '50%',
              animation: 'spin 1s linear infinite',
            }}
          />
          <span
            style={{
              color: designTokens.colors.text.secondary,
              fontSize: designTokens.typography.fontSize.base,
            }}
          >
            Loading document...
          </span>
        </div>
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
              <Icon name="Search" size="sm" />
            </Button>
          </Tooltip>

          <Tooltip content="Generate Content">
            <Button variant="ghost" size="sm">
              <Icon name="Generate" size="sm" />
            </Button>
          </Tooltip>

          <Tooltip content="Compare">
            <Button variant="ghost" size="sm">
              <Icon name="Compare" size="sm" />
            </Button>
          </Tooltip>

          {onClose && (
            <Tooltip content="Close">
              <Button variant="ghost" size="sm" onClick={onClose}>
                <Icon name="ChevronDown" size="sm" />
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
