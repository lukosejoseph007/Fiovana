// Document Generation Service
import { apiClient } from '../api'
import { DocumentGeneration, GenerationMetadata, ApiResponse } from '../types'

export class DocumentGenerationService {
  /**
   * Initialize the document generator (should be called on app startup)
   */
  async initializeGenerator(
    outputDirectory?: string,
    aiConfig?: unknown
  ): Promise<ApiResponse<boolean>> {
    return apiClient.invoke('init_document_generator', {
      output_directory: outputDirectory,
      ai_config: aiConfig,
    })
  }

  /**
   * Generate document from template
   */
  async generateFromTemplate(
    templateId: string,
    variables: Record<string, unknown>,
    outputFormat: string = 'docx'
  ): Promise<ApiResponse<DocumentGeneration>> {
    return apiClient.invoke('generate_from_template', {
      template_id: templateId,
      request: {
        variables,
        output_format: outputFormat,
      },
    })
  }

  /**
   * Generate document from prompt
   */
  async generateFromPrompt(
    prompt: string,
    format: string,
    options?: unknown
  ): Promise<ApiResponse<DocumentGeneration>> {
    return apiClient.invoke('generate_document_from_prompt', {
      prompt,
      format,
      options: options || {},
    })
  }

  /**
   * Generate document outline
   */
  async generateOutline(
    topic: string,
    documentType: string,
    options?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_document_outline', {
      topic,
      document_type: documentType,
      options: options || {},
    })
  }

  /**
   * Generate content sections
   */
  async generateContentSections(
    outline: unknown,
    parameters?: unknown
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('generate_content_sections', {
      outline,
      parameters: parameters || {},
    })
  }

  /**
   * Expand document section
   */
  async expandDocumentSection(
    sectionId: string,
    targetLength?: number
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('expand_document_section', {
      section_id: sectionId,
      target_length: targetLength,
    })
  }

  /**
   * Generate document summary
   */
  async generateDocumentSummary(
    documentId: string,
    options?: unknown
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('generate_document_summary', {
      document_id: documentId,
      options: options || {},
    })
  }

  /**
   * Generate bibliography
   */
  async generateBibliography(sources: unknown[], style: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('generate_bibliography', {
      sources,
      style,
    })
  }

  /**
   * Generate table of contents
   */
  async generateTableOfContents(
    documentId: string,
    options?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_table_of_contents', {
      document_id: documentId,
      options: options || {},
    })
  }

  /**
   * Generate document variations
   */
  async generateDocumentVariations(
    documentId: string,
    variationTypes: string[]
  ): Promise<ApiResponse<DocumentGeneration[]>> {
    return apiClient.invoke('generate_document_variations', {
      document_id: documentId,
      variation_types: variationTypes,
    })
  }

  /**
   * Auto-complete document
   */
  async autoCompleteDocument(
    documentId: string,
    fromPosition?: number
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('auto_complete_document', {
      document_id: documentId,
      from_position: fromPosition,
    })
  }

  /**
   * Generate document metadata
   */
  async generateDocumentMetadata(documentId: string): Promise<ApiResponse<GenerationMetadata>> {
    return apiClient.invoke('generate_document_metadata', { document_id: documentId })
  }

  /**
   * Validate generated content
   */
  async validateGeneratedContent(generationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_generated_content', { generation_id: generationId })
  }

  /**
   * Improve generated content
   */
  async improveGeneratedContent(
    generationId: string,
    improvementType: string
  ): Promise<ApiResponse<DocumentGeneration>> {
    return apiClient.invoke('improve_generated_content', {
      generation_id: generationId,
      improvement_type: improvementType,
    })
  }

  /**
   * Generate content with constraints
   */
  async generateWithConstraints(
    prompt: string,
    constraints: unknown
  ): Promise<ApiResponse<DocumentGeneration>> {
    return apiClient.invoke('generate_content_with_constraints', {
      prompt,
      constraints,
    })
  }

  /**
   * Batch document generation
   */
  async batchGenerate(requests: unknown[]): Promise<ApiResponse<DocumentGeneration[]>> {
    return apiClient.invoke('batch_document_generation', { requests })
  }

  /**
   * Get generation history
   */
  async getGenerationHistory(documentId?: string): Promise<ApiResponse<DocumentGeneration[]>> {
    return apiClient.invoke('get_generation_history', { document_id: documentId })
  }

  /**
   * Get generation status
   */
  async getGenerationStatus(generationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_generation_status', { generation_id: generationId })
  }

  /**
   * Cancel generation
   */
  async cancelGeneration(generationId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('cancel_generation', { generation_id: generationId })
  }

  /**
   * Generate document from text content
   */
  async generateFromText(
    title: string,
    content: string,
    outputFilename: string,
    outputFormat: string = 'docx',
    metadata?: Record<string, string>
  ): Promise<ApiResponse<DocumentGeneration>> {
    // Map the output format string to the proper OutputFormat enum value
    const formatMap: Record<string, string> = {
      docx: 'Docx',
      pdf: 'Pdf',
      html: 'Html',
      markdown: 'Markdown',
      md: 'Markdown',
      txt: 'PlainText',
      text: 'PlainText',
      pptx: 'Docx', // Fallback - PPTX not directly supported in basic generator
    }

    const mappedFormat = formatMap[outputFormat.toLowerCase()] || 'Docx'

    // Backend expects all parameters wrapped in a 'request' object
    return apiClient.invoke('generate_document_from_text', {
      request: {
        title,
        content,
        metadata: metadata || {},
        options: {
          format: mappedFormat,
          template: null,
          style_options: {},
          include_metadata: true,
        },
        output_filename: outputFilename,
      },
    })
  }

  /**
   * Generate multilingual content
   */
  async generateMultilingualContent(
    prompt: string,
    languages: string[]
  ): Promise<ApiResponse<DocumentGeneration[]>> {
    return apiClient.invoke('generate_multilingual_content', {
      prompt,
      languages,
    })
  }

  /**
   * Customize generation parameters
   */
  async customizeGenerationParameters(
    generationId: string,
    parameters: unknown
  ): Promise<ApiResponse<DocumentGeneration>> {
    return apiClient.invoke('customize_generation_parameters', {
      generation_id: generationId,
      parameters,
    })
  }
}

export const documentGenerationService = new DocumentGenerationService()
