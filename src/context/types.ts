// Separate file for types and interfaces to satisfy react-refresh requirements

// File Management State
export interface FileInfo {
  name: string
  size: number
  type: string
  lastModified: number
  path?: string
  validation?: {
    is_valid: boolean
    message: string
    warnings: string[]
  }
  hash?: {
    sha256: string
    algorithm: string
  }
  duplicate_check?: {
    is_duplicate: boolean
    existing_paths: string[]
  }
  metadata?: {
    file_type: string
    mime_type: string
    page_count?: number
    word_count?: number
    creation_date?: string
    author?: string
    language?: string
    binary_ratio?: number
    entropy?: number
    text_preview?: string
  }
  error?: string
}

// Chat State
export interface ChatMessage {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: Date
  intent?: string
  confidence?: number
  error?: string
}

// File Watcher State
export interface FileEvent {
  type: 'file-created' | 'file-modified' | 'file-deleted' | 'file-renamed' | 'file-moved'
  path: string
  old_path?: string
  timestamp: number
  size?: number
  is_directory: boolean
}

// Global App State
export interface AppState {
  // File Management
  fileManagement: {
    droppedFiles: FileInfo[]
    isDragOver: boolean
    isProcessing: boolean
  }

  // Chat
  chat: {
    messages: ChatMessage[]
    isLoading: boolean
    aiStatus: 'unknown' | 'available' | 'unavailable'
    currentProvider: string
    currentModel: string
  }

  // File Watcher
  fileWatcher: {
    isWatching: boolean
    watchedPaths: string[]
    fileEvents: FileEvent[]
    workspacePath: string
  }

  // Workspace
  workspace: {
    currentWorkspace: string
    recentWorkspaces: string[]
  }
}

// Action Types
export type AppAction =
  // File Management Actions
  | { type: 'FILE_MANAGEMENT_SET_FILES'; payload: FileInfo[] }
  | { type: 'FILE_MANAGEMENT_ADD_FILES'; payload: FileInfo[] }
  | { type: 'FILE_MANAGEMENT_CLEAR_FILES' }
  | { type: 'FILE_MANAGEMENT_SET_DRAG_OVER'; payload: boolean }
  | { type: 'FILE_MANAGEMENT_SET_PROCESSING'; payload: boolean }

  // Chat Actions
  | { type: 'CHAT_ADD_MESSAGE'; payload: ChatMessage }
  | { type: 'CHAT_SET_MESSAGES'; payload: ChatMessage[] }
  | { type: 'CHAT_CLEAR_MESSAGES' }
  | { type: 'CHAT_SET_LOADING'; payload: boolean }
  | { type: 'CHAT_SET_AI_STATUS'; payload: 'unknown' | 'available' | 'unavailable' }
  | { type: 'CHAT_SET_PROVIDER'; payload: string }
  | { type: 'CHAT_SET_MODEL'; payload: string }

  // File Watcher Actions
  | { type: 'FILE_WATCHER_SET_WATCHING'; payload: boolean }
  | { type: 'FILE_WATCHER_SET_PATHS'; payload: string[] }
  | { type: 'FILE_WATCHER_ADD_PATH'; payload: string }
  | { type: 'FILE_WATCHER_REMOVE_PATH'; payload: string }
  | { type: 'FILE_WATCHER_ADD_EVENT'; payload: FileEvent }
  | { type: 'FILE_WATCHER_SET_EVENTS'; payload: FileEvent[] }
  | { type: 'FILE_WATCHER_CLEAR_EVENTS' }
  | { type: 'FILE_WATCHER_SET_WORKSPACE_PATH'; payload: string }

  // Workspace Actions
  | { type: 'WORKSPACE_SET_CURRENT'; payload: string }
  | { type: 'WORKSPACE_ADD_RECENT'; payload: string }

  // Persistence Actions
  | { type: 'LOAD_PERSISTED_STATE'; payload: Partial<AppState> }
