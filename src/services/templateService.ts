// Template Service
import { apiClient } from '../api'
import {
  ContentTemplate,
  TemplateVariable,
  TemplateUsage,
  TemplatePerformance,
  ApiResponse
} from '../types'

export class TemplateService {
  /**
   * Create new template
   */
  async createTemplate(templateData: Omit<ContentTemplate, 'id' | 'createdAt'>): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('create_template', { template_data: templateData })
  }

  /**
   * Get template by ID
   */
  async getTemplate(templateId: string): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('get_template', { template_id: templateId })
  }

  /**
   * Update template
   */
  async updateTemplate(templateId: string, updates: Partial<ContentTemplate>): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('update_template', {
      template_id: templateId,
      ...updates
    })
  }

  /**
   * Delete template
   */
  async deleteTemplate(templateId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_template', { template_id: templateId })
  }

  /**
   * List templates
   */
  async listTemplates(category?: string, tags?: string[]): Promise<ApiResponse<ContentTemplate[]>> {
    return apiClient.invoke('list_templates', {
      category: category,
      tags: tags || []
    })
  }

  /**
   * Search templates
   */
  async searchTemplates(query: string, filters?: Record<string, unknown>): Promise<ApiResponse<ContentTemplate[]>> {
    return apiClient.invoke('search_templates', {
      query,
      filters: filters || {}
    })
  }

  /**
   * Validate template
   */
  async validateTemplate(templateId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_template', { template_id: templateId })
  }

  /**
   * Preview template with variables
   */
  async previewTemplate(templateId: string, variables: Record<string, unknown>): Promise<ApiResponse<string>> {
    return apiClient.invoke('preview_template', {
      template_id: templateId,
      variables
    })
  }

  /**
   * Render template
   */
  async renderTemplate(templateId: string, variables: Record<string, unknown>, options?: unknown): Promise<ApiResponse<string>> {
    return apiClient.invoke('render_template', {
      template_id: templateId,
      variables,
      options: options || {}
    })
  }

  /**
   * Clone template
   */
  async cloneTemplate(templateId: string, newName: string): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('clone_template', {
      template_id: templateId,
      new_name: newName
    })
  }

  /**
   * Get template variables
   */
  async getTemplateVariables(templateId: string): Promise<ApiResponse<TemplateVariable[]>> {
    return apiClient.invoke('get_template_variables', { template_id: templateId })
  }

  /**
   * Add template variable
   */
  async addTemplateVariable(templateId: string, variable: TemplateVariable): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('add_template_variable', {
      template_id: templateId,
      variable
    })
  }

  /**
   * Update template variable
   */
  async updateTemplateVariable(templateId: string, variableName: string, updates: Partial<TemplateVariable>): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('update_template_variable', {
      template_id: templateId,
      variable_name: variableName,
      ...updates
    })
  }

  /**
   * Remove template variable
   */
  async removeTemplateVariable(templateId: string, variableName: string): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('remove_template_variable', {
      template_id: templateId,
      variable_name: variableName
    })
  }

  /**
   * Get template usage statistics
   */
  async getTemplateUsage(templateId: string): Promise<ApiResponse<TemplateUsage>> {
    return apiClient.invoke('get_template_usage', { template_id: templateId })
  }

  /**
   * Get template performance metrics
   */
  async getTemplatePerformance(templateId: string): Promise<ApiResponse<TemplatePerformance>> {
    return apiClient.invoke('get_template_performance', { template_id: templateId })
  }

  /**
   * Create template from document
   */
  async createTemplateFromDocument(documentId: string, templateName: string, variables?: string[]): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('create_template_from_document', {
      document_id: documentId,
      template_name: templateName,
      variables: variables || []
    })
  }

  /**
   * Export template
   */
  async exportTemplate(templateId: string, format: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_template', {
      template_id: templateId,
      format
    })
  }

  /**
   * Import template
   */
  async importTemplate(templateData: unknown, format: string): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('import_template', {
      template_data: templateData,
      format
    })
  }

  /**
   * Get template categories
   */
  async getTemplateCategories(): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('get_template_categories')
  }

  /**
   * Get popular templates
   */
  async getPopularTemplates(limit?: number): Promise<ApiResponse<ContentTemplate[]>> {
    return apiClient.invoke('get_popular_templates', { limit: limit || 10 })
  }

  /**
   * Get recommended templates
   */
  async getRecommendedTemplates(documentId?: string): Promise<ApiResponse<ContentTemplate[]>> {
    return apiClient.invoke('get_recommended_templates', { document_id: documentId })
  }

  /**
   * Analyze template effectiveness
   */
  async analyzeTemplateEffectiveness(templateId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_template_effectiveness', { template_id: templateId })
  }

  /**
   * Optimize template performance
   */
  async optimizeTemplatePerformance(templateId: string): Promise<ApiResponse<ContentTemplate>> {
    return apiClient.invoke('optimize_template_performance', { template_id: templateId })
  }

  /**
   * Generate template variations
   */
  async generateTemplateVariations(templateId: string, variationTypes: string[]): Promise<ApiResponse<ContentTemplate[]>> {
    return apiClient.invoke('generate_template_variations', {
      template_id: templateId,
      variation_types: variationTypes
    })
  }
}

export const templateService = new TemplateService()