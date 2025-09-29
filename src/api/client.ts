// Centralized Tauri invoke wrapper
import { ApiResponse } from '../types'

interface CacheEntry {
  data: unknown
  timestamp: number
  ttl: number
}

// Mock implementation for development
const mockTauri = {
  invoke: async (command: string, args: unknown = {}) => {
    console.log(`[MOCK] Tauri command: ${command}`, args)
    return { success: true, data: null }
  },
}

// Safely import Tauri invoke
let tauriInvoke: (command: string, args?: unknown) => Promise<unknown>

async function initTauriInvoke() {
  try {
    // Check if we're in a Tauri environment at runtime
    if (typeof window !== 'undefined' && (window as unknown as { __TAURI__?: unknown }).__TAURI__) {
      // Dynamic import with proper Vite handling
      const { invoke } = await import('@tauri-apps/api/core')
      tauriInvoke = invoke as (command: string, args?: unknown) => Promise<unknown>
    } else {
      tauriInvoke = mockTauri.invoke
    }
  } catch {
    // Fallback to mock if Tauri is not available
    tauriInvoke = mockTauri.invoke
  }
}

// Initialize immediately
tauriInvoke = mockTauri.invoke
initTauriInvoke().catch(() => {
  tauriInvoke = mockTauri.invoke
})

export class TauriApiClient {
  private static instance: TauriApiClient
  private commandCache = new Map<string, CacheEntry>()
  private performanceMetrics = new Map<string, PerformanceMetric>()

  private constructor() {
    this.setupPerformanceMonitoring()
  }

  static getInstance(): TauriApiClient {
    if (!TauriApiClient.instance) {
      TauriApiClient.instance = new TauriApiClient()
    }
    return TauriApiClient.instance
  }

  /**
   * Universal command invoker with automatic error handling and type safety
   */
  async invoke<T = unknown>(
    command: string,
    args: Record<string, unknown> = {},
    options: InvokeOptions = {}
  ): Promise<ApiResponse<T>> {
    const startTime = performance.now()

    try {
      // Validate command exists
      if (!this.isValidCommand(command)) {
        throw new Error(`Unknown command: ${command}`)
      }

      // Apply caching if enabled
      if (options.cache) {
        const cacheKey = this.generateCacheKey(command, args)
        const cached = this.commandCache.get(cacheKey)
        if (cached && !this.isCacheExpired(cached)) {
          return this.createSuccessResponse(cached.data as T, { fromCache: true })
        }
      }

      // Execute command
      const result = (await tauriInvoke(command, args)) as T
      const endTime = performance.now()

      // Record performance metrics
      this.recordPerformance(command, endTime - startTime, true)

      // Cache result if enabled
      if (options.cache) {
        const cacheKey = this.generateCacheKey(command, args)
        this.commandCache.set(cacheKey, {
          data: result,
          timestamp: Date.now(),
          ttl: options.cacheTtl || 300000, // 5 minutes default
        })
      }

      return this.createSuccessResponse(result, {
        executionTime: endTime - startTime,
        command,
      })
    } catch (error) {
      const endTime = performance.now()
      this.recordPerformance(command, endTime - startTime, false)

      return this.createErrorResponse(
        error instanceof Error ? error.message : 'Unknown error occurred',
        {
          command,
          args,
          executionTime: endTime - startTime,
        }
      )
    }
  }

  /**
   * Batch command execution with parallel processing
   */
  async invokeBatch<T = unknown>(
    commands: BatchCommand[],
    options: BatchOptions = {}
  ): Promise<BatchResponse<T>> {
    const startTime = performance.now()
    const concurrency = options.concurrency || 5
    const results: BatchResult<T>[] = []

    try {
      // Process commands in batches based on concurrency limit
      for (let i = 0; i < commands.length; i += concurrency) {
        const batch = commands.slice(i, i + concurrency)
        const batchPromises = batch.map(async (cmd, index) => {
          try {
            const result = await this.invoke<T>(cmd.command, cmd.args, cmd.options)
            return {
              index: i + index,
              success: result.success,
              data: result.data,
              error: result.error,
            }
          } catch (error) {
            return {
              index: i + index,
              success: false,
              data: undefined,
              error: error instanceof Error ? error.message : 'Unknown error',
            }
          }
        })

        const batchResults = await Promise.all(batchPromises)
        results.push(...batchResults)
      }

      const endTime = performance.now()
      const successful = results.filter(r => r.success).length
      const failed = results.length - successful

      return {
        success: failed === 0,
        results,
        summary: {
          total: commands.length,
          successful,
          failed,
          executionTime: endTime - startTime,
        },
      }
    } catch (error) {
      const endTime = performance.now()
      return {
        success: false,
        results,
        error: error instanceof Error ? error.message : 'Batch execution failed',
        summary: {
          total: commands.length,
          successful: 0,
          failed: commands.length,
          executionTime: endTime - startTime,
        },
      }
    }
  }

