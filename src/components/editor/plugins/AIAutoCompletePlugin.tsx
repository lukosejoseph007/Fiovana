import React, { useEffect, useState, useRef } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import {
  $getSelection,
  $isRangeSelection,
  COMMAND_PRIORITY_LOW,
  KEY_TAB_COMMAND,
  KEY_ESCAPE_COMMAND,
  TextNode,
} from 'lexical'
import { $convertFromMarkdownString, TRANSFORMERS } from '@lexical/markdown'
import { useAISuggestions } from '../../../hooks/useAISuggestions'
import './AIAutoCompletePlugin.css'

interface AIAutoCompletePluginProps {
  enabled?: boolean
  documentId?: string
  documentTitle?: string
}

export function AIAutoCompletePlugin({
  enabled = true,
  documentId,
  documentTitle,
}: AIAutoCompletePluginProps): React.JSX.Element {
  const [editor] = useLexicalComposerContext()
  const [showSuggestion, setShowSuggestion] = useState(false)
  const [suggestionPosition, setSuggestionPosition] = useState({ top: 0, left: 0 })
  const [isDragging, setIsDragging] = useState(false)
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 })
  const suggestionRef = useRef<HTMLDivElement>(null)

  const { suggestion, isLoading, updateContext, clearSuggestion, acceptSuggestion } =
    useAISuggestions({
      enabled,
      debounceMs: 2000,
      minCharsToTrigger: 20,
      documentId,
      documentTitle,
    })

  // Track editor changes and update context
  useEffect(() => {
    if (!enabled) return

    return editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const selection = $getSelection()

        if (!$isRangeSelection(selection) || !selection.isCollapsed()) {
          clearSuggestion()
          setShowSuggestion(false)
          return
        }

        // Get the current paragraph or sentence as context
        const anchor = selection.anchor
        const node = anchor.getNode()

        if (node instanceof TextNode) {
          const text = node.getTextContent()
          const offset = anchor.offset

          // Get text before cursor as context
          const contextBefore = text.substring(0, offset).trim()

          if (contextBefore.length >= 20) {
            updateContext(contextBefore)
          } else {
            clearSuggestion()
            setShowSuggestion(false)
          }
        } else {
          clearSuggestion()
          setShowSuggestion(false)
        }
      })
    })
  }, [editor, enabled, updateContext, clearSuggestion])

  // Show suggestion when available
  useEffect(() => {
    if (!suggestion || !enabled) {
      setShowSuggestion(false)
      return
    }

    console.log('AI Auto-Complete: Suggestion received:', {
      text: suggestion.text.substring(0, 50) + '...',
      confidence: suggestion.confidence,
    })

    // Get cursor position for suggestion display
    editor.getEditorState().read(() => {
      const selection = $getSelection()

      if (!$isRangeSelection(selection) || !selection.isCollapsed()) {
        setShowSuggestion(false)
        return
      }

      const nativeSelection = window.getSelection()
      if (nativeSelection && nativeSelection.rangeCount > 0) {
        const range = nativeSelection.getRangeAt(0)
        const rect = range.getBoundingClientRect()

        // Calculate suggestion box dimensions (approximate)
        const suggestionWidth = 500 // max-width from CSS
        const suggestionHeight = 300 // approximate height
        const margin = 10

        // Calculate position with viewport boundary detection
        let top = rect.bottom + 8
        let left = rect.left

        // Check if suggestion would overflow right edge
        if (left + suggestionWidth > window.innerWidth - margin) {
          left = window.innerWidth - suggestionWidth - margin
        }

        // Ensure minimum left margin
        left = Math.max(margin, left)

        // Check if suggestion would overflow bottom edge
        if (top + suggestionHeight > window.innerHeight - margin) {
          // Position above cursor if there's not enough space below
          top = rect.top - suggestionHeight - 8
        }

        // Ensure minimum top margin
        top = Math.max(margin, top)

        setSuggestionPosition({ top, left })
        setShowSuggestion(true)
      }
    })
  }, [editor, suggestion, enabled])

  // Handle Tab key to accept suggestion
  useEffect(() => {
    return editor.registerCommand(
      KEY_TAB_COMMAND,
      (event: KeyboardEvent) => {
        if (!showSuggestion || !suggestion) {
          return false
        }

        event.preventDefault()

        // Accept the suggestion
        const accepted = acceptSuggestion()
        if (accepted) {
          editor.update(() => {
            const selection = $getSelection()
            if ($isRangeSelection(selection)) {
              // Insert the suggestion text
              $convertFromMarkdownString(accepted.text, TRANSFORMERS)
            }
          })
        }

        setShowSuggestion(false)
        return true
      },
      COMMAND_PRIORITY_LOW
    )
  }, [editor, showSuggestion, suggestion, acceptSuggestion])

  // Handle Escape key to dismiss suggestion
  useEffect(() => {
    return editor.registerCommand(
      KEY_ESCAPE_COMMAND,
      () => {
        if (!showSuggestion) {
          return false
        }

        clearSuggestion()
        setShowSuggestion(false)
        return true
      },
      COMMAND_PRIORITY_LOW
    )
  }, [editor, showSuggestion, clearSuggestion])

  // Handle click outside to dismiss
  useEffect(() => {
    if (!showSuggestion) return

    const handleClickOutside = (event: MouseEvent) => {
      if (suggestionRef.current && !suggestionRef.current.contains(event.target as Node)) {
        clearSuggestion()
        setShowSuggestion(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [showSuggestion, clearSuggestion])

  // Handle dragging
  useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (event: MouseEvent) => {
      event.preventDefault()
      setSuggestionPosition({
        top: event.clientY - dragOffset.y,
        left: event.clientX - dragOffset.x,
      })
    }

    const handleMouseUp = () => {
      setIsDragging(false)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, dragOffset])

  // Handle drag start
  const handleDragStart = (event: React.MouseEvent<HTMLDivElement>) => {
    // Only start drag if clicking on the header area
    const target = event.target as HTMLElement
    if (
      target.classList.contains('suggestion-header') ||
      target.classList.contains('suggestion-icon') ||
      target.classList.contains('suggestion-label')
    ) {
      event.preventDefault()
      const rect = suggestionRef.current?.getBoundingClientRect()
      if (rect) {
        setDragOffset({
          x: event.clientX - rect.left,
          y: event.clientY - rect.top,
        })
        setIsDragging(true)
      }
    }
  }

  // Don't render if not enabled or no suggestion
  if (!enabled || !showSuggestion || !suggestion) {
    return <></>
  }

  return (
    <div
      ref={suggestionRef}
      className={`ai-autocomplete-suggestion ${isDragging ? 'dragging' : ''}`}
      style={{
        top: `${suggestionPosition.top}px`,
        left: `${suggestionPosition.left}px`,
      }}
    >
      <div className="suggestion-header" onMouseDown={handleDragStart}>
        <div className="suggestion-icon">
          {isLoading ? (
            <span className="loading-spinner">⏳</span>
          ) : (
            <span className="ai-icon">✨</span>
          )}
        </div>
        <span className="suggestion-label">AI Suggestion</span>
        <span className="drag-hint">⋮⋮</span>
        <span className="suggestion-hint">(Tab to accept, Esc to dismiss)</span>
      </div>

      <div className="suggestion-content">
        <div className="suggestion-text">{suggestion.text}</div>

        {suggestion.confidence !== undefined && (
          <div className="suggestion-confidence">
            <div className="confidence-bar-container">
              <div
                className="confidence-bar"
                style={{
                  width: `${suggestion.confidence * 100}%`,
                  backgroundColor:
                    suggestion.confidence > 0.7
                      ? 'var(--success-color, #10b981)'
                      : suggestion.confidence > 0.4
                        ? 'var(--warning-color, #f59e0b)'
                        : 'var(--error-color, #ef4444)',
                }}
              />
            </div>
            <span className="confidence-text">
              {(suggestion.confidence * 100).toFixed(0)}% confidence
            </span>
          </div>
        )}
      </div>

      <div className="suggestion-actions">
        <button
          type="button"
          className="suggestion-accept-button"
          onClick={() => {
            const accepted = acceptSuggestion()
            if (accepted) {
              editor.update(() => {
                const selection = $getSelection()
                if ($isRangeSelection(selection)) {
                  $convertFromMarkdownString(accepted.text, TRANSFORMERS)
                }
              })
            }
            setShowSuggestion(false)
          }}
        >
          ✓ Accept
        </button>
        <button
          type="button"
          className="suggestion-dismiss-button"
          onClick={() => {
            clearSuggestion()
            setShowSuggestion(false)
          }}
        >
          ✕ Dismiss
        </button>
      </div>
    </div>
  )
}

export default AIAutoCompletePlugin
