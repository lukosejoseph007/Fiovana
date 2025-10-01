import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { designTokens } from '../../styles/tokens'
import Card from '../ui/Card'
import Progress from '../ui/Progress'
import Badge from '../ui/Badge'
import Button from '../ui/Button'
import Icon from '../ui/Icon'
// import { documentService } from '../../services/documentService'
import { structureService } from '../../services/structureService'
// import { contentClassificationService } from '../../services/contentClassificationService'
import { styleAnalysisService } from '../../services/styleAnalysisService'

export interface DocumentIntelligenceProps {
  contextData?: unknown
  documentId?: string
  className?: string
  style?: React.CSSProperties
}

interface DocumentAnalysis {
  structure: {
    score: number
    headings: number
    sections: number
    completeness: 'excellent' | 'good' | 'needs_improvement'
    issues: string[]
  }
  clarity: {
    score: number
    readabilityLevel: string
    avgSentenceLength: number
    complexWords: number
    suggestions: string[]
  }
  concepts: {
    keyTerms: Array<{ term: string; frequency: number; importance: number }>
    categories: Array<{ category: string; confidence: number }>
    relationships: Array<{ from: string; to: string; type: string }>
  }
  procedures: {
    identified: number
    complete: number
    missing: string[]
    quality: 'high' | 'medium' | 'low'
  }
  gaps: {
    critical: string[]
    moderate: string[]
    minor: string[]
    severity: 'high' | 'medium' | 'low'
  }
}

interface AnalysisCardProps {
  title: string
  icon: string
  score?: number
  status?: 'excellent' | 'good' | 'needs_improvement' | 'high' | 'medium' | 'low'
  children: React.ReactNode
}

const AnalysisCard: React.FC<AnalysisCardProps> = ({ title, icon, score, status, children }) => {
  const getStatusColor = (status?: string) => {
    switch (status) {
      case 'excellent':
      case 'high':
        return designTokens.colors.confidence.high
      case 'good':
      case 'medium':
        return designTokens.colors.confidence.medium
      case 'needs_improvement':
      case 'low':
        return designTokens.colors.confidence.low
      default:
        return designTokens.colors.text.secondary
    }
  }

  const cardStyles = {
    marginBottom: designTokens.spacing[4],
  }

  const headerStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: designTokens.spacing[3],
  }

  const titleStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    fontSize: designTokens.typography.fontSize.base,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
  }

  const scoreStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
  }

  return (
    <Card variant="elevated" style={cardStyles}>
      <div style={headerStyles}>
        <div style={titleStyles}>
          <Icon name={icon as never} size={18} />
          {title}
        </div>
        {(score !== undefined || status) && (
          <div style={scoreStyles}>
            {score !== undefined && (
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.lg,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: getStatusColor(status),
                }}
              >
                {score}%
              </span>
            )}
            {status && (
              <Badge
                variant="default"
                size="sm"
                style={{
                  color: getStatusColor(status),
                  borderColor: getStatusColor(status),
                }}
              >
                {status}
              </Badge>
            )}
          </div>
        )}
      </div>
      {children}
    </Card>
  )
}

