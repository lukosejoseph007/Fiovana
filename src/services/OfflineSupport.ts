// Offline Support & Caching Service
import { apiClient } from '../api'
import { ApiResponse } from '../types'

// ==================== Types & Interfaces ====================

export interface CachedDocument {
  id: string
  title: string
  content: string
  metadata: Record<string, unknown>
  cachedAt: number
  lastAccessed: number
}

export interface CachedConversation {
  id: string
  sessionId: string
  messages: ConversationMessage[]
  cachedAt: number
  lastAccessed: number
}

export interface ConversationMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  metadata?: Record<string, unknown>
}

export interface QueuedOperation {
  id: string
  type: 'document_update' | 'conversation_sync' | 'ai_request' | 'document_generation'
  payload: Record<string, unknown>
  timestamp: number
  retryCount: number
  maxRetries: number
  status: 'pending' | 'processing' | 'failed' | 'completed'
}

export interface OfflineStatus {
  isOnline: boolean
  lastOnlineCheck: number
  queuedOperationsCount: number
  cachedDocumentsCount: number
  cachedConversationsCount: number
  storageUsed: number
  storageLimit: number
}

export interface CacheConfig {
  maxDocuments: number
  maxConversations: number
  maxCacheAge: number // milliseconds
  autoCleanup: boolean
  cleanupInterval: number // milliseconds
}

export interface OllamaStatus {
  isAvailable: boolean
  models: string[]
  currentModel?: string
  lastCheck: number
}

// ==================== Storage Keys ====================

const STORAGE_KEYS = {
  DOCUMENTS: 'offline_cached_documents',
  CONVERSATIONS: 'offline_cached_conversations',
  OPERATIONS_QUEUE: 'offline_operations_queue',
  CONFIG: 'offline_cache_config',
  OLLAMA_STATUS: 'offline_ollama_status',
} as const

// ==================== Default Configuration ====================

const DEFAULT_CONFIG: CacheConfig = {
  maxDocuments: 50,
  maxConversations: 20,
  maxCacheAge: 7 * 24 * 60 * 60 * 1000, // 7 days
  autoCleanup: true,
  cleanupInterval: 60 * 60 * 1000, // 1 hour
}

// ==================== Main Service Class ====================

export class OfflineSupport {
  private static instance: OfflineSupport
  private isOnline: boolean = navigator.onLine
  private cleanupIntervalId?: number
  private onlineCheckIntervalId?: number
  private onStatusChangeCallbacks: Set<(status: OfflineStatus) => void> = new Set()

  private constructor() {
    this.initializeEventListeners()
    this.initializeCleanupScheduler()
    this.initializeOnlineChecker()

    // Perform initial online check after a brief delay
    setTimeout(() => this.checkOnlineStatus(), 100)
  }

  /**
   * Get singleton instance
   */
  static getInstance(): OfflineSupport {
    if (!OfflineSupport.instance) {
      OfflineSupport.instance = new OfflineSupport()
    }
    return OfflineSupport.instance
  }

  // ==================== Initialization ====================

  private initializeEventListeners(): void {
    window.addEventListener('online', () => this.handleOnlineStatusChange(true))
    window.addEventListener('offline', () => this.handleOnlineStatusChange(false))
  }

  private initializeCleanupScheduler(): void {
    const config = this.getConfig()
    if (config.autoCleanup) {
      this.cleanupIntervalId = window.setInterval(
        () => this.performCleanup(),
        config.cleanupInterval
      )
    }
  }

  private initializeOnlineChecker(): void {
    // Check online status every 30 seconds
    this.onlineCheckIntervalId = window.setInterval(() => this.checkOnlineStatus(), 30000)
  }

  /**
   * Cleanup intervals on destroy
   */
  destroy(): void {
    if (this.cleanupIntervalId) {
      clearInterval(this.cleanupIntervalId)
    }
    if (this.onlineCheckIntervalId) {
      clearInterval(this.onlineCheckIntervalId)
    }
    window.removeEventListener('online', () => this.handleOnlineStatusChange(true))
    window.removeEventListener('offline', () => this.handleOnlineStatusChange(false))
  }

  // ==================== Online/Offline Detection ====================

  private async checkOnlineStatus(): Promise<void> {
    // First check browser's online status
    if (!navigator.onLine) {
      this.handleOnlineStatusChange(false)
      return
    }

    // If browser says we're online, verify backend is reachable
    try {
      // Use a simple greet command as a lightweight health check with timeout
      await Promise.race([
        apiClient.invoke('greet', { name: 'health-check' }),
        new Promise<never>((_, reject) => setTimeout(() => reject(new Error('Timeout')), 3000)),
      ])

      // If we got a response, we're online (even if it's an error response)
      this.handleOnlineStatusChange(true)
    } catch {
      // Backend unreachable, but browser says online - could be backend down
      // Let's be optimistic and trust the browser's navigator.onLine
      console.warn('Backend health check failed, using navigator.onLine status')
      this.handleOnlineStatusChange(navigator.onLine)
    }
  }

