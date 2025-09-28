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
  ConversionResult,
  ConversionOptions,
  ConversionMetadata,
  GenerationMetadata
} from './document'
export type {
  StyleProfile,
  StyleAnalysis,
  StyleTransfer,
  OrganizationalStyle,
  StyleGuideline,
  StyleRecommendation,
  StyleMetrics,
  TransferResult
} from './style'
export * from './knowledge'
export * from './search'
export * from './embedding'
export * from './conversation'
export type {
  ContentItem,
  ContentAdaptation,
  ContentClassification,
  ContentTemplate,
  TemplateVariable,
  TemplateMetadata,
  TemplateUsage,
  TemplatePerformance
} from './content'
export * from './relationship'
export * from './deduplication'
export * from './fileWatcher'
export * from './notifications'
export * from './progress'

// Additional specialized service types
export interface ClusterResult {
  id: string
  centroid: number[]
  members: string[]
  size: number
  quality: number
  metadata?: Record<string, any>
}

export interface ClusterAnalysis {
  clusters: ClusterResult[]
  silhouetteScore: number
  inertia: number
  optimalK?: number
  recommendations: string[]
}

export interface SystemHealth {
  status: 'healthy' | 'warning' | 'critical'
  services: Record<string, boolean>
  metrics: HealthMetrics
  alerts: HealthAlert[]
  uptime: number
}

export interface HealthMetrics {
  cpu: number
  memory: number
  disk: number
  network: boolean
  database: boolean
  timestamp: Date
}

export interface HealthAlert {
  id: string
  level: 'info' | 'warning' | 'error' | 'critical'
  message: string
  service: string
  timestamp: Date
}

export interface Workspace {
  id: string
  name: string
  description?: string
  documents: Document[]
  createdAt: Date
  updatedAt: Date
  metadata?: Record<string, any>
}

export interface WorkspaceComparison {
  workspace1: Workspace
  workspace2: Workspace
  similarities: number
  differences: string[]
  recommendations: string[]
}

export interface MultiWorkspaceAnalysis {
  workspaces: Workspace[]
  crossReferences: string[]
  duplicates: string[]
  insights: string[]
  metrics: Record<string, number>
}

export interface BackupInfo {
  id: string
  workspaceId: string
  timestamp: Date
  size: number
  status: 'pending' | 'completed' | 'failed'
  metadata?: Record<string, any>
}

export interface PerformanceMetrics {
  responseTime: number
  throughput: number
  errorRate: number
  resourceUsage: Record<string, number>
  timestamp: Date
}

export interface AIAnalysis {
  insights: string[]
  confidence: number
  recommendations: string[]
  metadata?: Record<string, any>
}

export interface DocumentStructure {
  hierarchy: StructureNode[]
  sections: StructureSection[]
  metadata?: Record<string, any>
}

export interface StructureNode {
  id: string
  type: string
  content: string
  children: StructureNode[]
  level: number
}

export interface StructureSection {
  id: string
  title: string
  content: string
  startIndex: number
  endIndex: number
}

export interface VectorOperation {
  id: string
  operation: string
  input: number[][]
  output: number[][]
  metadata?: Record<string, any>
}

export interface ProcessingPipeline {
  id: string
  name: string
  stages: PipelineStage[]
  status: 'idle' | 'running' | 'paused' | 'error'
  metrics: PipelineMetrics
}

export interface PipelineStage {
  id: string
  name: string
  processor: string
  config: Record<string, any>
  status: 'pending' | 'running' | 'completed' | 'failed'
}

export interface PipelineMetrics {
  documentsProcessed: number
  averageProcessingTime: number
  errorRate: number
  throughput: number
}

export interface StreamProcessor {
  id: string
  type: string
  config: Record<string, any>
  isActive: boolean
}

export interface RealTimeEvent {
  id: string
  type: string
  data: any
  timestamp: Date
  source: string
}

export interface NLOperation {
  id: string
  operation: string
  input: string
  output: any
  confidence: number
  metadata?: Record<string, any>
}

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
