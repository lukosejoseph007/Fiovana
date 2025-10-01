// Document Processing Service
import { apiClient } from '../api'
import {
  Document,
  DocumentMetadata,
  DocumentChunk,
  DocumentIndex,
  DocumentComparison,
  DocumentGeneration,
  FormatConversion,
  ApiResponse,
} from '../types'

export class DocumentService {
  /**
   * Initialize the document indexer (should be called on app startup)
   */
  async initializeIndexer(indexDir?: string): Promise<ApiResponse<boolean>> {
    return apiClient.invoke('init_document_indexer', {
      index_dir: indexDir,
    })
  }

  /**
   * Get all indexed documents
   */
  async getAllDocuments(): Promise<ApiResponse<Document[]>> {
    return apiClient.invoke('get_all_documents', {})
  }

  /**
   * Index a document (add it to the system)
   */
  async indexDocumentFile(filePath: string): Promise<ApiResponse<Document>> {
    return apiClient.invoke('index_document', {
      request: {
        file_path: filePath,
      },
    })
  }

  /**
   * Get document details by file path
   */
  async getDocumentDetails(filePath: string): Promise<ApiResponse<Document>> {
    return apiClient.invoke('get_document_details', {
      filePath: filePath,
    })
  }

  /**
   * Remove document from index
   */
  async removeDocument(documentId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('remove_document_from_indexer', {
      document_id: documentId,
    })
  }

  /**
   * Get index statistics
   */
  async getIndexStats(): Promise<
    ApiResponse<{
      total_documents: number
      total_keywords: number
      total_content_size: number
      index_version: number
    }>
  > {
    return apiClient.invoke('get_index_stats', {})
  }

  /**
   * Process a document - extract content, metadata, and create chunks
   */
  async processDocument(
    filePath: string,
    options?: Record<string, unknown>
  ): Promise<ApiResponse<Document>> {
    return apiClient.invoke('process_document', {
      file_path: filePath,
      options: options || {},
    })
  }

  /**
   * Get document by ID (file path)
   */
  async getDocument(documentId: string): Promise<ApiResponse<Document>> {
    // First, try to get all documents to find the path by ID
    const allDocsResponse = await this.getAllDocuments()

    if (allDocsResponse.success && allDocsResponse.data) {
      // Find the document by ID to get its path
      const doc = allDocsResponse.data.find((d: Document) => d.id === documentId)

      if (doc && doc.path) {
        // Use the actual file path
        return apiClient.invoke('get_document_details', { filePath: doc.path })
      }
    }

    // Fallback: if documentId looks like a path, use it directly
    if (documentId.includes('/') || documentId.includes('\\')) {
      return apiClient.invoke('get_document_details', { filePath: documentId })
    }

    // If we can't find the document, return an error
    return {
      success: false,
      error: `Document with ID ${documentId} not found`,
      metadata: { executionTime: 0 }
    }
  }

  /**
   * Update document metadata
   */
  async updateDocument(
    documentId: string,
    updates: Partial<Document>
  ): Promise<ApiResponse<Document>> {
    return apiClient.invoke('update_document', {
      document_id: documentId,
      ...updates,
    })
  }

  /**
   * Delete a document
   */
  async deleteDocument(documentId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_document', { document_id: documentId })
  }

  /**
   * List documents in workspace
   * @deprecated Use getAllDocuments() instead - connects to real backend indexer
   */
  async listDocuments(
    _workspaceId?: string,
    _filters?: Record<string, unknown>
  ): Promise<ApiResponse<Document[]>> {
    // Redirect to the real backend implementation
    return this.getAllDocuments()
  }

  /**
   * Extract text content from document
   */
  async extractContent(filePath: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('extract_document_content', { file_path: filePath })
  }

  /**
   * Extract metadata from document
   */
  async extractMetadata(filePath: string): Promise<ApiResponse<DocumentMetadata>> {
    return apiClient.invoke('extract_document_metadata', { file_path: filePath })
  }

  /**
   * Chunk document into smaller pieces
   */
  async chunkDocument(
    documentId: string,
    options?: Record<string, unknown>
  ): Promise<ApiResponse<DocumentChunk[]>> {
    return apiClient.invoke('chunk_document', {
      document_id: documentId,
      options: options || {},
    })
  }

