import { useState, useCallback, useEffect } from 'react'

export interface DocumentStateOptions {
  onSave?: (content: string) => Promise<void>
  onDirtyChange?: (isDirty: boolean) => void
}

export function useDocumentState(initialContent: string, options: DocumentStateOptions = {}) {
  const [content, setContent] = useState(initialContent)
  const [originalContent, setOriginalContent] = useState(initialContent)
  const [isDirty, setIsDirty] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [lastSaved, setLastSaved] = useState<Date | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Update dirty state when content changes
  useEffect(() => {
    const dirty = content !== originalContent
    setIsDirty(dirty)
    options.onDirtyChange?.(dirty)
  }, [content, originalContent, options])

  const updateContent = useCallback((newContent: string) => {
    setContent(newContent)
  }, [])

  const save = useCallback(async () => {
    if (!isDirty || isSaving) return

    setIsSaving(true)
    setError(null)

    try {
      if (options.onSave) {
        await options.onSave(content)
      }

      setOriginalContent(content)
      setIsDirty(false)
      setLastSaved(new Date())
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to save document'
      setError(errorMessage)
      throw err
    } finally {
      setIsSaving(false)
    }
  }, [content, isDirty, isSaving, options])

  const reset = useCallback(() => {
    setContent(originalContent)
    setIsDirty(false)
    setError(null)
  }, [originalContent])

  const initialize = useCallback((newContent: string) => {
    setContent(newContent)
    setOriginalContent(newContent)
    setIsDirty(false)
    setLastSaved(null)
    setError(null)
  }, [])

  return {
    content,
    isDirty,
    isSaving,
    lastSaved,
    error,
    updateContent,
    save,
    reset,
    initialize,
  }
}