  /**
   * Stream-based command execution for long-running operations
   */
  async invokeStream<T = unknown>(
    command: string,
    args: Record<string, unknown> = {},
    _onProgress?: (progress: StreamProgress<T>) => void
  ): Promise<AsyncGenerator<StreamChunk<T>, void, unknown>> {
    // Implementation for streaming commands
    // This would typically use Tauri's event system for real-time updates
    async function* streamGenerator() {
      try {
        // For now, we'll simulate streaming by polling
        // In a real implementation, this would use Tauri events
        const result = (await tauriInvoke(command, args)) as T
        yield {
          type: 'data' as const,
          data: result,
          progress: 100,
          timestamp: Date.now(),
        }
      } catch (error) {
        yield {
          type: 'error' as const,
          error: error instanceof Error ? error.message : 'Stream error',
          timestamp: Date.now(),
        }
      }
    }

    return streamGenerator()
  }

  /**
   * Get available commands from the backend
   */
  async getAvailableCommands(): Promise<string[]> {
    try {
      // This would query the backend for available commands
      // For now, return a placeholder
      return []
    } catch (error) {
      console.error('Failed to get available commands:', error)
      return []
    }
  }

  /**
   * Get performance metrics for commands
   */
  getPerformanceMetrics(): Map<string, PerformanceMetric> {
    return new Map(this.performanceMetrics)
  }

  /**
   * Clear command cache
   */
  clearCache(): void {
    this.commandCache.clear()
  }

  /**
   * Validate if a command exists
   */
  private isValidCommand(_command: string): boolean {
    // In a real implementation, this would check against the command registry
    // For now, accept all commands
    return true
  }

  private generateCacheKey(command: string, args: Record<string, unknown>): string {
    return `${command}:${JSON.stringify(args)}`
  }

  private isCacheExpired(cacheEntry: CacheEntry): boolean {
    return Date.now() - cacheEntry.timestamp > cacheEntry.ttl
  }

  private createSuccessResponse<T>(
    data: T,
    metadata: Record<string, unknown> = {}
  ): ApiResponse<T> {
    return {
      success: true,
      data,
      metadata,
    }
  }

  private createErrorResponse<T = never>(
    error: string,
    metadata: Record<string, unknown> = {}
  ): ApiResponse<T> {
    return {
      success: false,
      error,
      metadata,
    }
  }

  private recordPerformance(command: string, duration: number, success: boolean): void {
    const metric = this.performanceMetrics.get(command) || {
      command,
      totalCalls: 0,
      successfulCalls: 0,
      failedCalls: 0,
      averageDuration: 0,
      minDuration: Infinity,
      maxDuration: 0,
      lastCall: Date.now(),
    }

    metric.totalCalls++
    if (success) {
      metric.successfulCalls++
    } else {
      metric.failedCalls++
    }

    metric.averageDuration =
      (metric.averageDuration * (metric.totalCalls - 1) + duration) / metric.totalCalls
    metric.minDuration = Math.min(metric.minDuration, duration)
    metric.maxDuration = Math.max(metric.maxDuration, duration)
    metric.lastCall = Date.now()

    this.performanceMetrics.set(command, metric)
  }

  private setupPerformanceMonitoring(): void {
    // Set up periodic cleanup of old cache entries
    setInterval(() => {
      for (const [key, entry] of this.commandCache.entries()) {
        if (this.isCacheExpired(entry)) {
          this.commandCache.delete(key)
        }
      }
    }, 60000) // Clean up every minute
  }
}

// Types for the API client
export interface InvokeOptions {
  cache?: boolean
  cacheTtl?: number
  timeout?: number
  retries?: number
}

export interface BatchCommand {
  command: string
  args: Record<string, unknown>
  options?: InvokeOptions
}

export interface BatchOptions {
  concurrency?: number
  stopOnError?: boolean
}

export interface BatchResponse<T = unknown> {
  success: boolean
  results: BatchResult<T>[]
  error?: string
  summary: BatchSummary
}

export interface BatchResult<T = unknown> {
  index: number
  success: boolean
  data?: T
  error?: string
}

export interface BatchSummary {
  total: number
  successful: number
  failed: number
  executionTime: number
}

export interface StreamProgress<T = unknown> {
  progress: number
  data?: T
  message?: string
}

export interface StreamChunk<T = unknown> {
  type: 'data' | 'progress' | 'error' | 'complete'
  data?: T
  progress?: number
  error?: string
  timestamp: number
}

export interface PerformanceMetric {
  command: string
  totalCalls: number
  successfulCalls: number
  failedCalls: number
  averageDuration: number
  minDuration: number
  maxDuration: number
  lastCall: number
}

// Export singleton instance
export const apiClient = TauriApiClient.getInstance()
