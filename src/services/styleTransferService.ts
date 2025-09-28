// Style Transfer Service
import { apiClient } from '../api'
import {
  StyleTransfer,
  TransferResult,
  StyleProfile,
  OrganizationalStyle,
  ApiResponse
} from '../types'

export class StyleTransferService {
  /**
   * Transfer style from one document to another
   */
  async transferStyle(sourceDocumentId: string, targetDocumentId: string, options?: any): Promise<ApiResponse<StyleTransfer>> {
    return apiClient.invoke('transfer_document_style', {
      source_document_id: sourceDocumentId,
      target_document_id: targetDocumentId,
      options: options || {}
    })
  }

  /**
   * Apply style profile to document
   */
  async applyStyleProfile(documentId: string, styleProfileId: string, options?: any): Promise<ApiResponse<TransferResult>> {
    return apiClient.invoke('apply_style_profile', {
      document_id: documentId,
      style_profile_id: styleProfileId,
      options: options || {}
    })
  }

  /**
   * Transform document to organizational style
   */
  async applyOrganizationalStyle(documentId: string, organizationId: string): Promise<ApiResponse<TransferResult>> {
    return apiClient.invoke('apply_organizational_style', {
      document_id: documentId,
      organization_id: organizationId
    })
  }

  /**
   * Preview style transfer before applying
   */
  async previewStyleTransfer(documentId: string, styleProfileId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('preview_style_transfer', {
      document_id: documentId,
      style_profile_id: styleProfileId
    })
  }

  /**
   * Batch style transfer for multiple documents
   */
  async batchStyleTransfer(documentIds: string[], styleProfileId: string): Promise<ApiResponse<TransferResult[]>> {
    return apiClient.invoke('batch_style_transfer', {
      document_ids: documentIds,
      style_profile_id: styleProfileId
    })
  }

  /**
   * Customize style transfer parameters
   */
  async customizeTransferParameters(transferId: string, parameters: any): Promise<ApiResponse<TransferResult>> {
    return apiClient.invoke('customize_transfer_parameters', {
      transfer_id: transferId,
      parameters
    })
  }

  /**
   * Revert style transfer
   */
  async revertStyleTransfer(transferId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('revert_style_transfer', { transfer_id: transferId })
  }

  /**
   * Get style transfer history
   */
  async getTransferHistory(documentId: string): Promise<ApiResponse<StyleTransfer[]>> {
    return apiClient.invoke('get_style_transfer_history', { document_id: documentId })
  }

  /**
   * Validate style transfer quality
   */
  async validateTransferQuality(transferId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('validate_transfer_quality', { transfer_id: transferId })
  }

  /**
   * Get transfer recommendations
   */
  async getTransferRecommendations(documentId: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('get_transfer_recommendations', { document_id: documentId })
  }

  /**
   * Fine-tune style transfer
   */
  async fineTuneTransfer(transferId: string, adjustments: any): Promise<ApiResponse<TransferResult>> {
    return apiClient.invoke('fine_tune_style_transfer', {
      transfer_id: transferId,
      adjustments
    })
  }

  /**
   * Create organizational style guide
   */
  async createOrganizationalStyle(organizationId: string, styleData: any): Promise<ApiResponse<OrganizationalStyle>> {
    return apiClient.invoke('create_organizational_style', {
      organization_id: organizationId,
      style_data: styleData
    })
  }

  /**
   * Update organizational style guide
   */
  async updateOrganizationalStyle(styleId: string, updates: any): Promise<ApiResponse<OrganizationalStyle>> {
    return apiClient.invoke('update_organizational_style', {
      style_id: styleId,
      updates
    })
  }

  /**
   * Get organizational style compliance
   */
  async checkStyleCompliance(documentId: string, organizationId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('check_style_compliance', {
      document_id: documentId,
      organization_id: organizationId
    })
  }

  /**
   * Generate style transfer report
   */
  async generateTransferReport(transferId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('generate_transfer_report', { transfer_id: transferId })
  }
}

export const styleTransferService = new StyleTransferService()