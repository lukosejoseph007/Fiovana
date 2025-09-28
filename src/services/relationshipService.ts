// Relationship Service
import { apiClient } from '../api'
import {
  Document,
  DocumentRelationship,
  ApiResponse
} from '../types'

export class RelationshipService {
  /**
   * Analyze relationships between documents
   */
  async analyzeDocumentRelationships(
    documentIds: string[]
  ): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('analyze_document_relationships', {
      document_ids: documentIds
    })
  }

  /**
   * Find similar documents
   */
  async findSimilarDocuments(
    documentId: string,
    similarityThreshold?: number,
    maxResults?: number
  ): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('find_similar_documents', {
      document_id: documentId,
      similarity_threshold: similarityThreshold || 0.7,
      max_results: maxResults || 10
    })
  }

  /**
   * Find document dependencies
   */
  async findDocumentDependencies(documentId: string): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('find_document_dependencies', {
      document_id: documentId
    })
  }

  /**
   * Find documents that reference a specific document
   */
  async findReferencingDocuments(documentId: string): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('find_referencing_documents', {
      document_id: documentId
    })
  }

  /**
   * Build document relationship graph
   */
  async buildRelationshipGraph(
    workspaceId: string,
    includeMetadata?: boolean
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('build_relationship_graph', {
      workspace_id: workspaceId,
      include_metadata: includeMetadata || false
    })
  }

  /**
   * Analyze semantic relationships
   */
  async analyzeSemanticRelationships(
    documentIds: string[]
  ): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('analyze_semantic_relationships', {
      document_ids: documentIds
    })
  }

  /**
   * Find content overlap between documents
   */
  async findContentOverlap(
    documentId1: string,
    documentId2: string
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('find_content_overlap', {
      document_id_1: documentId1,
      document_id_2: documentId2
    })
  }

  /**
   * Identify document clusters
   */
  async identifyDocumentClusters(
    workspaceId: string,
    clusteringOptions?: any
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('identify_document_clusters', {
      workspace_id: workspaceId,
      options: clusteringOptions || {}
    })
  }

  /**
   * Track document evolution relationships
   */
  async trackDocumentEvolution(documentId: string): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('track_document_evolution', {
      document_id: documentId
    })
  }

  /**
   * Analyze citation relationships
   */
  async analyzeCitationRelationships(
    workspaceId: string
  ): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('analyze_citation_relationships', {
      workspace_id: workspaceId
    })
  }

  /**
   * Find topically related documents
   */
  async findTopicallyRelated(
    documentId: string,
    topics?: string[]
  ): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('find_topically_related', {
      document_id: documentId,
      topics: topics || []
    })
  }

  /**
   * Calculate relationship strength
   */
  async calculateRelationshipStrength(
    documentId1: string,
    documentId2: string,
    relationshipType?: string
  ): Promise<ApiResponse<number>> {
    return apiClient.invoke('calculate_relationship_strength', {
      document_id_1: documentId1,
      document_id_2: documentId2,
      relationship_type: relationshipType
    })
  }

  /**
   * Get relationship history for document
   */
  async getRelationshipHistory(documentId: string): Promise<ApiResponse<DocumentRelationship[]>> {
    return apiClient.invoke('get_relationship_history', {
      document_id: documentId
    })
  }

  /**
   * Update relationship metadata
   */
  async updateRelationshipMetadata(
    relationshipId: string,
    metadata: any
  ): Promise<ApiResponse<DocumentRelationship>> {
    return apiClient.invoke('update_relationship_metadata', {
      relationship_id: relationshipId,
      metadata
    })
  }
}

export const relationshipService = new RelationshipService()