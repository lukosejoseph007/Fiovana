// src/types/fileWatcher.ts
// TypeScript types for file watcher functionality

export interface FileEvent {
  type: 'file-created' | 'file-modified' | 'file-deleted' | 'file-renamed' | 'file-moved'
  path: string
  old_path?: string // For rename/move events
  timestamp: number
  size?: number
  is_directory: boolean
}

export interface FileWatcherConfig {
  debounceDuration: number // milliseconds
  ignorePatterns: string[]
  maxEventsPerSecond: number
}

export interface WatcherStatus {
  isWatching: boolean
  watchedPaths: string[]
  totalEvents: number
  lastEvent?: FileEvent
  error?: string
}

export interface FileSystemEvent {
  type: 'create' | 'modify' | 'delete' | 'rename'
  path: string
  oldPath?: string // For rename events
  timestamp: number
  size?: number
  isDirectory: boolean
}

// Event types that can be emitted to the frontend
export interface FrontendFileEvent {
  type: 'file-created' | 'file-modified' | 'file-deleted' | 'file-renamed' | 'file-moved'
  path: string
  oldPath?: string
  timestamp: number
  size?: number
  isDirectory: boolean
}

// Response types for file watcher commands
export interface StartWatchingResponse {
  success: boolean
  message: string
  watchedPaths: string[]
}

export interface StopWatchingResponse {
  success: boolean
  message: string
}

export interface AddWatchPathResponse {
  success: boolean
  message: string
  watchedPaths: string[]
}

export interface RemoveWatchPathResponse {
  success: boolean
  message: string
  watchedPaths: string[]
}

export interface GetWatchedPathsResponse {
  paths: string[]
}

export interface FileWatcherStats {
  totalEvents: number
  eventsByType: {
    created: number
    modified: number
    deleted: number
    renamed: number
    moved: number
  }
  watchedPaths: number
  isWatching: boolean
  uptime: number // seconds
}