const DocumentIntelligence: React.FC<DocumentIntelligenceProps> = ({
  contextData: _contextData,
  documentId,
  className = '',
  style,
}) => {
  const [analysis, setAnalysis] = useState<DocumentAnalysis | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedDocument, setSelectedDocument] = useState<string | null>(documentId || null)

  // Load document analysis
  const loadAnalysis = useCallback(async (docId: string) => {
    if (!docId) return

    setIsLoading(true)
    setError(null)

    try {
      // Run multiple analysis operations in parallel
      const [structureResult] = await Promise.allSettled([
        structureService.analyzeDocumentStructure(docId),
        // contentClassificationService.classifyDocument(docId),
        Promise.resolve({ success: true, data: { type: 'document', confidence: 0.85 } }),
        styleAnalysisService.analyzeDocumentStyle(docId),
      ])

      // Process results and create comprehensive analysis
      const analysis: DocumentAnalysis = {
        structure: {
          score: structureResult.status === 'fulfilled' && structureResult.value.success ? 85 : 0, // Mock score, would come from actual analysis
          headings: 12, // Mock data
          sections: 5,
          completeness: 'good',
          issues: ['Missing conclusion section', 'Inconsistent heading hierarchy'],
        },
        clarity: {
          score: 78,
          readabilityLevel: 'College',
          avgSentenceLength: 18.2,
          complexWords: 45,
          suggestions: [
            'Consider shorter sentences in introduction',
            'Define technical terms on first use',
            'Add more examples for complex concepts',
          ],
        },
        concepts: {
          keyTerms: [
            { term: 'Machine Learning', frequency: 24, importance: 0.9 },
            { term: 'Neural Network', frequency: 18, importance: 0.8 },
            { term: 'Deep Learning', frequency: 15, importance: 0.7 },
            { term: 'Algorithm', frequency: 31, importance: 0.6 },
          ],
          categories: [
            { category: 'Technical Documentation', confidence: 0.92 },
            { category: 'Educational Content', confidence: 0.78 },
            { category: 'Process Description', confidence: 0.65 },
          ],
          relationships: [
            { from: 'Machine Learning', to: 'Neural Network', type: 'contains' },
            { from: 'Neural Network', to: 'Deep Learning', type: 'enables' },
          ],
        },
        procedures: {
          identified: 6,
          complete: 4,
          missing: ['Error handling procedure', 'Validation steps'],
          quality: 'medium',
        },
        gaps: {
          critical: ['Missing safety guidelines'],
          moderate: ['Incomplete examples', 'No troubleshooting section'],
          minor: ['Formatting inconsistencies', 'Minor typos'],
          severity: 'medium',
        },
      }

      setAnalysis(analysis)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to analyze document')
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Load analysis when document changes
  useEffect(() => {
    if (selectedDocument) {
      loadAnalysis(selectedDocument)
    }
  }, [selectedDocument, loadAnalysis])

  // Handle document selection change
  const handleDocumentChange = useCallback(() => {
    // This would open a document selector dialog
    // For now, we'll simulate selecting a document
    setSelectedDocument('sample-doc-id')
  }, [])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column' as const,
      height: '100%',
      width: '100%',
      overflow: 'hidden' as const,
      ...style,
    }),
    [style]
  )

  const headerStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.tertiary,
    borderRadius: designTokens.borderRadius.md,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    flexShrink: 0,
    marginBottom: designTokens.spacing[3],
  }

  const scrollContainerStyles = {
    flex: 1,
    overflowY: 'auto' as const,
    overflowX: 'hidden' as const,
    padding: `0 ${designTokens.spacing[1]}`,
    minHeight: 0,
  }

  const emptyStateStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '300px',
    textAlign: 'center' as const,
    color: designTokens.colors.text.secondary,
  }

  const loadingStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '200px',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[3],
  }

  if (!selectedDocument) {
    return (
      <div className={`proxemic-document-intelligence ${className}`} style={containerStyles}>
        <div style={emptyStateStyles}>
          <Icon name="FileText" size={48} />
          <h3
            style={{
              margin: `${designTokens.spacing[3]} 0`,
              color: designTokens.colors.text.primary,
            }}
          >
            No Document Selected
          </h3>
          <p style={{ margin: `0 0 ${designTokens.spacing[4]}`, maxWidth: '250px' }}>
            Select or open a document to view intelligence insights and analysis.
          </p>
          <Button variant="primary" size="sm" onClick={handleDocumentChange}>
            Select Document
          </Button>
        </div>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className={`proxemic-document-intelligence ${className}`} style={containerStyles}>
        <div style={loadingStyles}>
          <Icon name="Loader" size={32} className="animate-spin" />
          <span>Analyzing document...</span>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className={`proxemic-document-intelligence ${className}`} style={containerStyles}>
        <div style={emptyStateStyles}>
          <Icon name="AlertCircle" size={48} color={designTokens.colors.accent.alert} />
          <h3
            style={{
              margin: `${designTokens.spacing[3]} 0`,
              color: designTokens.colors.accent.alert,
            }}
          >
            Analysis Failed
          </h3>
          <p style={{ margin: `0 0 ${designTokens.spacing[4]}`, maxWidth: '250px' }}>{error}</p>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => selectedDocument && loadAnalysis(selectedDocument)}
          >
            Retry Analysis
          </Button>
        </div>
      </div>
    )
  }

  if (!analysis) {
    return null
  }

  return (
    <div className={`proxemic-document-intelligence ${className}`} style={containerStyles}>
      {/* Header */}
      <div style={headerStyles}>
        <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
          <Icon name="FileText" size={20} />
          <span
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              fontWeight: designTokens.typography.fontWeight.medium,
            }}
          >
            Document Analysis
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          // icon="RefreshCcw"
          onClick={() => selectedDocument && loadAnalysis(selectedDocument)}
        >
          Refresh
        </Button>
      </div>

      {/* Analysis Content */}
      <div className="scroll-container" style={scrollContainerStyles}>
        {/* Structure Analysis */}
        <AnalysisCard
          title="Structure Analysis"
          icon="layout"
          score={analysis.structure.score}
          status={analysis.structure.completeness}
        >
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
              gap: designTokens.spacing[3],
              marginBottom: designTokens.spacing[3],
            }}
          >
            <div style={{ textAlign: 'center' }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.lg,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {analysis.structure.headings}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Headings
              </div>
            </div>
            <div style={{ textAlign: 'center' }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.lg,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {analysis.structure.sections}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Sections
              </div>
            </div>
          </div>
          <Progress
            value={analysis.structure.score}
            max={100}
            size="sm"
            // color={analysis.structure.score >= 80 ? 'success' : analysis.structure.score >= 60 ? 'warning' : 'error'}
          />
          {analysis.structure.issues.length > 0 && (
            <div style={{ marginTop: designTokens.spacing[3] }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.secondary,
                  marginBottom: designTokens.spacing[2],
                }}
              >
                Issues Found:
              </div>
              {analysis.structure.issues.map((issue, index) => (
                <div
                  key={index}
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.tertiary,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  • {issue}
                </div>
              ))}
            </div>
          )}
        </AnalysisCard>

        {/* Clarity Analysis */}
        <AnalysisCard
          title="Clarity & Readability"
          icon="eye"
          score={analysis.clarity.score}
          status={
            analysis.clarity.score >= 80
              ? 'excellent'
              : analysis.clarity.score >= 60
                ? 'good'
                : 'needs_improvement'
          }
        >
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(100px, 1fr))',
              gap: designTokens.spacing[2],
              marginBottom: designTokens.spacing[3],
            }}
          >
            <div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {analysis.clarity.readabilityLevel}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Reading Level
              </div>
            </div>
            <div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {analysis.clarity.avgSentenceLength}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Avg Sentence Length
              </div>
            </div>
            <div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {analysis.clarity.complexWords}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xs,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Complex Words
              </div>
            </div>
          </div>
          <Progress
            value={analysis.clarity.score}
            max={100}
            size="sm"
            // color={analysis.clarity.score >= 80 ? 'success' : analysis.clarity.score >= 60 ? 'warning' : 'error'}
          />
        </AnalysisCard>

        {/* Concept Mapping */}
        <AnalysisCard title="Concept Mapping" icon="share-2">
          <div style={{ marginBottom: designTokens.spacing[3] }}>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                fontWeight: designTokens.typography.fontWeight.semibold,
                marginBottom: designTokens.spacing[2],
                color: designTokens.colors.text.primary,
              }}
            >
              Key Terms
            </div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: designTokens.spacing[2] }}>
              {analysis.concepts.keyTerms.slice(0, 4).map((term, index) => (
                <Badge
                  key={index}
                  variant="default"
                  size="sm"
                  style={{
                    backgroundColor: `${designTokens.colors.accent.semantic}15`,
                    color: designTokens.colors.accent.semantic,
                    borderColor: `${designTokens.colors.accent.semantic}40`,
                  }}
                >
                  {term.term} ({term.frequency})
                </Badge>
              ))}
            </div>
          </div>
          <div>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                fontWeight: designTokens.typography.fontWeight.semibold,
                marginBottom: designTokens.spacing[2],
                color: designTokens.colors.text.primary,
              }}
            >
              Categories
            </div>
            {analysis.concepts.categories.map((category, index) => (
              <div
                key={index}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  marginBottom: designTokens.spacing[2],
                }}
              >
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.sm,
                    color: designTokens.colors.text.primary,
                  }}
                >
                  {category.category}
                </span>
                <span
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  {Math.round(category.confidence * 100)}%
                </span>
              </div>
            ))}
          </div>
        </AnalysisCard>

        {/* Gap Detection */}
        <AnalysisCard title="Gap Detection" icon="alert-triangle" status={analysis.gaps.severity}>
          {analysis.gaps.critical.length > 0 && (
            <div style={{ marginBottom: designTokens.spacing[3] }}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.confidence.critical,
                  marginBottom: designTokens.spacing[2],
                }}
              >
                Critical Issues
              </div>
              {analysis.gaps.critical.map((gap, index) => (
                <div
                  key={index}
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  • {gap}
                </div>
              ))}
            </div>
          )}
          {analysis.gaps.moderate.length > 0 && (
            <div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.confidence.medium,
                  marginBottom: designTokens.spacing[2],
                }}
              >
                Moderate Issues
              </div>
              {analysis.gaps.moderate.map((gap, index) => (
                <div
                  key={index}
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    color: designTokens.colors.text.primary,
                    marginBottom: designTokens.spacing[1],
                  }}
                >
                  • {gap}
                </div>
              ))}
            </div>
          )}
        </AnalysisCard>
      </div>

      {/* Custom Animations and Scrollbars */}
      <style>
        {`
          .animate-spin {
            animation: spin 1s linear infinite;
          }

          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }

          /* Custom scrollbar for scroll container */
          .proxemic-document-intelligence .scroll-container::-webkit-scrollbar {
            width: 6px;
          }

          .proxemic-document-intelligence .scroll-container::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .proxemic-document-intelligence .scroll-container::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }

          .proxemic-document-intelligence .scroll-container::-webkit-scrollbar-thumb:hover {
            background: ${designTokens.colors.border.strong};
          }
        `}
      </style>
    </div>
  )
}

export default React.memo(DocumentIntelligence)
