// Document Processing Types
export interface Document {
  id: string
  name: string
  path: string
  type: string
  size: number
  createdAt: Date
  updatedAt: Date
  metadata: DocumentMetadata
  content?: string
  chunks?: DocumentChunk[]
}

export interface DocumentMetadata {
  title?: string
  author?: string
  tags: string[]
  language?: string
  encoding?: string
  wordCount?: number
  pageCount?: number
  extractedText?: string
  customFields: Record<string, unknown>
}

export interface DocumentChunk {
  id: string
  documentId: string
  content: string
  position: number
  size: number
  type: 'paragraph' | 'section' | 'page' | 'custom'
  metadata: ChunkMetadata
}

export interface ChunkMetadata {
  startPosition: number
  endPosition: number
  parentId?: string
  level: number
  tags: string[]
  confidence: number
}

export interface DocumentIndex {
  id: string
  documentId: string
  terms: IndexTerm[]
  vectors: number[]
  metadata: IndexMetadata
  lastUpdated: Date
}

export interface IndexTerm {
  term: string
  frequency: number
  positions: number[]
  importance: number
}

export interface IndexMetadata {
  version: string
  algorithm: string
  parameters: Record<string, unknown>
  processingTime: number
}

export interface DocumentComparison {
  documentA: string
  documentB: string
  similarity: number
  differences: DocumentDifference[]
  commonElements: CommonElement[]
  analysis: ComparisonAnalysis
}

export interface DocumentDifference {
  type: 'content' | 'structure' | 'metadata' | 'style'
  description: string
  position?: Position
  severity: 'minor' | 'moderate' | 'significant'
}

export interface Position {
  start: number
  end: number
  line?: number
  column?: number
}

export interface CommonElement {
  type: string
  content: string
  similarity: number
  positions: Position[]
}

export interface ComparisonAnalysis {
  contentSimilarity: number
  structureSimilarity: number
  styleSimilarity: number
  overallScore: number
  recommendations: string[]
}

export interface DocumentGeneration {
  templateId?: string
  parameters: Record<string, unknown>
  content: string
  format: 'markdown' | 'html' | 'pdf' | 'docx' | 'txt'
  metadata: GenerationMetadata
}

export interface GenerationMetadata {
  generatedAt: Date
  model?: string
  version: string
  processingTime: number
  wordCount: number
}

export interface FormatConversion {
  sourceFormat: string
  targetFormat: string
  options: ConversionOptions
  result: ConversionResult
}

export interface ConversionOptions {
  preserveFormatting: boolean
  includeImages: boolean
  customMapping?: Record<string, string>
  quality?: 'low' | 'medium' | 'high'
}

export interface ConversionResult {
  success: boolean
  outputPath?: string
  errors: string[]
  warnings: string[]
  metadata: ConversionMetadata
}

export interface ConversionMetadata {
  originalSize: number
  convertedSize: number
  processingTime: number
  lossyConversion: boolean
}
