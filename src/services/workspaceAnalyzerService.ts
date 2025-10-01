// Advanced Workspace Analysis Service
import { apiClient } from '../api'
import { WorkspaceAnalysis, ApiResponse } from '../types'

export class WorkspaceAnalyzerService {
  /**
   * Helper to convert workspace ID to path
   * For now, we use "." for current directory as the default workspace
   */
  private getWorkspacePath(workspaceId?: string): string {
    // If a path-like workspaceId is provided, use it
    if (workspaceId && (workspaceId.startsWith('/') || workspaceId.startsWith('.'))) {
      return workspaceId
    }
    // Default to current directory
    return '.'
  }

  /**
   * Perform comprehensive workspace analysis
   */
  async analyzeWorkspace(workspaceId: string): Promise<ApiResponse<WorkspaceAnalysis>> {
    return apiClient.invoke('analyze_workspace', {
      request: {
        workspace_path: this.getWorkspacePath(workspaceId),
      },
    })
  }

  /**
   * Analyze workspace productivity patterns
   */
  async analyzeProductivityPatterns(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_workspace_health_score', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze document usage patterns
   */
  async analyzeDocumentUsage(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_document_usage_patterns', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze collaboration patterns
   */
  async analyzeCollaborationPatterns(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_collaboration_patterns', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze content quality distribution
   */
  async analyzeContentQuality(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_content_quality_distribution', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze knowledge gaps
   */
  async analyzeKnowledgeGaps(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('analyze_workspace_knowledge_gaps', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze workflow efficiency
   */
  async analyzeWorkflowEfficiency(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_workflow_efficiency', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze content lifecycle patterns
   */
  async analyzeContentLifecycle(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_content_lifecycle_patterns', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze workspace growth trends
   */
  async analyzeGrowthTrends(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_workspace_growth_trends', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze content duplication
   */
  async analyzeContentDuplication(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_content_duplication', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze workspace structure optimization
   */
  async analyzeStructureOptimization(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_structure_optimization', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze user behavior patterns
   */
  async analyzeUserBehavior(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_user_behavior_patterns', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze search patterns
   */
  async analyzeSearchPatterns(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_search_patterns', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Analyze content accessibility
   */
  async analyzeContentAccessibility(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_content_accessibility', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Generate workspace insights report
   */
  async generateInsightsReport(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_workspace_insights_report', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Compare workspace metrics over time
   */
  async compareMetricsOverTime(
    workspaceId: string,
    timeRanges: string[]
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('compare_workspace_metrics_over_time', {
      workspacePath: this.getWorkspacePath(workspaceId),
      time_ranges: timeRanges,
    })
  }

  /**
   * Predict workspace trends
   */
  async predictWorkspaceTrends(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('predict_workspace_trends', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Benchmark workspace performance
   */
  async benchmarkPerformance(
    workspaceId: string,
    benchmarkType?: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('benchmark_workspace_performance', {
      workspacePath: this.getWorkspacePath(workspaceId),
      benchmark_type: benchmarkType || 'standard',
    })
  }

  /**
   * Analyze workspace complexity
   */
  async analyzeComplexity(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_workspace_complexity', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }

  /**
   * Generate optimization recommendations
   */
  async generateOptimizationRecommendations(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('generate_optimization_recommendations', {
      workspacePath: this.getWorkspacePath(workspaceId),
    })
  }
}

export const workspaceAnalyzerService = new WorkspaceAnalyzerService()