  private handleOnlineStatusChange(isOnline: boolean): void {
    const wasOnline = this.isOnline
    this.isOnline = isOnline

    if (!wasOnline && isOnline) {
      // Just came online - process queued operations
      this.processQueuedOperations()
    }

    // Notify listeners
    this.notifyStatusChange()
  }

  /**
   * Get current online status
   */
  getOnlineStatus(): boolean {
    return this.isOnline
  }

  /**
   * Subscribe to status changes
   */
  onStatusChange(callback: (status: OfflineStatus) => void): () => void {
    this.onStatusChangeCallbacks.add(callback)
    return () => this.onStatusChangeCallbacks.delete(callback)
  }

  private notifyStatusChange(): void {
    const status = this.getStatus()
    this.onStatusChangeCallbacks.forEach(callback => callback(status))
  }

  // ==================== Configuration Management ====================

  /**
   * Get cache configuration
   */
  getConfig(): CacheConfig {
    const stored = localStorage.getItem(STORAGE_KEYS.CONFIG)
    if (stored) {
      try {
        return { ...DEFAULT_CONFIG, ...JSON.parse(stored) }
      } catch {
        return DEFAULT_CONFIG
      }
    }
    return DEFAULT_CONFIG
  }

  /**
   * Update cache configuration
   */
  updateConfig(config: Partial<CacheConfig>): void {
    const current = this.getConfig()
    const updated = { ...current, ...config }
    localStorage.setItem(STORAGE_KEYS.CONFIG, JSON.stringify(updated))

    // Restart cleanup scheduler if interval changed
    if (config.cleanupInterval || config.autoCleanup !== undefined) {
      if (this.cleanupIntervalId) {
        clearInterval(this.cleanupIntervalId)
      }
      this.initializeCleanupScheduler()
    }
  }

  // ==================== Document Caching ====================

  /**
   * Cache a document for offline access
   */
  cacheDocument(document: Omit<CachedDocument, 'cachedAt' | 'lastAccessed'>): void {
    const documents = this.getCachedDocuments()
    const now = Date.now()

    const cached: CachedDocument = {
      ...document,
      cachedAt: now,
      lastAccessed: now,
    }

    // Remove existing entry if present
    const filtered = documents.filter(d => d.id !== document.id)

    // Add new entry
    filtered.push(cached)

    // Enforce max documents limit
    const config = this.getConfig()
    const sorted = filtered.sort((a, b) => b.lastAccessed - a.lastAccessed)
    const limited = sorted.slice(0, config.maxDocuments)

    localStorage.setItem(STORAGE_KEYS.DOCUMENTS, JSON.stringify(limited))
    this.notifyStatusChange()
  }

  /**
   * Get all cached documents
   */
  getCachedDocuments(): CachedDocument[] {
    const stored = localStorage.getItem(STORAGE_KEYS.DOCUMENTS)
    if (stored) {
      try {
        return JSON.parse(stored)
      } catch {
        return []
      }
    }
    return []
  }

  /**
   * Get a specific cached document
   */
  getCachedDocument(id: string): CachedDocument | null {
    const documents = this.getCachedDocuments()
    const document = documents.find(d => d.id === id)

    if (document) {
      // Update last accessed time
      document.lastAccessed = Date.now()
      localStorage.setItem(STORAGE_KEYS.DOCUMENTS, JSON.stringify(documents))
      return document
    }

    return null
  }

  /**
   * Remove a document from cache
   */
  removeCachedDocument(id: string): void {
    const documents = this.getCachedDocuments()
    const filtered = documents.filter(d => d.id !== id)
    localStorage.setItem(STORAGE_KEYS.DOCUMENTS, JSON.stringify(filtered))
    this.notifyStatusChange()
  }

  /**
   * Clear all cached documents
   */
  clearCachedDocuments(): void {
    localStorage.removeItem(STORAGE_KEYS.DOCUMENTS)
    this.notifyStatusChange()
  }

  // ==================== Conversation Caching ====================

  /**
   * Cache a conversation for offline access
   */
  cacheConversation(conversation: Omit<CachedConversation, 'cachedAt' | 'lastAccessed'>): void {
    const conversations = this.getCachedConversations()
    const now = Date.now()

    const cached: CachedConversation = {
      ...conversation,
      cachedAt: now,
      lastAccessed: now,
    }

    // Remove existing entry if present
    const filtered = conversations.filter(c => c.id !== conversation.id)

    // Add new entry
    filtered.push(cached)

    // Enforce max conversations limit
    const config = this.getConfig()
    const sorted = filtered.sort((a, b) => b.lastAccessed - a.lastAccessed)
    const limited = sorted.slice(0, config.maxConversations)

    localStorage.setItem(STORAGE_KEYS.CONVERSATIONS, JSON.stringify(limited))
    this.notifyStatusChange()
  }

