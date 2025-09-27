import React, { createContext, useReducer, useEffect } from 'react'
import type { ChatState, ChatAction, ChatSession, ChatMessage } from './chatTypes'

// Initial state
const initialChatState: ChatState = {
  sessions: [],
  activeChatId: null,
  isLoading: false,
  aiStatus: 'unknown',
  currentProvider: 'local',
  currentModel: '',
  sidebarCollapsed: false,
}

// Utility functions
const generateSessionId = (): string =>
  `chat_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

const generateSessionTitle = (messages: ChatMessage[]): string => {
  if (messages.length === 0) return 'New Chat'

  const firstUserMessage = messages.find(m => m.type === 'user')
  if (!firstUserMessage) return 'New Chat'

  // Use first 50 characters of the first user message as title
  const title = firstUserMessage.content.trim().slice(0, 50)
  return title.length < firstUserMessage.content.trim().length ? `${title}...` : title
}

const createNewSession = (title?: string): ChatSession => {
  const now = new Date()
  return {
    id: generateSessionId(),
    title: title || 'New Chat',
    messages: [],
    createdAt: now,
    updatedAt: now,
    isActive: false,
    messageCount: 0,
  }
}

// Reducer
function chatReducer(state: ChatState, action: ChatAction): ChatState {
  switch (action.type) {
    case 'CHAT_CREATE_SESSION': {
      const newSession = createNewSession(action.payload.title)
      const sessions = [newSession, ...state.sessions]
      return {
        ...state,
        sessions,
        activeChatId: newSession.id,
      }
    }

    case 'CHAT_SET_ACTIVE_SESSION': {
      return {
        ...state,
        activeChatId: action.payload,
      }
    }

    case 'CHAT_DELETE_SESSION': {
      const sessions = state.sessions.filter(s => s.id !== action.payload)
      let activeChatId = state.activeChatId

      // If we deleted the active session, switch to the first remaining session
      if (state.activeChatId === action.payload) {
        activeChatId = sessions.length > 0 ? sessions[0]?.id || null : null
      }

      return {
        ...state,
        sessions,
        activeChatId,
      }
    }

    case 'CHAT_UPDATE_SESSION_TITLE': {
      const sessions = state.sessions.map(session =>
        session.id === action.payload.sessionId
          ? { ...session, title: action.payload.title, updatedAt: new Date() }
          : session
      )
      return {
        ...state,
        sessions,
      }
    }

    case 'CHAT_CLEAR_ALL_SESSIONS': {
      return {
        ...state,
        sessions: [],
        activeChatId: null,
      }
    }

    case 'CHAT_ADD_MESSAGE': {
      const { sessionId, message } = action.payload
      const sessions = state.sessions.map(session => {
        if (session.id !== sessionId) return session

        const messages = [...session.messages, message]
        const title = session.title === 'New Chat' ? generateSessionTitle(messages) : session.title

        return {
          ...session,
          messages,
          title,
          updatedAt: new Date(),
          messageCount: messages.length,
        }
      })

      return {
        ...state,
        sessions,
      }
    }

    case 'CHAT_DELETE_MESSAGE': {
      const { sessionId, messageId } = action.payload
      const sessions = state.sessions.map(session => {
        if (session.id !== sessionId) return session

        const messages = session.messages.filter(m => m.id !== messageId)

        return {
          ...session,
          messages,
          updatedAt: new Date(),
          messageCount: messages.length,
        }
      })

      return {
        ...state,
        sessions,
      }
    }

    case 'CHAT_CLEAR_SESSION_MESSAGES': {
      const sessions = state.sessions.map(session =>
        session.id === action.payload
          ? {
              ...session,
              messages: [],
              title: 'New Chat',
              updatedAt: new Date(),
              messageCount: 0,
            }
          : session
      )

      return {
        ...state,
        sessions,
      }
    }

    case 'CHAT_ADD_RESPONSE': {
      const { sessionId, messageId, response } = action.payload
      const sessions = state.sessions.map(session => {
        if (session.id !== sessionId) return session

        const messages = session.messages.map(message => {
          if (message.id !== messageId) return message

          const responses = message.responses || []
          const newResponses = [...responses, response]

          return {
            ...message,
            responses: newResponses,
            activeResponseIndex: newResponses.length - 1, // Set new response as active
          }
        })

        return {
          ...session,
          messages,
          updatedAt: new Date(),
        }
      })

      return {
        ...state,
        sessions,
      }
    }

    case 'CHAT_SET_ACTIVE_RESPONSE': {
      const { sessionId, messageId, responseIndex } = action.payload
      const sessions = state.sessions.map(session => {
        if (session.id !== sessionId) return session

        const messages = session.messages.map(message => {
          if (message.id !== messageId) return message

          return {
            ...message,
            activeResponseIndex: responseIndex,
          }
        })

        return {
          ...session,
          messages,
          updatedAt: new Date(),
        }
      })

      return {
        ...state,
        sessions,
      }
    }

    case 'CHAT_SET_LOADING': {
      return {
        ...state,
        isLoading: action.payload,
      }
    }

    case 'CHAT_SET_AI_STATUS': {
      return {
        ...state,
        aiStatus: action.payload,
      }
    }

    case 'CHAT_SET_PROVIDER': {
      return {
        ...state,
        currentProvider: action.payload,
      }
    }

    case 'CHAT_SET_MODEL': {
      return {
        ...state,
        currentModel: action.payload,
      }
    }

    case 'CHAT_TOGGLE_SIDEBAR': {
      return {
        ...state,
        sidebarCollapsed: !state.sidebarCollapsed,
      }
    }

    case 'CHAT_SET_SIDEBAR_COLLAPSED': {
      return {
        ...state,
        sidebarCollapsed: action.payload,
      }
    }

    case 'CHAT_LOAD_PERSISTED_STATE': {
      const persistedState = action.payload
      // Convert timestamp strings back to Date objects
      if (persistedState.sessions) {
        persistedState.sessions = persistedState.sessions.map(session => ({
          ...session,
          createdAt: new Date(session.createdAt),
          updatedAt: new Date(session.updatedAt),
          messages:
            session.messages?.map(msg => ({
              ...msg,
              timestamp: new Date(msg.timestamp),
              responses: msg.responses?.map(response => ({
                ...response,
                timestamp: new Date(response.timestamp),
              })),
            })) || [],
        }))
      }

      return {
        ...state,
        ...persistedState,
      }
    }

    default:
      return state
  }
}

// Context
export interface ChatContextType {
  state: ChatState
  dispatch: React.Dispatch<ChatAction>

  // Helper functions
  createNewChat: (title?: string) => void
  switchToChat: (sessionId: string) => void
  deleteChat: (sessionId: string) => void
  updateChatTitle: (sessionId: string, title: string) => void
  addMessage: (sessionId: string, message: ChatMessage) => void
  deleteMessage: (sessionId: string, messageId: string) => void
  clearChatHistory: (sessionId: string) => void
  addResponse: (sessionId: string, messageId: string, response: ChatMessage) => void
  setActiveResponse: (sessionId: string, messageId: string, responseIndex: number) => void
  getActiveSession: () => ChatSession | null
  getTotalMessageCount: () => number
}

// eslint-disable-next-line react-refresh/only-export-components
export const ChatContext = createContext<ChatContextType | undefined>(undefined)

// Provider Component
interface ChatProviderProps {
  children: React.ReactNode
}

export const ChatProvider: React.FC<ChatProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(chatReducer, initialChatState)

  // Load persisted state on mount
  useEffect(() => {
    try {
      const persistedState = localStorage.getItem('chat_state')
      if (persistedState) {
        const parsedState = JSON.parse(persistedState)
        dispatch({ type: 'CHAT_LOAD_PERSISTED_STATE', payload: parsedState })
      } else {
        // Create initial session if no persisted state
        dispatch({ type: 'CHAT_CREATE_SESSION', payload: {} })
      }
    } catch (error) {
      console.error('Failed to load persisted chat state:', error)
      // Create initial session on error
      dispatch({ type: 'CHAT_CREATE_SESSION', payload: {} })
    }
  }, [])

  // Persist state changes to localStorage
  useEffect(() => {
    // Only persist if we have sessions (avoid persisting initial empty state)
    if (state.sessions.length > 0) {
      try {
        const persistableState = {
          ...state,
          isLoading: false, // Don't persist loading state
        }
        localStorage.setItem('chat_state', JSON.stringify(persistableState))
      } catch (error) {
        console.error('Failed to persist chat state:', error)
      }
    }
  }, [state])

  // Helper functions
  const createNewChat = (title?: string) => {
    dispatch({ type: 'CHAT_CREATE_SESSION', payload: { title } })
  }

  const switchToChat = (sessionId: string) => {
    dispatch({ type: 'CHAT_SET_ACTIVE_SESSION', payload: sessionId })
  }

  const deleteChat = (sessionId: string) => {
    dispatch({ type: 'CHAT_DELETE_SESSION', payload: sessionId })
  }

  const updateChatTitle = (sessionId: string, title: string) => {
    dispatch({ type: 'CHAT_UPDATE_SESSION_TITLE', payload: { sessionId, title } })
  }

  const addMessage = (sessionId: string, message: ChatMessage) => {
    dispatch({ type: 'CHAT_ADD_MESSAGE', payload: { sessionId, message } })
  }

  const deleteMessage = (sessionId: string, messageId: string) => {
    dispatch({ type: 'CHAT_DELETE_MESSAGE', payload: { sessionId, messageId } })
  }

  const clearChatHistory = (sessionId: string) => {
    dispatch({ type: 'CHAT_CLEAR_SESSION_MESSAGES', payload: sessionId })
  }

  const getActiveSession = (): ChatSession | null => {
    if (!state.activeChatId) return null
    return state.sessions.find(s => s.id === state.activeChatId) || null
  }

  const getTotalMessageCount = (): number => {
    return state.sessions.reduce((total, session) => total + session.messageCount, 0)
  }

  const addResponse = (sessionId: string, messageId: string, response: ChatMessage) => {
    dispatch({ type: 'CHAT_ADD_RESPONSE', payload: { sessionId, messageId, response } })
  }

  const setActiveResponse = (sessionId: string, messageId: string, responseIndex: number) => {
    dispatch({ type: 'CHAT_SET_ACTIVE_RESPONSE', payload: { sessionId, messageId, responseIndex } })
  }

  const contextValue: ChatContextType = {
    state,
    dispatch,
    createNewChat,
    switchToChat,
    deleteChat,
    updateChatTitle,
    addMessage,
    deleteMessage,
    clearChatHistory,
    addResponse,
    setActiveResponse,
    getActiveSession,
    getTotalMessageCount,
  }

  return <ChatContext.Provider value={contextValue}>{children}</ChatContext.Provider>
}

// Note: useChatContext hook is now in src/hooks/useChatContext.ts
