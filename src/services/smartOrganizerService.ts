// Smart Organizer Service
import { apiClient } from '../api'
import {
  SmartOrganization,
  OrganizationSuggestion,
  AutomationRule,
  OrganizationPerformance,
  ApiResponse
} from '../types'

export class SmartOrganizerService {
  /**
   * Get smart organization analysis for workspace
   */
  async getSmartOrganization(workspaceId: string): Promise<ApiResponse<SmartOrganization>> {
    return apiClient.invoke('get_smart_organization', { workspace_id: workspaceId })
  }

  /**
   * Generate organization suggestions
   */
  async generateOrganizationSuggestions(workspaceId: string, options?: unknown): Promise<ApiResponse<OrganizationSuggestion[]>> {
    return apiClient.invoke('generate_organization_suggestions', {
      workspace_id: workspaceId,
      options: options || {}
    })
  }

  /**
   * Apply organization suggestion
   */
  async applyOrganizationSuggestion(suggestionId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('apply_organization_suggestion', { suggestion_id: suggestionId })
  }

  /**
   * Create automation rule
   */
  async createAutomationRule(workspaceId: string, ruleData: unknown): Promise<ApiResponse<AutomationRule>> {
    return apiClient.invoke('create_automation_rule', {
      workspace_id: workspaceId,
      rule_data: ruleData
    })
  }

  /**
   * Update automation rule
   */
  async updateAutomationRule(ruleId: string, updates: Partial<AutomationRule>): Promise<ApiResponse<AutomationRule>> {
    return apiClient.invoke('update_automation_rule', {
      rule_id: ruleId,
      ...updates
    })
  }

  /**
   * Delete automation rule
   */
  async deleteAutomationRule(ruleId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_automation_rule', { rule_id: ruleId })
  }

  /**
   * List automation rules
   */
  async listAutomationRules(workspaceId: string): Promise<ApiResponse<AutomationRule[]>> {
    return apiClient.invoke('list_automation_rules', { workspace_id: workspaceId })
  }

  /**
   * Enable automation rule
   */
  async enableAutomationRule(ruleId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('enable_automation_rule', { rule_id: ruleId })
  }

  /**
   * Disable automation rule
   */
  async disableAutomationRule(ruleId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('disable_automation_rule', { rule_id: ruleId })
  }

  /**
   * Test automation rule
   */
  async testAutomationRule(ruleId: string, testData?: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('test_automation_rule', {
      rule_id: ruleId,
      test_data: testData || {}
    })
  }

  /**
   * Get automation rule performance
   */
  async getRulePerformance(ruleId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_rule_performance', { rule_id: ruleId })
  }

  /**
   * Analyze file organization patterns
   */
  async analyzeOrganizationPatterns(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('analyze_organization_patterns', { workspace_id: workspaceId })
  }

  /**
   * Suggest file categorization
   */
  async suggestFileCategorization(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('suggest_file_categorization', { document_id: documentId })
  }

  /**
   * Auto-organize workspace
   */
  async autoOrganizeWorkspace(workspaceId: string, options?: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('auto_organize_workspace', {
      workspace_id: workspaceId,
      options: options || {}
    })
  }

  /**
   * Preview organization changes
   */
  async previewOrganizationChanges(workspaceId: string, suggestionIds: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('preview_organization_changes', {
      workspace_id: workspaceId,
      suggestion_ids: suggestionIds
    })
  }

  /**
   * Rollback organization changes
   */
  async rollbackOrganizationChanges(changeId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('rollback_organization_changes', { change_id: changeId })
  }

  /**
   * Get organization performance metrics
   */
  async getOrganizationPerformance(workspaceId: string): Promise<ApiResponse<OrganizationPerformance>> {
    return apiClient.invoke('get_organization_performance', { workspace_id: workspaceId })
  }

  /**
   * Analyze duplicate files
   */
  async analyzeDuplicateFiles(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('analyze_duplicate_files', { workspace_id: workspaceId })
  }

  /**
   * Suggest duplicate resolution
   */
  async suggestDuplicateResolution(duplicateGroupId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('suggest_duplicate_resolution', { duplicate_group_id: duplicateGroupId })
  }

  /**
   * Create organization template
   */
  async createOrganizationTemplate(workspaceId: string, templateName: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('create_organization_template', {
      workspace_id: workspaceId,
      template_name: templateName
    })
  }

  /**
   * Apply organization template
   */
  async applyOrganizationTemplate(workspaceId: string, templateId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('apply_organization_template', {
      workspace_id: workspaceId,
      template_id: templateId
    })
  }

  /**
   * Generate organization report
   */
  async generateOrganizationReport(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_organization_report', { workspace_id: workspaceId })
  }
}

export const smartOrganizerService = new SmartOrganizerService()