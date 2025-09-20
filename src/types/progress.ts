// src/types/progress.ts
// TypeScript types for progress tracking system

export interface ImportProgress {
  operation_id: string
  current_step: string
  progress_percentage: number
  files_processed: number
  total_files: number
  current_file?: string
  started_at: string
  eta_seconds?: number
  cancellable: boolean
  status: OperationStatus
  steps: ProgressStep[]
  errors: string[]
  warnings: string[]
}

export enum OperationStatus {
  Pending = 'Pending',
  Running = 'Running',
  Completed = 'Completed',
  Failed = 'Failed',
  Cancelled = 'Cancelled',
  Paused = 'Paused',
}

export interface ProgressStep {
  name: string
  description: string
  status: StepStatus
  progress: number
  started_at?: string
  completed_at?: string
  error?: string
}

export enum StepStatus {
  Pending = 'Pending',
  Running = 'Running',
  Completed = 'Completed',
  Failed = 'Failed',
  Skipped = 'Skipped',
}

export interface ProgressSummary {
  active_operations: number
  completed_operations: number
  failed_operations: number
  total_files_processing: number
  total_files_completed: number
  overall_progress: number
}

// Service interface for progress operations
export interface ProgressService {
  getAllOperations(): Promise<ImportProgress[]>
  getOperationProgress(operationId: string): Promise<ImportProgress | null>
  cancelOperation(operationId: string): Promise<boolean>
  getProgressSummary(): Promise<ProgressSummary>
  cleanupCompletedOperations(): Promise<number>
  subscribeToUpdates(): Promise<void>
  getOperationHistory(limit?: number): Promise<ImportProgress[]>
  getEstimatedCompletionTime(): Promise<number | null>
  updateOperationProgress(
    operationId: string,
    progressPercentage: number,
    currentStep?: string,
    currentFile?: string
  ): Promise<void>
}

// UI-specific types
export interface ProgressCardProps {
  progress: ImportProgress
  onCancel?: (operationId: string) => void
  onRetry?: (operationId: string) => void
  compact?: boolean
}

export interface ProgressListProps {
  operations: ImportProgress[]
  onCancel?: (operationId: string) => void
  onRetry?: (operationId: string) => void
  showCompleted?: boolean
  maxItems?: number
}

export interface ProgressSummaryProps {
  summary: ProgressSummary
  onRefresh?: () => void
  onCleanup?: () => void
}

export interface ProgressStepProps {
  step: ProgressStep
  isActive?: boolean
  showProgress?: boolean
}

// Hooks return types
export interface UseProgressReturn {
  operations: ImportProgress[]
  summary: ProgressSummary
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
  cancelOperation: (operationId: string) => Promise<void>
  cleanupCompleted: () => Promise<void>
  subscribeToUpdates: () => void
  unsubscribeFromUpdates: () => void
}

export interface UseOperationProgressReturn {
  progress: ImportProgress | null
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
  cancel: () => Promise<void>
}

// Event types for real-time updates
export interface ProgressUpdateEvent {
  type: 'progress-update'
  payload: ImportProgress
}

// Configuration types
export interface ProgressUIConfig {
  autoRefreshInterval?: number
  showCompletedOperations?: boolean
  maxDisplayedOperations?: number
  enableNotifications?: boolean
  compactMode?: boolean
}

// Utility types
export type ProgressStatus = OperationStatus
export type ProgressFilter = 'all' | 'active' | 'completed' | 'failed'

export interface ProgressFilterOptions {
  status?: ProgressFilter[]
  dateRange?: {
    start: Date
    end: Date
  }
  searchTerm?: string
}
