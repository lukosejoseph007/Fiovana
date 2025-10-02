import { useContext } from 'react'
import CollaborationContext, { CollaborationContextValue } from './CollaborationContext'

export const useCollaboration = (): CollaborationContextValue => {
  const context = useContext(CollaborationContext)
  if (!context) {
    throw new Error('useCollaboration must be used within a CollaborationProvider')
  }
  return context
}
