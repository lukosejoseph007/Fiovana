import { useContext } from 'react'
import { DocumentEditorContext } from '../context/DocumentEditorContext'

export function useDocumentEditor() {
  const context = useContext(DocumentEditorContext)
  if (context === undefined) {
    throw new Error('useDocumentEditor must be used within a DocumentEditorProvider')
  }
  return context
}
