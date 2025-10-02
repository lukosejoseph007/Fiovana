/**
 * MarkdownRenderer Component
 *
 * Professional markdown rendering with:
 * - GitHub Flavored Markdown support
 * - Syntax highlighting for code blocks
 * - Security sanitization
 * - Raw HTML support (sanitized)
 * - Custom styling
 */

import React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeRaw from 'rehype-raw'
import rehypeSanitize from 'rehype-sanitize'
import rehypeHighlight from 'rehype-highlight'
import '../../styles/markdown.css'
import 'highlight.js/styles/github.css' // GitHub-style code highlighting

interface MarkdownRendererProps {
  /**
   * The markdown content to render
   */
  content: string

  /**
   * Optional CSS class name
   */
  className?: string

  /**
   * Optional inline styles
   */
  style?: React.CSSProperties

  /**
   * Whether to enable raw HTML rendering (will be sanitized)
   * @default true
   */
  allowRawHtml?: boolean

  /**
   * Whether to enable syntax highlighting
   * @default true
   */
  enableSyntaxHighlight?: boolean

  /**
   * Whether to enable GitHub Flavored Markdown features
   * (tables, strikethrough, task lists, etc.)
   * @default true
   */
  enableGfm?: boolean
}

/**
 * MarkdownRenderer renders markdown content with professional styling
 * and comprehensive feature support including syntax highlighting,
 * GitHub Flavored Markdown, and security sanitization.
 */
const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({
  content,
  className = '',
  style,
  allowRawHtml = true,
  enableSyntaxHighlight = true,
  enableGfm = true,
}) => {
  // Build the rehype plugins array based on options
  const rehypePlugins = []

  // Add raw HTML support if enabled (must come before sanitize)
  if (allowRawHtml) {
    rehypePlugins.push(rehypeRaw)
  }

  // Always sanitize for security (removes dangerous HTML)
  rehypePlugins.push(rehypeSanitize)

  // Add syntax highlighting if enabled
  if (enableSyntaxHighlight) {
    rehypePlugins.push(rehypeHighlight)
  }

  // Build the remark plugins array
  const remarkPlugins = []

  // Add GitHub Flavored Markdown if enabled
  if (enableGfm) {
    remarkPlugins.push(remarkGfm)
  }

  return (
    <div className={`markdown-renderer markdown-body ${className}`} style={style}>
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        components={{
          // Custom component renderers for additional control
          a: ({ node: _node, ...props }) => (
            <a {...props} target="_blank" rel="noopener noreferrer" />
          ),
          // Add more custom components as needed
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}

export default MarkdownRenderer
