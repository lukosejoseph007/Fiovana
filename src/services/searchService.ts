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
    const startTime = Date.now()

    const response = await apiClient.invoke<{
      success: boolean
      results?: Array<{
        document: {
          id: string
          title: string
          content: string
          path: string
          metadata?: Record<string, unknown>
        }
        score: number
        snippets: string[]
      }>
      total_found: number
      error?: string
    }>('search_documents', {
      request: {
        query: query.text,
        filter: query.filters?.[0] || null,
        limit: query.options?.limit || 50,
      },
    })

    const executionTime = Date.now() - startTime

    // Map backend response to frontend SearchResult structure
    if (response.success && response.data?.success && response.data.results) {
      const mappedResults: SearchResult = {
        query,
        results: response.data.results.map(item => ({
          id: item.document.id,
          documentId: item.document.id,
          title: item.document.title,
          content: item.document.content,
          score: item.score,
          highlights: item.snippets.map(snippet => ({
            field: 'content',
            fragments: [snippet],
            positions: [],
          })),
          metadata: item.document.metadata || {},
          path: item.document.path,
        })),
        totalCount: response.data.total_found,
        executionTime,
        metadata: {
          algorithm: query.type,
          indexVersion: '1.0',
          performance: {
            queryTime: executionTime,
            indexTime: 0,
            postProcessingTime: 0,
            totalDocuments: response.data.total_found,
            documentsScanned: response.data.total_found,
          },
        },
      }

      return {
        success: true,
        data: mappedResults,
      }
    }

    // Return error response
    return {
      success: false,
      error: response.data?.error || response.error || 'Search failed',
    }
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
      workspaceId: workspaceId,
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
      documentIds: documentIds,
      query,
    })
  }

  /**
   * Get search suggestions
   */
  async getSearchSuggestions(partialQuery: string): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('get_search_suggestions', {
      partialQuery: partialQuery,
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
    return apiClient.invoke('delete_vector_space', { spaceId: spaceId })
  }

  /**
   * Add document to vector space
   */
  async addDocumentToVectorSpace(spaceId: string, documentId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('add_document_to_vector_space', {
      spaceId: spaceId,
      documentId: documentId,
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
      spaceId: spaceId,
      documentId: documentId,
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
      spaceId: spaceId,
      query,
    })
  }

  /**
   * Find similar documents using vectors
   */
  async findSimilarDocuments(documentId: string, limit?: number): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('find_similar_documents', {
      documentId: documentId,
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
      documentAId: documentAId,
      documentBId: documentBId,
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
      documentIds: documentIds,
      options: options || {},
    })
  }

  /**
   * Get document vector
   */
  async getDocumentVector(documentId: string): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('get_document_vector', { documentId: documentId })
  }

  /**
   * Update document vector
   */
  async updateDocumentVector(documentId: string, vector?: number[]): Promise<ApiResponse<void>> {
    return apiClient.invoke('update_document_vector', {
      documentId: documentId,
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
    return apiClient.invoke('get_vector_space_stats', { spaceId: spaceId })
  }

  /**
   * Export vector space
   */
  async exportVectorSpace(spaceId: string, format: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_vector_space', {
      spaceId: spaceId,
      format,
    })
  }

  /**
   * Import vector space
   */
  async importVectorSpace(filePath: string, format: string): Promise<ApiResponse<VectorSpace>> {
    return apiClient.invoke('import_vector_space', {
      filePath: filePath,
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
