// Content Lifecycle Service
import { apiClient } from '../api'
import {
  ContentLifecycle,
  LifecycleAction,
  LifecycleMetrics,
  LifecycleEvent,
  ApiResponse
} from '../types'

export class ContentLifecycleService {
  /**
   * Get content lifecycle status
   */
  async getContentLifecycle(documentId: string): Promise<ApiResponse<ContentLifecycle>> {
    return apiClient.invoke('get_content_lifecycle', { document_id: documentId })
  }

  /**
   * Update content lifecycle stage
   */
  async updateLifecycleStage(documentId: string, stage: string, notes?: string): Promise<ApiResponse<ContentLifecycle>> {
    return apiClient.invoke('update_lifecycle_stage', {
      document_id: documentId,
      stage,
      notes: notes
    })
  }

  /**
   * Schedule lifecycle action
   */
  async scheduleLifecycleAction(documentId: string, action: LifecycleAction): Promise<ApiResponse<void>> {
    return apiClient.invoke('schedule_lifecycle_action', {
      document_id: documentId,
      action
    })
  }

  /**
   * Execute lifecycle action
   */
  async executeLifecycleAction(actionId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('execute_lifecycle_action', { action_id: actionId })
  }

  /**
   * Cancel scheduled action
   */
  async cancelScheduledAction(actionId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('cancel_scheduled_action', { action_id: actionId })
  }

  /**
   * Get lifecycle metrics
   */
  async getLifecycleMetrics(documentId: string): Promise<ApiResponse<LifecycleMetrics>> {
    return apiClient.invoke('get_lifecycle_metrics', { document_id: documentId })
  }

  /**
   * Get lifecycle history
   */
  async getLifecycleHistory(documentId: string): Promise<ApiResponse<LifecycleEvent[]>> {
    return apiClient.invoke('get_lifecycle_history', { document_id: documentId })
  }

  /**
   * Analyze content aging
   */
  async analyzeContentAging(workspaceId?: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_content_aging', { workspace_id: workspaceId })
  }

  /**
   * Get lifecycle recommendations
   */
  async getLifecycleRecommendations(documentId: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('get_lifecycle_recommendations', { document_id: documentId })
  }

  /**
   * Set lifecycle policies
   */
  async setLifecyclePolicies(workspaceId: string, policies: any): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_lifecycle_policies', {
      workspace_id: workspaceId,
      policies
    })
  }

  /**
   * Get lifecycle policies
   */
  async getLifecyclePolicies(workspaceId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_lifecycle_policies', { workspace_id: workspaceId })
  }

  /**
   * Archive content
   */
  async archiveContent(documentId: string, archiveLocation?: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('archive_content', {
      document_id: documentId,
      archive_location: archiveLocation
    })
  }

  /**
   * Restore content from archive
   */
  async restoreContent(documentId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('restore_content', { document_id: documentId })
  }

  /**
   * Get content retention status
   */
  async getRetentionStatus(documentId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_retention_status', { document_id: documentId })
  }

  /**
   * Apply retention policy
   */
  async applyRetentionPolicy(documentId: string, policyId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('apply_retention_policy', {
      document_id: documentId,
      policy_id: policyId
    })
  }

  /**
   * Monitor content usage
   */
  async monitorContentUsage(documentIds: string[]): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('monitor_content_usage', { document_ids: documentIds })
  }

  /**
   * Predict content lifecycle
   */
  async predictContentLifecycle(documentId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('predict_content_lifecycle', { document_id: documentId })
  }

  /**
   * Get lifecycle analytics
   */
  async getLifecycleAnalytics(workspaceId: string, timeframe?: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_lifecycle_analytics', {
      workspace_id: workspaceId,
      timeframe: timeframe || 'last_30_days'
    })
  }

  /**
   * Trigger content review
   */
  async triggerContentReview(documentId: string, reviewType: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('trigger_content_review', {
      document_id: documentId,
      review_type: reviewType
    })
  }

  /**
   * Complete content review
   */
  async completeContentReview(reviewId: string, reviewData: any): Promise<ApiResponse<void>> {
    return apiClient.invoke('complete_content_review', {
      review_id: reviewId,
      review_data: reviewData
    })
  }

  /**
   * Get pending actions
   */
  async getPendingActions(workspaceId?: string): Promise<ApiResponse<LifecycleAction[]>> {
    return apiClient.invoke('get_pending_lifecycle_actions', { workspace_id: workspaceId })
  }

  /**
   * Bulk lifecycle operations
   */
  async bulkLifecycleOperations(operations: any[]): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('bulk_lifecycle_operations', { operations })
  }
}

export const contentLifecycleService = new ContentLifecycleService()