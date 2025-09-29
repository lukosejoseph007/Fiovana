// Content Adaptation Service
import { apiClient } from '../api'
import {
  ContentAdaptation,
  ApiResponse
} from '../types'

export class ContentAdaptationService {
  /**
   * Adapt content for different audiences
   */
  async adaptContentForAudience(
    documentId: string,
    targetAudience: string,
    adaptationOptions?: unknown
  ): Promise<ApiResponse<ContentAdaptation>> {
    return apiClient.invoke('adapt_content_for_audience', {
      document_id: documentId,
      target_audience: targetAudience,
      options: adaptationOptions || {}
    })
  }

  /**
   * Adapt content for different formats
   */
  async adaptContentForFormat(
    documentId: string,
    targetFormat: string,
    formatOptions?: unknown
  ): Promise<ApiResponse<ContentAdaptation>> {
    return apiClient.invoke('adapt_content_for_format', {
      document_id: documentId,
      target_format: targetFormat,
      options: formatOptions || {}
    })
  }

  /**
   * Adapt content complexity level
   */
  async adaptContentComplexity(
    documentId: string,
    complexityLevel: 'simple' | 'intermediate' | 'advanced',
    adaptationOptions?: unknown
  ): Promise<ApiResponse<ContentAdaptation>> {
    return apiClient.invoke('adapt_content_complexity', {
      document_id: documentId,
      complexity_level: complexityLevel,
      options: adaptationOptions || {}
    })
  }

  /**
   * Adapt content length
   */
  async adaptContentLength(
    documentId: string,
    targetLength: number,
    lengthOptions?: unknown
  ): Promise<ApiResponse<ContentAdaptation>> {
    return apiClient.invoke('adapt_content_length', {
      document_id: documentId,
      target_length: targetLength,
      options: lengthOptions || {}
    })
  }

  /**
   * Adapt content tone and style
   */
  async adaptContentTone(
    documentId: string,
    targetTone: string,
    toneOptions?: unknown
  ): Promise<ApiResponse<ContentAdaptation>> {
    return apiClient.invoke('adapt_content_tone', {
      document_id: documentId,
      target_tone: targetTone,
      options: toneOptions || {}
    })
  }

  /**
   * Batch adapt content with multiple adaptations
   */
  async batchAdaptContent(
    documentId: string,
    adaptations: Array<{
      type: 'audience' | 'format' | 'complexity' | 'length' | 'tone',
      target: string | number,
      options?: unknown
    }>
  ): Promise<ApiResponse<ContentAdaptation[]>> {
    return apiClient.invoke('batch_adapt_content', {
      document_id: documentId,
      adaptations
    })
  }

  /**
   * Get adaptation history for a document
   */
  async getAdaptationHistory(documentId: string): Promise<ApiResponse<ContentAdaptation[]>> {
    return apiClient.invoke('get_adaptation_history', {
      document_id: documentId
    })
  }

  /**
   * Preview content adaptation without applying
   */
  async previewContentAdaptation(
    documentId: string,
    adaptationType: string,
    adaptationTarget: string | number,
    previewOptions?: unknown
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('preview_content_adaptation', {
      document_id: documentId,
      adaptation_type: adaptationType,
      adaptation_target: adaptationTarget,
      options: previewOptions || {}
    })
  }

  /**
   * Analyze content adaptation requirements
   */
  async analyzeAdaptationRequirements(
    documentId: string,
    targetContext: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_adaptation_requirements', {
      document_id: documentId,
      target_context: targetContext
    })
  }
}

export const contentAdaptationService = new ContentAdaptationService()