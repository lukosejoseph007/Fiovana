// Service Layer Exports
export { workspaceService, WorkspaceService } from './workspaceService'
export { documentService, DocumentService } from './documentService'
export { aiService, AIService } from './aiService'
export { searchService, SearchService } from './searchService'

// Import service instances for use in health checks
import { workspaceService } from './workspaceService'
import { documentService } from './documentService'
import { aiService } from './aiService'
import { searchService } from './searchService'

// Service initialization and management
export class ServiceManager {
  private static instance: ServiceManager
  private initialized = false

  private constructor() {}

  static getInstance(): ServiceManager {
    if (!ServiceManager.instance) {
      ServiceManager.instance = new ServiceManager()
    }
    return ServiceManager.instance
  }

  /**
   * Initialize all services
   */
  async initialize(): Promise<void> {
    if (this.initialized) return

    try {
      // Initialize API system first
      const { initializeApiSystem } = await import('../api')
      await initializeApiSystem()

      // Services are ready to use (they use the initialized API client)
      this.initialized = true
      console.log('Service layer initialized successfully')
    } catch (error) {
      console.error('Failed to initialize service layer:', error)
      throw error
    }
  }

  /**
   * Check if services are initialized
   */
  isInitialized(): boolean {
    return this.initialized
  }

  /**
   * Get service health status
   */
  async getServiceHealth(): Promise<Record<string, boolean>> {
    const health: Record<string, boolean> = {}

    try {
      // Test each service with a simple operation
      const tests = [
        { name: 'workspace', test: () => workspaceService.listWorkspaces() },
        { name: 'document', test: () => documentService.listDocuments() },
        { name: 'ai', test: () => aiService.getModels() },
        { name: 'search', test: () => searchService.getIndexStatus() }
      ]

      for (const { name, test } of tests) {
        try {
          await test()
          health[name] = true
        } catch (error) {
          health[name] = false
          console.warn(`Service ${name} health check failed:`, error)
        }
      }
    } catch (error) {
      console.error('Error checking service health:', error)
    }

    return health
  }
}

// Export singleton instance
export const serviceManager = ServiceManager.getInstance()

// Convenience function to ensure services are initialized
export async function ensureServicesInitialized(): Promise<void> {
  if (!serviceManager.isInitialized()) {
    await serviceManager.initialize()
  }
}