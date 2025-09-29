// Content Classification Service
import { apiClient } from '../api'
import {
  ContentClassification,
  ApiResponse
} from '../types'

export class ContentClassificationService {
  /**
   * Classify document content type
   */
  async classifyContentType(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_content_type', {
      document_id: documentId
    })
  }

  /**
   * Classify document topic
   */
  async classifyTopic(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_topic', {
      document_id: documentId
    })
  }

  /**
   * Classify document sentiment
   */
  async classifySentiment(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_sentiment', {
      document_id: documentId
    })
  }

  /**
   * Classify document complexity level
   */
  async classifyComplexity(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_complexity', {
      document_id: documentId
    })
  }

  /**
   * Classify document audience suitability
   */
  async classifyAudience(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_audience', {
      document_id: documentId
    })
  }

  /**
   * Classify document urgency/priority
   */
  async classifyUrgency(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_urgency', {
      document_id: documentId
    })
  }

  /**
   * Classify document format and structure
   */
  async classifyFormat(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_format', {
      document_id: documentId
    })
  }

  /**
   * Classify document quality metrics
   */
  async classifyQuality(documentId: string): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('classify_quality', {
      document_id: documentId
    })
  }

  /**
   * Multi-dimensional content classification
   */
  async classifyMultiDimensional(
    documentId: string,
    dimensions: string[]
  ): Promise<ApiResponse<ContentClassification[]>> {
    return apiClient.invoke('classify_multi_dimensional', {
      document_id: documentId,
      dimensions
    })
  }

  /**
   * Batch classify multiple documents
   */
  async batchClassify(
    documentIds: string[],
    classificationTypes: string[]
  ): Promise<ApiResponse<ContentClassification[]>> {
    return apiClient.invoke('batch_classify_content', {
      document_ids: documentIds,
      classification_types: classificationTypes
    })
  }

  /**
   * Get classification confidence scores
   */
  async getClassificationConfidence(
    documentId: string,
    classificationType: string
  ): Promise<ApiResponse<number>> {
    return apiClient.invoke('get_classification_confidence', {
      document_id: documentId,
      classification_type: classificationType
    })
  }

  /**
   * Train custom classification model
   */
  async trainCustomClassifier(
    trainingData: unknown[],
    classifierConfig: Record<string, unknown>
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('train_custom_classifier', {
      training_data: trainingData,
      classifier_config: classifierConfig
    })
  }

  /**
   * Apply custom classification model
   */
  async applyCustomClassifier(
    documentId: string,
    classifierModelId: string
  ): Promise<ApiResponse<ContentClassification>> {
    return apiClient.invoke('apply_custom_classifier', {
      document_id: documentId,
      classifier_model_id: classifierModelId
    })
  }

  /**
   * Get classification history for document
   */
  async getClassificationHistory(documentId: string): Promise<ApiResponse<ContentClassification[]>> {
    return apiClient.invoke('get_classification_history', {
      document_id: documentId
    })
  }
}

export const contentClassificationService = new ContentClassificationService()