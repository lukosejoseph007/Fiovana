// Workspace AI Integration Service
import { getWorkspacePath } from './workspacePathHelper'
import { apiClient } from '../api'
import { AIAnalysis, ApiResponse } from '../types'

export class WorkspaceAiService {
  /**
   * Generate AI-powered workspace insights
   */
  async generateWorkspaceInsights(workspaceId: string): Promise<ApiResponse<AIAnalysis>> {
    return apiClient.invoke('generate_workspace_ai_insights', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered content recommendation for workspace
   */
  async recommendContent(
    workspaceId: string,
    _userPreferences?: unknown
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('get_workspace_recommendations', {
      request: {
        workspace_path: getWorkspacePath(workspaceId),
      },
    })
  }

  /**
   * AI-powered workspace organization suggestions
   */
  async suggestOrganization(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_suggest_workspace_organization', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered content gap analysis
   */
  async analyzeContentGaps(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_analyze_content_gaps', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered workflow optimization
   */
  async optimizeWorkflow(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_optimize_workspace_workflow', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered document classification for workspace
   */
  async classifyWorkspaceDocuments(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_classify_workspace_documents', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered duplicate detection
   */
  async detectDuplicates(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_detect_workspace_duplicates', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered content quality assessment
   */
  async assessContentQuality(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_assess_content_quality', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered knowledge extraction from workspace
   */
  async extractKnowledge(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_extract_workspace_knowledge', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered productivity analysis
   */
  async analyzeProductivity(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_analyze_workspace_productivity', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered collaboration insights
   */
  async analyzeCollaboration(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_analyze_collaboration_patterns', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered content summarization for workspace
   */
  async summarizeWorkspace(workspaceId: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('ai_summarize_workspace_content', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered trend prediction
   */
  async predictTrends(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_predict_workspace_trends', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered anomaly detection
   */
  async detectAnomalies(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_detect_workspace_anomalies', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered content generation suggestions
   */
  async suggestContentGeneration(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_suggest_content_generation', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered workspace health scoring
   */
  async scoreWorkspaceHealth(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_score_workspace_health', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered personalized workspace dashboard
   */
  async generatePersonalizedDashboard(
    workspaceId: string,
    userId: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_generate_personalized_dashboard', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
      user_id: userId,
    })
  }

  /**
   * AI-powered content maintenance suggestions
   */
  async suggestContentMaintenance(workspaceId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('ai_suggest_content_maintenance', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
    })
  }

  /**
   * AI-powered workspace benchmarking
   */
  async benchmarkWorkspace(
    workspaceId: string,
    benchmarkCriteria?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('ai_benchmark_workspace', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
      benchmark_criteria: benchmarkCriteria || {},
    })
  }

  /**
   * AI-powered question answering about workspace
   */
  async answerWorkspaceQuestion(
    workspaceId: string,
    question: string
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('ai_answer_workspace_question', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
      question,
    })
  }

  /**
   * Train custom AI model for workspace
   */
  async trainCustomModel(workspaceId: string, modelConfig: unknown): Promise<ApiResponse<string>> {
    return apiClient.invoke('ai_train_custom_workspace_model', {
      request: { workspace_path: getWorkspacePath(workspaceId) },
      model_config: modelConfig,
    })
  }
}

export const workspaceAiService = new WorkspaceAiService()