  /**
   * Get all cached conversations
   */
  getCachedConversations(): CachedConversation[] {
    const stored = localStorage.getItem(STORAGE_KEYS.CONVERSATIONS)
    if (stored) {
      try {
        return JSON.parse(stored)
      } catch {
        return []
      }
    }
    return []
  }

  /**
   * Get a specific cached conversation
   */
  getCachedConversation(id: string): CachedConversation | null {
    const conversations = this.getCachedConversations()
    const conversation = conversations.find(c => c.id === id)

    if (conversation) {
      // Update last accessed time
      conversation.lastAccessed = Date.now()
      localStorage.setItem(STORAGE_KEYS.CONVERSATIONS, JSON.stringify(conversations))
      return conversation
    }

    return null
  }

  /**
   * Remove a conversation from cache
   */
  removeCachedConversation(id: string): void {
    const conversations = this.getCachedConversations()
    const filtered = conversations.filter(c => c.id !== id)
    localStorage.setItem(STORAGE_KEYS.CONVERSATIONS, JSON.stringify(filtered))
    this.notifyStatusChange()
  }

  /**
   * Clear all cached conversations
   */
  clearCachedConversations(): void {
    localStorage.removeItem(STORAGE_KEYS.CONVERSATIONS)
    this.notifyStatusChange()
  }

  // ==================== Operations Queue ====================

