import React, { createContext, useState, useCallback, ReactNode } from 'react'
import type { LexicalEditor } from 'lexical'

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
  updateSettings: (updates: Partial<CollaborationSettings>) => void
  setActiveDocument: (documentId: string | null, editor: LexicalEditor | null) => void
  enableCollaboration: (documentId: string, editor: LexicalEditor) => void
  disableCollaboration: () => void
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
    enabled: false,
    username: defaultUsername,
    userColor: defaultUserColor,
    showPresence: true,
    showCursors: true,
  })

  const [activeDocumentId, setActiveDocumentId] = useState<string | null>(null)
  const [activeEditor, setActiveEditor] = useState<LexicalEditor | null>(null)

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

  const enableCollaboration = useCallback(
    (documentId: string, editor: LexicalEditor) => {
      setActiveDocumentId(documentId)
      setActiveEditor(editor)
      setSettings(prev => ({ ...prev, enabled: true }))
    },
    []
  )

  const disableCollaboration = useCallback(() => {
    setActiveDocumentId(null)
    setActiveEditor(null)
    setSettings(prev => ({ ...prev, enabled: false }))
  }, [])

  const value: CollaborationContextValue = {
    settings,
    activeDocumentId,
    activeEditor,
    updateSettings,
    setActiveDocument,
    enableCollaboration,
    disableCollaboration,
  }

  return <CollaborationContext.Provider value={value}>{children}</CollaborationContext.Provider>
}

export default CollaborationContext
