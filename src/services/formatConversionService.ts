// Format Conversion Service
import { apiClient } from '../api'
import {
  FormatConversion,
  ConversionOptions,
  ConversionResult,
  ConversionMetadata,
  ApiResponse,
} from '../types'

export class FormatConversionService {
  /**
   * Convert document format
   */
  async convertDocument(
    documentId: string,
    targetFormat: string,
    options?: ConversionOptions
  ): Promise<ApiResponse<FormatConversion>> {
    return apiClient.invoke('convert_document_format', {
      document_id: documentId,
      target_format: targetFormat,
      options: options || {},
    })
  }

  /**
   * Convert file by path
   */
  async convertFile(
    filePath: string,
    targetFormat: string,
    options?: ConversionOptions
  ): Promise<ApiResponse<ConversionResult>> {
    return apiClient.invoke('convert_file_format', {
      file_path: filePath,
      target_format: targetFormat,
      options: options || {},
    })
  }

  /**
   * Get supported formats
   */
  async getSupportedFormats(): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('get_supported_formats')
  }

  /**
   * Get conversion capabilities
   */
  async getConversionCapabilities(sourceFormat: string): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('get_conversion_capabilities', { source_format: sourceFormat })
  }

  /**
   * Batch convert documents
   */
  async batchConvert(
    documentIds: string[],
    targetFormat: string,
    options?: ConversionOptions
  ): Promise<ApiResponse<FormatConversion[]>> {
    return apiClient.invoke('batch_convert_documents', {
      document_ids: documentIds,
      target_format: targetFormat,
      options: options || {},
    })
  }

  /**
   * Preview conversion
   */
  async previewConversion(documentId: string, targetFormat: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('preview_format_conversion', {
      document_id: documentId,
      target_format: targetFormat,
    })
  }

  /**
   * Get conversion status
   */
  async getConversionStatus(conversionId: string): Promise<ApiResponse<FormatConversion>> {
    return apiClient.invoke('get_conversion_status', { conversion_id: conversionId })
  }

  /**
   * Cancel conversion
   */
  async cancelConversion(conversionId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('cancel_conversion', { conversion_id: conversionId })
  }

  /**
   * Get conversion history
   */
  async getConversionHistory(documentId?: string): Promise<ApiResponse<FormatConversion[]>> {
    return apiClient.invoke('get_conversion_history', { document_id: documentId })
  }

  /**
   * Validate conversion quality
   */
  async validateConversionQuality(conversionId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_conversion_quality', { conversion_id: conversionId })
  }

  /**
   * Optimize conversion settings
   */
  async optimizeConversionSettings(
    sourceFormat: string,
    targetFormat: string
  ): Promise<ApiResponse<ConversionOptions>> {
    return apiClient.invoke('optimize_conversion_settings', {
      source_format: sourceFormat,
      target_format: targetFormat,
    })
  }

  /**
   * Convert with custom mapping
   */
  async convertWithMapping(
    documentId: string,
    targetFormat: string,
    mapping: Record<string, string>
  ): Promise<ApiResponse<FormatConversion>> {
    return apiClient.invoke('convert_with_custom_mapping', {
      document_id: documentId,
      target_format: targetFormat,
      mapping,
    })
  }

  /**
   * Extract conversion metadata
   */
  async extractConversionMetadata(conversionId: string): Promise<ApiResponse<ConversionMetadata>> {
    return apiClient.invoke('extract_conversion_metadata', { conversion_id: conversionId })
  }

  /**
   * Compare conversion results
   */
  async compareConversionResults(
    conversionAId: string,
    conversionBId: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('compare_conversion_results', {
      conversion_a_id: conversionAId,
      conversion_b_id: conversionBId,
    })
  }

  /**
   * Get format specifications
   */
  async getFormatSpecifications(format: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_format_specifications', { format })
  }

  /**
   * Validate format compatibility
   */
  async validateFormatCompatibility(
    sourceFormat: string,
    targetFormat: string
  ): Promise<ApiResponse<boolean>> {
    return apiClient.invoke('validate_format_compatibility', {
      source_format: sourceFormat,
      target_format: targetFormat,
    })
  }

  /**
   * Convert with preservation rules
   */
  async convertWithPreservation(
    documentId: string,
    targetFormat: string,
    preservationRules: unknown
  ): Promise<ApiResponse<FormatConversion>> {
    return apiClient.invoke('convert_with_preservation', {
      document_id: documentId,
      target_format: targetFormat,
      preservation_rules: preservationRules,
    })
  }

  /**
   * Auto-detect source format
   */
  async autoDetectFormat(filePath: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('auto_detect_format', { file_path: filePath })
  }

  /**
   * Generate conversion report
   */
  async generateConversionReport(conversionId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_conversion_report', { conversion_id: conversionId })
  }

  /**
   * Set conversion preferences
   */
  async setConversionPreferences(preferences: unknown): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_conversion_preferences', { preferences })
  }

  /**
   * Get conversion analytics
   */
  async getConversionAnalytics(timeframe?: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_conversion_analytics', {
      timeframe: timeframe || 'last_30_days',
    })
  }
}

export const formatConversionService = new FormatConversionService()
