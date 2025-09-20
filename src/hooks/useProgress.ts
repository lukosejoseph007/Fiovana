// src/hooks/useProgress.ts
// React hook for progress tracking

import { useState, useEffect, useCallback, useRef } from 'react'
import { progressService } from '../services/progressService'
import type {
  ImportProgress,
  ProgressSummary,
  UseProgressReturn,
  UseOperationProgressReturn,
} from '../types/progress'
import { OperationStatus } from '../types/progress'

export function useProgress(): UseProgressReturn {
  const [operations, setOperations] = useState<ImportProgress[]>([])
  const [summary, setSummary] = useState<ProgressSummary>({
    active_operations: 0,
    completed_operations: 0,
    failed_operations: 0,
    total_files_processing: 0,
    total_files_completed: 0,
    overall_progress: 0,
  })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const isSubscribed = useRef(false)

  const refresh = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      const [operationsData, summaryData] = await Promise.all([
        progressService.getAllOperations(),
        progressService.getProgressSummary(),
      ])

      setOperations(operationsData)
      setSummary(summaryData)
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch progress data'
      setError(message)
      console.error('Error refreshing progress data:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  const cancelOperation = useCallback(
    async (operationId: string) => {
      try {
        const success = await progressService.cancelOperation(operationId)
        if (success) {
          // Update local state immediately for better UX
          setOperations(prev =>
            prev.map(op =>
              op.operation_id === operationId
                ? { ...op, status: OperationStatus.Cancelled, cancellable: false }
                : op
            )
          )
          // Refresh to get the actual state
          await refresh()
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to cancel operation'
        setError(message)
        console.error('Error cancelling operation:', err)
      }
    },
    [refresh]
  )

  const cleanupCompleted = useCallback(async () => {
    try {
      const cleanedCount = await progressService.cleanupCompletedOperations()
      console.log(`Cleaned up ${cleanedCount} completed operations`)
      await refresh()
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to cleanup operations'
      setError(message)
      console.error('Error cleaning up operations:', err)
    }
  }, [refresh])

  const subscribeToUpdates = useCallback(() => {
    if (isSubscribed.current) return

    isSubscribed.current = true

    // Subscribe to backend progress updates
    progressService
      .subscribeToUpdates()
      .then(() => {
        // Add listener for real-time updates
        const removeListener = progressService.addUpdateListener((progress: ImportProgress) => {
          setOperations(prev => {
            // Update or add the progress item
            const existingIndex = prev.findIndex(op => op.operation_id === progress.operation_id)

            if (existingIndex >= 0) {
              // Update existing operation
              const updated = [...prev]
              updated[existingIndex] = progress
              return updated
            } else {
              // Add new operation
              return [...prev, progress]
            }
          })

          // Update summary when operations change
          progressService.getProgressSummary().then(setSummary).catch(console.error)
        })

        // Store cleanup function
        return removeListener
      })
      .catch(err => {
        console.error('Failed to subscribe to progress updates:', err)
        setError('Failed to subscribe to real-time updates')
      })
  }, [])

  const unsubscribeFromUpdates = useCallback(() => {
    if (!isSubscribed.current) return

    isSubscribed.current = false
    progressService.unsubscribeFromUpdates()
  }, [])

  // Initial data load
  useEffect(() => {
    refresh()
  }, [refresh])

  // Auto-subscribe to updates
  useEffect(() => {
    subscribeToUpdates()

    // Cleanup on unmount
    return () => {
      unsubscribeFromUpdates()
    }
  }, [subscribeToUpdates, unsubscribeFromUpdates])

  return {
    operations,
    summary,
    loading,
    error,
    refresh,
    cancelOperation,
    cleanupCompleted,
    subscribeToUpdates,
    unsubscribeFromUpdates,
  }
}

export function useOperationProgress(operationId: string): UseOperationProgressReturn {
  const [progress, setProgress] = useState<ImportProgress | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const isSubscribed = useRef(false)

  const refresh = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      const progressData = await progressService.getOperationProgress(operationId)
      setProgress(progressData)
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch operation progress'
      setError(message)
      console.error('Error refreshing operation progress:', err)
    } finally {
      setLoading(false)
    }
  }, [operationId])

  const cancel = useCallback(async () => {
    try {
      const success = await progressService.cancelOperation(operationId)
      if (success) {
        // Update local state immediately
        setProgress(prev =>
          prev ? { ...prev, status: OperationStatus.Cancelled, cancellable: false } : null
        )
        // Refresh to get actual state
        await refresh()
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to cancel operation'
      setError(message)
      console.error('Error cancelling operation:', err)
    }
  }, [operationId, refresh])

  // Subscribe to updates for this specific operation
  useEffect(() => {
    if (isSubscribed.current) return

    isSubscribed.current = true

    const removeListener = progressService.addUpdateListener((progressUpdate: ImportProgress) => {
      if (progressUpdate.operation_id === operationId) {
        setProgress(progressUpdate)
      }
    })

    // Cleanup function
    return () => {
      isSubscribed.current = false
      removeListener()
    }
  }, [operationId])

  // Initial data load
  useEffect(() => {
    refresh()
  }, [refresh])

  return {
    progress,
    loading,
    error,
    refresh,
    cancel,
  }
}

// Hook for progress summary only (lighter weight)
export function useProgressSummary() {
  const [summary, setSummary] = useState<ProgressSummary>({
    active_operations: 0,
    completed_operations: 0,
    failed_operations: 0,
    total_files_processing: 0,
    total_files_completed: 0,
    overall_progress: 0,
  })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const summaryData = await progressService.getProgressSummary()
      setSummary(summaryData)
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch progress summary'
      setError(message)
      console.error('Error refreshing progress summary:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    refresh()

    // Set up periodic refresh for summary
    const interval = setInterval(refresh, 5000) // Refresh every 5 seconds

    return () => clearInterval(interval)
  }, [refresh])

  return { summary, loading, error, refresh }
}
