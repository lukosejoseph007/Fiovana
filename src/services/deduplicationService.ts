import { invoke } from '@tauri-apps/api/core'
import {
  DeduplicationResult,
  StorageStats,
  GarbageCollectionResult,
  DuplicateGroup,
} from '../types/deduplication'

export class DeduplicationService {
  /**
   * Initialize deduplication system for a workspace
   */
  static async initializeDeduplication(workspacePath: string): Promise<void> {
    return invoke('initialize_deduplication', { workspacePath })
  }

  /**
   * Deduplicate a single file
   */
  static async deduplicateFile(
    sourcePath: string,
    workspacePath: string
  ): Promise<DeduplicationResult> {
    return invoke('deduplicate_file', { sourcePath, workspacePath })
  }

  /**
   * Batch deduplicate multiple files
   */
  static async batchDeduplicateFiles(
    sourcePaths: string[],
    workspacePath: string
  ): Promise<DeduplicationResult[]> {
    return invoke('batch_deduplicate_files', { sourcePaths, workspacePath })
  }

  /**
   * Check if a file would be deduplicated without performing the operation
   */
  static async checkFileDeduplication(sourcePath: string, workspacePath: string): Promise<boolean> {
    return invoke('check_file_deduplication', { sourcePath, workspacePath })
  }

  /**
   * Get storage statistics for deduplication
   */
  static async getDeduplicationStats(workspacePath: string): Promise<StorageStats> {
    return invoke('get_deduplication_stats', { workspacePath })
  }

  /**
   * Get storage statistics for all workspaces
   */
  static async getAllDeduplicationStats(): Promise<Record<string, StorageStats>> {
    return invoke('get_all_deduplication_stats')
  }

  /**
   * Run garbage collection for unreferenced files
   */
  static async runGarbageCollection(workspacePath: string): Promise<GarbageCollectionResult> {
    return invoke('run_garbage_collection', { workspacePath })
  }

  /**
   * Check if garbage collection should run
   */
  static async shouldRunGarbageCollection(workspacePath: string): Promise<boolean> {
    return invoke('should_run_garbage_collection', { workspacePath })
  }

  /**
   * Clean up deduplication manager for a workspace
   */
  static async cleanupDeduplication(workspacePath: string): Promise<void> {
    return invoke('cleanup_deduplication', { workspacePath })
  }

  /**
   * Helper method to format file size
   */
  static formatFileSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let size = bytes
    let unitIndex = 0

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024
      unitIndex++
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`
  }

  /**
   * Helper method to calculate potential savings from duplicates
   */
  static calculatePotentialSavings(duplicateGroups: DuplicateGroup[]): number {
    return duplicateGroups.reduce((total, group) => {
      // Savings = (number of duplicates - 1) * file size
      if (group.files.length > 0) {
        return total + (group.files.length - 1) * group.files[0]!.size
      }
      return total
    }, 0)
  }
}
