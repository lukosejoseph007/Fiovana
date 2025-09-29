// Comprehensive error handling system
export enum ErrorType {
  NETWORK = 'NETWORK',
  VALIDATION = 'VALIDATION',
  AUTHENTICATION = 'AUTHENTICATION',
  AUTHORIZATION = 'AUTHORIZATION',
  NOT_FOUND = 'NOT_FOUND',
  TIMEOUT = 'TIMEOUT',
  RATE_LIMIT = 'RATE_LIMIT',
  SERVER_ERROR = 'SERVER_ERROR',
  CLIENT_ERROR = 'CLIENT_ERROR',
  UNKNOWN = 'UNKNOWN'
}

export enum ErrorSeverity {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL'
}

export interface ErrorContext {
  command?: string
  parameters?: Record<string, unknown>
  timestamp: number
  userAgent?: string
  sessionId?: string
  userId?: string
  stackTrace?: string
  additionalData?: Record<string, unknown>
}

export interface ErrorDetail {
  id: string
  type: ErrorType
  severity: ErrorSeverity
  message: string
  code?: string
  context: ErrorContext
  retryable: boolean
  retryAfter?: number
  suggestions?: string[]
  documentation?: string
}

export interface ErrorRecoveryStrategy {
  type: 'retry' | 'fallback' | 'ignore' | 'escalate'
  config: Record<string, unknown>
  description: string
}

export class ApiError extends Error {
  public readonly id: string
  public readonly type: ErrorType
  public readonly severity: ErrorSeverity
  public readonly code?: string
  public readonly context: ErrorContext
  public readonly retryable: boolean
  public readonly retryAfter?: number
  public readonly suggestions: string[]
  public readonly documentation?: string

  constructor(detail: ErrorDetail) {
    super(detail.message)
    this.name = 'ApiError'
    this.id = detail.id
    this.type = detail.type
    this.severity = detail.severity
    this.code = detail.code
    this.context = detail.context
    this.retryable = detail.retryable
    this.retryAfter = detail.retryAfter
    this.suggestions = detail.suggestions || []
    this.documentation = detail.documentation
  }

  toJSON(): ErrorDetail {
    return {
      id: this.id,
      type: this.type,
      severity: this.severity,
      message: this.message,
      code: this.code,
      context: this.context,
      retryable: this.retryable,
      retryAfter: this.retryAfter,
      suggestions: this.suggestions,
      documentation: this.documentation
    }
  }
}

export class ErrorHandler {
  private static instance: ErrorHandler
  private errorLog: ErrorDetail[] = []
  private recoveryStrategies = new Map<string, ErrorRecoveryStrategy>()
  private errorCallbacks = new Set<(error: ApiError) => void>()

  private constructor() {
    this.setupDefaultRecoveryStrategies()
  }

  static getInstance(): ErrorHandler {
    if (!ErrorHandler.instance) {
      ErrorHandler.instance = new ErrorHandler()
    }
    return ErrorHandler.instance
  }

  /**
   * Handle and process an error
   */
  handleError(error: unknown, context: Partial<ErrorContext> = {}): ApiError {
    const errorDetail = this.classifyError(error, context)
    const apiError = new ApiError(errorDetail)

    // Log the error
    this.logError(errorDetail)

    // Notify error callbacks
    this.notifyErrorCallbacks(apiError)

    // Apply recovery strategy if applicable
    this.applyRecoveryStrategy(apiError)

    return apiError
  }

  /**
   * Register an error callback
   */
  onError(callback: (error: ApiError) => void): void {
    this.errorCallbacks.add(callback)
  }

  /**
   * Remove an error callback
   */
  removeErrorCallback(callback: (error: ApiError) => void): void {
    this.errorCallbacks.delete(callback)
  }

  /**
   * Get error history
   */
  getErrorHistory(limit?: number): ErrorDetail[] {
    return limit ? this.errorLog.slice(-limit) : [...this.errorLog]
  }

