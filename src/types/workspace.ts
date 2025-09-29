// Workspace Intelligence Types
export interface WorkspaceConfig {
  id: string
  name: string
  path: string
  description?: string
  createdAt: Date
  updatedAt: Date
}

export interface WorkspaceMetrics {
  totalFiles: number
  totalSize: number
  documentTypes: Record<string, number>
  lastActivity: Date
  activeProjects: number
}

export interface WorkspaceHealth {
  score: number
  status: 'excellent' | 'good' | 'fair' | 'poor'
  issues: HealthIssue[]
  recommendations: string[]
  lastChecked: Date
}

export interface HealthIssue {
  id: string
  type: 'performance' | 'organization' | 'security' | 'maintenance'
  severity: 'low' | 'medium' | 'high' | 'critical'
  description: string
  resolution?: string
}

export interface WorkspaceInsight {
  id: string
  type: 'pattern' | 'trend' | 'anomaly' | 'opportunity'
  title: string
  description: string
  confidence: number
  actionable: boolean
  suggestedActions?: string[]
}

export interface WorkspaceAnalysis {
  overview: WorkspaceMetrics
  health: WorkspaceHealth
  insights: WorkspaceInsight[]
  performanceMetrics: PerformanceMetrics
  organizationSuggestions: OrganizationSuggestion[]
}

export interface PerformanceMetrics {
  searchLatency: number
  indexingSpeed: number
  memoryUsage: number
  diskUsage: number
  operationThroughput: number
}

export interface OrganizationSuggestion {
  id: string
  type: 'folder_structure' | 'file_naming' | 'categorization' | 'tagging'
  priority: 'low' | 'medium' | 'high'
  description: string
  implementation: string
  estimatedImpact: number
}

export interface WorkspaceComparison {
  workspaceA: string
  workspaceB: string
  similarities: Similarity[]
  differences: Difference[]
  recommendations: ComparisonRecommendation[]
}

export interface Similarity {
  type: string
  description: string
  score: number
}

export interface Difference {
  type: string
  description: string
  impact: 'minor' | 'moderate' | 'significant'
}

export interface ComparisonRecommendation {
  type: 'adopt' | 'standardize' | 'optimize'
  description: string
  targetWorkspace: string
  priority: number
}

export interface WorkspaceBackup {
  id: string
  workspaceId: string
  timestamp: Date
  size: number
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  path: string
  metadata: BackupMetadata
}

export interface BackupMetadata {
  version: string
  fileCount: number
  compressionRatio: number
  checksums: Record<string, string>
}
