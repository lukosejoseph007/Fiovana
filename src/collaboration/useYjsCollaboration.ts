import { useEffect, useState, useCallback, useRef } from 'react'
import * as Y from 'yjs'
import { WebsocketProvider } from 'y-websocket'
import { WebrtcProvider } from 'y-webrtc'
import type { LexicalEditor } from 'lexical'

export interface CollaborationUser {
  clientID: number
  name: string
  color: string
}

export interface CollaborationState {
  isConnected: boolean
  isSynced: boolean
  users: CollaborationUser[]
  error: Error | null
}

export interface UseYjsCollaborationOptions {
  documentId: string
  editor: LexicalEditor | null
  username?: string
  userColor?: string
  enabled?: boolean
}

/**
 * Hook for Yjs-based real-time collaboration
 *
 * Note: Full Lexical-Yjs integration requires additional setup.
 * This hook sets up the Yjs infrastructure. The actual Lexical binding
 * will be implemented in Phase 4.2 with custom plugins.
 */
export const useYjsCollaboration = ({
  documentId,
  editor,
  username = 'Anonymous',
  userColor = '#' + Math.floor(Math.random() * 16777215).toString(16),
  enabled = true,
}: UseYjsCollaborationOptions) => {
  const [state, setState] = useState<CollaborationState>({
    isConnected: false,
    isSynced: false,
    users: [],
    error: null,
  })

  const docRef = useRef<Y.Doc | null>(null)
  const providerRef = useRef<WebsocketProvider | WebrtcProvider | null>(null)

  // Update users from awareness
  const updateUsers = useCallback((awarenessStates: Map<number, Record<string, unknown>>) => {
    const users: CollaborationUser[] = []

    awarenessStates.forEach((state, clientID) => {
      if (state && typeof state === 'object' && 'name' in state) {
        users.push({
          clientID,
          name: String(state.name || 'Anonymous'),
          color: String(state.color || '#808080'),
        })
      }
    })

    setState(prev => ({ ...prev, users }))
  }, [])

  useEffect(() => {
    if (!enabled || !editor || !documentId) {
      return
    }

    // Create Yjs document
    const ydoc = new Y.Doc()
    docRef.current = ydoc

    let provider: WebsocketProvider | WebrtcProvider | null = null

    try {
      // WebSocket provider configuration
      const wsUrl =
        (import.meta as { env?: { VITE_COLLABORATION_WS_URL?: string } }).env
          ?.VITE_COLLABORATION_WS_URL || 'ws://localhost:1234'

      provider = new WebsocketProvider(wsUrl, documentId, ydoc, {
        connect: true,
      })

      // Set user info in awareness
      provider.awareness.setLocalStateField('name', username)
      provider.awareness.setLocalStateField('color', userColor)

      // Connection handlers
      provider.on('status', (event: { status: string }) => {
        const isConnected = event.status === 'connected'
        setState(prev => ({ ...prev, isConnected }))
      })

      provider.on('sync', (isSynced: boolean) => {
        setState(prev => ({ ...prev, isSynced }))
      })

      // Awareness change handler
      const awarenessChangeHandler = () => {
        updateUsers(provider!.awareness.getStates())
      }

      provider.awareness.on('change', awarenessChangeHandler)

      // Initial users update
      updateUsers(provider.awareness.getStates())

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
          })

          webrtcProvider.awareness.setLocalStateField('name', username)
          webrtcProvider.awareness.setLocalStateField('color', userColor)

          webrtcProvider.awareness.on('change', () => {
            updateUsers(webrtcProvider.awareness.getStates())
          })

          providerRef.current = webrtcProvider
          setState(prev => ({ ...prev, isConnected: true, error: null }))
        } catch (webrtcError) {
          const error =
            webrtcError instanceof Error ? webrtcError : new Error('WebRTC initialization failed')
          setState(prev => ({ ...prev, error }))
        }
      })

      providerRef.current = provider
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Provider initialization failed')
      setState(prev => ({ ...prev, error: err }))

      // Try WebRTC as immediate fallback
      try {
        provider = new WebrtcProvider(documentId, ydoc, {
          signaling: [
            'wss://signaling.yjs.dev',
            'wss://y-webrtc-signaling-eu.herokuapp.com',
            'wss://y-webrtc-signaling-us.herokuapp.com',
          ],
        })

        provider.awareness.setLocalStateField('name', username)
        provider.awareness.setLocalStateField('color', userColor)

        provider.awareness.on('change', () => {
          updateUsers(provider!.awareness.getStates())
        })

        providerRef.current = provider
        setState(prev => ({ ...prev, isConnected: true, error: null }))
      } catch (webrtcError) {
        const error = webrtcError instanceof Error ? webrtcError : new Error('All providers failed')
        setState(prev => ({ ...prev, error }))
      }
    }

    // Cleanup on unmount
    return () => {
      providerRef.current?.destroy()
      docRef.current?.destroy()
    }
  }, [documentId, editor, username, userColor, enabled, updateUsers])

  const disconnect = useCallback(() => {
    providerRef.current?.destroy()
    docRef.current?.destroy()
    setState({
      isConnected: false,
      isSynced: false,
      users: [],
      error: null,
    })
  }, [])

  return {
    ...state,
    disconnect,
    doc: docRef.current,
    provider: providerRef.current,
  }
}
