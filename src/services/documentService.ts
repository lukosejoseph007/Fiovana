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
   * Get document by ID
   */
  async getDocument(documentId: string): Promise<ApiResponse<Document>> {
    return apiClient.invoke('get_document', { document_id: documentId })
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
   */
  async listDocuments(
    workspaceId?: string,
    filters?: Record<string, unknown>
  ): Promise<ApiResponse<Document[]>> {
    return apiClient.invoke('list_documents', {
      workspace_id: workspaceId,
      filters: filters || {},
    })
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
    return apiClient.invoke('index_document', { document_id: documentId })
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
}

export const documentService = new DocumentService()
