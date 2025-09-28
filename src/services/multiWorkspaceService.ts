// Multi-Workspace Analysis Service
import { apiClient } from '../api'
import {
  Workspace,
  WorkspaceComparison,
  MultiWorkspaceAnalysis,
  ApiResponse
} from '../types'

export class MultiWorkspaceService {
  /**
   * Compare multiple workspaces
   */
  async compareWorkspaces(
    workspaceIds: string[],
    comparisonMetrics?: string[]
  ): Promise<ApiResponse<WorkspaceComparison[]>> {
    return apiClient.invoke('compare_multiple_workspaces', {
      workspace_ids: workspaceIds,
      comparison_metrics: comparisonMetrics || []
    })
  }

  /**
   * Analyze cross-workspace content relationships
   */
  async analyzeCrossWorkspaceRelationships(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_cross_workspace_relationships', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Perform multi-workspace search
   */
  async searchAcrossWorkspaces(
    workspaceIds: string[],
    query: string,
    searchOptions?: any
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('search_across_workspaces', {
      workspace_ids: workspaceIds,
      query,
      options: searchOptions || {}
    })
  }

  /**
   * Analyze content distribution across workspaces
   */
  async analyzeContentDistribution(
    workspaceIds: string[]
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('analyze_content_distribution', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Find duplicate content across workspaces
   */
  async findCrossWorkspaceDuplicates(
    workspaceIds: string[],
    duplicateThreshold?: number
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('find_cross_workspace_duplicates', {
      workspace_ids: workspaceIds,
      duplicate_threshold: duplicateThreshold || 0.8
    })
  }

  /**
   * Analyze collaboration patterns across workspaces
   */
  async analyzeCollaborationPatterns(
    workspaceIds: string[]
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('analyze_multi_workspace_collaboration', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Generate multi-workspace insights
   */
  async generateMultiWorkspaceInsights(
    workspaceIds: string[],
    analysisScope?: string
  ): Promise<ApiResponse<MultiWorkspaceAnalysis>> {
    return apiClient.invoke('generate_multi_workspace_insights', {
      workspace_ids: workspaceIds,
      analysis_scope: analysisScope || 'comprehensive'
    })
  }

  /**
   * Benchmark workspace performance across multiple workspaces
   */
  async benchmarkWorkspacePerformance(
    workspaceIds: string[],
    benchmarkMetrics?: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('benchmark_multi_workspace_performance', {
      workspace_ids: workspaceIds,
      benchmark_metrics: benchmarkMetrics || []
    })
  }

  /**
   * Analyze workspace growth trends
   */
  async analyzeGrowthTrends(
    workspaceIds: string[],
    timeRange?: string
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_multi_workspace_growth_trends', {
      workspace_ids: workspaceIds,
      time_range: timeRange || '6m'
    })
  }

  /**
   * Find knowledge gaps across workspaces
   */
  async findKnowledgeGaps(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('find_multi_workspace_knowledge_gaps', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Analyze content quality distribution
   */
  async analyzeContentQualityDistribution(
    workspaceIds: string[]
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('analyze_multi_workspace_content_quality', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Perform cross-workspace style analysis
   */
  async analyzeCrossWorkspaceStyles(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_cross_workspace_styles', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Generate workspace migration recommendations
   */
  async generateMigrationRecommendations(
    sourceWorkspaceId: string,
    targetWorkspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('generate_workspace_migration_recommendations', {
      source_workspace_id: sourceWorkspaceId,
      target_workspace_ids: targetWorkspaceIds
    })
  }

  /**
   * Analyze workspace usage patterns
   */
  async analyzeUsagePatterns(
    workspaceIds: string[],
    timeRange?: string
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_multi_workspace_usage_patterns', {
      workspace_ids: workspaceIds,
      time_range: timeRange || '30d'
    })
  }

  /**
   * Perform multi-workspace security analysis
   */
  async performSecurityAnalysis(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('perform_multi_workspace_security_analysis', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Generate consolidation recommendations
   */
  async generateConsolidationRecommendations(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('generate_workspace_consolidation_recommendations', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Analyze resource utilization across workspaces
   */
  async analyzeResourceUtilization(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_multi_workspace_resource_utilization', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Perform cross-workspace compliance check
   */
  async performComplianceCheck(
    workspaceIds: string[],
    complianceRules?: any[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('perform_multi_workspace_compliance_check', {
      workspace_ids: workspaceIds,
      compliance_rules: complianceRules || []
    })
  }

  /**
   * Generate multi-workspace reports
   */
  async generateMultiWorkspaceReport(
    workspaceIds: string[],
    reportType?: string,
    reportOptions?: any
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('generate_multi_workspace_report', {
      workspace_ids: workspaceIds,
      report_type: reportType || 'comprehensive',
      options: reportOptions || {}
    })
  }

  /**
   * Analyze workspace interdependencies
   */
  async analyzeWorkspaceInterdependencies(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('analyze_workspace_interdependencies', {
      workspace_ids: workspaceIds
    })
  }

  /**
   * Perform multi-workspace backup coordination
   */
  async coordinateMultiWorkspaceBackup(
    workspaceIds: string[],
    backupOptions?: any
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('coordinate_multi_workspace_backup', {
      workspace_ids: workspaceIds,
      options: backupOptions || {}
    })
  }

  /**
   * Synchronize workspace configurations
   */
  async synchronizeWorkspaceConfigurations(
    sourceWorkspaceId: string,
    targetWorkspaceIds: string[],
    configurationTypes?: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('synchronize_workspace_configurations', {
      source_workspace_id: sourceWorkspaceId,
      target_workspace_ids: targetWorkspaceIds,
      configuration_types: configurationTypes || []
    })
  }

  /**
   * Perform cross-workspace data migration
   */
  async performCrossWorkspaceMigration(
    migrationPlan: any
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('perform_cross_workspace_migration', {
      migration_plan: migrationPlan
    })
  }

  /**
   * Monitor multi-workspace health
   */
  async monitorMultiWorkspaceHealth(
    workspaceIds: string[]
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('monitor_multi_workspace_health', {
      workspace_ids: workspaceIds
    })
  }
}

export const multiWorkspaceService = new MultiWorkspaceService()