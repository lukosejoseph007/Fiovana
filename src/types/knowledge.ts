// Knowledge Management Types
export interface KnowledgeBase {
  id: string
  name: string
  description: string
  domains: KnowledgeDomain[]
  totalDocuments: number
  lastUpdated: Date
  status: 'active' | 'maintenance' | 'archived'
}

export interface KnowledgeDomain {
  id: string
  name: string
  description: string
  keywords: string[]
  documentCount: number
  confidence: number
}

export interface KnowledgeGap {
  id: string
  type: 'content' | 'expertise' | 'process' | 'tool'
  severity: 'low' | 'medium' | 'high' | 'critical'
  description: string
  impact: string
  suggestedSources: string[]
  priority: number
  identifiedAt: Date
}

export interface KnowledgeGapAnalysis {
  workspaceId: string
  gaps: KnowledgeGap[]
  completeness: number
  recommendations: GapRecommendation[]
  analysisDate: Date
  methodology: string
}

export interface GapRecommendation {
  gapId: string
  action: 'research' | 'training' | 'documentation' | 'acquisition'
  description: string
  resources: RecommendedResource[]
  estimatedEffort: string
  priority: number
}

export interface RecommendedResource {
  type: 'document' | 'expert' | 'tool' | 'course' | 'external'
  name: string
  description: string
  url?: string
  confidence: number
}

export interface ContentLifecycle {
  documentId: string
  stage: 'creation' | 'review' | 'active' | 'maintenance' | 'archive' | 'deprecated'
  lastTransition: Date
  nextAction?: LifecycleAction
  metrics: LifecycleMetrics
  history: LifecycleEvent[]
}

export interface LifecycleAction {
  type: 'review' | 'update' | 'archive' | 'delete' | 'promote'
  scheduledDate: Date
  assignee?: string
  description: string
  priority: 'low' | 'medium' | 'high'
}

export interface LifecycleMetrics {
  lastAccessed: Date
  accessCount: number
  updateFrequency: number
  relevanceScore: number
  qualityScore: number
}

export interface LifecycleEvent {
  timestamp: Date
  action: string
  actor: string
  details: string
  metadata: Record<string, any>
}

export interface SmartOrganization {
  id: string
  workspaceId: string
  suggestions: OrganizationSuggestion[]
  automationRules: AutomationRule[]
  performance: OrganizationPerformance
  lastAnalysis: Date
}

export interface OrganizationSuggestion {
  id: string
  type: 'categorization' | 'tagging' | 'restructure' | 'cleanup'
  target: string[]
  description: string
  rationale: string
  confidence: number
  impact: 'low' | 'medium' | 'high'
  effort: 'minimal' | 'moderate' | 'significant'
}

export interface AutomationRule {
  id: string
  name: string
  condition: RuleCondition
  action: RuleAction
  enabled: boolean
  lastTriggered?: Date
  performance: RulePerformance
}

export interface RuleCondition {
  type: string
  parameters: Record<string, any>
  description: string
}

export interface RuleAction {
  type: 'move' | 'tag' | 'categorize' | 'notify' | 'archive'
  parameters: Record<string, any>
  description: string
}

export interface RulePerformance {
  executionCount: number
  successRate: number
  averageExecutionTime: number
  lastExecution: Date
}

export interface OrganizationPerformance {
  automationEfficiency: number
  userSatisfaction: number
  timesSaved: number
  errorsReduced: number
  lastMeasurement: Date
}