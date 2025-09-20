// src/services/progressService.ts
// Service for progress tracking operations

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ImportProgress, ProgressService, ProgressSummary } from '../types/progress'

class ProgressServiceImpl implements ProgressService {
  private updateListeners: Set<(progress: ImportProgress) => void> = new Set()
  private unsubscribeFromTauri?: () => void

  async getAllOperations(): Promise<ImportProgress[]> {
    try {
      return await invoke<ImportProgress[]>('get_all_operations')
    } catch (error) {
      console.error('Failed to get all operations:', error)
      throw new Error(`Failed to get operations: ${error}`)
    }
  }

  async getOperationProgress(operationId: string): Promise<ImportProgress | null> {
    try {
      return await invoke<ImportProgress | null>('get_operation_progress', {
        operationId,
      })
    } catch (error) {
      console.error('Failed to get operation progress:', error)
      throw new Error(`Failed to get progress for operation ${operationId}: ${error}`)
    }
  }

  async cancelOperation(operationId: string): Promise<boolean> {
    try {
      return await invoke<boolean>('cancel_operation', { operationId })
    } catch (error) {
      console.error('Failed to cancel operation:', error)
      throw new Error(`Failed to cancel operation ${operationId}: ${error}`)
    }
  }

  async getProgressSummary(): Promise<ProgressSummary> {
    try {
      return await invoke<ProgressSummary>('get_progress_summary')
    } catch (error) {
      console.error('Failed to get progress summary:', error)
      throw new Error(`Failed to get progress summary: ${error}`)
    }
  }

  async cleanupCompletedOperations(): Promise<number> {
    try {
      return await invoke<number>('cleanup_completed_operations')
    } catch (error) {
      console.error('Failed to cleanup completed operations:', error)
      throw new Error(`Failed to cleanup operations: ${error}`)
    }
  }

  async subscribeToUpdates(): Promise<void> {
    try {
      // Subscribe to backend events
      await invoke('subscribe_progress_updates')

      // Listen for progress update events from Tauri
      this.unsubscribeFromTauri = await listen<ImportProgress>(
        'progress-update',
        (event: { payload: ImportProgress }) => {
          // Notify all listeners
          this.updateListeners.forEach(listener => {
            try {
              listener(event.payload)
            } catch (error) {
              console.error('Error in progress update listener:', error)
            }
          })
        }
      )
    } catch (error) {
      console.error('Failed to subscribe to progress updates:', error)
      throw new Error(`Failed to subscribe to updates: ${error}`)
    }
  }

  async getOperationHistory(limit?: number): Promise<ImportProgress[]> {
    try {
      return await invoke<ImportProgress[]>('get_operation_history', { limit })
    } catch (error) {
      console.error('Failed to get operation history:', error)
      throw new Error(`Failed to get operation history: ${error}`)
    }
  }

  async getEstimatedCompletionTime(): Promise<number | null> {
    try {
      return await invoke<number | null>('get_estimated_completion_time')
    } catch (error) {
      console.error('Failed to get estimated completion time:', error)
      throw new Error(`Failed to get completion time: ${error}`)
    }
  }

  async updateOperationProgress(
    operationId: string,
    progressPercentage: number,
    currentStep?: string,
    currentFile?: string
  ): Promise<void> {
    try {
      await invoke('update_operation_progress', {
        operationId,
        progressPercentage,
        currentStep,
        currentFile,
      })
    } catch (error) {
      console.error('Failed to update operation progress:', error)
      throw new Error(`Failed to update progress for operation ${operationId}: ${error}`)
    }
  }

  // Additional methods for managing listeners
  addUpdateListener(listener: (progress: ImportProgress) => void): () => void {
    this.updateListeners.add(listener)
    return () => {
      this.updateListeners.delete(listener)
    }
  }

  removeUpdateListener(listener: (progress: ImportProgress) => void): void {
    this.updateListeners.delete(listener)
  }

  unsubscribeFromUpdates(): void {
    if (this.unsubscribeFromTauri) {
      this.unsubscribeFromTauri()
      this.unsubscribeFromTauri = undefined
    }
    this.updateListeners.clear()
  }

  // Utility methods
  async isOperationActive(operationId: string): Promise<boolean> {
    const progress = await this.getOperationProgress(operationId)
    return progress?.status === 'Running' || progress?.status === 'Pending'
  }

  async waitForOperationCompletion(
    operationId: string,
    timeout: number = 30000
  ): Promise<ImportProgress> {
    return new Promise((resolve, reject) => {
      const timeoutHandle = setTimeout(() => {
        cleanup()
        reject(new Error(`Operation ${operationId} did not complete within ${timeout}ms`))
      }, timeout)

      const cleanup = this.addUpdateListener(progress => {
        if (progress.operation_id === operationId) {
          if (
            progress.status === 'Completed' ||
            progress.status === 'Failed' ||
            progress.status === 'Cancelled'
          ) {
            clearTimeout(timeoutHandle)
            cleanup()
            resolve(progress)
          }
        }
      })
    })
  }

  // Formatting utilities
  formatProgress(progress: ImportProgress): string {
    return `${progress.progress_percentage.toFixed(1)}% (${progress.files_processed}/${progress.total_files})`
  }

  formatETA(etaSeconds?: number): string {
    if (!etaSeconds) return 'Unknown'

    if (etaSeconds < 60) {
      return `${etaSeconds}s`
    } else if (etaSeconds < 3600) {
      const minutes = Math.floor(etaSeconds / 60)
      const seconds = etaSeconds % 60
      return `${minutes}m ${seconds}s`
    } else {
      const hours = Math.floor(etaSeconds / 3600)
      const minutes = Math.floor((etaSeconds % 3600) / 60)
      return `${hours}h ${minutes}m`
    }
  }

  formatDuration(startTime: string): string {
    const start = new Date(startTime)
    const now = new Date()
    const durationMs = now.getTime() - start.getTime()
    const durationSeconds = Math.floor(durationMs / 1000)

    return this.formatETA(durationSeconds)
  }

  getStatusColor(status: string): string {
    switch (status) {
      case 'Running':
        return 'blue'
      case 'Completed':
        return 'green'
      case 'Failed':
        return 'red'
      case 'Cancelled':
        return 'orange'
      case 'Pending':
        return 'yellow'
      case 'Paused':
        return 'gray'
      default:
        return 'gray'
    }
  }

  getStatusIcon(status: string): string {
    switch (status) {
      case 'Running':
        return '⚡'
      case 'Completed':
        return '✅'
      case 'Failed':
        return '❌'
      case 'Cancelled':
        return '🚫'
      case 'Pending':
        return '⏳'
      case 'Paused':
        return '⏸️'
      default:
        return '❓'
    }
  }
}

// Singleton instance
export const progressService = new ProgressServiceImpl()

// Export the class for testing
export { ProgressServiceImpl }
