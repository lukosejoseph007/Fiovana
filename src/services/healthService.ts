// System Health Service
import { apiClient } from '../api'
import { SystemHealth, HealthMetrics, ApiResponse } from '../types'

export class HealthService {
  /**
   * Get overall system health status
   */
  async getSystemHealth(): Promise<ApiResponse<SystemHealth>> {
    return apiClient.invoke('get_system_health', {})
  }

  /**
   * Get workspace health metrics
   */
  async getWorkspaceHealth(workspaceId: string): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('get_workspace_health', {
      workspace_id: workspaceId,
    })
  }

  /**
   * Check document processing health
   */
  async checkDocumentProcessingHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_document_processing_health', {})
  }

  /**
   * Check AI service health
   */
  async checkAIServiceHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_ai_service_health', {})
  }

  /**
   * Check search service health
   */
  async checkSearchServiceHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_search_service_health', {})
  }

  /**
   * Check embedding service health
   */
  async checkEmbeddingServiceHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_embedding_service_health', {})
  }

  /**
   * Monitor system performance metrics
   */
  async getPerformanceMetrics(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('get_performance_metrics', {})
  }

  /**
   * Check database health
   */
  async checkDatabaseHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_database_health', {})
  }

  /**
   * Check file system health
   */
  async checkFileSystemHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_file_system_health', {})
  }

  /**
   * Check memory usage
   */
  async checkMemoryUsage(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_memory_usage', {})
  }

  /**
   * Check CPU usage
   */
  async checkCPUUsage(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_cpu_usage', {})
  }

  /**
   * Check disk usage
   */
  async checkDiskUsage(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_disk_usage', {})
  }

  /**
   * Check network connectivity
   */
  async checkNetworkHealth(): Promise<ApiResponse<HealthMetrics>> {
    return apiClient.invoke('check_network_health', {})
  }

  /**
   * Run comprehensive health diagnostics
   */
  async runHealthDiagnostics(): Promise<ApiResponse<SystemHealth>> {
    return apiClient.invoke('run_health_diagnostics', {})
  }

  /**
   * Get health alerts and warnings
   */
  async getHealthAlerts(): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('get_health_alerts', {})
  }

  /**
   * Set health monitoring thresholds
   */
  async setHealthThresholds(thresholds: unknown): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_health_thresholds', { thresholds })
  }

  /**
   * Get health monitoring history
   */
  async getHealthHistory(
    timeRange?: string,
    metrics?: string[]
  ): Promise<ApiResponse<HealthMetrics[]>> {
    return apiClient.invoke('get_health_history', {
      time_range: timeRange || '24h',
      metrics: metrics || [],
    })
  }

  /**
   * Export health report
   */
  async exportHealthReport(format?: 'json' | 'csv' | 'pdf'): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_health_report', {
      format: format || 'json',
    })
  }

  /**
   * Schedule health checks
   */
  async scheduleHealthCheck(schedule: string, checkTypes: string[]): Promise<ApiResponse<string>> {
    return apiClient.invoke('schedule_health_check', {
      schedule,
      check_types: checkTypes,
    })
  }

  /**
   * Get system resource utilization
   */
  async getResourceUtilization(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_resource_utilization', {})
  }

  /**
   * Check service dependencies
   */
  async checkServiceDependencies(): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('check_service_dependencies', {})
  }

  /**
   * Validate system configuration
   */
  async validateSystemConfiguration(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_system_configuration', {})
  }

  /**
   * Get uptime statistics
   */
  async getUptimeStatistics(): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_uptime_statistics', {})
  }

  /**
   * Clear health alerts
   */
  async clearHealthAlerts(alertIds?: string[]): Promise<ApiResponse<void>> {
    return apiClient.invoke('clear_health_alerts', {
      alert_ids: alertIds || [],
    })
  }
}

export const healthService = new HealthService()
