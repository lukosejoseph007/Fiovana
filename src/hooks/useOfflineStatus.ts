// Hook for offline status monitoring and management
import { useState, useEffect, useCallback } from 'react'
import { offlineSupport, OfflineStatus } from '../services'

export interface UseOfflineStatusReturn {
  status: OfflineStatus
  isOnline: boolean
  queueOperation: (operation: {
    type: 'document_update' | 'conversation_sync' | 'ai_request' | 'document_generation'
    payload: Record<string, unknown>
    maxRetries?: number
  }) => string
  processQueue: () => Promise<void>
  clearCache: () => void
  getCachedDocument: (id: string) => unknown
  getCachedConversation: (id: string) => unknown
  checkOllama: () => Promise<void>
}

export function useOfflineStatus(): UseOfflineStatusReturn {
  const [status, setStatus] = useState<OfflineStatus>(offlineSupport.getStatus())
  const [isOnline, setIsOnline] = useState<boolean>(offlineSupport.getOnlineStatus())

  useEffect(() => {
    // Subscribe to status changes
    const unsubscribe = offlineSupport.onStatusChange(newStatus => {
      setStatus(newStatus)
      setIsOnline(newStatus.isOnline)
    })

    // Initial status update
    setStatus(offlineSupport.getStatus())
    setIsOnline(offlineSupport.getOnlineStatus())

    return () => {
      unsubscribe()
    }
  }, [])

  const queueOperation = useCallback(
    (operation: {
      type: 'document_update' | 'conversation_sync' | 'ai_request' | 'document_generation'
      payload: Record<string, unknown>
      maxRetries?: number
    }) => {
      return offlineSupport.queueOperation({
        type: operation.type,
        payload: operation.payload,
        maxRetries: operation.maxRetries || 3,
      })
    },
    []
  )

  const processQueue = useCallback(async () => {
    await offlineSupport.processQueuedOperations()
  }, [])

  const clearCache = useCallback(() => {
    offlineSupport.clearAllData()
  }, [])

  const getCachedDocument = useCallback((id: string) => {
    return offlineSupport.getCachedDocument(id)
  }, [])

  const getCachedConversation = useCallback((id: string) => {
    return offlineSupport.getCachedConversation(id)
  }, [])

  const checkOllama = useCallback(async () => {
    await offlineSupport.checkOllamaStatus()
  }, [])

  return {
    status,
    isOnline,
    queueOperation,
    processQueue,
    clearCache,
    getCachedDocument,
    getCachedConversation,
    checkOllama,
  }
}
