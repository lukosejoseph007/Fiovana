// Chat-specific types for enhanced chat management

export interface ChatMessage {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: Date
  intent?: string
  confidence?: number
  error?: string
  responses?: ChatMessage[]
  activeResponseIndex?: number
  parentMessageId?: string
}

export interface ChatSession {
  id: string
  title: string
  messages: ChatMessage[]
  createdAt: Date
  updatedAt: Date
  isActive: boolean
  summary?: string
  messageCount: number
}

export interface ChatState {
  sessions: ChatSession[]
  activeChatId: string | null
  isLoading: boolean
  aiStatus: 'unknown' | 'available' | 'unavailable'
  currentProvider: string
  currentModel: string
  sidebarCollapsed: boolean
}

export type ChatAction =
  // Session Management
  | { type: 'CHAT_CREATE_SESSION'; payload: { title?: string } }
  | { type: 'CHAT_SET_ACTIVE_SESSION'; payload: string }
  | { type: 'CHAT_DELETE_SESSION'; payload: string }
  | { type: 'CHAT_UPDATE_SESSION_TITLE'; payload: { sessionId: string; title: string } }
  | { type: 'CHAT_CLEAR_ALL_SESSIONS' }

  // Message Management
  | { type: 'CHAT_ADD_MESSAGE'; payload: { sessionId: string; message: ChatMessage } }
  | { type: 'CHAT_DELETE_MESSAGE'; payload: { sessionId: string; messageId: string } }
  | { type: 'CHAT_CLEAR_SESSION_MESSAGES'; payload: string }
  | {
      type: 'CHAT_ADD_RESPONSE'
      payload: { sessionId: string; messageId: string; response: ChatMessage }
    }
  | {
      type: 'CHAT_SET_ACTIVE_RESPONSE'
      payload: { sessionId: string; messageId: string; responseIndex: number }
    }

  // UI State
  | { type: 'CHAT_SET_LOADING'; payload: boolean }
  | { type: 'CHAT_SET_AI_STATUS'; payload: 'unknown' | 'available' | 'unavailable' }
  | { type: 'CHAT_SET_PROVIDER'; payload: string }
  | { type: 'CHAT_SET_MODEL'; payload: string }
  | { type: 'CHAT_TOGGLE_SIDEBAR' }
  | { type: 'CHAT_SET_SIDEBAR_COLLAPSED'; payload: boolean }

  // Persistence
  | { type: 'CHAT_LOAD_PERSISTED_STATE'; payload: Partial<ChatState> }
