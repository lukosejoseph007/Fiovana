import React from 'react'
import MarkdownRenderer from './MarkdownRenderer'
import PlainTextRenderer from './PlainTextRenderer'
import { Icon } from '../ui'
import { designTokens } from '../../styles/tokens'

interface DocumentRendererProps {
  content: string
  documentType?: string
  documentName?: string
  style?: React.CSSProperties
}

/**
 * DocumentRenderer - Smart router for document rendering
 *
 * Automatically detects document type and routes to the appropriate renderer:
 * - Markdown files (.md, .markdown) → MarkdownRenderer
 * - PDF files (.pdf) → PDF.js viewer (future implementation)
 * - DOCX files (.docx, .doc) → Convert to HTML/Markdown (future implementation)
 * - Plain text files (.txt, unknown) → PlainTextRenderer
 *
 * Features:
 * - Type detection from file extension or MIME type
 * - Graceful fallback to plain text for unknown types
 * - Consistent styling across all renderers
 */
const DocumentRenderer: React.FC<DocumentRendererProps> = ({
  content,
  documentType,
  documentName,
  style,
}) => {
  /**
   * Detect document format from type or file name
   */
  const detectFormat = (): 'markdown' | 'pdf' | 'docx' | 'plaintext' => {
    // Check file name extension FIRST (most reliable)
    if (documentName) {
      const lowerName = documentName.toLowerCase()

      if (lowerName.endsWith('.md') || lowerName.endsWith('.markdown')) {
        return 'markdown'
      }

      if (lowerName.endsWith('.pdf')) {
        return 'pdf'
      }

      if (lowerName.endsWith('.docx') || lowerName.endsWith('.doc')) {
        return 'docx'
      }
    }

    // Check document type (MIME type or extension)
    if (documentType) {
      const lowerType = documentType.toLowerCase()

      // Markdown detection
      if (
        lowerType.includes('markdown') ||
        lowerType === 'md' ||
        lowerType === '.md' ||
        lowerType === 'text/markdown' ||
        lowerType === 'text/x-markdown'
      ) {
        return 'markdown'
      }

      // PDF detection
      if (lowerType.includes('pdf') || lowerType === 'application/pdf') {
        return 'pdf'
      }

      // DOCX detection
      if (
        lowerType.includes('docx') ||
        lowerType.includes('doc') ||
        lowerType === 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' ||
        lowerType === 'application/msword'
      ) {
        return 'docx'
      }
    }

    // Content-based detection as fallback
    if (content) {
      const contentStart = content.substring(0, 500)
      // Check for markdown patterns
      if (
        /^#{1,6}\s/.test(contentStart) || // Headers
        /\*\*.*?\*\*/.test(contentStart) || // Bold
        /\[.*?\]\(.*?\)/.test(contentStart) || // Links
        /^[-*+]\s/m.test(contentStart) // Lists
      ) {
        return 'markdown'
      }
    }

    // Default to plain text for unknown types
    return 'plaintext'
  }

  const format = detectFormat()

  /**
   * Render based on detected format
   */
  switch (format) {
    case 'markdown':
      return <MarkdownRenderer content={content} style={style} />

    case 'pdf':
      // PDF rendering will be implemented in a future phase
      // For now, show a placeholder
      return (
        <div
          style={{
            padding: designTokens.spacing[8],
            textAlign: 'center',
            ...style,
          }}
        >
          <div
            style={{
              display: 'inline-flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: designTokens.spacing[4],
              padding: designTokens.spacing[8],
              backgroundColor: designTokens.colors.surface.secondary,
              borderRadius: designTokens.borderRadius.lg,
              border: `1px solid ${designTokens.colors.border.subtle}`,
            }}
          >
            <Icon name="Document" size="2xl" />
            <div
              style={{
                fontSize: designTokens.typography.fontSize.lg,
                fontWeight: designTokens.typography.fontWeight.medium,
                color: designTokens.colors.text.primary,
              }}
            >
              PDF Viewer
            </div>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                maxWidth: '400px',
              }}
            >
              PDF rendering support is coming in a future update. For now, the extracted text
              content is shown below.
            </div>
            <div
              style={{
                marginTop: designTokens.spacing[4],
                width: '100%',
              }}
            >
              <PlainTextRenderer content={content} />
            </div>
          </div>
        </div>
      )

    case 'docx':
      // DOCX rendering will be implemented in a future phase
      // For now, show a placeholder with extracted text
      return (
        <div
          style={{
            padding: designTokens.spacing[8],
            textAlign: 'center',
            ...style,
          }}
        >
          <div
            style={{
              display: 'inline-flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: designTokens.spacing[4],
              padding: designTokens.spacing[8],
              backgroundColor: designTokens.colors.surface.secondary,
              borderRadius: designTokens.borderRadius.lg,
              border: `1px solid ${designTokens.colors.border.subtle}`,
            }}
          >
            <Icon name="Document" size="2xl" />
            <div
              style={{
                fontSize: designTokens.typography.fontSize.lg,
                fontWeight: designTokens.typography.fontWeight.medium,
                color: designTokens.colors.text.primary,
              }}
            >
              Word Document Viewer
            </div>
            <div
              style={{
                fontSize: designTokens.typography.fontSize.sm,
                color: designTokens.colors.text.secondary,
                maxWidth: '400px',
              }}
            >
              Word document rendering with full formatting support is coming in a future update. For
              now, the extracted text content is shown below.
            </div>
            <div
              style={{
                marginTop: designTokens.spacing[4],
                width: '100%',
              }}
            >
              <PlainTextRenderer content={content} />
            </div>
          </div>
        </div>
      )

    case 'plaintext':
    default:
      return <PlainTextRenderer content={content} style={style} />
  }
}

export default DocumentRenderer
