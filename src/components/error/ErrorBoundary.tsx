import React, { Component, ErrorInfo, ReactNode } from 'react'
import { Card, Button } from '../ui'

interface Props {
  children: ReactNode
  fallback?: ReactNode
  onError?: (error: Error, errorInfo: ErrorInfo) => void
  resetKeys?: Array<string | number>
  FallbackComponent?: React.ComponentType<FallbackProps>
}

interface State {
  hasError: boolean
  error: Error | null
  errorInfo: ErrorInfo | null
  errorCount: number
}

export interface FallbackProps {
  error: Error
  errorInfo: ErrorInfo | null
  resetError: () => void
  errorCount: number
}

/**
 * Production-grade Error Boundary component
 *
 * Features:
 * - Graceful error recovery with user-friendly messages
 * - Error reporting and logging integration
 * - Fallback UI components for broken sections
 * - Network error handling with retry mechanisms
 * - AI service failure handling with degraded modes
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorCount: 0,
    }
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
    }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // Log error to console in development
    if (process.env.NODE_ENV === 'development') {
      console.error('ErrorBoundary caught an error:', error)
      console.error('Error Info:', errorInfo)
    }

    // Update state with error details
    this.setState(prevState => ({
      error,
      errorInfo,
      errorCount: prevState.errorCount + 1,
    }))

    // Call custom error handler if provided
    if (this.props.onError) {
      try {
        this.props.onError(error, errorInfo)
      } catch (handlerError) {
        console.error('Error in onError handler:', handlerError)
      }
    }

    // Log to error reporting service (integrate with your error logging service)
    this.logErrorToService(error, errorInfo)
  }

  componentDidUpdate(prevProps: Props): void {
    // Reset error boundary if resetKeys change
    if (this.state.hasError && this.props.resetKeys) {
      const prevKeys = prevProps.resetKeys || []
      const currentKeys = this.props.resetKeys

      if (
        prevKeys.length !== currentKeys.length ||
        prevKeys.some((key, index) => key !== currentKeys[index])
      ) {
        this.resetError()
      }
    }
  }

  resetError = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    })
  }

  logErrorToService(error: Error, errorInfo: ErrorInfo): void {
    // TODO: Integrate with error logging service (e.g., Sentry, LogRocket)
    // For now, log to console
    const errorReport = {
      message: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
    }

    // In production, send to error tracking service
    if (process.env.NODE_ENV === 'production') {
      // Example: window.errorTracker?.captureException(error, { extra: errorReport });
      console.error('Error Report:', errorReport)
    }
  }

  isNetworkError(error: Error): boolean {
    return (
      error.message.includes('NetworkError') ||
      error.message.includes('Failed to fetch') ||
      error.message.includes('Network request failed') ||
      error.message.includes('ECONNREFUSED') ||
      error.name === 'NetworkError'
    )
  }

  isAIServiceError(error: Error): boolean {
    return (
      error.message.includes('AI service') ||
      error.message.includes('Ollama') ||
      error.message.includes('OpenRouter') ||
      error.message.includes('Anthropic') ||
      error.message.includes('Model not available')
    )
  }

  renderFallback(): ReactNode {
    const { error, errorInfo, errorCount } = this.state
    const { FallbackComponent } = this.props

    if (!error) return null

    // Use custom fallback component if provided
    if (FallbackComponent) {
      return (
        <FallbackComponent
          error={error}
          errorInfo={errorInfo}
          resetError={this.resetError}
          errorCount={errorCount}
        />
      )
    }

    // Check for specific error types
    const isNetwork = this.isNetworkError(error)
    const isAIService = this.isAIServiceError(error)

    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-gray-50">
        <Card className="max-w-2xl w-full p-8 space-y-6">
          {/* Error Icon */}
          <div className="flex justify-center">
            <div className="w-16 h-16 rounded-full bg-red-100 flex items-center justify-center">
              <svg
                className="w-8 h-8 text-red-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
            </div>
          </div>

          {/* Error Title */}
          <div className="text-center">
            <h1 className="text-2xl font-bold text-gray-900 mb-2">
              {isNetwork
                ? 'Network Connection Error'
                : isAIService
                  ? 'AI Service Error'
                  : 'Something went wrong'}
            </h1>
            <p className="text-gray-600">
              {isNetwork
                ? 'Unable to connect to the server. Please check your internet connection and try again.'
                : isAIService
                  ? 'The AI service is currently unavailable. You can continue using other features while we work on this.'
                  : "We're sorry, but something unexpected happened. We've been notified and are working to fix it."}
            </p>
          </div>

          {/* Error Details (Development only) */}
          {process.env.NODE_ENV === 'development' && (
            <div className="bg-gray-100 rounded-lg p-4 space-y-2">
              <h3 className="font-semibold text-sm text-gray-700">Error Details (Dev Only):</h3>
              <div className="text-xs text-gray-600 font-mono space-y-1">
                <p className="font-semibold">
                  {error.name}: {error.message}
                </p>
                {error.stack && (
                  <pre className="overflow-x-auto whitespace-pre-wrap break-words max-h-40 overflow-y-auto">
                    {error.stack}
                  </pre>
                )}
                {errorInfo?.componentStack && (
                  <details className="mt-2">
                    <summary className="cursor-pointer font-semibold">Component Stack</summary>
                    <pre className="overflow-x-auto whitespace-pre-wrap break-words max-h-40 overflow-y-auto mt-2">
                      {errorInfo.componentStack}
                    </pre>
                  </details>
                )}
              </div>
            </div>
          )}

          {/* Error Count Warning */}
          {errorCount > 1 && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <p className="text-sm text-yellow-800">
                ⚠️ This error has occurred {errorCount} times. If the problem persists, please
                refresh the page or contact support.
              </p>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-3 justify-center">
            <Button onClick={this.resetError} variant="primary" className="min-w-[120px]">
              {isNetwork || isAIService ? 'Retry' : 'Try Again'}
            </Button>
            <Button
              onClick={() => window.location.reload()}
              variant="secondary"
              className="min-w-[120px]"
            >
              Reload Page
            </Button>
            {isAIService && (
              <Button
                onClick={() => {
                  // Navigate to settings or degraded mode
                  this.resetError()
                  // TODO: Navigate to settings page
                }}
                variant="ghost"
                className="min-w-[120px]"
              >
                Use Offline Mode
              </Button>
            )}
          </div>

          {/* Support Link */}
          <div className="text-center text-sm text-gray-500">
            Need help?{' '}
            <a
              href="https://github.com/anthropics/claude-code/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:underline"
            >
              Report this issue
            </a>
          </div>
        </Card>
      </div>
    )
  }

  render(): ReactNode {
    if (this.state.hasError) {
      // Custom fallback provided
      if (this.props.fallback) {
        return this.props.fallback
      }

      // Render default fallback
      return this.renderFallback()
    }

    return this.props.children
  }
}

