import React, { useEffect, useRef, useState } from 'react'
import * as Y from 'yjs'
import { WebsocketProvider } from 'y-websocket'
import { WebrtcProvider } from 'y-webrtc'
import { Awareness } from 'y-protocols/awareness'

export interface YjsProviderProps {
  documentId: string
  username?: string
  userColor?: string
  onSync?: (isSynced: boolean) => void
  onError?: (error: Error) => void
  onStatusChange?: (status: ProviderStatus) => void
  children: (
    doc: Y.Doc,
    provider: WebsocketProvider | WebrtcProvider | null,
    awareness: Awareness,
    status: ProviderStatus
  ) => React.ReactNode
}

export interface ProviderStatus {
  connected: boolean
  synced: boolean
  error: Error | null
  connectionState: 'disconnected' | 'connecting' | 'connected' | 'reconnecting'
  reconnectAttempts: number
}

const YjsProvider: React.FC<YjsProviderProps> = ({
  documentId,
  username = 'Anonymous',
  userColor = '#' + Math.floor(Math.random() * 16777215).toString(16),
  onSync,
  onError,
  onStatusChange,
  children,
}) => {
  const docRef = useRef<Y.Doc | null>(null)
  const providerRef = useRef<WebsocketProvider | WebrtcProvider | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const reconnectAttemptsRef = useRef(0)
  const maxReconnectAttempts = 10

  const [status, setStatus] = useState<ProviderStatus>({
    connected: false,
    synced: false,
    error: null,
    connectionState: 'disconnected',
    reconnectAttempts: 0,
  })

  // Notify parent of status changes
  useEffect(() => {
    onStatusChange?.(status)
  }, [status, onStatusChange])

  // Calculate exponential backoff delay (max 30 seconds)
  const getReconnectDelay = (attempt: number): number => {
    const baseDelay = 1000 // 1 second
    const maxDelay = 30000 // 30 seconds
    const delay = Math.min(baseDelay * Math.pow(2, attempt), maxDelay)
    // Add jitter (±20%)
    const jitter = delay * 0.2 * (Math.random() - 0.5)
    return delay + jitter
  }

  // WebRTC fallback function
  const tryWebRTCFallback = React.useCallback(() => {
    if (!docRef.current) return

    console.log('Attempting WebRTC fallback...')
    const ydoc = docRef.current

    try {
      const webrtcProvider = new WebrtcProvider(documentId, ydoc, {
        signaling: [
          'wss://signaling.yjs.dev',
          'wss://y-webrtc-signaling-eu.herokuapp.com',
          'wss://y-webrtc-signaling-us.herokuapp.com',
        ],
        awareness: new Awareness(ydoc),
      })

      webrtcProvider.awareness.setLocalStateField('user', {
        name: username,
        color: userColor,
      })

      providerRef.current = webrtcProvider
      setStatus(prev => ({
        ...prev,
        connected: true,
        connectionState: 'connected',
        error: null,
      }))
    } catch (webrtcError) {
      const error =
        webrtcError instanceof Error ? webrtcError : new Error('WebRTC initialization failed')
      console.error('WebRTC fallback failed:', error)
      setStatus(prev => ({ ...prev, error, connectionState: 'disconnected' }))
      onError?.(error)
    }
  }, [documentId, username, userColor, onError])

  // Initialize provider with reconnection logic
  const initializeProvider = React.useCallback(
    (isReconnect = false) => {
      // Update connection state
      setStatus(prev => ({
        ...prev,
        connectionState: isReconnect ? 'reconnecting' : 'connecting',
        reconnectAttempts: reconnectAttemptsRef.current,
      }))

      // Create Yjs document if not exists
      if (!docRef.current) {
        docRef.current = new Y.Doc()
      }
      const ydoc = docRef.current

      // WebSocket provider configuration
      const wsUrl =
        (import.meta as { env?: { VITE_COLLABORATION_WS_URL?: string } }).env
          ?.VITE_COLLABORATION_WS_URL || 'ws://localhost:1234'

      try {
        const provider = new WebsocketProvider(wsUrl, documentId, ydoc, {
          connect: true,
          awareness: new Awareness(ydoc),
        })

        // Set user info in awareness
        provider.awareness.setLocalStateField('user', {
          name: username,
          color: userColor,
        })

        // Connection handlers
        provider.on('status', (event: { status: string }) => {
          const connected = event.status === 'connected'
          if (connected) {
            reconnectAttemptsRef.current = 0
            setStatus(prev => ({
              ...prev,
              connected: true,
              connectionState: 'connected',
              reconnectAttempts: 0,
              error: null,
            }))
          } else {
            setStatus(prev => ({
              ...prev,
              connected: false,
              connectionState: 'disconnected',
            }))
          }
        })

        provider.on('sync', (isSynced: boolean) => {
          setStatus(prev => ({ ...prev, synced: isSynced }))
          onSync?.(isSynced)
        })

        // Error handling with reconnection
        provider.on('connection-error', (event: Error) => {
          console.warn('WebSocket connection error:', event)
          setStatus(prev => ({
            ...prev,
            connected: false,
            connectionState: 'disconnected',
            error: event,
          }))
          onError?.(event)

          // Attempt reconnection with exponential backoff
          if (reconnectAttemptsRef.current < maxReconnectAttempts) {
            reconnectAttemptsRef.current++
            const delay = getReconnectDelay(reconnectAttemptsRef.current)

            console.log(
              `Reconnecting in ${delay}ms (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})...`
            )

            reconnectTimeoutRef.current = setTimeout(() => {
              provider?.destroy()
              providerRef.current = null
              initializeProvider(true)
            }, delay)
          } else {
            console.error('Max reconnection attempts reached, falling back to WebRTC')
            // Try WebRTC as final fallback
            tryWebRTCFallback()
          }
        })

        // Connection close handler
        provider.on('connection-close', (event: { code: number }) => {
          console.log('WebSocket connection closed with code:', event.code)

          // Don't reconnect on normal closure (code 1000)
          if (event.code !== 1000 && reconnectAttemptsRef.current < maxReconnectAttempts) {
            reconnectAttemptsRef.current++
            const delay = getReconnectDelay(reconnectAttemptsRef.current)

            reconnectTimeoutRef.current = setTimeout(() => {
              provider?.destroy()
              providerRef.current = null
              initializeProvider(true)
            }, delay)
          }
        })

        providerRef.current = provider
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Provider initialization failed')
        console.error('Failed to initialize WebSocket provider:', err)
        setStatus(prev => ({ ...prev, error: err, connectionState: 'disconnected' }))
        onError?.(err)

        // Try WebRTC as fallback
        tryWebRTCFallback()
      }
    },
    [documentId, username, userColor, onSync, onError, tryWebRTCFallback]
  )

  useEffect(() => {
    // Initialize provider
    initializeProvider(false)

    // Cleanup on unmount
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      providerRef.current?.destroy()
      docRef.current?.destroy()
    }
  }, [initializeProvider])

  if (!docRef.current || !providerRef.current) {
    return null
  }

  return <>{children(docRef.current, providerRef.current, providerRef.current.awareness, status)}</>
}

export default YjsProvider
