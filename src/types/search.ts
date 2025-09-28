// Search & Vector Operations Types
export interface SearchQuery {
  text: string
  type: 'semantic' | 'keyword' | 'hybrid'
  filters?: SearchFilter[]
  options: SearchOptions
}

export interface SearchFilter {
  field: string
  operator: 'equals' | 'contains' | 'gt' | 'lt' | 'range' | 'in'
  value: any
}

export interface SearchOptions {
  limit?: number
  offset?: number
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
  includeMetadata?: boolean
  includeContent?: boolean
  threshold?: number
}

export interface SearchResult {
  query: SearchQuery
  results: SearchResultItem[]
  totalCount: number
  executionTime: number
  metadata: SearchMetadata
}

export interface SearchResultItem {
  id: string
  documentId: string
  title: string
  content: string
  score: number
  highlights: Highlight[]
  metadata: any
  path: string
}

export interface Highlight {
  field: string
  fragments: string[]
  positions: HighlightPosition[]
}

export interface HighlightPosition {
  start: number
  end: number
  score: number
}

export interface SearchMetadata {
  algorithm: string
  indexVersion: string
  debugInfo?: any
  performance: PerformanceMetrics
}

export interface PerformanceMetrics {
  queryTime: number
  indexTime: number
  postProcessingTime: number
  totalDocuments: number
  documentsScanned: number
}

export interface VectorSpace {
  id: string
  name: string
  dimensions: number
  model: string
  documents: number
  createdAt: Date
  lastUpdated: Date
}

export interface DocumentVector {
  documentId: string
  vector: number[]
  metadata: VectorMetadata
  timestamp: Date
}

export interface VectorMetadata {
  model: string
  version: string
  processingTime: number
  chunkId?: string
  confidence: number
}

export interface VectorSimilarity {
  documentA: string
  documentB: string
  similarity: number
  method: string
  timestamp: Date
}

export interface VectorQuery {
  vector?: number[]
  text?: string
  k: number
  threshold?: number
  filters?: VectorFilter[]
  includeMetadata?: boolean
}

export interface VectorFilter {
  field: string
  value: any
  operator: 'equals' | 'in' | 'range'
}

export interface VectorSearchResult {
  items: VectorResultItem[]
  executionTime: number
  totalCount: number
  metadata: VectorSearchMetadata
}

export interface VectorResultItem {
  documentId: string
  similarity: number
  vector?: number[]
  metadata?: any
  content?: string
}

export interface VectorSearchMetadata {
  space: string
  algorithm: string
  indexSize: number
  performance: VectorPerformanceMetrics
}

export interface VectorPerformanceMetrics {
  searchTime: number
  indexLookupTime: number
  scoringTime: number
  vectorsCompared: number
}

export interface IndexConfig {
  type: 'inverted' | 'vector' | 'hybrid'
  parameters: Record<string, any>
  updateFrequency: 'realtime' | 'batch' | 'manual'
  optimization: IndexOptimization
}

export interface IndexOptimization {
  compressionEnabled: boolean
  cacheSize: number
  parallelization: boolean
  memoryLimit: number
}