  /**
   * Get document chunks
   */
  async getDocumentChunks(documentId: string): Promise<ApiResponse<DocumentChunk[]>> {
    return apiClient.invoke('get_document_chunks', { document_id: documentId })
  }

  /**
   * Index document for search
   */
  async indexDocument(documentId: string): Promise<ApiResponse<DocumentIndex>> {
    return apiClient.invoke('index_document', {
      request: {
        file_path: documentId,
      },
    })
  }

  /**
   * Get document index
   */
  async getDocumentIndex(documentId: string): Promise<ApiResponse<DocumentIndex>> {
    return apiClient.invoke('get_document_index', { document_id: documentId })
  }

  /**
   * Compare two documents
   */
  async compareDocuments(
    documentAId: string,
    documentBId: string,
    options?: Record<string, unknown>
  ): Promise<ApiResponse<DocumentComparison>> {
    return apiClient.invoke('compare_documents', {
      document_a_id: documentAId,
      document_b_id: documentBId,
      options: options || {},
    })
  }

  /**
   * Generate document from template
   */
  async generateDocument(
    templateId: string,
    parameters: Record<string, unknown>
  ): Promise<ApiResponse<DocumentGeneration>> {
    return apiClient.invoke('generate_document', {
      template_id: templateId,
      parameters,
    })
  }

  /**
   * Convert document format
   */
  async convertFormat(
    documentId: string,
    targetFormat: string,
    options?: Record<string, unknown>
  ): Promise<ApiResponse<FormatConversion>> {
    return apiClient.invoke('convert_document_format', {
      document_id: documentId,
      target_format: targetFormat,
      options: options || {},
    })
  }

  /**
   * Validate document structure
   */
  async validateDocument(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_document', { document_id: documentId })
  }

  /**
   * Repair corrupted document
   */
  async repairDocument(documentId: string): Promise<ApiResponse<Document>> {
    return apiClient.invoke('repair_document', { document_id: documentId })
  }

  /**
   * Merge multiple documents
   */
  async mergeDocuments(
    documentIds: string[],
    options?: Record<string, unknown>
  ): Promise<ApiResponse<Document>> {
    return apiClient.invoke('merge_documents', {
      document_ids: documentIds,
      options: options || {},
    })
  }

  /**
   * Split document into multiple parts
   */
  async splitDocument(documentId: string, criteria: unknown): Promise<ApiResponse<Document[]>> {
    return apiClient.invoke('split_document', {
      document_id: documentId,
      criteria,
    })
  }

  /**
   * Analyze document structure
   */
  async analyzeStructure(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_document_structure', { document_id: documentId })
  }

  /**
   * Get document statistics
   */
  async getDocumentStats(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_document_stats', { document_id: documentId })
  }

  /**
   * Detect document language
   */
  async detectLanguage(documentId: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('detect_document_language', { document_id: documentId })
  }

  /**
   * Classify document content
   */
  async classifyDocument(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('classify_document', { document_id: documentId })
  }

  /**
   * Extract entities from document
   */
  async extractEntities(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_document_entities', { document_id: documentId })
  }

  /**
   * Summarize document content
   */
  async summarizeDocument(
    documentId: string,
    options?: Record<string, unknown>
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('summarize_document', {
      document_id: documentId,
      options: options || {},
    })
  }

  /**
   * Translate document content
   */
  async translateDocument(
    documentId: string,
    targetLanguage: string
  ): Promise<ApiResponse<Document>> {
    return apiClient.invoke('translate_document', {
      document_id: documentId,
      target_language: targetLanguage,
    })
  }

  /**
   * Check document for plagiarism
   */
  async checkPlagiarism(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('check_document_plagiarism', { document_id: documentId })
  }

  /**
   * Get document version history
   */
  async getVersionHistory(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('get_document_versions', { document_id: documentId })
  }

  /**
   * Create document version
   */
  async createVersion(documentId: string, versionInfo: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('create_document_version', {
      document_id: documentId,
      version_info: versionInfo,
    })
  }

  /**
   * Comprehensive document analysis (structure + content + style)
   */
  async analyzeDocument(
    documentId: string,
    options?: Record<string, unknown>
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_document', {
      document_id: documentId,
      options: options || {},
    })
  }
}

export const documentService = new DocumentService()
