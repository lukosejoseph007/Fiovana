// Centralized Tauri invoke wrapper
import { ApiResponse } from '../types'

interface CacheEntry {
  data: unknown
  timestamp: number
  ttl: number
}

// Mock AI settings storage for development with localStorage persistence
const SETTINGS_STORAGE_KEY = 'fiovana_ai_settings'

const loadMockSettings = (): Record<string, unknown> => {
  try {
    const stored = localStorage.getItem(SETTINGS_STORAGE_KEY)
    if (stored) {
      console.log('[MOCK] Loading AI settings from localStorage')
      return JSON.parse(stored)
    }
  } catch (error) {
    console.warn('[MOCK] Failed to load settings from localStorage:', error)
  }

  // Return defaults if nothing stored
  return {
    provider: 'local',
    openrouterApiKey: '',
    anthropicApiKey: '',
    selectedModel: 'llama3.2-3b',
    preferLocalModels: true,
    recentModels: [],
  }
}

const saveMockSettings = (settings: Record<string, unknown>) => {
  try {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings))
    console.log('[MOCK] Saved AI settings to localStorage')
  } catch (error) {
    console.warn('[MOCK] Failed to save settings to localStorage:', error)
  }
}

let mockAISettings: Record<string, unknown> = loadMockSettings()

