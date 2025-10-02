import React from 'react'
import { designTokens } from '../../styles/tokens'

interface PlainTextRendererProps {
  content: string
  style?: React.CSSProperties
}

/**
 * PlainTextRenderer - Renders plain text documents with basic formatting
 *
 * Features:
 * - Preserves whitespace and line breaks
 * - Monospace font for code-like content
 * - Handles long lines with word wrapping
 * - Syntax-highlighted display for better readability
 */
const PlainTextRenderer: React.FC<PlainTextRendererProps> = ({ content, style }) => {
  return (
    <div
      style={{
        fontFamily: designTokens.typography.fonts.mono.join(', '),
        fontSize: designTokens.typography.fontSize.sm,
        lineHeight: designTokens.typography.lineHeight.relaxed,
        color: designTokens.colors.text.primary,
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        padding: designTokens.spacing[4],
        backgroundColor: designTokens.colors.surface.secondary,
        borderRadius: designTokens.borderRadius.md,
        border: `1px solid ${designTokens.colors.border.subtle}`,
        overflowX: 'auto',
        ...style,
      }}
    >
      {content}
    </div>
  )
}

export default PlainTextRenderer
