// Conversation Intelligence Types
export interface Conversation {
  id: string
  title: string
  participants: Participant[]
  messages: Message[]
  context: ConversationContext
  metadata: ConversationMetadata
  createdAt: Date
  updatedAt: Date
  status: 'active' | 'archived' | 'deleted'
}

export interface Participant {
  id: string
  name: string
  role: 'user' | 'assistant' | 'system' | 'guest'
  avatar?: string
  metadata?: Record<string, any>
}

export interface Message {
  id: string
  conversationId: string
  senderId: string
  content: string
  type: 'text' | 'file' | 'image' | 'code' | 'system'
  timestamp: Date
  metadata: MessageMetadata
  reactions?: Reaction[]
  references?: MessageReference[]
}

export interface MessageMetadata {
  tokens?: number
  model?: string
  processingTime?: number
  confidence?: number
  intent?: Intent
  entities?: Entity[]
}

export interface Intent {
  name: string
  confidence: number
  parameters: Record<string, any>
}

export interface Entity {
  type: string
  value: string
  start: number
  end: number
  confidence: number
}

export interface Reaction {
  emoji: string
  userId: string
  timestamp: Date
}

export interface MessageReference {
  messageId: string
  type: 'reply' | 'quote' | 'thread' | 'reference'
  snippet?: string
}

export interface ConversationContext {
  workspaceId?: string
  documentIds: string[]
  tags: string[]
  topic?: string
  summary?: string
  keyPoints: string[]
  actionItems: ActionItem[]
}

export interface ActionItem {
  id: string
  description: string
  assignee?: string
  dueDate?: Date
  status: 'pending' | 'in_progress' | 'completed' | 'cancelled'
  priority: 'low' | 'medium' | 'high'
}

export interface ConversationMetadata {
  language: string
  domain?: string
  classification: ConversationClassification
  analytics: ConversationAnalytics
  settings: ConversationSettings
}

export interface ConversationClassification {
  category: string
  subcategory?: string
  confidence: number
  tags: string[]
  sensitivity: 'public' | 'internal' | 'confidential' | 'restricted'
}

export interface ConversationAnalytics {
  messageCount: number
  participantCount: number
  averageResponseTime: number
  sentimentScore: number
  engagementLevel: number
  topicDrift: number
  resolutionStatus?: 'resolved' | 'unresolved' | 'pending'
}

export interface ConversationSettings {
  autoArchive: boolean
  archiveAfterDays?: number
  notificationsEnabled: boolean
  searchable: boolean
  retentionPolicy?: RetentionPolicy
}

export interface RetentionPolicy {
  keepDays: number
  autoDelete: boolean
  archiveBeforeDelete: boolean
  exceptions: string[]
}

export interface ConversationIntelligence {
  conversationId: string
  insights: ConversationInsight[]
  recommendations: ConversationRecommendation[]
  patterns: ConversationPattern[]
  performance: ConversationPerformance
  generatedAt: Date
}

export interface ConversationInsight {
  type: 'sentiment' | 'intent' | 'topic' | 'engagement' | 'resolution'
  description: string
  confidence: number
  evidence: string[]
  timestamp: Date
}

export interface ConversationRecommendation {
  type: 'improvement' | 'follow_up' | 'escalation' | 'automation'
  priority: 'low' | 'medium' | 'high'
  description: string
  action: string
  rationale: string
}

export interface ConversationPattern {
  id: string
  type: string
  frequency: number
  examples: string[]
  significance: number
}

export interface ConversationPerformance {
  responseQuality: number
  userSatisfaction: number
  goalAchievement: number
  efficiency: number
  lastMeasured: Date
}