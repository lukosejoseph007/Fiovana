/* eslint-disable react-refresh/only-export-components */
import React, { createContext, useState, useCallback, ReactNode } from 'react'

export interface DocumentState {
  id: string
  content: string
  originalContent: string
  isDirty: boolean
  isSaving: boolean
  lastSaved: Date | null
  error: string | null
}

export interface DocumentEditorContextType {
  documentState: DocumentState | null
  setDocumentContent: (content: string) => void
  saveDocument: () => Promise<void>
  resetDocument: () => void
  initializeDocument: (id: string, content: string) => void
}

export const DocumentEditorContext = createContext<DocumentEditorContextType | undefined>(undefined)

export function DocumentEditorProvider({ children }: { children: ReactNode }) {
  const [documentState, setDocumentState] = useState<DocumentState | null>(null)

  const initializeDocument = useCallback((id: string, content: string) => {
    setDocumentState({
      id,
      content,
      originalContent: content,
      isDirty: false,
      isSaving: false,
      lastSaved: null,
      error: null,
    })
  }, [])

  const setDocumentContent = useCallback((content: string) => {
    setDocumentState(prev => {
      if (!prev) return null
      return {
        ...prev,
        content,
        isDirty: content !== prev.originalContent,
      }
    })
  }, [])

  const saveDocument = useCallback(async () => {
    if (!documentState) return

    setDocumentState(prev => {
      if (!prev) return null
      return { ...prev, isSaving: true, error: null }
    })

    try {
      // TODO: Implement actual save logic with Tauri backend
      // For now, simulate save operation
      await new Promise(resolve => setTimeout(resolve, 500))

      setDocumentState(prev => {
        if (!prev) return null
        return {
          ...prev,
          originalContent: prev.content,
          isDirty: false,
          isSaving: false,
          lastSaved: new Date(),
          error: null,
        }
      })
    } catch (error) {
      setDocumentState(prev => {
        if (!prev) return null
        return {
          ...prev,
          isSaving: false,
          error: error instanceof Error ? error.message : 'Failed to save document',
        }
      })
    }
  }, [documentState])

  const resetDocument = useCallback(() => {
    setDocumentState(prev => {
      if (!prev) return null
      return {
        ...prev,
        content: prev.originalContent,
        isDirty: false,
      }
    })
  }, [])

  return (
    <DocumentEditorContext.Provider
      value={{
        documentState,
        setDocumentContent,
        saveDocument,
        resetDocument,
        initializeDocument,
      }}
    >
      {children}
    </DocumentEditorContext.Provider>
  )
}
