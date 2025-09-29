// Embedding Service
import { apiClient } from '../api'
import {
  EmbeddingModel,
  EmbeddingRequest,
  EmbeddingResponse,
  EmbeddingCache,
  EmbeddingSettings,
  EmbeddingJob,
  EmbeddingComparison,
  ApiResponse,
} from '../types'

export class EmbeddingService {
  /**
   * Generate embeddings for text
   */
  async generateEmbeddings(request: EmbeddingRequest): Promise<ApiResponse<EmbeddingResponse>> {
    return apiClient.invoke('generate_embeddings', {
      text: request.text,
      model: request.model,
      options: request.options,
    })
  }

  /**
   * Get available embedding models
   */
  async getEmbeddingModels(): Promise<ApiResponse<EmbeddingModel[]>> {
    return apiClient.invoke('get_embedding_models')
  }

  /**
   * Get specific embedding model
   */
  async getEmbeddingModel(modelId: string): Promise<ApiResponse<EmbeddingModel>> {
    return apiClient.invoke('get_embedding_model', { model_id: modelId })
  }

  /**
   * Set default embedding model
   */
  async setDefaultEmbeddingModel(modelId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_default_embedding_model', { model_id: modelId })
  }

  /**
   * Generate document embeddings
   */
  async generateDocumentEmbeddings(
    documentId: string,
    model?: string
  ): Promise<ApiResponse<EmbeddingResponse>> {
    return apiClient.invoke('generate_document_embeddings', {
      document_id: documentId,
      model: model,
    })
  }

  /**
   * Batch generate embeddings
   */
  async batchGenerateEmbeddings(
    texts: string[],
    model?: string
  ): Promise<ApiResponse<EmbeddingJob>> {
    return apiClient.invoke('batch_generate_embeddings', {
      texts,
      model: model,
    })
  }

  /**
   * Get embedding job status
   */
  async getEmbeddingJobStatus(jobId: string): Promise<ApiResponse<EmbeddingJob>> {
    return apiClient.invoke('get_embedding_job_status', { job_id: jobId })
  }

  /**
   * Cancel embedding job
   */
  async cancelEmbeddingJob(jobId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('cancel_embedding_job', { job_id: jobId })
  }

  /**
   * Get cached embeddings
   */
  async getCachedEmbeddings(
    textHash: string,
    model: string
  ): Promise<ApiResponse<EmbeddingCache | null>> {
    return apiClient.invoke('get_cached_embeddings', {
      text_hash: textHash,
      model: model,
    })
  }

  /**
   * Clear embedding cache
   */
  async clearEmbeddingCache(model?: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('clear_embedding_cache', { model: model })
  }

  /**
   * Compare embeddings
   */
  async compareEmbeddings(
    embeddingA: number[],
    embeddingB: number[],
    method?: string
  ): Promise<ApiResponse<EmbeddingComparison>> {
    return apiClient.invoke('compare_embeddings', {
      embedding_a: embeddingA,
      embedding_b: embeddingB,
      method: method || 'cosine',
    })
  }

  /**
   * Find similar embeddings
   */
  async findSimilarEmbeddings(
    embedding: number[],
    threshold?: number,
    limit?: number
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('find_similar_embeddings', {
      embedding,
      threshold: threshold || 0.7,
      limit: limit || 10,
    })
  }

  /**
   * Get embedding settings
   */
  async getEmbeddingSettings(): Promise<ApiResponse<EmbeddingSettings>> {
    return apiClient.invoke('get_embedding_settings')
  }

  /**
   * Update embedding settings
   */
  async updateEmbeddingSettings(
    settings: Partial<EmbeddingSettings>
  ): Promise<ApiResponse<EmbeddingSettings>> {
    return apiClient.invoke('update_embedding_settings', settings)
  }

  /**
   * Validate embedding model
   */
  async validateEmbeddingModel(modelId: string): Promise<ApiResponse<boolean>> {
    return apiClient.invoke('validate_embedding_model', { model_id: modelId })
  }

  /**
   * Get embedding statistics
   */
  async getEmbeddingStatistics(timeframe?: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_embedding_statistics', {
      timeframe: timeframe || 'last_30_days',
    })
  }

  /**
   * Optimize embedding storage
   */
  async optimizeEmbeddingStorage(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('optimize_embedding_storage')
  }

  /**
   * Export embeddings
   */
  async exportEmbeddings(
    format: string,
    filters?: Record<string, unknown>
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_embeddings', {
      format,
      filters: filters || {},
    })
  }

  /**
   * Import embeddings
   */
  async importEmbeddings(filePath: string, format: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('import_embeddings', {
      file_path: filePath,
      format,
    })
  }

  /**
   * Cluster embeddings
   */
  async clusterEmbeddings(
    embeddings: number[][],
    options?: unknown
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('cluster_embeddings', {
      embeddings,
      options: options || {},
    })
  }

  /**
   * Reduce embedding dimensions
   */
  async reduceEmbeddingDimensions(
    embeddings: number[][],
    targetDimensions: number
  ): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('reduce_embedding_dimensions', {
      embeddings,
      target_dimensions: targetDimensions,
    })
  }

  /**
   * Analyze embedding quality
   */
  async analyzeEmbeddingQuality(embeddings: number[][]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_embedding_quality', { embeddings })
  }

  /**
   * Get embedding usage report
   */
  async getEmbeddingUsageReport(
    startDate?: string,
    endDate?: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_embedding_usage_report', {
      start_date: startDate,
      end_date: endDate,
    })
  }
}

export const embeddingService = new EmbeddingService()
