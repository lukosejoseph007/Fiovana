// Document Editing Service - Backend integration for document save/versioning
import { apiClient } from '../api'
import { ApiResponse } from '../types'

// Rust backend uses camelCase for enum variants
export type DocumentFormatType = 'markdown' | 'plainText' | 'html'

export interface SaveDocumentRequest {
  documentId: string
  content: string
  format: DocumentFormatType
}

export interface SaveDocumentResponse {
  success: boolean
  documentId: string
  path: string
  size: number
  modifiedAt: string
  message: string
}

export interface VersionInfo {
  versionId: string
  documentId: string
  createdAt: string
  size: number
  hash: string
  description: string
}

export class DocumentEditingService {
  /**
   * Save document content to file system
   */
  async saveDocument(
    documentId: string,
    content: string,
    format: DocumentFormatType = 'markdown'
  ): Promise<ApiResponse<SaveDocumentResponse>> {
    return apiClient.invoke('save_document', {
      request: {
        documentId,
        content,
        format,
      },
    })
  }

  /**
   * Create a version snapshot of the document
   */
  async createDocumentVersion(
    documentId: string,
    content: string
  ): Promise<ApiResponse<VersionInfo>> {
    return apiClient.invoke('create_document_version', {
      documentId,
      content,
    })
  }

  /**
   * Get all versions of a document
   */
  async getDocumentVersions(documentId: string): Promise<ApiResponse<VersionInfo[]>> {
    return apiClient.invoke('get_document_versions', {
      documentId,
    })
  }

  /**
   * Restore a document to a previous version
   */
  async restoreDocumentVersion(
    documentId: string,
    versionId: string
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('restore_document_version', {
      documentId,
      versionId,
    })
  }
}

export const documentEditingService = new DocumentEditingService()
