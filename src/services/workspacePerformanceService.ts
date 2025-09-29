// Workspace Performance Service
import { apiClient } from '../api'
import { PerformanceMetrics, ApiResponse } from '../types'

export class WorkspacePerformanceService {
  /**
   * Monitor workspace performance metrics
   */
  async monitorPerformance(workspaceId: string): Promise<ApiResponse<PerformanceMetrics>> {
    return apiClient.invoke('monitor_workspace_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Get workspace performance history
   */
  async getPerformanceHistory(
    workspaceId: string,
    timeRange?: string
  ): Promise<ApiResponse<PerformanceMetrics[]>> {
    return apiClient.invoke('get_workspace_performance_history', {
      workspace_id: workspaceId,
      time_range: timeRange || '7d',
    })
  }

  /**
   * Analyze performance bottlenecks
   */
  async analyzeBottlenecks(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('analyze_performance_bottlenecks', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Optimize workspace performance
   */
  async optimizePerformance(
    workspaceId: string,
    optimizationOptions?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('optimize_workspace_performance', {
      workspace_id: workspaceId,
      options: optimizationOptions || {},
    })
  }

  /**
   * Monitor document processing performance
   */
  async monitorDocumentProcessing(workspaceId: string): Promise<ApiResponse<PerformanceMetrics>> {
    return apiClient.invoke('monitor_document_processing_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Monitor search performance
   */
  async monitorSearchPerformance(workspaceId: string): Promise<ApiResponse<PerformanceMetrics>> {
    return apiClient.invoke('monitor_search_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Monitor AI service performance
   */
  async monitorAIPerformance(workspaceId: string): Promise<ApiResponse<PerformanceMetrics>> {
    return apiClient.invoke('monitor_ai_service_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Get performance alerts
   */
  async getPerformanceAlerts(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('get_performance_alerts', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Set performance thresholds
   */
  async setPerformanceThresholds(
    workspaceId: string,
    thresholds: unknown
  ): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_performance_thresholds', {
      workspace_id: workspaceId,
      thresholds,
    })
  }

  /**
   * Generate performance report
   */
  async generatePerformanceReport(
    workspaceId: string,
    reportOptions?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_performance_report', {
      workspace_id: workspaceId,
      options: reportOptions || {},
    })
  }

  /**
   * Compare performance across workspaces
   */
  async compareWorkspacePerformance(workspaceIds: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('compare_workspace_performance', {
      workspace_ids: workspaceIds,
    })
  }

  /**
   * Predict performance trends
   */
  async predictPerformanceTrends(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('predict_performance_trends', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Monitor resource utilization
   */
  async monitorResourceUtilization(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('monitor_resource_utilization', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Analyze query performance
   */
  async analyzeQueryPerformance(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_query_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Optimize index performance
   */
  async optimizeIndexPerformance(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('optimize_index_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Monitor cache performance
   */
  async monitorCachePerformance(workspaceId: string): Promise<ApiResponse<PerformanceMetrics>> {
    return apiClient.invoke('monitor_cache_performance', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Clear performance alerts
   */
  async clearPerformanceAlerts(
    workspaceId: string,
    alertIds?: string[]
  ): Promise<ApiResponse<void>> {
    return apiClient.invoke('clear_performance_alerts', {
      workspace_id: workspaceId,
      alert_ids: alertIds || [],
    })
  }

  /**
   * Schedule performance analysis
   */
  async schedulePerformanceAnalysis(
    workspaceId: string,
    schedule: string
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('schedule_performance_analysis', {
      workspace_id: workspaceId,
      schedule,
    })
  }

  /**
   * Export performance metrics
   */
  async exportPerformanceMetrics(
    workspaceId: string,
    format?: 'json' | 'csv' | 'xlsx'
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_performance_metrics', {
      workspace_id: workspaceId,
      format: format || 'json',
    })
  }
}

export const workspacePerformanceService = new WorkspacePerformanceService()
