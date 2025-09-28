// Workspace Backup Service
import { apiClient } from '../api'
import {
  Workspace,
  BackupInfo,
  ApiResponse
} from '../types'

export class WorkspaceBackupService {
  /**
   * Create workspace backup
   */
  async createBackup(
    workspaceId: string,
    backupOptions?: any
  ): Promise<ApiResponse<BackupInfo>> {
    return apiClient.invoke('create_workspace_backup', {
      workspace_id: workspaceId,
      options: backupOptions || {}
    })
  }

  /**
   * Restore workspace from backup
   */
  async restoreBackup(
    workspaceId: string,
    backupId: string,
    restoreOptions?: any
  ): Promise<ApiResponse<void>> {
    return apiClient.invoke('restore_workspace_backup', {
      workspace_id: workspaceId,
      backup_id: backupId,
      options: restoreOptions || {}
    })
  }

  /**
   * List available backups
   */
  async listBackups(workspaceId: string): Promise<ApiResponse<BackupInfo[]>> {
    return apiClient.invoke('list_workspace_backups', {
      workspace_id: workspaceId
    })
  }

  /**
   * Delete backup
   */
  async deleteBackup(backupId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('delete_workspace_backup', {
      backup_id: backupId
    })
  }

  /**
   * Get backup details
   */
  async getBackupDetails(backupId: string): Promise<ApiResponse<BackupInfo>> {
    return apiClient.invoke('get_backup_details', {
      backup_id: backupId
    })
  }

  /**
   * Schedule automatic backups
   */
  async scheduleBackup(
    workspaceId: string,
    schedule: string,
    backupOptions?: any
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('schedule_workspace_backup', {
      workspace_id: workspaceId,
      schedule,
      options: backupOptions || {}
    })
  }

  /**
   * Cancel scheduled backup
   */
  async cancelScheduledBackup(scheduleId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('cancel_scheduled_backup', {
      schedule_id: scheduleId
    })
  }

  /**
   * Verify backup integrity
   */
  async verifyBackup(backupId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('verify_backup_integrity', {
      backup_id: backupId
    })
  }

  /**
   * Create incremental backup
   */
  async createIncrementalBackup(
    workspaceId: string,
    lastBackupId?: string
  ): Promise<ApiResponse<BackupInfo>> {
    return apiClient.invoke('create_incremental_backup', {
      workspace_id: workspaceId,
      last_backup_id: lastBackupId
    })
  }

  /**
   * Export backup to external storage
   */
  async exportBackup(
    backupId: string,
    exportDestination: string,
    exportOptions?: any
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('export_backup', {
      backup_id: backupId,
      export_destination: exportDestination,
      options: exportOptions || {}
    })
  }

  /**
   * Import backup from external storage
   */
  async importBackup(
    importSource: string,
    workspaceId: string,
    importOptions?: any
  ): Promise<ApiResponse<BackupInfo>> {
    return apiClient.invoke('import_backup', {
      import_source: importSource,
      workspace_id: workspaceId,
      options: importOptions || {}
    })
  }

  /**
   * Compare backup versions
   */
  async compareBackups(
    backupId1: string,
    backupId2: string
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('compare_backup_versions', {
      backup_id_1: backupId1,
      backup_id_2: backupId2
    })
  }

  /**
   * Get backup storage usage
   */
  async getBackupStorageUsage(workspaceId?: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_backup_storage_usage', {
      workspace_id: workspaceId
    })
  }

  /**
   * Cleanup old backups
   */
  async cleanupOldBackups(
    workspaceId: string,
    retentionPolicy: any
  ): Promise<ApiResponse<any>> {
    return apiClient.invoke('cleanup_old_backups', {
      workspace_id: workspaceId,
      retention_policy: retentionPolicy
    })
  }

  /**
   * Set backup retention policy
   */
  async setRetentionPolicy(
    workspaceId: string,
    retentionPolicy: any
  ): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_backup_retention_policy', {
      workspace_id: workspaceId,
      retention_policy: retentionPolicy
    })
  }

  /**
   * Get backup status
   */
  async getBackupStatus(backupId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_backup_status', {
      backup_id: backupId
    })
  }

  /**
   * Pause/Resume backup operation
   */
  async pauseBackup(backupId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('pause_backup_operation', {
      backup_id: backupId
    })
  }

  /**
   * Resume backup operation
   */
  async resumeBackup(backupId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('resume_backup_operation', {
      backup_id: backupId
    })
  }

  /**
   * Get backup logs
   */
  async getBackupLogs(
    backupId: string,
    logLevel?: string
  ): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('get_backup_logs', {
      backup_id: backupId,
      log_level: logLevel || 'info'
    })
  }
}

export const workspaceBackupService = new WorkspaceBackupService()