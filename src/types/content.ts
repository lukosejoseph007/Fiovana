// Content Management Types
export interface ContentItem {
  id: string
  type: 'document' | 'image' | 'video' | 'audio' | 'code' | 'data'
  path: string
  name: string
  title?: string
  description?: string
  content?: string
  metadata: ContentMetadata
  version: string
  status: ContentStatus
  createdAt: Date
  updatedAt: Date
}

export interface ContentMetadata {
  size: number
  mimeType: string
  encoding?: string
  checksum: string
  language?: string
  author?: string
  tags: string[]
  category?: string
  customFields: Record<string, any>
}

export interface ContentStatus {
  state: 'draft' | 'review' | 'approved' | 'published' | 'archived' | 'deleted'
  lastModified: Date
  modifiedBy: string
  reviewers?: string[]
  approvers?: string[]
}

export interface ContentAdaptation {
  sourceId: string
  targetFormat: string
  adaptationRules: AdaptationRule[]
  result: AdaptationResult
  metadata: AdaptationMetadata
}

export interface AdaptationRule {
  type: 'format' | 'style' | 'length' | 'audience' | 'platform'
  parameters: Record<string, any>
  priority: number
  description: string
}

export interface AdaptationResult {
  success: boolean
  adaptedContent: string
  changes: ContentChange[]
  quality: QualityMetrics
  warnings: string[]
}

export interface ContentChange {
  type: string
  position: ContentPosition
  original: string
  adapted: string
  rationale: string
  confidence: number
}

export interface ContentPosition {
  start: number
  end: number
  line?: number
  section?: string
}

export interface QualityMetrics {
  readability: number
  coherence: number
  completeness: number
  accuracy: number
  overallScore: number
}

export interface AdaptationMetadata {
  processingTime: number
  model?: string
  version: string
  parameters: Record<string, any>
}

export interface ContentClassification {
  contentId: string
  categories: ClassificationCategory[]
  confidence: number
  method: 'rule_based' | 'ml_model' | 'hybrid'
  timestamp: Date
}

export interface ClassificationCategory {
  name: string
  confidence: number
  subcategories?: SubCategory[]
  attributes: CategoryAttribute[]
}

export interface SubCategory {
  name: string
  confidence: number
  path: string[]
}

export interface CategoryAttribute {
  name: string
  value: any
  confidence: number
  description: string
}

export interface ContentTemplate {
  id: string
  name: string
  description: string
  type: string
  content: string
  variables: TemplateVariable[]
  constraints: TemplateConstraint[]
  metadata: TemplateMetadata
  version: string
  createdAt: Date
}

export interface TemplateVariable {
  name: string
  type: 'text' | 'number' | 'date' | 'boolean' | 'list' | 'object'
  required: boolean
  defaultValue?: any
  validation?: ValidationRule
  description: string
}

export interface ValidationRule {
  pattern?: string
  minLength?: number
  maxLength?: number
  min?: number
  max?: number
  options?: any[]
}

export interface TemplateConstraint {
  type: 'length' | 'format' | 'content' | 'style'
  parameters: Record<string, any>
  description: string
  severity: 'warning' | 'error'
}

export interface TemplateMetadata {
  category: string
  tags: string[]
  usage: TemplateUsage
  performance: TemplatePerformance
  lastModified: Date
}

export interface TemplateUsage {
  timesUsed: number
  lastUsed: Date
  averageGenerationTime: number
  successRate: number
}

export interface TemplatePerformance {
  generationSpeed: number
  outputQuality: number
  userSatisfaction: number
  errorRate: number
}