// Mock implementation for development
const mockTauri = {
  invoke: async (command: string, args: unknown = {}) => {
    console.log(`[MOCK] Tauri command: ${command}`, args)

    // Return mock data for relationship graph commands
    if (command === 'build_relationship_graph') {
      return {
        id: 'mock-graph-1',
        name: 'Mock Knowledge Graph',
        nodes: [
          {
            id: 'node1',
            documentId: 'doc1',
            label: 'Document 1',
            type: 'document',
            properties: { importance: 2 },
          },
          {
            id: 'node2',
            documentId: 'doc2',
            label: 'Document 2',
            type: 'concept',
            properties: { importance: 1 },
          },
          {
            id: 'node3',
            documentId: 'doc3',
            label: 'Document 3',
            type: 'procedure',
            properties: { importance: 3 },
          },
          {
            id: 'node4',
            documentId: 'doc4',
            label: 'Reference Doc',
            type: 'reference',
            properties: { importance: 1 },
          },
        ],
        edges: [
          {
            id: 'edge1',
            sourceId: 'node1',
            targetId: 'node2',
            relationshipId: 'rel1',
            weight: 0.8,
            properties: {},
          },
          {
            id: 'edge2',
            sourceId: 'node2',
            targetId: 'node3',
            relationshipId: 'rel2',
            weight: 0.6,
            properties: {},
          },
          {
            id: 'edge3',
            sourceId: 'node1',
            targetId: 'node3',
            relationshipId: 'rel3',
            weight: 0.9,
            properties: {},
          },
        ],
        metadata: {
          nodeCount: 4,
          edgeCount: 3,
          density: 0.5,
          avgDegree: 1.5,
          clustering: 0.3,
          algorithms: ['force-directed'],
          lastAnalysis: new Date(),
        },
        createdAt: new Date(),
        updatedAt: new Date(),
      }
    }

    if (command === 'identify_document_clusters') {
      return [
        {
          id: 'cluster1',
          documents: ['doc1', 'doc2'],
          centerDocument: 'doc1',
          coherence: 0.85,
          size: 2,
          topics: [
            {
              name: 'Topic A',
              keywords: ['keyword1', 'keyword2'],
              confidence: 0.9,
              prevalence: 0.7,
            },
          ],
          relationships: ['rel1'],
        },
        {
          id: 'cluster2',
          documents: ['doc3'],
          coherence: 0.75,
          size: 1,
          topics: [{ name: 'Topic B', keywords: ['keyword3'], confidence: 0.8, prevalence: 0.6 }],
          relationships: ['rel2'],
        },
      ]
    }

    // Workspace analysis command
    if (command === 'analyze_workspace') {
      return {
        workspaceId: 'default',
        health: {
          score: 75,
          status: 'good',
          recommendations: [
            'Add more API documentation',
            'Improve code coverage',
            'Update outdated dependencies',
          ],
        },
        insights: ['gap1', 'gap2', 'gap3'],
        documents: {
          total: 25,
          active: 18,
          archived: 7,
        },
        lastAnalyzed: new Date().toISOString(),
      }
    }

    // List documents command
    if (command === 'list_documents') {
      return [
        {
          id: 'doc1',
          title: 'API Documentation',
          type: 'markdown',
          metadata: { status: 'active', recentlyEdited: true, updatedAt: new Date().toISOString() },
        },
        {
          id: 'doc2',
          title: 'User Guide',
          type: 'pdf',
          metadata: {
            status: 'active',
            recentlyEdited: false,
            updatedAt: new Date().toISOString(),
          },
        },
        {
          id: 'doc3',
          title: 'Technical Specification',
          type: 'document',
          metadata: { status: 'active', recentlyEdited: true, updatedAt: new Date().toISOString() },
        },
      ]
    }

    // Conversation analytics command
    if (command === 'get_conversation_analytics') {
      return {
        totalConversations: 12,
        recentActivity: true,
      }
    }

    // Smart collections command
    if (command === 'list_smart_collections' || command === 'get_smart_collections') {
      return [
        {
          id: 'collection1',
          name: 'Recent Updates',
          documentCount: 5,
        },
        {
          id: 'collection2',
          name: 'High Priority',
          documentCount: 3,
        },
      ]
    }

    // Organization suggestions command
    if (command === 'generate_organization_suggestions') {
      return [
        {
          id: 'org-sugg-1',
          type: 'restructure',
          description:
            'Group related documentation: API and User guides could be organized together',
          rationale: 'These documents share similar topics and would benefit from being co-located',
          confidence: 0.85,
          impact: 'medium',
          effort: 'low',
          target: ['doc1', 'doc2'],
        },
        {
          id: 'org-sugg-2',
          type: 'tagging',
          description: 'Add tags for better discoverability: Consider tagging by feature area',
          rationale: 'Consistent tagging would improve search and organization',
          confidence: 0.75,
          impact: 'high',
          effort: 'medium',
          target: ['doc1', 'doc2', 'doc3'],
        },
        {
          id: 'org-sugg-3',
          type: 'restructure',
          description: 'Archive inactive content: Move older documents to archive',
          rationale: 'Documents not accessed in 90+ days could be archived',
          confidence: 0.9,
          impact: 'low',
          effort: 'low',
          target: ['doc4', 'doc5'],
        },
      ]
    }

    // Smart organization command
    if (command === 'get_smart_organization') {
      return {
        workspaceId: 'default',
        categories: [
          {
            id: 'cat-1',
            name: 'Documentation',
            documentCount: 15,
            tags: ['docs', 'reference'],
          },
          {
            id: 'cat-2',
            name: 'Technical',
            documentCount: 8,
            tags: ['technical', 'specs'],
          },
        ],
        suggestions: [
          {
            id: 'sugg-1',
            type: 'categorization',
            description: 'Recent documents needing review',
            confidence: 0.8,
            impact: 'high',
            effort: 'medium',
            target: ['doc1', 'doc2', 'doc3'],
          },
          {
            id: 'sugg-2',
            type: 'categorization',
            description: 'Related concepts that should be linked',
            confidence: 0.75,
            impact: 'medium',
            effort: 'low',
            target: ['doc4', 'doc5'],
          },
        ],
        lastUpdated: new Date().toISOString(),
      }
    }

    // AI Settings commands
    if (command === 'get_ai_settings') {
      // Reload from localStorage to get latest settings
      mockAISettings = loadMockSettings()
      console.log('[MOCK] Loading AI settings:', mockAISettings)
      return { ...mockAISettings }
    }

    if (command === 'save_ai_settings') {
      const argsObj = args as Record<string, unknown>
      if (argsObj.settings && typeof argsObj.settings === 'object') {
        mockAISettings = { ...mockAISettings, ...(argsObj.settings as Record<string, unknown>) }
        saveMockSettings(mockAISettings) // Persist to localStorage
      }
      console.log('[MOCK] Saved AI settings:', mockAISettings)
      return true
    }

    if (command === 'init_ai_system') {
      const argsObj = args as Record<string, unknown>
      if (argsObj.config && typeof argsObj.config === 'object') {
        mockAISettings = { ...mockAISettings, ...(argsObj.config as Record<string, unknown>) }
        saveMockSettings(mockAISettings) // Persist to localStorage
      }
      console.log('[MOCK] AI system initialized with config')
      return true
    }

    if (command === 'test_ollama_connection') {
      console.log('[MOCK] Testing Ollama connection')
      return { available: mockAISettings.provider === 'local', message: 'Mock connection test' }
    }

    // AI Chat command
    if (command === 'ai_chat') {
      const argsObj = args as Record<string, unknown>
      const messages = (argsObj.messages || []) as Array<{ role: string; content: string }>
      const lastUserMessage = messages.filter(m => m.role === 'user').pop()
      const userContent = lastUserMessage?.content || 'Hello'

      console.log('[MOCK] AI Chat - responding to:', userContent)

      return {
        message: {
          role: 'assistant',
          content: `This is a mock AI response to: "${userContent}". In production, this would be a real AI response from ${mockAISettings.selectedModel || 'the configured AI model'}.`,
          metadata: {},
        },
        usage: {
          promptTokens: 50,
          completionTokens: 100,
          totalTokens: 150,
          cost: 0.001,
        },
        model: mockAISettings.selectedModel || 'mock-model',
        finishReason: 'stop',
        metadata: {
          provider: mockAISettings.provider || 'local',
        },
      }
    }

    // Get workspace health
    if (command === 'get_workspace_health') {
      return {
        score: 85,
        status: 'good',
        issues: [],
        recommendations: ['Keep up the good work!'],
      }
    }

    // Get conversation suggestions
    if (command === 'get_conversation_suggestions') {
      console.log('[MOCK] Getting conversation suggestions')
      return [
        'Can you elaborate on that?',
        'What would you like to know more about?',
        'How can I help you with this?',
      ]
    }

    // Analyze workspace comprehensive
    if (command === 'analyze_workspace_comprehensive') {
      return {
        summary: 'Mock workspace analysis',
        totalDocuments: 10,
        totalSize: 1024000,
        documentTypes: { markdown: 5, pdf: 3, text: 2 },
        tags: ['work', 'personal', 'project'],
      }
    }

    // AI recommend workspace content
    if (command === 'ai_recommend_workspace_content') {
      return {
        recommendations: [
          {
            id: 'rec-1',
            type: 'document',
            title: 'Recommended Document',
            reason: 'Based on your recent activity',
            confidence: 0.85,
          },
        ],
      }
    }

    // Analyze productivity patterns
    if (command === 'analyze_productivity_patterns') {
      return {
        patterns: [
          {
            type: 'peak_hours',
            description: 'Most productive between 9 AM - 12 PM',
            confidence: 0.8,
          },
        ],
        insights: ['You work best in the morning'],
      }
    }

    // Default fallback - return empty object instead of null
    console.warn(`[MOCK] No mock data for command: ${command}`)
    return {}
  },
}

