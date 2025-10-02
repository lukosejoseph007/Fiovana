import { useEffect, useRef, useCallback } from 'react'

export interface AutoSaveOptions {
  enabled?: boolean
  delay?: number // milliseconds
  onAutoSave?: () => Promise<void>
  onError?: (error: Error) => void
}

export function useAutoSave(content: string, isDirty: boolean, options: AutoSaveOptions = {}) {
  const { enabled = true, delay = 5000, onAutoSave, onError } = options

  const timeoutRef = useRef<NodeJS.Timeout | null>(null)
  const isSavingRef = useRef(false)
  const lastContentRef = useRef(content)

  const triggerAutoSave = useCallback(async () => {
    if (isSavingRef.current || !onAutoSave) return

    isSavingRef.current = true

    try {
      await onAutoSave()
      lastContentRef.current = content
    } catch (error) {
      onError?.(error instanceof Error ? error : new Error('Auto-save failed'))
    } finally {
      isSavingRef.current = false
    }
  }, [content, onAutoSave, onError])

  useEffect(() => {
    // Clear existing timeout
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
      timeoutRef.current = null
    }

    // Only schedule auto-save if enabled, dirty, and content has changed
    if (enabled && isDirty && content !== lastContentRef.current) {
      timeoutRef.current = setTimeout(() => {
        triggerAutoSave()
      }, delay)
    }

    // Cleanup on unmount or when dependencies change
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
        timeoutRef.current = null
      }
    }
  }, [content, isDirty, enabled, delay, triggerAutoSave])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  const cancelAutoSave = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
      timeoutRef.current = null
    }
  }, [])

  return {
    cancelAutoSave,
  }
}
