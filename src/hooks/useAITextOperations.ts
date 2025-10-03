// src/hooks/useAITextOperations.ts
import { useState, useCallback } from 'react'
import {
  executeTextOperation,
  TextOperation,
  TextOperationResult,
  DocumentContext,
} from '../services/textOperationService'

export interface AITextOperationState {
  isLoading: boolean
  result: TextOperationResult | null
  error: string | null
}

/**
 * Hook to manage AI text operations
 */
export function useAITextOperations() {
  const [state, setState] = useState<AITextOperationState>({
    isLoading: false,
    result: null,
    error: null,
  })

  const execute = useCallback(
    async (text: string, operation: TextOperation, context?: DocumentContext) => {
      setState({ isLoading: true, result: null, error: null })

      try {
        const result = await executeTextOperation({
          text,
          operation,
          context,
        })

        setState({ isLoading: false, result, error: null })
        return result
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error'
        setState({ isLoading: false, result: null, error: errorMessage })
        throw error
      }
    },
    []
  )

  const reset = useCallback(() => {
    setState({ isLoading: false, result: null, error: null })
  }, [])

  return {
    ...state,
    execute,
    reset,
  }
}
