// Knowledge Analyzer Service
import { apiClient } from '../api'
import {
  KnowledgeGap,
  KnowledgeGapAnalysis,
  KnowledgeBase,
  KnowledgeDomain,
  GapRecommendation,
  ApiResponse
} from '../types'

export class KnowledgeAnalyzerService {
  /**
   * Analyze knowledge gaps in workspace
   */
  async analyzeKnowledgeGaps(workspaceId: string, options?: any): Promise<ApiResponse<KnowledgeGapAnalysis>> {
    return apiClient.invoke('analyze_knowledge_gaps', {
      workspace_id: workspaceId,
      options: options || {}
    })
  }

  /**
   * Identify specific knowledge gap
   */
  async identifyKnowledgeGap(documentId: string, domain?: string): Promise<ApiResponse<KnowledgeGap[]>> {
    return apiClient.invoke('identify_knowledge_gap', {
      document_id: documentId,
      domain: domain
    })
  }

  /**
   * Create knowledge base
   */
  async createKnowledgeBase(name: string, description: string, workspaceId?: string): Promise<ApiResponse<KnowledgeBase>> {
    return apiClient.invoke('create_knowledge_base', {
      name,
      description,
      workspace_id: workspaceId
    })
  }

  /**
   * Get knowledge base
   */
  async getKnowledgeBase(knowledgeBaseId: string): Promise<ApiResponse<KnowledgeBase>> {
    return apiClient.invoke('get_knowledge_base', { knowledge_base_id: knowledgeBaseId })
  }

  /**
   * Update knowledge base
   */
  async updateKnowledgeBase(knowledgeBaseId: string, updates: Partial<KnowledgeBase>): Promise<ApiResponse<KnowledgeBase>> {
    return apiClient.invoke('update_knowledge_base', {
      knowledge_base_id: knowledgeBaseId,
      ...updates
    })
  }

  /**
   * List knowledge bases
   */
  async listKnowledgeBases(workspaceId?: string): Promise<ApiResponse<KnowledgeBase[]>> {
    return apiClient.invoke('list_knowledge_bases', { workspace_id: workspaceId })
  }

  /**
   * Add document to knowledge base
   */
  async addDocumentToKnowledgeBase(knowledgeBaseId: string, documentId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('add_document_to_knowledge_base', {
      knowledge_base_id: knowledgeBaseId,
      document_id: documentId
    })
  }

  /**
   * Remove document from knowledge base
   */
  async removeDocumentFromKnowledgeBase(knowledgeBaseId: string, documentId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('remove_document_from_knowledge_base', {
      knowledge_base_id: knowledgeBaseId,
      document_id: documentId
    })
  }

  /**
   * Analyze knowledge domain coverage
   */
  async analyzeDomainCoverage(knowledgeBaseId: string): Promise<ApiResponse<KnowledgeDomain[]>> {
    return apiClient.invoke('analyze_domain_coverage', { knowledge_base_id: knowledgeBaseId })
  }

  /**
   * Get gap recommendations
   */
  async getGapRecommendations(gapId: string): Promise<ApiResponse<GapRecommendation[]>> {
    return apiClient.invoke('get_gap_recommendations', { gap_id: gapId })
  }

  /**
   * Prioritize knowledge gaps
   */
  async prioritizeKnowledgeGaps(workspaceId: string, criteria?: any): Promise<ApiResponse<KnowledgeGap[]>> {
    return apiClient.invoke('prioritize_knowledge_gaps', {
      workspace_id: workspaceId,
      criteria: criteria || {}
    })
  }

  /**
   * Track gap resolution progress
   */
  async trackGapResolution(gapId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('track_gap_resolution', { gap_id: gapId })
  }

  /**
   * Mark knowledge gap as resolved
   */
  async markGapAsResolved(gapId: string, resolutionDetails: any): Promise<ApiResponse<void>> {
    return apiClient.invoke('mark_gap_as_resolved', {
      gap_id: gapId,
      resolution_details: resolutionDetails
    })
  }

  /**
   * Generate knowledge coverage report
   */
  async generateCoverageReport(knowledgeBaseId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('generate_coverage_report', { knowledge_base_id: knowledgeBaseId })
  }

  /**
   * Suggest knowledge acquisition sources
   */
  async suggestAcquisitionSources(gapId: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('suggest_acquisition_sources', { gap_id: gapId })
  }

  /**
   * Analyze knowledge redundancy
   */
  async analyzeKnowledgeRedundancy(knowledgeBaseId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('analyze_knowledge_redundancy', { knowledge_base_id: knowledgeBaseId })
  }

  /**
   * Extract knowledge from documents
   */
  async extractKnowledge(documentIds: string[], domain?: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('extract_knowledge', {
      document_ids: documentIds,
      domain: domain
    })
  }

  /**
   * Validate knowledge accuracy
   */
  async validateKnowledgeAccuracy(knowledgeBaseId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('validate_knowledge_accuracy', { knowledge_base_id: knowledgeBaseId })
  }

  /**
   * Get knowledge trends
   */
  async getKnowledgeTrends(knowledgeBaseId: string, timeframe?: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('get_knowledge_trends', {
      knowledge_base_id: knowledgeBaseId,
      timeframe: timeframe || 'last_30_days'
    })
  }
}

export const knowledgeAnalyzerService = new KnowledgeAnalyzerService()