import React, { createContext, useContext, useReducer, ReactNode, useEffect } from 'react'
import type { AppState, AppAction, ChatMessage } from './types'

// Initial State
const initialState: AppState = {
  fileManagement: {
    droppedFiles: [],
    isDragOver: false,
    isProcessing: false,
  },
  chat: {
    messages: [],
    isLoading: false,
    aiStatus: 'unknown',
    currentProvider: 'local',
    currentModel: '',
  },
  fileWatcher: {
    isWatching: false,
    watchedPaths: [],
    fileEvents: [],
    workspacePath: '',
  },
  workspace: {
    currentWorkspace: '',
    recentWorkspaces: [],
  },
}

// Reducer
function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    // File Management
    case 'FILE_MANAGEMENT_SET_FILES':
      return {
        ...state,
        fileManagement: {
          ...state.fileManagement,
          droppedFiles: action.payload,
        },
      }

    case 'FILE_MANAGEMENT_ADD_FILES':
      return {
        ...state,
        fileManagement: {
          ...state.fileManagement,
          droppedFiles: [...state.fileManagement.droppedFiles, ...action.payload],
        },
      }

    case 'FILE_MANAGEMENT_CLEAR_FILES':
      return {
        ...state,
        fileManagement: {
          ...state.fileManagement,
          droppedFiles: [],
        },
      }

    case 'FILE_MANAGEMENT_SET_DRAG_OVER':
      return {
        ...state,
        fileManagement: {
          ...state.fileManagement,
          isDragOver: action.payload,
        },
      }

    case 'FILE_MANAGEMENT_SET_PROCESSING':
      return {
        ...state,
        fileManagement: {
          ...state.fileManagement,
          isProcessing: action.payload,
        },
      }

    // Chat
    case 'CHAT_ADD_MESSAGE':
      return {
        ...state,
        chat: {
          ...state.chat,
          messages: [...state.chat.messages, action.payload],
        },
      }

    case 'CHAT_SET_MESSAGES':
      return {
        ...state,
        chat: {
          ...state.chat,
          messages: action.payload,
        },
      }

    case 'CHAT_CLEAR_MESSAGES':
      return {
        ...state,
        chat: {
          ...state.chat,
          messages: [],
        },
      }

    case 'CHAT_SET_LOADING':
      return {
        ...state,
        chat: {
          ...state.chat,
          isLoading: action.payload,
        },
      }

    case 'CHAT_SET_AI_STATUS':
      return {
        ...state,
        chat: {
          ...state.chat,
          aiStatus: action.payload,
        },
      }

    case 'CHAT_SET_PROVIDER':
      return {
        ...state,
        chat: {
          ...state.chat,
          currentProvider: action.payload,
        },
      }

    case 'CHAT_SET_MODEL':
      return {
        ...state,
        chat: {
          ...state.chat,
          currentModel: action.payload,
        },
      }

    // File Watcher
    case 'FILE_WATCHER_SET_WATCHING':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          isWatching: action.payload,
        },
      }

    case 'FILE_WATCHER_SET_PATHS':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          watchedPaths: action.payload,
        },
      }

    case 'FILE_WATCHER_ADD_PATH':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          watchedPaths: [...state.fileWatcher.watchedPaths, action.payload],
        },
      }

    case 'FILE_WATCHER_REMOVE_PATH':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          watchedPaths: state.fileWatcher.watchedPaths.filter(path => path !== action.payload),
        },
      }

    case 'FILE_WATCHER_ADD_EVENT':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          fileEvents: [...state.fileWatcher.fileEvents, action.payload],
        },
      }

    case 'FILE_WATCHER_SET_EVENTS':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          fileEvents: action.payload,
        },
      }

    case 'FILE_WATCHER_CLEAR_EVENTS':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          fileEvents: [],
        },
      }

    case 'FILE_WATCHER_SET_WORKSPACE_PATH':
      return {
        ...state,
        fileWatcher: {
          ...state.fileWatcher,
          workspacePath: action.payload,
        },
      }

    // Workspace
    case 'WORKSPACE_SET_CURRENT':
      return {
        ...state,
        workspace: {
          ...state.workspace,
          currentWorkspace: action.payload,
        },
      }

    case 'WORKSPACE_ADD_RECENT': {
      const newRecent = [
        action.payload,
        ...state.workspace.recentWorkspaces.filter(w => w !== action.payload),
      ].slice(0, 10)
      return {
        ...state,
        workspace: {
          ...state.workspace,
          recentWorkspaces: newRecent,
        },
      }
    }

    // Persistence
    case 'LOAD_PERSISTED_STATE':
      return {
        ...state,
        ...action.payload,
      }

    default:
      return state
  }
}

// Context
interface AppStateContextType {
  state: AppState
  dispatch: React.Dispatch<AppAction>
}

const AppStateContext = createContext<AppStateContextType | undefined>(undefined)

// Provider Component
interface AppStateProviderProps {
  children: ReactNode
}

export const AppStateProvider: React.FC<AppStateProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(appReducer, initialState)

  // Load persisted state on mount
  useEffect(() => {
    try {
      const persistedState = localStorage.getItem('app_state')
      if (persistedState) {
        const parsedState = JSON.parse(persistedState)
        // Convert timestamp strings back to Date objects for chat messages
        if (parsedState.chat?.messages) {
          parsedState.chat.messages = parsedState.chat.messages.map((msg: ChatMessage) => ({
            ...msg,
            timestamp: new Date(msg.timestamp),
          }))
        }
        dispatch({ type: 'LOAD_PERSISTED_STATE', payload: parsedState })
      }
    } catch (error) {
      console.error('Failed to load persisted state:', error)
    }
  }, [])

  // Persist state changes to localStorage
  useEffect(() => {
    try {
      // Create a serializable version of the state
      const persistableState = {
        ...state,
        // Don't persist temporary UI states
        fileManagement: {
          ...state.fileManagement,
          isDragOver: false,
          isProcessing: false,
        },
        chat: {
          ...state.chat,
          isLoading: false,
        },
      }
      localStorage.setItem('app_state', JSON.stringify(persistableState))
    } catch (error) {
      console.error('Failed to persist state:', error)
    }
  }, [state])

  return <AppStateContext.Provider value={{ state, dispatch }}>{children}</AppStateContext.Provider>
}

// Hook to use the context
// eslint-disable-next-line react-refresh/only-export-components
export const useAppState = (): AppStateContextType => {
  const context = useContext(AppStateContext)
  if (context === undefined) {
    throw new Error('useAppState must be used within an AppStateProvider')
  }
  return context
}
