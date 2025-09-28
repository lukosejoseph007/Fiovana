// Comprehensive TypeScript types for all 321+ backend commands

// Core application types
export interface AppState {
  currentProject: Project | null
  isLoading: boolean
  error: string | null
}

export interface Project {
  id: string
  name: string
  description?: string
  documents: Document[]
  createdAt: Date
}

// Re-export all specialized type modules
export * from './ai'
export type {
  WorkspaceConfig,
  WorkspaceAnalysis,
  WorkspaceHealth,
  WorkspaceMetrics,
  WorkspaceComparison,
  WorkspaceBackup,
  WorkspaceInsight,
  HealthIssue,
  BackupMetadata
} from './workspace'
export type {
  Document,
  DocumentMetadata,
  DocumentChunk,
  DocumentIndex,
  DocumentComparison,
  DocumentGeneration,
  FormatConversion,
  ComparisonAnalysis,
  ConversionResult
} from './document'
export type {
  StyleProfile,
  StyleAnalysis,
  StyleTransfer,
  OrganizationalStyle,
  StyleGuideline,
  StyleRecommendation
} from './style'
export * from './knowledge'
export * from './search'
export * from './embedding'
export * from './conversation'
export type {
  ContentItem,
  ContentAdaptation,
  ContentClassification,
  ContentTemplate
} from './content'
export * from './relationship'
export * from './deduplication'
export * from './fileWatcher'
export * from './notifications'
export * from './progress'

// Common utility types used across modules
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  metadata?: Record<string, any>
}

export interface PaginatedResponse<T = any> {
  items: T[]
  total: number
  page: number
  limit: number
  hasNext: boolean
  hasPrevious: boolean
}

export interface BatchOperation<T = any> {
  id: string
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled'
  items: T[]
  progress: number
  results?: BatchResult<T>
  error?: string
  createdAt: Date
  completedAt?: Date
}

export interface BatchResult<T = any> {
  successful: T[]
  failed: FailedItem<T>[]
  summary: BatchSummary
}

export interface FailedItem<T = any> {
  item: T
  error: string
  retryable: boolean
}

export interface BatchSummary {
  total: number
  successful: number
  failed: number
  skipped: number
  processingTime: number
}
