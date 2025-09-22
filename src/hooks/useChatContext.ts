import { useContext } from 'react'
import { ChatContext } from '../context/ChatContext'
import type { ChatContextType } from '../context/ChatContext'

// Hook to use the context
export const useChatContext = (): ChatContextType => {
  const context = useContext(ChatContext)
  if (context === undefined) {
    throw new Error('useChatContext must be used within a ChatProvider')
  }
  return context
}
