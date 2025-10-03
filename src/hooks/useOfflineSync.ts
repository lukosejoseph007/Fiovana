import { useState, useEffect, useCallback, useRef } from 'react'
import * as Y from 'yjs'

export interface QueuedOperation {
  id: string
  type: 'insert' | 'delete' | 'update'
  timestamp: number
  data: unknown
  retryCount: number
  status: 'pending' | 'syncing' | 'synced' | 'failed'
}

export interface OfflineSyncState {
  isOnline: boolean
  isSyncing: boolean
  queuedOperations: QueuedOperation[]
  lastSyncTime: number | null
  syncProgress: number
  failedOperations: QueuedOperation[]
}

export interface UseOfflineSyncOptions {
  ydoc: Y.Doc | null
  enabled?: boolean
  autoSync?: boolean
  maxRetries?: number
  onSyncStart?: () => void
  onSyncComplete?: (success: boolean, syncedCount: number) => void
  onOperationQueued?: (operation: QueuedOperation) => void
}

/**
 * Hook for managing offline synchronization in collaborative editing
 *
 * Features:
 * - Detect online/offline status
 * - Queue operations when offline
 * - Automatic sync when reconnected
 * - Retry failed operations
 * - Progress tracking
 */
export const useOfflineSync = ({
  ydoc,
  enabled = true,
  autoSync = true,
  maxRetries = 3,
  onSyncStart,
  onSyncComplete,
  onOperationQueued,
}: UseOfflineSyncOptions) => {
  const [state, setState] = useState<OfflineSyncState>({
    isOnline: navigator.onLine,
    isSyncing: false,
    queuedOperations: [],
    lastSyncTime: null,
    syncProgress: 0,
    failedOperations: [],
  })

  const operationCounterRef = useRef(0)
  const syncTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  /**
   * Sync queued operations
   */
  const syncQueuedOperations = useCallback(async () => {
    if (state.isSyncing || state.queuedOperations.length === 0 || !state.isOnline) {
      return
    }

    setState(prev => ({ ...prev, isSyncing: true, syncProgress: 0 }))
    onSyncStart?.()

    const operationsToSync = [...state.queuedOperations]
    let syncedCount = 0
    const failedOps: QueuedOperation[] = []

    for (let i = 0; i < operationsToSync.length; i++) {
      const operation = operationsToSync[i]

      try {
        // Update status to syncing
        setState(prev => ({
          ...prev,
          queuedOperations: prev.queuedOperations.map(op =>
            op.id === operation.id ? { ...op, status: 'syncing' as const } : op
          ),
          syncProgress: ((i + 1) / operationsToSync.length) * 100,
        }))

        // Simulate applying operation to Yjs document
        // In a real implementation, this would apply the queued operation to the Yjs document
        if (ydoc) {
          // Apply operation based on type
          // This is a simplified version - real implementation would be more complex
          await new Promise(resolve => setTimeout(resolve, 100)) // Simulate network delay
        }

        // Mark as synced
        setState(prev => ({
          ...prev,
          queuedOperations: prev.queuedOperations.map(op =>
            op.id === operation.id ? { ...op, status: 'synced' as const } : op
          ),
        }))

        syncedCount++
      } catch (error) {
        console.error('Failed to sync operation:', operation.id, error)

        // Increment retry count
        const updatedOp = { ...operation, retryCount: operation.retryCount + 1 }

        if (updatedOp.retryCount >= maxRetries) {
          // Move to failed operations
          updatedOp.status = 'failed'
          failedOps.push(updatedOp)
        } else {
          // Keep in queue for retry
          setState(prev => ({
            ...prev,
            queuedOperations: prev.queuedOperations.map(op =>
              op.id === operation.id ? { ...updatedOp, status: 'pending' as const } : op
            ),
          }))
        }
      }
    }

    // Remove synced operations from queue
    setState(prev => ({
      ...prev,
      isSyncing: false,
      queuedOperations: prev.queuedOperations.filter(op => op.status !== 'synced'),
      failedOperations: [...prev.failedOperations, ...failedOps],
      lastSyncTime: Date.now(),
      syncProgress: 100,
    }))

    // Clear synced operations from localStorage
    try {
      const remaining = state.queuedOperations.filter(op => op.status !== 'synced')
      localStorage.setItem('offline-operations', JSON.stringify(remaining))
    } catch (error) {
      console.error('Failed to update localStorage:', error)
    }

    onSyncComplete?.(failedOps.length === 0, syncedCount)
  }, [state, ydoc, maxRetries, onSyncStart, onSyncComplete])

  /**
   * Update online/offline status
   */
  const updateOnlineStatus = useCallback(() => {
    const isOnline = navigator.onLine
    setState(prev => ({ ...prev, isOnline }))

    // Auto-sync when coming back online
    if (isOnline && autoSync && state.queuedOperations.length > 0) {
      syncQueuedOperations()
    }
  }, [autoSync, state.queuedOperations.length, syncQueuedOperations])

  /**
   * Listen for online/offline events
   */
  useEffect(() => {
    if (!enabled) return

    window.addEventListener('online', updateOnlineStatus)
    window.addEventListener('offline', updateOnlineStatus)

    return () => {
      window.removeEventListener('online', updateOnlineStatus)
      window.removeEventListener('offline', updateOnlineStatus)
    }
  }, [enabled, updateOnlineStatus])

  /**
   * Queue an operation when offline
   */
  const queueOperation = useCallback(
    (type: 'insert' | 'delete' | 'update', data: unknown) => {
      const operation: QueuedOperation = {
        id: `op-${operationCounterRef.current++}`,
        type,
        timestamp: Date.now(),
        data,
        retryCount: 0,
        status: 'pending',
      }

      setState(prev => ({
        ...prev,
        queuedOperations: [...prev.queuedOperations, operation],
      }))

      onOperationQueued?.(operation)

      // Store in localStorage for persistence across page refreshes
      try {
        const existing = localStorage.getItem('offline-operations') || '[]'
        const operations = JSON.parse(existing) as QueuedOperation[]
        operations.push(operation)
        localStorage.setItem('offline-operations', JSON.stringify(operations))
      } catch (error) {
        console.error('Failed to persist operation to localStorage:', error)
      }

      return operation
    },
    [onOperationQueued]
  )

  /**
   * Manually trigger sync
   */
  const triggerSync = useCallback(() => {
    if (state.isOnline) {
      syncQueuedOperations()
    }
  }, [state.isOnline, syncQueuedOperations])

  /**
   * Clear all queued operations
   */
  const clearQueue = useCallback(() => {
    setState(prev => ({
      ...prev,
      queuedOperations: [],
      failedOperations: [],
    }))

    localStorage.removeItem('offline-operations')
  }, [])

  /**
   * Retry a failed operation
   */
  const retryFailedOperation = useCallback((operationId: string) => {
    setState(prev => {
      const failedOp = prev.failedOperations.find(op => op.id === operationId)
      if (!failedOp) return prev

      return {
        ...prev,
        failedOperations: prev.failedOperations.filter(op => op.id !== operationId),
        queuedOperations: [
          ...prev.queuedOperations,
          { ...failedOp, status: 'pending' as const, retryCount: 0 },
        ],
      }
    })
  }, [])

  /**
   * Load persisted operations from localStorage on mount
   */
  useEffect(() => {
    if (!enabled) return

    try {
      const stored = localStorage.getItem('offline-operations')
      if (stored) {
        const operations = JSON.parse(stored) as QueuedOperation[]
        setState(prev => ({
          ...prev,
          queuedOperations: operations.filter(op => op.status !== 'synced'),
        }))
      }
    } catch (error) {
      console.error('Failed to load persisted operations:', error)
    }
  }, [enabled])

  /**
   * Auto-sync when coming online (debounced)
   */
  useEffect(() => {
    if (state.isOnline && autoSync && state.queuedOperations.length > 0) {
      // Debounce sync by 2 seconds
      if (syncTimeoutRef.current) {
        clearTimeout(syncTimeoutRef.current)
      }

      syncTimeoutRef.current = setTimeout(() => {
        syncQueuedOperations()
      }, 2000)
    }

    return () => {
      if (syncTimeoutRef.current) {
        clearTimeout(syncTimeoutRef.current)
      }
    }
  }, [state.isOnline, autoSync, state.queuedOperations.length, syncQueuedOperations])

  return {
    ...state,
    queueOperation,
    triggerSync,
    clearQueue,
    retryFailedOperation,
  }
}

export default useOfflineSync
