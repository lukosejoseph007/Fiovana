// Service Layer Exports - Core Services
export { workspaceService, WorkspaceService } from './workspaceService'
export { documentService, DocumentService } from './documentService'
export { documentEditingService, DocumentEditingService } from './documentEditingService'
export { aiService, AIService } from './aiService'
export { searchService, SearchService } from './searchService'

// Advanced Services
export { styleAnalysisService, StyleAnalysisService } from './styleAnalysisService'
export { styleTransferService, StyleTransferService } from './styleTransferService'
export { knowledgeAnalyzerService, KnowledgeAnalyzerService } from './knowledgeAnalyzerService'
export { smartOrganizerService, SmartOrganizerService } from './smartOrganizerService'
export { contentLifecycleService, ContentLifecycleService } from './contentLifecycleService'
export {
  conversationIntelligenceService,
  ConversationIntelligenceService,
} from './conversationIntelligenceService'
export { embeddingService, EmbeddingService } from './embeddingService'
export { documentGenerationService, DocumentGenerationService } from './documentGenerationService'
export { formatConversionService, FormatConversionService } from './formatConversionService'
export { templateService, TemplateService } from './templateService'

// Additional Specialized Services
export { contentAdaptationService, ContentAdaptationService } from './contentAdaptationService'
export {
  contentClassificationService,
  ContentClassificationService,
} from './contentClassificationService'
export { relationshipService, RelationshipService } from './relationshipService'
export { nlOperationsService, NLOperationsService } from './nlOperationsService'
export { structureService, StructureService } from './structureService'
export { healthService, HealthService } from './healthService'
export { workspaceAnalyzerService, WorkspaceAnalyzerService } from './workspaceAnalyzerService'
export {
  workspacePerformanceService,
  WorkspacePerformanceService,
} from './workspacePerformanceService'
export { workspaceBackupService, WorkspaceBackupService } from './workspaceBackupService'
export { workspaceAiService, WorkspaceAiService } from './workspaceAiService'

// Advanced Integration Services
export { vectorOperationsService, VectorOperationsService } from './vectorOperationsService'
export { clusteringService, ClusteringService } from './clusteringService'
export { multiWorkspaceService, MultiWorkspaceService } from './multiWorkspaceService'
export { realTimeProcessingService, RealTimeProcessingService } from './realTimeProcessingService'

// Offline & Caching Services
export { offlineSupport, OfflineSupport } from './OfflineSupport'
export type {
  OfflineStatus,
  CachedDocument,
  CachedConversation,
  QueuedOperation,
  CacheConfig,
  OllamaStatus,
} from './OfflineSupport'

// Import service instances for use in health checks
import { workspaceService } from './workspaceService'
import { documentService } from './documentService'
import { aiService } from './aiService'
import { searchService } from './searchService'
import { styleAnalysisService } from './styleAnalysisService'
import { styleTransferService } from './styleTransferService'
import { knowledgeAnalyzerService } from './knowledgeAnalyzerService'
import { smartOrganizerService } from './smartOrganizerService'
import { contentLifecycleService } from './contentLifecycleService'
import { conversationIntelligenceService } from './conversationIntelligenceService'
import { embeddingService } from './embeddingService'
import { documentGenerationService } from './documentGenerationService'
import { formatConversionService } from './formatConversionService'
import { templateService } from './templateService'
// The following services are available but not exported as they're used internally
// import { contentAdaptationService } from './contentAdaptationService'
// import { contentClassificationService } from './contentClassificationService'
// import { relationshipService } from './relationshipService'
// import { nlOperationsService } from './nlOperationsService'
// import { structureService } from './structureService'
// import { healthService } from './healthService'
// import { workspaceAnalyzerService } from './workspaceAnalyzerService'
// import { workspacePerformanceService } from './workspacePerformanceService'
// import { workspaceBackupService } from './workspaceBackupService'
// import { workspaceAiService } from './workspaceAiService'
// import { vectorOperationsService } from './vectorOperationsService'
// import { clusteringService } from './clusteringService'
// import { multiWorkspaceService } from './multiWorkspaceService'
// import { realTimeProcessingService } from './realTimeProcessingService'

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
        // Core Services
        { name: 'workspace', test: () => workspaceService.listWorkspaces() },
        { name: 'document', test: () => documentService.listDocuments() },
        { name: 'ai', test: () => aiService.getModels() },
        { name: 'search', test: () => searchService.getIndexStatus() },

        // Advanced Services
        { name: 'styleAnalysis', test: () => styleAnalysisService.listStyleProfiles() },
        { name: 'styleTransfer', test: () => styleTransferService.getTransferHistory('test') },
        { name: 'knowledgeAnalyzer', test: () => knowledgeAnalyzerService.listKnowledgeBases() },
        {
          name: 'smartOrganizer',
          test: () => smartOrganizerService.getOrganizationPerformance('test'),
        },
        { name: 'contentLifecycle', test: () => contentLifecycleService.getPendingActions() },
        {
          name: 'conversationIntelligence',
          test: () => conversationIntelligenceService.getConversationAnalytics(),
        },
        { name: 'embedding', test: () => embeddingService.getEmbeddingModels() },
        {
          name: 'documentGeneration',
          test: () => documentGenerationService.getGenerationHistory(),
        },
        { name: 'formatConversion', test: () => formatConversionService.getSupportedFormats() },
        { name: 'template', test: () => templateService.getTemplateCategories() },
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