  /**
   * Get error statistics
   */
  getErrorStatistics(): ErrorStatistics {
    const typeCount = new Map<ErrorType, number>()
    const severityCount = new Map<ErrorSeverity, number>()
    const totalErrors = this.errorLog.length
    let retryableErrors = 0

    for (const error of this.errorLog) {
      typeCount.set(error.type, (typeCount.get(error.type) || 0) + 1)
      severityCount.set(error.severity, (severityCount.get(error.severity) || 0) + 1)
      if (error.retryable) retryableErrors++
    }

    return {
      totalErrors,
      retryableErrors,
      errorsByType: Object.fromEntries(typeCount),
      errorsBySeverity: Object.fromEntries(severityCount),
      lastError: this.errorLog[this.errorLog.length - 1],
      errorRate: this.calculateErrorRate()
    }
  }

  /**
   * Clear error history
   */
  clearErrorHistory(): void {
    this.errorLog = []
  }

  /**
   * Register a recovery strategy
   */
  registerRecoveryStrategy(errorPattern: string, strategy: ErrorRecoveryStrategy): void {
    this.recoveryStrategies.set(errorPattern, strategy)
  }

  /**
   * Classify an error into our error system
   */
  private classifyError(error: unknown, context: Partial<ErrorContext>): ErrorDetail {
    const id = this.generateErrorId()
    const timestamp = Date.now()
    const fullContext: ErrorContext = {
      timestamp,
      ...context
    }

    if (error instanceof ApiError) {
      return {
        ...error.toJSON(),
        context: { ...error.context, ...fullContext }
      }
    }

    if (error instanceof Error) {
      const { type, severity, retryable, suggestions } = this.analyzeError(error)

      return {
        id,
        type,
        severity,
        message: error.message,
        context: {
          ...fullContext,
          stackTrace: error.stack
        },
        retryable,
        suggestions
      }
    }

    // Handle non-Error objects
    const message = typeof error === 'string' ? error : 'Unknown error occurred'
    return {
      id,
      type: ErrorType.UNKNOWN,
      severity: ErrorSeverity.MEDIUM,
      message,
      context: fullContext,
      retryable: false,
      suggestions: ['Check the error details and try again']
    }
  }

  /**
   * Analyze an Error object to determine type and severity
   */
  private analyzeError(error: Error): {
    type: ErrorType
    severity: ErrorSeverity
    retryable: boolean
    suggestions: string[]
  } {
    const message = error.message.toLowerCase()
    const name = error.name.toLowerCase()

    // Network-related errors
    if (message.includes('network') || message.includes('connection') || message.includes('fetch')) {
      return {
        type: ErrorType.NETWORK,
        severity: ErrorSeverity.HIGH,
        retryable: true,
        suggestions: ['Check your internet connection', 'Try again in a few moments']
      }
    }

    // Timeout errors
    if (message.includes('timeout') || message.includes('timed out')) {
      return {
        type: ErrorType.TIMEOUT,
        severity: ErrorSeverity.MEDIUM,
        retryable: true,
        suggestions: ['The operation took too long', 'Try breaking the task into smaller parts']
      }
    }

    // Validation errors
    if (message.includes('validation') || message.includes('invalid') || name.includes('validation')) {
      return {
        type: ErrorType.VALIDATION,
        severity: ErrorSeverity.LOW,
        retryable: false,
        suggestions: ['Check the input parameters', 'Ensure all required fields are provided']
      }
    }

    // Authentication errors
    if (message.includes('unauthorized') || message.includes('authentication')) {
      return {
        type: ErrorType.AUTHENTICATION,
        severity: ErrorSeverity.HIGH,
        retryable: false,
        suggestions: ['Check your authentication credentials', 'You may need to log in again']
      }
    }

    // Authorization errors
    if (message.includes('forbidden') || message.includes('permission')) {
      return {
        type: ErrorType.AUTHORIZATION,
        severity: ErrorSeverity.HIGH,
        retryable: false,
        suggestions: ['You do not have permission for this operation', 'Contact an administrator']
      }
    }

    // Not found errors
    if (message.includes('not found') || message.includes('404')) {
      return {
        type: ErrorType.NOT_FOUND,
        severity: ErrorSeverity.MEDIUM,
        retryable: false,
        suggestions: ['The requested resource was not found', 'Check the path or identifier']
      }
    }

    // Rate limit errors
    if (message.includes('rate limit') || message.includes('too many requests')) {
      return {
        type: ErrorType.RATE_LIMIT,
        severity: ErrorSeverity.MEDIUM,
        retryable: true,
        suggestions: ['You are making requests too quickly', 'Wait a moment before trying again']
      }
    }

    // Server errors
    if (message.includes('internal server error') || message.includes('500')) {
      return {
        type: ErrorType.SERVER_ERROR,
        severity: ErrorSeverity.HIGH,
        retryable: true,
        suggestions: ['There was a server error', 'Try again later or contact support']
      }
    }

    // Default classification
    return {
      type: ErrorType.CLIENT_ERROR,
      severity: ErrorSeverity.MEDIUM,
      retryable: false,
      suggestions: ['An unexpected error occurred', 'Check the error details for more information']
    }
  }

