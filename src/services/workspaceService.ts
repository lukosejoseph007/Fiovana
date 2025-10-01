// Workspace Intelligence Service
import { getWorkspacePath } from './workspacePathHelper'
import { apiClient } from '../api'
import {
  WorkspaceConfig,
  WorkspaceAnalysis,
  WorkspaceHealth,
  WorkspaceMetrics,
  WorkspaceComparison,
  WorkspaceBackup,
  ApiResponse,
} from '../types'

export class WorkspaceService {
  /**
   * Create a new workspace
   */
  async createWorkspace(
    config: Omit<WorkspaceConfig, 'id' | 'createdAt' | 'updatedAt'>
  ): Promise<ApiResponse<WorkspaceConfig>> {
    return apiClient.invoke('create_workspace', {
      name: config.name,
      path: config.path,
      description: config.description,
    })
  }

  /**
   * Get workspace configuration
   */
  async getWorkspace(workspaceId: string): Promise<ApiResponse<WorkspaceConfig>> {
    return apiClient.invoke('get_workspace', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Update workspace configuration
   */
  async updateWorkspace(
    workspaceId: string,
    updates: Partial<WorkspaceConfig>
  ): Promise<ApiResponse<WorkspaceConfig>> {
    return apiClient.invoke('update_workspace', {
      workspace_path: getWorkspacePath(workspaceId),
      ...updates,
    })
  }

  /**
   * Delete a workspace
   */
  async deleteWorkspace(workspaceId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_workspace', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * List all workspaces
   */
  async listWorkspaces(): Promise<ApiResponse<WorkspaceConfig[]>> {
    return apiClient.invoke('list_workspaces')
  }

  /**
   * Analyze workspace structure and provide comprehensive insights
   */
  async analyzeWorkspace(workspaceId: string): Promise<ApiResponse<WorkspaceAnalysis>> {
    return apiClient.invoke('analyze_workspace', {
      request: {
        workspace_path: getWorkspacePath(workspaceId),
      },
    })
  }

  /**
   * Get workspace health metrics
   */
  async getWorkspaceHealth(workspaceId: string): Promise<ApiResponse<WorkspaceHealth>> {
    return apiClient.invoke('get_workspace_health_score', {
      workspacePath: getWorkspacePath(workspaceId),
    })
  }

  /**
   * Get workspace metrics overview
   */
  async getWorkspaceMetrics(workspaceId: string): Promise<ApiResponse<WorkspaceMetrics>> {
    return apiClient.invoke('get_workspace_metrics', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Compare two workspaces
   */
  async compareWorkspaces(
    workspaceAId: string,
    workspaceBId: string
  ): Promise<ApiResponse<WorkspaceComparison>> {
    return apiClient.invoke('compare_workspaces', {
      workspace_a_id: workspaceAId,
      workspace_b_id: workspaceBId,
    })
  }

  /**
   * Get workspace insights and recommendations
   */
  async getWorkspaceInsights(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_workspace_insights', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Optimize workspace organization
   */
  async optimizeWorkspace(workspaceId: string, options?: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('optimize_workspace', {
      workspace_path: getWorkspacePath(workspaceId),
      options: options || {},
    })
  }

  /**
   * Backup workspace
   */
  async backupWorkspace(
    workspaceId: string,
    options?: unknown
  ): Promise<ApiResponse<WorkspaceBackup>> {
    return apiClient.invoke('backup_workspace', {
      workspace_path: getWorkspacePath(workspaceId),
      options: options || {},
    })
  }

  /**
   * Restore workspace from backup
   */
  async restoreWorkspace(backupId: string): Promise<ApiResponse<WorkspaceConfig>> {
    return apiClient.invoke('restore_workspace', { backup_id: backupId })
  }

  /**
   * Get workspace backup history
   */
  async getBackupHistory(workspaceId: string): Promise<ApiResponse<WorkspaceBackup[]>> {
    return apiClient.invoke('get_workspace_backup_history', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Scan workspace for issues
   */
  async scanWorkspace(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('scan_workspace', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Get workspace performance metrics
   */
  async getPerformanceMetrics(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_workspace_performance', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Set workspace AI configuration
   */
  async configureWorkspaceAI(workspaceId: string, config: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('configure_workspace_ai', {
      workspace_path: getWorkspacePath(workspaceId),
      config,
    })
  }

  /**
   * Get workspace AI status
   */
  async getWorkspaceAIStatus(workspaceId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_workspace_ai_status', { workspace_path: getWorkspacePath(workspaceId) })
  }

  /**
   * Generate workspace report
   */
  async generateWorkspaceReport(
    workspaceId: string,
    reportType: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_workspace_report', {
      workspace_path: getWorkspacePath(workspaceId),
      report_type: reportType,
    })
  }

  /**
   * Get workspace activity feed
   */
  async getActivityFeed(workspaceId: string, options?: unknown): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_workspace_activity', {
      workspace_path: getWorkspacePath(workspaceId),
      options: options || {},
    })
  }
}

export const workspaceService = new WorkspaceService()
