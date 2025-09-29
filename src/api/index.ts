// Centralized API exports
export { apiClient, TauriApiClient } from './client'
export { commandRegistry, CommandRegistry } from './commandRegistry'
export { errorHandler, ErrorHandler, ApiError, ErrorType, ErrorSeverity } from './errorHandler'
export { eventBus, EventBus, EventTypes } from './eventBus'
export * from './typeGuards'

// Import instances for use in initialization
import { commandRegistry } from './commandRegistry'
import { errorHandler, ErrorSeverity } from './errorHandler'
import { eventBus } from './eventBus'
import { apiClient } from './client'

export type {
  InvokeOptions,
  BatchCommand,
  BatchOptions,
  BatchResponse,
  BatchResult,
  BatchSummary,
  StreamProgress,
  StreamChunk,
  PerformanceMetric,
} from './client'

export type {
  CommandDefinition,
  ParameterDefinition,
  ValidationRule,
  CommandExample,
  CommandModule,
  ValidationResult,
  CommandHelp,
  ParameterInfo,
} from './commandRegistry'

export type {
  ErrorContext,
  ErrorDetail,
  ErrorRecoveryStrategy,
  ErrorStatistics,
} from './errorHandler'

export type { EventSubscriber, SubscriptionOptions, Event, EventMetrics } from './eventBus'

// Convenience function to initialize the entire API system
export async function initializeApiSystem(): Promise<void> {
  try {
    // Initialize command registry
    await commandRegistry.initialize()

    // Set up global error handling
    errorHandler.onError(error => {
      // Emit error events for system-wide error handling
      eventBus.emit('system:error', {
        error: error.toJSON(),
        timestamp: new Date(),
      })

      // Log critical errors
      if (error.severity === ErrorSeverity.CRITICAL) {
        console.error('[CRITICAL ERROR]', error)
      }
    })

    // Set up performance monitoring
    setupPerformanceMonitoring()

    console.log('API system initialized successfully')
  } catch (error) {
    console.error('Failed to initialize API system:', error)
    throw error
  }
}

function setupPerformanceMonitoring(): void {
  // Monitor API performance
  setInterval(() => {
    const metrics = apiClient.getPerformanceMetrics()
    const eventMetrics = eventBus.getMetrics()
    const errorStats = errorHandler.getErrorStatistics()

    eventBus.emit('system:performance', {
      api: {
        commandMetrics: Object.fromEntries(metrics),
        totalCommands: metrics.size,
      },
      events: eventMetrics,
      errors: errorStats,
      timestamp: new Date(),
    })
  }, 30000) // Every 30 seconds
}
