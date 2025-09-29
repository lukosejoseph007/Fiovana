// Search & Vector Operations Service
import { apiClient } from '../api'
import {
  SearchQuery,
  SearchResult,
  VectorSpace,
  VectorQuery,
  VectorSearchResult,
  IndexConfig,
  ApiResponse,
} from '../types'

export class SearchService {
  /**
   * Perform semantic search across documents
   */
  async search(query: SearchQuery): Promise<ApiResponse<SearchResult>> {
    return apiClient.invoke('search_documents', {
      text: query.text,
      type: query.type,
      filters: query.filters || [],
      options: query.options,
    })
  }

  /**
   * Perform keyword-based search
   */
  async keywordSearch(text: string, options?: unknown): Promise<ApiResponse<SearchResult>> {
    return apiClient.invoke('keyword_search', {
      text,
      options: options || {},
    })
  }

  /**
   * Perform semantic search using embeddings
   */
  async semanticSearch(text: string, options?: unknown): Promise<ApiResponse<SearchResult>> {
    return apiClient.invoke('semantic_search', {
      text,
      options: options || {},
    })
  }

  /**
   * Perform hybrid search (keyword + semantic)
   */
  async hybridSearch(text: string, options?: unknown): Promise<ApiResponse<SearchResult>> {
    return apiClient.invoke('hybrid_search', {
      text,
      options: options || {},
    })
  }

  /**
   * Search within a specific workspace
   */
  async searchWorkspace(
    workspaceId: string,
    query: SearchQuery
  ): Promise<ApiResponse<SearchResult>> {
    return apiClient.invoke('search_workspace', {
      workspace_id: workspaceId,
      query,
    })
  }

  /**
   * Search within specific documents
   */
  async searchDocuments(
    documentIds: string[],
    query: SearchQuery
  ): Promise<ApiResponse<SearchResult>> {
    return apiClient.invoke('search_in_documents', {
      document_ids: documentIds,
      query,
    })
  }

  /**
   * Get search suggestions
   */
  async getSearchSuggestions(partialQuery: string): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('get_search_suggestions', {
      partial_query: partialQuery,
    })
  }

  /**
   * Get search history
   */
  async getSearchHistory(limit?: number): Promise<ApiResponse<SearchQuery[]>> {
    return apiClient.invoke('get_search_history', {
      limit: limit || 50,
    })
  }

  /**
   * Save search query
   */
  async saveSearchQuery(query: SearchQuery, name: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('save_search_query', {
      query,
      name,
    })
  }

  /**
   * Get saved search queries
   */
  async getSavedQueries(): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('get_saved_queries')
  }

  /**
   * Create or update search index
   */
  async updateIndex(config?: IndexConfig): Promise<ApiResponse<void>> {
    return apiClient.invoke('update_search_index', {
      config: config || {},
    })
  }

  /**
   * Get index status
   */
  async getIndexStatus(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_index_status')
  }

  /**
   * Rebuild search index
   */
  async rebuildIndex(): Promise<ApiResponse<void>> {
    return apiClient.invoke('rebuild_search_index')
  }

  /**
   * Optimize search index
   */
  async optimizeIndex(): Promise<ApiResponse<void>> {
    return apiClient.invoke('optimize_search_index')
  }

  /**
   * Get index statistics
   */
  async getIndexStats(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_index_stats')
  }

  // Vector Operations

  /**
   * Create vector space
   */
  async createVectorSpace(
    name: string,
    dimensions: number,
    model: string
  ): Promise<ApiResponse<VectorSpace>> {
    return apiClient.invoke('create_vector_space', {
      name,
      dimensions,
      model,
    })
  }

  /**
   * Get vector spaces
   */
  async getVectorSpaces(): Promise<ApiResponse<VectorSpace[]>> {
    return apiClient.invoke('get_vector_spaces')
  }

  /**
   * Delete vector space
   */
  async deleteVectorSpace(spaceId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_vector_space', { space_id: spaceId })
  }

  /**
   * Add document to vector space
   */
  async addDocumentToVectorSpace(spaceId: string, documentId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('add_document_to_vector_space', {
      space_id: spaceId,
      document_id: documentId,
    })
  }

  /**
   * Remove document from vector space
   */
  async removeDocumentFromVectorSpace(
    spaceId: string,
    documentId: string
  ): Promise<ApiResponse<void>> {
    return apiClient.invoke('remove_document_from_vector_space', {
      space_id: spaceId,
      document_id: documentId,
    })
  }

  /**
   * Perform vector similarity search
   */
  async vectorSearch(
    spaceId: string,
    query: VectorQuery
  ): Promise<ApiResponse<VectorSearchResult>> {
    return apiClient.invoke('vector_search', {
      space_id: spaceId,
      query,
    })
  }

  /**
   * Find similar documents using vectors
   */
  async findSimilarDocuments(documentId: string, limit?: number): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('find_similar_documents', {
      document_id: documentId,
      limit: limit || 10,
    })
  }

  /**
   * Calculate document similarity
   */
  async calculateSimilarity(
    documentAId: string,
    documentBId: string
  ): Promise<ApiResponse<number>> {
    return apiClient.invoke('calculate_document_similarity', {
      document_a_id: documentAId,
      document_b_id: documentBId,
    })
  }

  /**
   * Cluster documents by similarity
   */
  async clusterDocuments(
    documentIds: string[],
    options?: unknown
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('cluster_documents', {
      document_ids: documentIds,
      options: options || {},
    })
  }

  /**
   * Get document vector
   */
  async getDocumentVector(documentId: string): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('get_document_vector', { document_id: documentId })
  }

  /**
   * Update document vector
   */
  async updateDocumentVector(documentId: string, vector?: number[]): Promise<ApiResponse<void>> {
    return apiClient.invoke('update_document_vector', {
      document_id: documentId,
      vector,
    })
  }

  /**
   * Batch vector operations
   */
  async batchVectorOperations(operations: unknown[]): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('batch_vector_operations', { operations })
  }

  /**
   * Get vector space statistics
   */
  async getVectorSpaceStats(spaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_vector_space_stats', { space_id: spaceId })
  }

  /**
   * Export vector space
   */
  async exportVectorSpace(spaceId: string, format: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_vector_space', {
      space_id: spaceId,
      format,
    })
  }

  /**
   * Import vector space
   */
  async importVectorSpace(filePath: string, format: string): Promise<ApiResponse<VectorSpace>> {
    return apiClient.invoke('import_vector_space', {
      file_path: filePath,
      format,
    })
  }

  /**
   * Clear search cache
   */
  async clearSearchCache(): Promise<ApiResponse<void>> {
    return apiClient.invoke('clear_search_cache')
  }

  /**
   * Get search performance metrics
   */
  async getSearchMetrics(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_search_metrics')
  }
}

export const searchService = new SearchService()
