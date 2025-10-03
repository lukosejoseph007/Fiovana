// src/hooks/useTextSelection.ts
import { useState, useEffect, useCallback } from 'react'

export interface TextSelection {
  text: string
  range: Range | null
  rect: DOMRect | null
}

/**
 * Hook to track text selection in a container
 */
export function useTextSelection(containerRef: React.RefObject<HTMLElement>) {
  const [selection, setSelection] = useState<TextSelection>({
    text: '',
    range: null,
    rect: null,
  })

  const updateSelection = useCallback(() => {
    const windowSelection = window.getSelection()

    if (!windowSelection || windowSelection.rangeCount === 0) {
      setSelection({ text: '', range: null, rect: null })
      return
    }

    const selectedText = windowSelection.toString().trim()

    if (!selectedText) {
      setSelection({ text: '', range: null, rect: null })
      return
    }

    // Check if selection is within our container
    const range = windowSelection.getRangeAt(0)
    const container = containerRef.current

    if (!container) {
      setSelection({ text: '', range: null, rect: null })
      return
    }

    // Verify selection is within container
    if (!container.contains(range.commonAncestorContainer)) {
      setSelection({ text: '', range: null, rect: null })
      return
    }

    const rect = range.getBoundingClientRect()

    setSelection({
      text: selectedText,
      range: range,
      rect: rect,
    })
  }, [containerRef])

  useEffect(() => {
    const handleSelectionChange = () => {
      updateSelection()
    }

    document.addEventListener('selectionchange', handleSelectionChange)

    return () => {
      document.removeEventListener('selectionchange', handleSelectionChange)
    }
  }, [updateSelection])

  const clearSelection = useCallback(() => {
    setSelection({ text: '', range: null, rect: null })
    window.getSelection()?.removeAllRanges()
  }, [])

  return {
    selection,
    clearSelection,
    hasSelection: selection.text.length > 0,
  }
}
