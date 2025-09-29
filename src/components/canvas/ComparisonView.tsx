import React, { useState, useEffect, useCallback, useRef } from 'react'
import { Card, Button, Icon, Badge, Tooltip } from '../ui'
import { documentService } from '../../services'
import { designTokens } from '../../styles/tokens'
import { Document, DocumentComparison } from '../../types'
import type { DocumentDifference } from '../../types/document'

interface ComparisonViewProps {
  documentAId: string
  documentBId: string
  onClose?: () => void
  onSelectDocument?: (documentId: string) => void
}

interface ViewMode {
  type: 'side-by-side' | 'overlay'
  label: string
  icon: string
}

interface ComparisonResult {
  comparison: DocumentComparison
  documentA: Document
  documentB: Document
}

const viewModes: ViewMode[] = [
  { type: 'side-by-side', label: 'Side by Side', icon: 'Columns' },
  { type: 'overlay', label: 'Overlay', icon: 'Layers' },
]

const ComparisonView: React.FC<ComparisonViewProps> = ({
  documentAId,
  documentBId,
  onClose,
  onSelectDocument,
}) => {
  const [result, setResult] = useState<ComparisonResult | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [viewMode, setViewMode] = useState<'side-by-side' | 'overlay'>('side-by-side')
  const [selectedDifference, setSelectedDifference] = useState<DocumentDifference | null>(null)
  const [syncScroll, setSyncScroll] = useState(true)
  const [showOnlyDifferences, setShowOnlyDifferences] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const panelARef = useRef<HTMLDivElement>(null)
  const panelBRef = useRef<HTMLDivElement>(null)
  const isScrollingSyncRef = useRef(false)

  // Load comparison data
  const loadComparison = useCallback(async () => {
    if (!documentAId || !documentBId) return

    setIsLoading(true)
    setError(null)

    try {
      // Load both documents and comparison in parallel
      const [docAResponse, docBResponse, comparisonResponse] = await Promise.all([
        documentService.getDocument(documentAId),
        documentService.getDocument(documentBId),
        documentService.compareDocuments(documentAId, documentBId, {
          include_ai_analysis: true,
          comparison_types: ['TextDiff', 'StructuralDiff', 'SemanticSimilarity'],
        }),
      ])

      if (!docAResponse.success || !docAResponse.data) {
        throw new Error(`Failed to load document A: ${docAResponse.error}`)
      }

      if (!docBResponse.success || !docBResponse.data) {
        throw new Error(`Failed to load document B: ${docBResponse.error}`)
      }

      if (!comparisonResponse.success || !comparisonResponse.data) {
        throw new Error(`Failed to compare documents: ${comparisonResponse.error}`)
      }

      setResult({
        comparison: comparisonResponse.data,
        documentA: docAResponse.data,
        documentB: docBResponse.data,
      })
    } catch (error) {
      console.error('Failed to load comparison:', error)
      setError(error instanceof Error ? error.message : 'Failed to load comparison')
    } finally {
      setIsLoading(false)
    }
  }, [documentAId, documentBId])

  // Handle synchronized scrolling
  const handleScroll = useCallback(
    (source: 'A' | 'B') => {
      if (!syncScroll || isScrollingSyncRef.current) return

      const sourcePanel = source === 'A' ? panelARef.current : panelBRef.current
      const targetPanel = source === 'A' ? panelBRef.current : panelARef.current

      if (!sourcePanel || !targetPanel) return

      isScrollingSyncRef.current = true

      const scrollPercentage =
        sourcePanel.scrollTop / (sourcePanel.scrollHeight - sourcePanel.clientHeight)

      targetPanel.scrollTop =
        scrollPercentage * (targetPanel.scrollHeight - targetPanel.clientHeight)

      // Reset the flag after a short delay
      setTimeout(() => {
        isScrollingSyncRef.current = false
      }, 100)
    },
    [syncScroll]
  )

  // Handle difference selection
  const handleDifferenceClick = useCallback((difference: DocumentDifference) => {
    setSelectedDifference(difference)
    // TODO: Scroll to difference position
  }, [])

  // Render document content with difference highlighting
  const renderDocumentContent = useCallback(
    (document: Document, isDocumentA: boolean) => {
      if (!document.content || !result?.comparison) return null

      const content = document.content
      const differences = result.comparison.differences.filter(
        diff => diff.position && (isDocumentA ? true : true) // Show all differences for now
      )

      let renderedContent = content

      // Apply difference highlighting
      differences.forEach((diff, index) => {
        if (!diff.position) return

        const beforeText = content.substring(0, diff.position.start)
        const diffText = content.substring(diff.position.start, diff.position.end)
        const afterText = content.substring(diff.position.end)

        const severityColor = {
          minor: designTokens.colors.accent.info,
          moderate: designTokens.colors.accent.warning,
          significant: designTokens.colors.accent.alert,
        }[diff.severity]

        const isSelected = selectedDifference?.description === diff.description
        const opacity = isSelected ? '40' : '20'

        renderedContent =
          beforeText +
          `<span
            class="difference-highlight difference-${diff.type}"
            data-difference-index="${index}"
            style="
              background: ${severityColor}${opacity};
              border-left: 3px solid ${severityColor};
              padding: 2px 4px;
              margin: 0 1px;
              border-radius: ${designTokens.borderRadius.sm};
              cursor: pointer;
              transition: all ${designTokens.animation.duration.fast} ease;
            "
            title="${diff.description}"
          >${diffText}</span>` +
          afterText
      })

      return renderedContent
    },
    [result?.comparison, selectedDifference]
  )

  // Get severity badge color
  const getSeverityColor = useCallback((severity: DocumentDifference['severity']) => {
    switch (severity) {
      case 'minor':
        return designTokens.colors.accent.info
      case 'moderate':
        return designTokens.colors.accent.warning
      case 'significant':
        return designTokens.colors.accent.alert
      default:
        return designTokens.colors.text.secondary
    }
  }, [])

  // Load data on mount
  useEffect(() => {
    loadComparison()
  }, [loadComparison])

  // Add event listeners for difference clicks
  useEffect(() => {
    const handleContentClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement
      const differenceIndex = target.getAttribute('data-difference-index')

      if (differenceIndex !== null && result?.comparison) {
        const index = parseInt(differenceIndex, 10)
        const difference = result.comparison.differences[index]
        if (difference) {
          handleDifferenceClick(difference)
        }
      }
    }

    const panelA = panelARef.current
    const panelB = panelBRef.current

    if (panelA && panelB) {
      panelA.addEventListener('click', handleContentClick)
      panelB.addEventListener('click', handleContentClick)

      return () => {
        panelA.removeEventListener('click', handleContentClick)
        panelB.removeEventListener('click', handleContentClick)
      }
    }

    return undefined
  }, [result?.comparison, handleDifferenceClick])

  // Add scroll event listeners
  useEffect(() => {
    const panelA = panelARef.current
    const panelB = panelBRef.current

    if (!panelA || !panelB) return

    const handleScrollA = () => handleScroll('A')
    const handleScrollB = () => handleScroll('B')

    panelA.addEventListener('scroll', handleScrollA)
    panelB.addEventListener('scroll', handleScrollB)

    return () => {
      panelA.removeEventListener('scroll', handleScrollA)
      panelB.removeEventListener('scroll', handleScrollB)
    }
  }, [handleScroll])

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
            Comparing documents...
          </span>
        </div>
      </div>
    )
  }

  if (error) {
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
            <Icon name="Alert" size={24} />
          </div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              marginBottom: designTokens.spacing[2],
            }}
          >
            Comparison Failed
          </div>
          <div style={{ fontSize: designTokens.typography.fontSize.sm }}>{error}</div>
          <Button
            variant="primary"
            onClick={loadComparison}
            style={{ marginTop: designTokens.spacing[4] }}
          >
            Try Again
          </Button>
        </div>
      </div>
    )
  }

  if (!result) {
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
            <Icon name="Compare" size="3xl" />
          </div>
          <div style={{ fontSize: designTokens.typography.fontSize.lg }}>No comparison data</div>
        </div>
      </div>
    )
  }

  const { comparison, documentA, documentB } = result

  return (
    <div
      style={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        background: designTokens.colors.background.canvas,
      }}
    >
      {/* Comparison Toolbar */}
      <div
        style={{
          height: '48px',
          padding: `0 ${designTokens.spacing[4]}`,
          background: designTokens.colors.surface.secondary,
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        {/* Document Info */}
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[4] }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onSelectDocument?.(documentAId)}
              style={{ color: designTokens.colors.accent.info }}
            >
              {documentA.name}
            </Button>
            <div style={{ transform: 'rotate(-90deg)' }}>
              <Icon name="ChevronDown" size={16} />
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onSelectDocument?.(documentBId)}
              style={{ color: designTokens.colors.accent.success }}
            >
              {documentB.name}
            </Button>
          </div>

          <Badge
            variant="confidence"
            style={{
              background: `${designTokens.colors.accent.ai}20`,
              color: designTokens.colors.accent.ai,
              fontSize: designTokens.typography.fontSize.xs,
            }}
          >
            {Math.round(comparison.similarity * 100)}% similar
          </Badge>

          <Badge
            variant="default"
            style={{
              fontSize: designTokens.typography.fontSize.xs,
            }}
          >
            {comparison.differences.length} differences
          </Badge>
        </div>

        {/* View Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
          {/* View Mode Selector */}
          <div
            style={{
              display: 'flex',
              background: designTokens.colors.surface.primary,
              borderRadius: designTokens.borderRadius.md,
              padding: '2px',
            }}
          >
            {viewModes.map(mode => (
              <Button
                key={mode.type}
                variant={viewMode === mode.type ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => setViewMode(mode.type)}
                style={{
                  borderRadius: designTokens.borderRadius.sm,
                }}
              >
                <Icon name={mode.type === 'side-by-side' ? 'Columns' : 'Layers'} size={12} />
                {mode.label}
              </Button>
            ))}
          </div>

          {/* Options */}
          <Tooltip content="Sync Scrolling">
            <Button
              variant={syncScroll ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setSyncScroll(!syncScroll)}
            >
              <Icon name="Link" size={16} />
            </Button>
          </Tooltip>

          <Tooltip content="Show Only Differences">
            <Button
              variant={showOnlyDifferences ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setShowOnlyDifferences(!showOnlyDifferences)}
            >
              <Icon name="Filter" size={16} />
            </Button>
          </Tooltip>

          {onClose && (
            <Tooltip content="Close Comparison">
              <Button variant="ghost" size="sm" onClick={onClose}>
                <Icon name="X" size={16} />
              </Button>
            </Tooltip>
          )}
        </div>
      </div>

      {/* Comparison Content */}
      <div style={{ flex: 1, display: 'flex', position: 'relative' }}>
        {viewMode === 'side-by-side' ? (
          <>
            {/* Document A Panel */}
            <div
              style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                borderRight: `1px solid ${designTokens.colors.border.subtle}`,
              }}
            >
              <div
                style={{
                  height: '32px',
                  padding: `0 ${designTokens.spacing[4]}`,
                  background: designTokens.colors.surface.tertiary,
                  borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
                  display: 'flex',
                  alignItems: 'center',
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  color: designTokens.colors.accent.info,
                }}
              >
                <Icon name="Document" size={12} />
                <span style={{ marginLeft: designTokens.spacing[2] }}>{documentA.name}</span>
              </div>
              <div
                ref={panelARef}
                style={{
                  flex: 1,
                  padding: designTokens.spacing[6],
                  overflowY: 'auto',
                  fontSize: designTokens.typography.fontSize.base,
                  lineHeight: designTokens.typography.lineHeight.relaxed,
                  color: designTokens.colors.text.primary,
                  fontFamily: designTokens.typography.fonts.sans.join(', '),
                }}
                dangerouslySetInnerHTML={{
                  __html: renderDocumentContent(documentA, true) || documentA.content || '',
                }}
              />
            </div>

            {/* Drag Handle */}
            <div
              style={{
                width: '2px',
                background: designTokens.colors.border.subtle,
                cursor: 'col-resize',
                position: 'relative',
              }}
            >
              <div
                style={{
                  position: 'absolute',
                  top: '50%',
                  left: '-3px',
                  width: '8px',
                  height: '32px',
                  background: designTokens.colors.surface.secondary,
                  border: `1px solid ${designTokens.colors.border.subtle}`,
                  borderRadius: designTokens.borderRadius.md,
                  transform: 'translateY(-50%)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                <div
                  style={{
                    width: '2px',
                    height: '16px',
                    background: designTokens.colors.text.secondary,
                    borderRadius: '1px',
                  }}
                />
              </div>
            </div>

            {/* Document B Panel */}
            <div
              style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
              }}
            >
              <div
                style={{
                  height: '32px',
                  padding: `0 ${designTokens.spacing[4]}`,
                  background: designTokens.colors.surface.tertiary,
                  borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
                  display: 'flex',
                  alignItems: 'center',
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  color: designTokens.colors.accent.success,
                }}
              >
                <Icon name="Document" size={12} />
                <span style={{ marginLeft: designTokens.spacing[2] }}>{documentB.name}</span>
              </div>
              <div
                ref={panelBRef}
                style={{
                  flex: 1,
                  padding: designTokens.spacing[6],
                  overflowY: 'auto',
                  fontSize: designTokens.typography.fontSize.base,
                  lineHeight: designTokens.typography.lineHeight.relaxed,
                  color: designTokens.colors.text.primary,
                  fontFamily: designTokens.typography.fonts.sans.join(', '),
                }}
                dangerouslySetInnerHTML={{
                  __html: renderDocumentContent(documentB, false) || documentB.content || '',
                }}
              />
            </div>
          </>
        ) : (
          /* Overlay Mode */
          <div style={{ flex: 1, position: 'relative' }}>
            {/* TODO: Implement overlay comparison mode */}
            <div
              style={{
                height: '100%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: designTokens.colors.text.secondary,
              }}
            >
              Overlay mode coming soon
            </div>
          </div>
        )}

        {/* Connection Lines for Moved Content */}
        {viewMode === 'side-by-side' && (
          <svg
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              pointerEvents: 'none',
              zIndex: 1,
            }}
          >
            {/* TODO: Draw SVG curves connecting related content */}
          </svg>
        )}
      </div>

      {/* Differences Sidebar */}
      {selectedDifference && (
        <Card
          variant="glass"
          style={{
            position: 'absolute',
            right: designTokens.spacing[4],
            top: '60px',
            width: '320px',
            maxHeight: '400px',
            zIndex: designTokens.zIndex.popover,
            padding: designTokens.spacing[4],
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: designTokens.spacing[3],
            }}
          >
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                fontWeight: designTokens.typography.fontWeight.medium,
                color: designTokens.colors.text.primary,
              }}
            >
              Difference Details
            </div>
            <Button variant="ghost" size="sm" onClick={() => setSelectedDifference(null)}>
              <Icon name="X" size={12} />
            </Button>
          </div>

          <div style={{ marginBottom: designTokens.spacing[3] }}>
            <Badge
              style={{
                background: `${getSeverityColor(selectedDifference.severity)}20`,
                color: getSeverityColor(selectedDifference.severity),
                fontSize: designTokens.typography.fontSize.xs,
                marginBottom: designTokens.spacing[2],
              }}
            >
              {selectedDifference.severity} • {selectedDifference.type}
            </Badge>
          </div>

          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
              lineHeight: designTokens.typography.lineHeight.relaxed,
            }}
          >
            {selectedDifference.description}
          </div>
        </Card>
      )}
    </div>
  )
}

export default ComparisonView
