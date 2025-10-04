import React, { useState, useEffect, useRef, useCallback } from 'react'
import { X, ArrowLeft, ArrowRight, Copy, Check } from 'lucide-react'
import Button from '../ui/Button'

interface DiffLine {
  type: 'added' | 'removed' | 'unchanged' | 'modified'
  oldLineNumber: number | null
  newLineNumber: number | null
  oldContent: string
  newContent: string
}

interface DocumentDiffProps {
  oldContent: string
  newContent: string
  oldTitle?: string
  newTitle?: string
  onClose: () => void
  onMergeLeft?: () => void
  onMergeRight?: () => void
}

/**
 * DocumentDiff - Side-by-side diff view for comparing document versions
 *
 * Features:
 * - Two-pane side-by-side comparison
 * - Line-by-line diff highlighting
 * - Synchronized scrolling between panes
 * - Merge changes from either side
 * - Copy content functionality
 * - Statistics display
 */
export const DocumentDiff: React.FC<DocumentDiffProps> = ({
  oldContent,
  newContent,
  oldTitle = 'Original Version',
  newTitle = 'New Version',
  onClose,
  onMergeLeft,
  onMergeRight,
}) => {
  const [diffLines, setDiffLines] = useState<DiffLine[]>([])
  const [stats, setStats] = useState({ added: 0, removed: 0, modified: 0, unchanged: 0 })
  const [copiedSide, setCopiedSide] = useState<'left' | 'right' | null>(null)

  const leftPaneRef = useRef<HTMLDivElement>(null)
  const rightPaneRef = useRef<HTMLDivElement>(null)
  const syncScrolling = useRef(true)

  // Calculate diff
  useEffect(() => {
    const oldLines = oldContent.split('\n')
    const newLines = newContent.split('\n')
    const lines: DiffLine[] = []
    let addedCount = 0
    let removedCount = 0
    let modifiedCount = 0
    let unchangedCount = 0

    // Simple line-by-line comparison
    const maxLength = Math.max(oldLines.length, newLines.length)

    for (let i = 0; i < maxLength; i++) {
      const oldLine = oldLines[i] ?? ''
      const newLine = newLines[i] ?? ''

      if (i >= oldLines.length) {
        // Line added
        lines.push({
          type: 'added',
          oldLineNumber: null,
          newLineNumber: i + 1,
          oldContent: '',
          newContent: newLine,
        })
        addedCount++
      } else if (i >= newLines.length) {
        // Line removed
        lines.push({
          type: 'removed',
          oldLineNumber: i + 1,
          newLineNumber: null,
          oldContent: oldLine,
          newContent: '',
        })
        removedCount++
      } else if (oldLine === newLine) {
        // Line unchanged
        lines.push({
          type: 'unchanged',
          oldLineNumber: i + 1,
          newLineNumber: i + 1,
          oldContent: oldLine,
          newContent: newLine,
        })
        unchangedCount++
      } else {
        // Line modified
        lines.push({
          type: 'modified',
          oldLineNumber: i + 1,
          newLineNumber: i + 1,
          oldContent: oldLine,
          newContent: newLine,
        })
        modifiedCount++
      }
    }

    setDiffLines(lines)
    setStats({
      added: addedCount,
      removed: removedCount,
      modified: modifiedCount,
      unchanged: unchangedCount,
    })
  }, [oldContent, newContent])

  // Synchronized scrolling
  const handleScroll = useCallback((source: 'left' | 'right') => {
    if (!syncScrolling.current) return

    const sourcePane = source === 'left' ? leftPaneRef.current : rightPaneRef.current
    const targetPane = source === 'left' ? rightPaneRef.current : leftPaneRef.current

    if (sourcePane && targetPane) {
      // Temporarily disable sync to prevent infinite loop
      syncScrolling.current = false
      targetPane.scrollTop = sourcePane.scrollTop
      setTimeout(() => {
        syncScrolling.current = true
      }, 50)
    }
  }, [])

  // Copy content to clipboard
  const handleCopy = useCallback(
    async (side: 'left' | 'right') => {
      const content = side === 'left' ? oldContent : newContent
      try {
        await navigator.clipboard.writeText(content)
        setCopiedSide(side)
        setTimeout(() => setCopiedSide(null), 2000)
      } catch (err) {
        console.error('Failed to copy:', err)
      }
    },
    [oldContent, newContent]
  )

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-2xl w-full max-w-7xl max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              Document Comparison
            </h2>

            {/* Statistics */}
            <div className="flex items-center gap-3 text-sm">
              {stats.added > 0 && (
                <span className="px-2 py-1 rounded bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300">
                  +{stats.added} added
                </span>
              )}
              {stats.removed > 0 && (
                <span className="px-2 py-1 rounded bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300">
                  -{stats.removed} removed
                </span>
              )}
              {stats.modified > 0 && (
                <span className="px-2 py-1 rounded bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300">
                  ~{stats.modified} modified
                </span>
              )}
              <span className="text-gray-500 dark:text-gray-400">{stats.unchanged} unchanged</span>
            </div>
          </div>

          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
            className="hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <X className="w-5 h-5" />
          </Button>
        </div>

        {/* Comparison panes */}
        <div className="flex-1 flex overflow-hidden">
          {/* Left pane - Original */}
          <div className="flex-1 flex flex-col border-r border-gray-200 dark:border-gray-700">
            {/* Left header */}
            <div className="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
              <h3 className="font-medium text-gray-900 dark:text-white">{oldTitle}</h3>
              <div className="flex items-center gap-2">
                {onMergeRight && (
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={onMergeRight}
                    title="Use this version"
                  >
                    <ArrowRight className="w-4 h-4 mr-1" />
                    Use this
                  </Button>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleCopy('left')}
                  title="Copy to clipboard"
                >
                  {copiedSide === 'left' ? (
                    <Check className="w-4 h-4 text-green-600" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </Button>
              </div>
            </div>

            {/* Left content */}
            <div
              ref={leftPaneRef}
              onScroll={() => handleScroll('left')}
              className="flex-1 overflow-auto font-mono text-sm"
            >
              {diffLines.map((line, index) => (
                <div
                  key={index}
                  className={`flex ${
                    line.type === 'removed'
                      ? 'bg-red-50 dark:bg-red-900/10'
                      : line.type === 'modified'
                        ? 'bg-yellow-50 dark:bg-yellow-900/10'
                        : ''
                  }`}
                >
                  <div className="w-12 flex-shrink-0 px-2 py-1 text-gray-400 dark:text-gray-600 text-right border-r border-gray-200 dark:border-gray-700">
                    {line.oldLineNumber || ''}
                  </div>
                  <div
                    className={`flex-1 px-3 py-1 ${
                      line.type === 'removed'
                        ? 'text-red-700 dark:text-red-300'
                        : line.type === 'modified'
                          ? 'text-yellow-700 dark:text-yellow-300'
                          : 'text-gray-700 dark:text-gray-300'
                    }`}
                  >
                    {line.oldContent || '\u00A0'}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Right pane - New */}
          <div className="flex-1 flex flex-col">
            {/* Right header */}
            <div className="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
              <h3 className="font-medium text-gray-900 dark:text-white">{newTitle}</h3>
              <div className="flex items-center gap-2">
                {onMergeLeft && (
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={onMergeLeft}
                    title="Use this version"
                  >
                    <ArrowLeft className="w-4 h-4 mr-1" />
                    Use this
                  </Button>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleCopy('right')}
                  title="Copy to clipboard"
                >
                  {copiedSide === 'right' ? (
                    <Check className="w-4 h-4 text-green-600" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </Button>
              </div>
            </div>

            {/* Right content */}
            <div
              ref={rightPaneRef}
              onScroll={() => handleScroll('right')}
              className="flex-1 overflow-auto font-mono text-sm"
            >
              {diffLines.map((line, index) => (
                <div
                  key={index}
                  className={`flex ${
                    line.type === 'added'
                      ? 'bg-green-50 dark:bg-green-900/10'
                      : line.type === 'modified'
                        ? 'bg-yellow-50 dark:bg-yellow-900/10'
                        : ''
                  }`}
                >
                  <div className="w-12 flex-shrink-0 px-2 py-1 text-gray-400 dark:text-gray-600 text-right border-r border-gray-200 dark:border-gray-700">
                    {line.newLineNumber || ''}
                  </div>
                  <div
                    className={`flex-1 px-3 py-1 ${
                      line.type === 'added'
                        ? 'text-green-700 dark:text-green-300'
                        : line.type === 'modified'
                          ? 'text-yellow-700 dark:text-yellow-300'
                          : 'text-gray-700 dark:text-gray-300'
                    }`}
                  >
                    {line.newContent || '\u00A0'}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-3 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            Total lines: {diffLines.length} | Changes:{' '}
            {stats.added + stats.removed + stats.modified}
          </div>
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>
    </div>
  )
}
