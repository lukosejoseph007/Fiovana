import { useState, useEffect, useRef, useCallback } from 'react'
import { executeTextOperation } from '../services/textOperationService'

interface SuggestionOptions {
  debounceMs?: number
  minCharsToTrigger?: number
  enabled?: boolean
  documentId?: string
  documentTitle?: string
}

interface AISuggestion {
  text: string
  confidence: number
  reasoning?: string
}

export function useAISuggestions(options: SuggestionOptions = {}) {
  const {
    debounceMs = 2000,
    minCharsToTrigger = 20,
    enabled = true,
    documentId,
    documentTitle,
  } = options

  const [suggestion, setSuggestion] = useState<AISuggestion | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [currentContext, setCurrentContext] = useState('')
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const abortControllerRef = useRef<AbortController | null>(null)

  // Generate AI suggestion based on context
  const generateSuggestion = useCallback(
    async (context: string) => {
      if (!enabled || context.length < minCharsToTrigger) {
        setSuggestion(null)
        return
      }

      console.log('AI Auto-Complete: Generating suggestion for context:', {
        contextLength: context.length,
        contextPreview: context.substring(Math.max(0, context.length - 50)),
      })

      // Cancel any pending request
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }

      abortControllerRef.current = new AbortController()
      setIsLoading(true)

      try {
        // Use "Expand" operation to generate continuation suggestions
        const result = await executeTextOperation({
          text: context,
          operation: { type: 'Expand' },
          context: {
            document_id: documentId,
            document_title: documentTitle,
          },
        })

        // Extract only the new content (remove the original context)
        let suggestionText = result.result
        if (suggestionText.startsWith(context)) {
          suggestionText = suggestionText.substring(context.length).trim()
        }

        // Only show suggestion if we got meaningful new content
        if (suggestionText && suggestionText.length > 5) {
          console.log('AI Auto-Complete: Suggestion generated successfully:', {
            suggestionLength: suggestionText.length,
            confidence: result.confidence,
          })
          setSuggestion({
            text: suggestionText,
            confidence: result.confidence || 0.5,
            reasoning: result.reasoning,
          })
        } else {
          console.log('AI Auto-Complete: Suggestion too short or empty, ignoring')
          setSuggestion(null)
        }
      } catch (error) {
        // Ignore aborted requests
        if (error instanceof Error && error.name === 'AbortError') {
          console.log('AI Auto-Complete: Request aborted')
          return
        }
        console.error('AI Auto-Complete: Error generating suggestion:', error)
        setSuggestion(null)
      } finally {
        setIsLoading(false)
        abortControllerRef.current = null
      }
    },
    [enabled, minCharsToTrigger, documentId, documentTitle]
  )

  // Debounced suggestion generation
  useEffect(() => {
    if (!enabled) {
      setSuggestion(null)
      return
    }

    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current)
    }

    // Only trigger if we have enough context
    if (currentContext.length >= minCharsToTrigger) {
      debounceTimerRef.current = setTimeout(() => {
        generateSuggestion(currentContext)
      }, debounceMs)
    } else {
      setSuggestion(null)
    }

    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current)
      }
    }
  }, [currentContext, enabled, debounceMs, minCharsToTrigger, generateSuggestion])

  // Update context (called from editor)
  const updateContext = useCallback((newContext: string) => {
    setCurrentContext(newContext)
  }, [])

  // Clear suggestion
  const clearSuggestion = useCallback(() => {
    setSuggestion(null)
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current)
    }
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }
  }, [])

  // Accept suggestion
  const acceptSuggestion = useCallback(() => {
    const accepted = suggestion
    setSuggestion(null)
    return accepted
  }, [suggestion])

  return {
    suggestion,
    isLoading,
    updateContext,
    clearSuggestion,
    acceptSuggestion,
  }
}
