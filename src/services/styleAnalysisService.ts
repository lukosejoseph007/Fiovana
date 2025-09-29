// Style Analysis Service
import { apiClient } from '../api'
import {
  StyleProfile,
  StyleAnalysis,
  StyleMetrics,
  StyleRecommendation,
  ApiResponse
} from '../types'

export class StyleAnalysisService {
  /**
   * Analyze document style and create style profile
   */
  async analyzeDocumentStyle(documentId: string, options?: unknown): Promise<ApiResponse<StyleAnalysis>> {
    return apiClient.invoke('analyze_document_style', {
      document_id: documentId,
      options: options || {}
    })
  }

  /**
   * Create style profile from document
   */
  async createStyleProfile(documentId: string, profileName: string): Promise<ApiResponse<StyleProfile>> {
    return apiClient.invoke('create_style_profile', {
      document_id: documentId,
      profile_name: profileName
    })
  }

  /**
   * Get existing style profile
   */
  async getStyleProfile(profileId: string): Promise<ApiResponse<StyleProfile>> {
    return apiClient.invoke('get_style_profile', { profile_id: profileId })
  }

  /**
   * List all style profiles
   */
  async listStyleProfiles(): Promise<ApiResponse<StyleProfile[]>> {
    return apiClient.invoke('list_style_profiles')
  }

  /**
   * Update style profile
   */
  async updateStyleProfile(profileId: string, updates: Partial<StyleProfile>): Promise<ApiResponse<StyleProfile>> {
    return apiClient.invoke('update_style_profile', {
      profile_id: profileId,
      ...updates
    })
  }

  /**
   * Delete style profile
   */
  async deleteStyleProfile(profileId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_style_profile', { profile_id: profileId })
  }

  /**
   * Compare styles between documents
   */
  async compareDocumentStyles(documentAId: string, documentBId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('compare_document_styles', {
      document_a_id: documentAId,
      document_b_id: documentBId
    })
  }

  /**
   * Get style recommendations for document
   */
  async getStyleRecommendations(documentId: string): Promise<ApiResponse<StyleRecommendation[]>> {
    return apiClient.invoke('get_style_recommendations', { document_id: documentId })
  }

  /**
   * Analyze writing style patterns
   */
  async analyzeWritingPatterns(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_writing_patterns', { document_id: documentId })
  }

  /**
   * Extract style features from document
   */
  async extractStyleFeatures(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_style_features', { document_id: documentId })
  }

  /**
   * Calculate style metrics
   */
  async calculateStyleMetrics(documentId: string): Promise<ApiResponse<StyleMetrics>> {
    return apiClient.invoke('calculate_style_metrics', { document_id: documentId })
  }

  /**
   * Learn style from multiple documents
   */
  async learnStyleFromDocuments(documentIds: string[], styleName: string): Promise<ApiResponse<StyleProfile>> {
    return apiClient.invoke('learn_style_from_documents', {
      document_ids: documentIds,
      style_name: styleName
    })
  }

  /**
   * Validate style consistency
   */
  async validateStyleConsistency(documentIds: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_style_consistency', { document_ids: documentIds })
  }

  /**
   * Get style evolution over time
   */
  async getStyleEvolution(documentIds: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_style_evolution', { document_ids: documentIds })
  }

  /**
   * Detect style anomalies
   */
  async detectStyleAnomalies(documentId: string, referenceStyleId?: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('detect_style_anomalies', {
      document_id: documentId,
      reference_style_id: referenceStyleId
    })
  }

  /**
   * Generate style report
   */
  async generateStyleReport(documentId: string, options?: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_style_report', {
      document_id: documentId,
      options: options || {}
    })
  }
}

export const styleAnalysisService = new StyleAnalysisService()