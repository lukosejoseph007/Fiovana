import React, { createContext, useState, useCallback, ReactNode } from 'react'
import type { LexicalEditor } from 'lexical'

export interface CollaborationUser {
  id: string
  name: string
  color: string
  isTyping: boolean
  cursor?: {
    x: number
    y: number
  }
  lastSeen: number
}

export interface CollaborationSettings {
  enabled: boolean
  username: string
  userColor: string
  showPresence: boolean
  showCursors: boolean
}

export interface CollaborationContextValue {
  settings: CollaborationSettings
  activeDocumentId: string | null
  activeEditor: LexicalEditor | null
  users: Map<string, CollaborationUser>
  currentUserId: string
  isConnected: boolean
  updateSettings: (updates: Partial<CollaborationSettings>) => void
  setActiveDocument: (documentId: string | null, editor: LexicalEditor | null) => void
  enableCollaboration: (documentId: string, editor: LexicalEditor) => void
  disableCollaboration: () => void
  updateUserCursor: (userId: string, x: number, y: number) => void
  updateUserTyping: (userId: string, isTyping: boolean) => void
  addUser: (user: CollaborationUser) => void
  removeUser: (userId: string) => void
  setConnectionStatus: (connected: boolean) => void
}

const CollaborationContext = createContext<CollaborationContextValue | undefined>(undefined)

export interface CollaborationProviderProps {
  children: ReactNode
  defaultUsername?: string
  defaultUserColor?: string
}

export const CollaborationProvider: React.FC<CollaborationProviderProps> = ({
  children,
  defaultUsername = 'Anonymous',
  defaultUserColor = '#' + Math.floor(Math.random() * 16777215).toString(16),
}) => {
  const [settings, setSettings] = useState<CollaborationSettings>({
    enabled: false, // ✅ ENABLED BY DEFAULT for demo/testing
    username: defaultUsername,
    userColor: defaultUserColor,
    showPresence: true,
    showCursors: true,
  })

  const [activeDocumentId, setActiveDocumentId] = useState<string | null>(null)
  const [activeEditor, setActiveEditor] = useState<LexicalEditor | null>(null)
  const [currentUserId] = useState<string>(() => `user-${Date.now()}-${Math.random()}`)

  // Initialize with demo users for testing (you can see the UI even with one user)
  const [users, setUsers] = useState<Map<string, CollaborationUser>>(() => {
    const demoUsers = new Map<string, CollaborationUser>()
    // Add a demo collaborator so the UI is visible
    demoUsers.set('demo-user-1', {
      id: 'demo-user-1',
      name: 'Sarah Chen',
      color: '#10b981', // Green
      isTyping: false,
      cursor: { x: 100, y: 100 },
      lastSeen: Date.now(),
    })
    demoUsers.set('demo-user-2', {
      id: 'demo-user-2',
      name: 'John Smith',
      color: '#3b82f6', // Blue
      isTyping: true, // Show typing indicator
      cursor: { x: 200, y: 150 },
      lastSeen: Date.now(),
    })
    return demoUsers
  })

  const [isConnected, setIsConnected] = useState(true) // ✅ CONNECTED BY DEFAULT for demo

  const updateSettings = useCallback((updates: Partial<CollaborationSettings>) => {
    setSettings(prev => ({ ...prev, ...updates }))
  }, [])

  const setActiveDocument = useCallback(
    (documentId: string | null, editor: LexicalEditor | null) => {
      setActiveDocumentId(documentId)
      setActiveEditor(editor)
    },
    []
  )

  const enableCollaboration = useCallback((documentId: string, editor: LexicalEditor) => {
    setActiveDocumentId(documentId)
    setActiveEditor(editor)
    setSettings(prev => ({ ...prev, enabled: true }))
  }, [])

  const disableCollaboration = useCallback(() => {
    setActiveDocumentId(null)
    setActiveEditor(null)
    setSettings(prev => ({ ...prev, enabled: false }))
    setUsers(new Map())
    setIsConnected(false)
  }, [])

  const updateUserCursor = useCallback((userId: string, x: number, y: number) => {
    setUsers(prev => {
      const next = new Map(prev)
      const user = next.get(userId)
      if (user) {
        next.set(userId, { ...user, cursor: { x, y }, lastSeen: Date.now() })
      }
      return next
    })
  }, [])

  const updateUserTyping = useCallback((userId: string, isTyping: boolean) => {
    setUsers(prev => {
      const next = new Map(prev)
      const user = next.get(userId)
      if (user) {
        next.set(userId, { ...user, isTyping, lastSeen: Date.now() })
      }
      return next
    })
  }, [])

  const addUser = useCallback((user: CollaborationUser) => {
    setUsers(prev => {
      const next = new Map(prev)
      next.set(user.id, user)
      return next
    })
  }, [])

  const removeUser = useCallback((userId: string) => {
    setUsers(prev => {
      const next = new Map(prev)
      next.delete(userId)
      return next
    })
  }, [])

  const setConnectionStatus = useCallback((connected: boolean) => {
    setIsConnected(connected)
  }, [])

  const value: CollaborationContextValue = {
    settings,
    activeDocumentId,
    activeEditor,
    users,
    currentUserId,
    isConnected,
    updateSettings,
    setActiveDocument,
    enableCollaboration,
    disableCollaboration,
    updateUserCursor,
    updateUserTyping,
    addUser,
    removeUser,
    setConnectionStatus,
  }

  return <CollaborationContext.Provider value={value}>{children}</CollaborationContext.Provider>
}

export default CollaborationContext
