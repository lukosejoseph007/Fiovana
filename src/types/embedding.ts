// Embedding System Types
export interface EmbeddingModel {
  id: string
  name: string
  provider: 'openai' | 'huggingface' | 'local' | 'custom'
  dimensions: number
  maxTokens: number
  costPerToken?: number
  capabilities: EmbeddingCapability[]
  status: 'active' | 'inactive' | 'deprecated'
}

export interface EmbeddingCapability {
  type: 'text' | 'code' | 'multimodal' | 'multilingual'
  languages?: string[]
  domains?: string[]
  quality: 'high' | 'medium' | 'low'
}

export interface EmbeddingRequest {
  text: string | string[]
  model: string
  options: EmbeddingOptions
}

export interface EmbeddingOptions {
  normalize?: boolean
  truncate?: boolean
  batchSize?: number
  timeout?: number
  retries?: number
}

export interface EmbeddingResponse {
  embeddings: number[][]
  model: string
  dimensions: number
  tokenUsage: TokenUsage
  processingTime: number
  metadata: EmbeddingMetadata
}

export interface TokenUsage {
  promptTokens: number
  totalTokens: number
  cost?: number
}

export interface EmbeddingMetadata {
  version: string
  truncated: boolean
  normalizations: string[]
  warnings: string[]
}

export interface EmbeddingCache {
  id: string
  textHash: string
  model: string
  embedding: number[]
  metadata: CacheMetadata
  createdAt: Date
  accessCount: number
  lastAccessed: Date
}

export interface CacheMetadata {
  textLength: number
  tokenCount: number
  compressionRatio?: number
  tags: string[]
}

export interface EmbeddingSettings {
  defaultModel: string
  cacheEnabled: boolean
  batchProcessing: boolean
  maxBatchSize: number
  retryPolicy: RetryPolicy
  costLimits: CostLimits
}

export interface RetryPolicy {
  maxRetries: number
  backoffStrategy: 'linear' | 'exponential' | 'fixed'
  baseDelay: number
  maxDelay: number
}

export interface CostLimits {
  dailyLimit?: number
  monthlyLimit?: number
  perRequestLimit?: number
  alertThresholds: number[]
}

export interface EmbeddingJob {
  id: string
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled'
  documents: string[]
  model: string
  progress: JobProgress
  result?: EmbeddingJobResult
  error?: string
  createdAt: Date
  completedAt?: Date
}

export interface JobProgress {
  total: number
  completed: number
  failed: number
  currentDocument?: string
  estimatedCompletion?: Date
}

export interface EmbeddingJobResult {
  successful: number
  failed: number
  totalCost?: number
  averageProcessingTime: number
  errors: JobError[]
}

export interface JobError {
  documentId: string
  error: string
  retryable: boolean
  timestamp: Date
}

export interface EmbeddingComparison {
  documentA: string
  documentB: string
  similarity: number
  distance: number
  method: 'cosine' | 'euclidean' | 'manhattan' | 'dot'
  metadata: ComparisonMetadata
}

export interface ComparisonMetadata {
  model: string
  dimensions: number
  computationTime: number
  confidence: number
}
