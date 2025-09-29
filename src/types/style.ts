// Style Analysis & Transfer Types
export interface StyleProfile {
  id: string
  name: string
  source: string
  features: StyleFeature[]
  patterns: StylePattern[]
  confidence: number
  createdAt: Date
}

export interface StyleFeature {
  type: 'vocabulary' | 'syntax' | 'structure' | 'tone' | 'formatting'
  name: string
  value: unknown
  weight: number
  description: string
}

export interface StylePattern {
  id: string
  type: string
  pattern: string
  frequency: number
  context: string[]
  examples: string[]
}

export interface StyleAnalysis {
  documentId: string
  profile: StyleProfile
  metrics: StyleMetrics
  recommendations: StyleRecommendation[]
  timestamp: Date
}

export interface StyleMetrics {
  readabilityScore: number
  complexityIndex: number
  formalityLevel: number
  sentimentScore: number
  coherenceScore: number
  vocabularyRichness: number
}

export interface StyleRecommendation {
  type: 'improvement' | 'consistency' | 'adaptation'
  priority: 'low' | 'medium' | 'high'
  description: string
  examples: string[]
  impact: string
}

export interface StyleTransfer {
  sourceDocumentId: string
  targetStyleId: string
  result: TransferResult
  preservedElements: string[]
  modifiedElements: ModifiedElement[]
}

export interface TransferResult {
  success: boolean
  transformedContent: string
  confidence: number
  appliedChanges: StyleChange[]
  warnings: string[]
}

export interface ModifiedElement {
  type: string
  original: string
  transformed: string
  confidence: number
  rationale: string
}

export interface StyleChange {
  type: string
  position: Position
  description: string
  severity: 'minor' | 'moderate' | 'significant'
}

export interface Position {
  start: number
  end: number
  line?: number
  column?: number
}

export interface OrganizationalStyle {
  id: string
  organizationId: string
  name: string
  guidelines: StyleGuideline[]
  templates: StyleTemplate[]
  approved: boolean
  version: string
}

export interface StyleGuideline {
  category: string
  rule: string
  description: string
  examples: GuidelineExample[]
  priority: 'required' | 'recommended' | 'optional'
}

export interface GuidelineExample {
  correct: string
  incorrect?: string
  context: string
  explanation: string
}

export interface StyleTemplate {
  id: string
  name: string
  type: string
  content: string
  variables: TemplateVariable[]
  metadata: TemplateMetadata
}

export interface TemplateVariable {
  name: string
  type: 'string' | 'number' | 'date' | 'list' | 'object'
  required: boolean
  defaultValue?: unknown
  description: string
}

export interface TemplateMetadata {
  category: string
  tags: string[]
  usage: string
  lastModified: Date
  version: string
}
