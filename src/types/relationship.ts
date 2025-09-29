// Relationship Analysis Types
export interface DocumentRelationship {
  id: string
  sourceDocumentId: string
  targetDocumentId: string
  type: RelationshipType
  strength: number
  confidence: number
  metadata: RelationshipMetadata
  discoveredAt: Date
  lastVerified: Date
}

export interface RelationshipType {
  category: 'content' | 'structural' | 'temporal' | 'semantic' | 'authorship' | 'reference'
  subcategory: string
  description: string
  bidirectional: boolean
}

export interface RelationshipMetadata {
  method: 'semantic_similarity' | 'citation_analysis' | 'content_overlap' | 'temporal_proximity'
  evidence: Evidence[]
  parameters: Record<string, unknown>
  qualityScore: number
}

export interface Evidence {
  type: string
  description: string
  location?: Location[]
  strength: number
  verified: boolean
}

export interface Location {
  documentId: string
  position: number
  length: number
  context: string
}

export interface RelationshipGraph {
  id: string
  name: string
  nodes: GraphNode[]
  edges: GraphEdge[]
  metadata: GraphMetadata
  createdAt: Date
  updatedAt: Date
}

export interface GraphNode {
  id: string
  documentId: string
  label: string
  type: string
  properties: Record<string, unknown>
  position?: GraphPosition
  style?: NodeStyle
}

export interface GraphPosition {
  x: number
  y: number
  z?: number
}

export interface NodeStyle {
  color?: string
  size?: number
  shape?: string
  icon?: string
}

export interface GraphEdge {
  id: string
  sourceId: string
  targetId: string
  relationshipId: string
  label?: string
  weight: number
  style?: EdgeStyle
  properties: Record<string, unknown>
}

export interface EdgeStyle {
  color?: string
  width?: number
  style?: 'solid' | 'dashed' | 'dotted'
  arrowType?: string
}

export interface GraphMetadata {
  nodeCount: number
  edgeCount: number
  density: number
  avgDegree: number
  clustering: number
  algorithms: string[]
  lastAnalysis: Date
}

export interface RelationshipCluster {
  id: string
  documents: string[]
  centerDocument?: string
  coherence: number
  size: number
  topics: ClusterTopic[]
  relationships: string[]
}

export interface ClusterTopic {
  name: string
  keywords: string[]
  confidence: number
  prevalence: number
}

export interface PathAnalysis {
  source: string
  target: string
  paths: DocumentPath[]
  shortestPath?: DocumentPath
  analysis: PathMetrics
}

export interface DocumentPath {
  documents: string[]
  relationships: string[]
  length: number
  strength: number
  confidence: number
}

export interface PathMetrics {
  averagePathLength: number
  pathStrength: number
  alternatives: number
  reliability: number
}

export interface RelationshipQuery {
  documentIds?: string[]
  types?: string[]
  strengthThreshold?: number
  confidenceThreshold?: number
  maxDepth?: number
  includeMetadata?: boolean
  filters?: RelationshipFilter[]
}

export interface RelationshipFilter {
  field: string
  operator: 'equals' | 'contains' | 'gt' | 'lt' | 'in'
  value: unknown
}

export interface RelationshipAnalysis {
  documentId: string
  inbound: RelationshipSummary
  outbound: RelationshipSummary
  patterns: RelationshipPattern[]
  recommendations: RelationshipRecommendation[]
  centrality: CentralityMetrics
}

export interface RelationshipSummary {
  count: number
  types: Record<string, number>
  averageStrength: number
  strongestRelationship?: DocumentRelationship
}

export interface RelationshipPattern {
  type: string
  description: string
  frequency: number
  significance: number
  examples: string[]
}

export interface RelationshipRecommendation {
  type: 'connection' | 'organization' | 'exploration' | 'validation'
  description: string
  targetDocuments: string[]
  confidence: number
  rationale: string
}

export interface CentralityMetrics {
  degree: number
  betweenness: number
  closeness: number
  eigenvector: number
  pageRank: number
}