  private logError(error: ErrorDetail): void {
    this.errorLog.push(error)

    // Keep only last 1000 errors to prevent memory issues
    if (this.errorLog.length > 1000) {
      this.errorLog = this.errorLog.slice(-1000)
    }

    // Log to console for development
    if (process.env.NODE_ENV === 'development') {
      console.error('[ErrorHandler]', error)
    }
  }

  private notifyErrorCallbacks(error: ApiError): void {
    for (const callback of this.errorCallbacks) {
      try {
        callback(error)
      } catch (callbackError) {
        console.error('Error in error callback:', callbackError)
      }
    }
  }

  private applyRecoveryStrategy(error: ApiError): void {
    for (const [pattern, strategy] of this.recoveryStrategies) {
      if (this.matchesPattern(error, pattern)) {
        this.executeRecoveryStrategy(error, strategy)
        break
      }
    }
  }

  private matchesPattern(error: ApiError, pattern: string): boolean {
    // Simple pattern matching - could be enhanced with regex or more complex rules
    return error.type === pattern || error.code === pattern || error.message.includes(pattern)
  }

  private executeRecoveryStrategy(error: ApiError, strategy: ErrorRecoveryStrategy): void {
    switch (strategy.type) {
      case 'retry':
        // Implementation would depend on the calling context
        console.log(`Applying retry strategy for error ${error.id}`)
        break
      case 'fallback':
        console.log(`Applying fallback strategy for error ${error.id}`)
        break
      case 'ignore':
        console.log(`Ignoring error ${error.id}`)
        break
      case 'escalate':
        console.log(`Escalating error ${error.id}`)
        break
    }
  }

  private setupDefaultRecoveryStrategies(): void {
    this.registerRecoveryStrategy(ErrorType.NETWORK, {
      type: 'retry',
      config: { maxRetries: 3, backoff: 'exponential' },
      description: 'Retry network requests with exponential backoff'
    })

    this.registerRecoveryStrategy(ErrorType.TIMEOUT, {
      type: 'retry',
      config: { maxRetries: 2, timeout: 'extended' },
      description: 'Retry with extended timeout'
    })

    this.registerRecoveryStrategy(ErrorType.RATE_LIMIT, {
      type: 'retry',
      config: { respectRetryAfter: true },
      description: 'Retry after rate limit reset'
    })
  }

  private generateErrorId(): string {
    return `err_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private calculateErrorRate(): number {
    if (this.errorLog.length === 0) return 0

    const oneHourAgo = Date.now() - 60 * 60 * 1000
    const recentErrors = this.errorLog.filter(error => error.context.timestamp > oneHourAgo)

    return recentErrors.length / 60 // Errors per minute
  }
}

export interface ErrorStatistics {
  totalErrors: number
  retryableErrors: number
  errorsByType: Record<string, number>
  errorsBySeverity: Record<string, number>
  lastError?: ErrorDetail
  errorRate: number
}

// Export singleton instance
export const errorHandler = ErrorHandler.getInstance()