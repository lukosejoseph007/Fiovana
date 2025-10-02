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
  children: (doc: Y.Doc, provider: WebsocketProvider | WebrtcProvider | null, awareness: Awareness) => React.ReactNode
}

export interface ProviderStatus {
  connected: boolean
  synced: boolean
  error: Error | null
}

const YjsProvider: React.FC<YjsProviderProps> = ({
  documentId,
  username = 'Anonymous',
  userColor = '#' + Math.floor(Math.random() * 16777215).toString(16),
  onSync,
  onError,
  children,
}) => {
  const docRef = useRef<Y.Doc | null>(null)
  const providerRef = useRef<WebsocketProvider | WebrtcProvider | null>(null)
  const [, setStatus] = useState<ProviderStatus>({
    connected: false,
    synced: false,
    error: null,
  })

  useEffect(() => {
    // Create Yjs document
    const ydoc = new Y.Doc()
    docRef.current = ydoc

    // Try WebSocket first, fallback to WebRTC
    let provider: WebsocketProvider | WebrtcProvider | null = null

    try {
      // WebSocket provider configuration
      const wsUrl = (import.meta as { env?: { VITE_COLLABORATION_WS_URL?: string } }).env?.VITE_COLLABORATION_WS_URL || 'ws://localhost:1234'

      provider = new WebsocketProvider(wsUrl, documentId, ydoc, {
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
        setStatus(prev => ({ ...prev, connected }))
      })

      provider.on('sync', (isSynced: boolean) => {
        setStatus(prev => ({ ...prev, synced: isSynced }))
        onSync?.(isSynced)
      })

      // Error handling with WebRTC fallback
      provider.on('connection-error', (event: Error) => {
        console.warn('WebSocket connection failed, attempting WebRTC fallback...', event)

        // Cleanup WebSocket provider
        provider?.destroy()

        // Create WebRTC provider as fallback
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
          setStatus(prev => ({ ...prev, connected: true, error: null }))
        } catch (webrtcError) {
          const error = webrtcError instanceof Error ? webrtcError : new Error('WebRTC initialization failed')
          setStatus(prev => ({ ...prev, error }))
          onError?.(error)
        }
      })

      providerRef.current = provider
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Provider initialization failed')
      setStatus(prev => ({ ...prev, error: err }))
      onError?.(err)

      // Try WebRTC as immediate fallback
      try {
        provider = new WebrtcProvider(documentId, ydoc, {
          signaling: [
            'wss://signaling.yjs.dev',
            'wss://y-webrtc-signaling-eu.herokuapp.com',
            'wss://y-webrtc-signaling-us.herokuapp.com',
          ],
          awareness: new Awareness(ydoc),
        })

        provider.awareness.setLocalStateField('user', {
          name: username,
          color: userColor,
        })

        providerRef.current = provider
        setStatus(prev => ({ ...prev, connected: true, error: null }))
      } catch (webrtcError) {
        const error = webrtcError instanceof Error ? webrtcError : new Error('All providers failed')
        setStatus(prev => ({ ...prev, error }))
        onError?.(error)
      }
    }

    // Cleanup on unmount
    return () => {
      providerRef.current?.destroy()
      docRef.current?.destroy()
    }
  }, [documentId, username, userColor, onSync, onError])

  if (!docRef.current || !providerRef.current) {
    return null
  }

  return <>{children(docRef.current, providerRef.current, providerRef.current.awareness)}</>
}

export default YjsProvider