  /**
   * Queue an operation for when connection is restored
   */
  queueOperation(
    operation: Omit<QueuedOperation, 'id' | 'timestamp' | 'retryCount' | 'status'>
  ): string {
    const operations = this.getQueuedOperations()
    const id = `op_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

    const queued: QueuedOperation = {
      id,
      ...operation,
      timestamp: Date.now(),
      retryCount: 0,
      status: 'pending',
    }

    operations.push(queued)
    localStorage.setItem(STORAGE_KEYS.OPERATIONS_QUEUE, JSON.stringify(operations))
    this.notifyStatusChange()

    return id
  }

  /**
   * Get all queued operations
   */
  getQueuedOperations(): QueuedOperation[] {
    const stored = localStorage.getItem(STORAGE_KEYS.OPERATIONS_QUEUE)
    if (stored) {
      try {
        return JSON.parse(stored)
      } catch {
        return []
      }
    }
    return []
  }

  /**
   * Process queued operations (when back online)
   */
  async processQueuedOperations(): Promise<void> {
    if (!this.isOnline) {
      return
    }

    const operations = this.getQueuedOperations()
    const pending = operations.filter(op => op.status === 'pending')

    for (const operation of pending) {
      try {
        operation.status = 'processing'
        this.updateQueuedOperation(operation)

        await this.executeOperation(operation)

        operation.status = 'completed'
        this.updateQueuedOperation(operation)
      } catch (error) {
        operation.retryCount++
        if (operation.retryCount >= operation.maxRetries) {
          operation.status = 'failed'
        } else {
          operation.status = 'pending'
        }
        this.updateQueuedOperation(operation)
        console.error(`Failed to process operation ${operation.id}:`, error)
      }
    }

    // Remove completed operations
    this.cleanupCompletedOperations()
  }

  private async executeOperation(operation: QueuedOperation): Promise<void> {
    switch (operation.type) {
      case 'document_update':
        await apiClient.invoke('update_document', operation.payload)
        break
      case 'conversation_sync':
        await apiClient.invoke('sync_conversation', operation.payload)
        break
      case 'ai_request':
        await apiClient.invoke('chat_with_ai', operation.payload)
        break
      case 'document_generation':
        await apiClient.invoke('generate_document', operation.payload)
        break
      default:
        throw new Error(`Unknown operation type: ${operation.type}`)
    }
  }

  private updateQueuedOperation(operation: QueuedOperation): void {
    const operations = this.getQueuedOperations()
    const index = operations.findIndex(op => op.id === operation.id)
    if (index !== -1) {
      operations[index] = operation
      localStorage.setItem(STORAGE_KEYS.OPERATIONS_QUEUE, JSON.stringify(operations))
      this.notifyStatusChange()
    }
  }

  private cleanupCompletedOperations(): void {
    const operations = this.getQueuedOperations()
    const active = operations.filter(op => op.status !== 'completed')
    localStorage.setItem(STORAGE_KEYS.OPERATIONS_QUEUE, JSON.stringify(active))
    this.notifyStatusChange()
  }

  /**
   * Remove a specific operation from queue
   */
  removeQueuedOperation(id: string): void {
    const operations = this.getQueuedOperations()
    const filtered = operations.filter(op => op.id !== id)
    localStorage.setItem(STORAGE_KEYS.OPERATIONS_QUEUE, JSON.stringify(filtered))
    this.notifyStatusChange()
  }

  /**
   * Clear all queued operations
   */
  clearQueuedOperations(): void {
    localStorage.removeItem(STORAGE_KEYS.OPERATIONS_QUEUE)
    this.notifyStatusChange()
  }

  // ==================== Ollama Integration ====================

  /**
   * Check Ollama availability for local AI operations
   */
  async checkOllamaStatus(): Promise<OllamaStatus> {
    try {
      const response: ApiResponse<{ available: boolean; models?: string[] }> =
        await apiClient.invoke('check_ollama_connection', {})

      if (response.success && response.data) {
        const status: OllamaStatus = {
          isAvailable: response.data.available,
          models: response.data.models || [],
          lastCheck: Date.now(),
        }
        localStorage.setItem(STORAGE_KEYS.OLLAMA_STATUS, JSON.stringify(status))
        return status
      }
    } catch (error) {
      console.error('Failed to check Ollama status:', error)
    }

    return {
      isAvailable: false,
      models: [],
      lastCheck: Date.now(),
    }
  }

  /**
   * Get cached Ollama status
   */
  getCachedOllamaStatus(): OllamaStatus | null {
    const stored = localStorage.getItem(STORAGE_KEYS.OLLAMA_STATUS)
    if (stored) {
      try {
        return JSON.parse(stored)
      } catch {
        return null
      }
    }
    return null
  }

  /**
   * Execute AI operation locally using Ollama (when offline)
   */
  async executeLocalAI(request: {
    message: string
    model?: string
    context?: Record<string, unknown>
  }): Promise<ApiResponse<unknown>> {
    const ollamaStatus = this.getCachedOllamaStatus()

    if (!ollamaStatus?.isAvailable) {
      return {
        success: false,
        error: 'Ollama is not available for local AI operations',
      }
    }

    try {
      return await apiClient.invoke('chat_with_ai', {
        request: {
          message: request.message,
          context: request.context,
        },
      })
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Failed to execute local AI operation',
      }
    }
  }

  // ==================== Cache Maintenance ====================

  /**
   * Perform cleanup of old cached data
   */
  performCleanup(): void {
    const config = this.getConfig()
    const now = Date.now()
    const maxAge = config.maxCacheAge

    // Cleanup old documents
    const documents = this.getCachedDocuments()
    const validDocuments = documents.filter(doc => now - doc.cachedAt < maxAge)
    localStorage.setItem(STORAGE_KEYS.DOCUMENTS, JSON.stringify(validDocuments))

    // Cleanup old conversations
    const conversations = this.getCachedConversations()
    const validConversations = conversations.filter(conv => now - conv.cachedAt < maxAge)
    localStorage.setItem(STORAGE_KEYS.CONVERSATIONS, JSON.stringify(validConversations))

    // Cleanup failed operations (older than 24 hours)
    const operations = this.getQueuedOperations()
    const validOperations = operations.filter(
      op => op.status !== 'failed' || now - op.timestamp < 24 * 60 * 60 * 1000
    )
    localStorage.setItem(STORAGE_KEYS.OPERATIONS_QUEUE, JSON.stringify(validOperations))

    this.notifyStatusChange()
  }

  /**
   * Get storage usage statistics
   */
  getStorageStats(): { used: number; total: number; percentage: number } {
    let used = 0

    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key) {
        const value = localStorage.getItem(key)
        if (value) {
          used += key.length + value.length
        }
      }
    }

    // Approximate localStorage limit (most browsers: 5-10 MB)
    const total = 5 * 1024 * 1024 // 5 MB
    const percentage = (used / total) * 100

    return { used, total, percentage }
  }

  /**
   * Get comprehensive offline status
   */
  getStatus(): OfflineStatus {
    const storage = this.getStorageStats()

    return {
      isOnline: this.isOnline,
      lastOnlineCheck: Date.now(),
      queuedOperationsCount: this.getQueuedOperations().length,
      cachedDocumentsCount: this.getCachedDocuments().length,
      cachedConversationsCount: this.getCachedConversations().length,
      storageUsed: storage.used,
      storageLimit: storage.total,
    }
  }

  /**
   * Clear all offline data
   */
  clearAllData(): void {
    this.clearCachedDocuments()
    this.clearCachedConversations()
    this.clearQueuedOperations()
    localStorage.removeItem(STORAGE_KEYS.OLLAMA_STATUS)
    this.notifyStatusChange()
  }
}

// Export singleton instance
export const offlineSupport = OfflineSupport.getInstance()
