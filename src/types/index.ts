// Global type definitions
export interface Document {
  id: string
  name: string
  path: string
  createdAt: Date
  updatedAt: Date
}

export interface Project {
  id: string
  name: string
  description?: string
  documents: Document[]
  createdAt: Date
}

export interface AppState {
  currentProject: Project | null
  isLoading: boolean
  error: string | null
}

// Re-export deduplication types
export * from './deduplication'