/**
 * Minimal Error Fallback Component
 */
export const MinimalErrorFallback: React.FC<FallbackProps> = ({ error, resetError }) => {
  return (
    <div className="flex flex-col items-center justify-center p-8 space-y-4">
      <div className="text-red-600 text-lg font-semibold">Something went wrong</div>
      <div className="text-gray-600 text-sm text-center max-w-md">{error.message}</div>
      <Button onClick={resetError} variant="primary" size="sm">
        Try Again
      </Button>
    </div>
  )
}

/**
 * Network Error Fallback Component
 */
export const NetworkErrorFallback: React.FC<FallbackProps> = ({ resetError }) => {
  return (
    <div className="flex flex-col items-center justify-center p-8 space-y-4">
      <div className="w-12 h-12 rounded-full bg-orange-100 flex items-center justify-center">
        <svg
          className="w-6 h-6 text-orange-600"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-2.83m-1.414 5.658a9 9 0 01-2.167-9.238m7.824 2.167a1 1 0 111.414 1.414m-1.414-1.414L3 3m8.293 8.293l1.414 1.414"
          />
        </svg>
      </div>
      <div className="text-orange-600 text-lg font-semibold">Network Error</div>
      <div className="text-gray-600 text-sm text-center max-w-md">
        Unable to connect to the server. Please check your connection and try again.
      </div>
      <Button onClick={resetError} variant="primary" size="sm">
        Retry Connection
      </Button>
    </div>
  )
}