// Safely import Tauri invoke with initialization promise
let tauriInvoke: (command: string, args?: unknown) => Promise<unknown>

async function initTauriInvoke() {
  try {
    // Check if we're in a Tauri environment at runtime (Tauri v2 uses __TAURI_INTERNALS__)
    if (
      typeof window !== 'undefined' &&
      (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
    ) {
      console.log('[API] Running in Tauri environment, using real backend')
      // Dynamic import with proper Vite handling
      const { invoke } = await import('@tauri-apps/api/core')
      tauriInvoke = invoke as (command: string, args?: unknown) => Promise<unknown>
      console.log('[API] Real Tauri backend initialized successfully')
    } else {
      console.warn('[API] Not in Tauri environment, using mock backend')
      tauriInvoke = mockTauri.invoke
    }
  } catch (error) {
    console.error('[API] Failed to initialize Tauri, using mock backend:', error)
    // Fallback to mock if Tauri is not available
    tauriInvoke = mockTauri.invoke
  }
}

// Initialize immediately with mock, then switch to real Tauri if available
tauriInvoke = mockTauri.invoke
const initPromise = initTauriInvoke().catch(error => {
  console.error('[API] initTauriInvoke failed:', error)
  tauriInvoke = mockTauri.invoke
})

// Helper to ensure initialization is complete before invoking
async function ensureInitialized() {
  await initPromise
}

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
      // Ensure Tauri is initialized before invoking
      await ensureInitialized()

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

      // Extract detailed error information
      let errorMessage = 'Unknown error occurred'
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      } else if (error && typeof error === 'object' && 'message' in error) {
        errorMessage = String((error as { message: unknown }).message)
      }

      // Log detailed error information
      console.error(`[API] Command ${command} failed:`, errorMessage)
      console.error('[API] Error details:', {
        error: errorMessage,
        command,
        args,
        executionTime: endTime - startTime,
        rawError: error,
      })

      // Try to extract more info from the error
      if (error && typeof error === 'object') {
        console.error('[API] Raw error object:', JSON.stringify(error, null, 2))
      }

      return this.createErrorResponse(errorMessage, {
        command,
        args,
        executionTime: endTime - startTime,
        originalError: error,
      })
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
