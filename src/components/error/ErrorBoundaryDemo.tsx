import React, { useState } from 'react'
import Button from '../ui/Button'
import { ErrorBoundary } from './ErrorBoundary'

/**
 * Demo component to test Error Boundary functionality
 *
 * This component demonstrates:
 * - Network errors
 * - AI service errors
 * - Generic errors
 * - Error recovery
 */

// Component that throws an error
const ThrowError: React.FC<{ errorType: string }> = ({ errorType }) => {
  if (errorType === 'network') {
    throw new Error('NetworkError: Failed to fetch data from server')
  }
  if (errorType === 'ai') {
    throw new Error('AI service unavailable: Ollama connection failed')
  }
  if (errorType === 'generic') {
    throw new Error('An unexpected error occurred in the application')
  }
  return <div>No error</div>
}

export const ErrorBoundaryDemo: React.FC = () => {
  const [errorType, setErrorType] = useState<string | null>(null)
  const [resetKey, setResetKey] = useState(0)

  const triggerError = (type: string) => {
    setResetKey(prev => prev + 1)
    setErrorType(type)
  }

  const clearError = () => {
    setErrorType(null)
    setResetKey(prev => prev + 1)
  }

  return (
    <div className="p-8 space-y-6 max-w-4xl mx-auto">
      <div className="bg-white rounded-lg shadow-sm p-6 space-y-4">
        <h2 className="text-2xl font-bold text-gray-900">Error Boundary Demo</h2>
        <p className="text-gray-600">
          Click the buttons below to test different error scenarios. The ErrorBoundary component
          will catch these errors and display appropriate fallback UI.
        </p>

        <div className="flex gap-3 flex-wrap">
          <Button
            onClick={() => triggerError('network')}
            variant="secondary"
            className="min-w-[160px]"
          >
            Trigger Network Error
          </Button>
          <Button onClick={() => triggerError('ai')} variant="secondary" className="min-w-[160px]">
            Trigger AI Service Error
          </Button>
          <Button
            onClick={() => triggerError('generic')}
            variant="secondary"
            className="min-w-[160px]"
          >
            Trigger Generic Error
          </Button>
          <Button onClick={clearError} variant="ghost" className="min-w-[160px]">
            Clear Error
          </Button>
        </div>
      </div>

      {/* Error Boundary Test Area */}
      <div className="bg-gray-50 rounded-lg p-6 min-h-[300px]">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Test Area</h3>
        <ErrorBoundary key={resetKey} resetKeys={[resetKey]}>
          {errorType ? (
            <ThrowError errorType={errorType} />
          ) : (
            <div className="flex items-center justify-center h-48 text-gray-500">
              Click a button above to trigger an error
            </div>
          )}
        </ErrorBoundary>
      </div>

      {/* Feature List */}
      <div className="bg-white rounded-lg shadow-sm p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">ErrorBoundary Features</h3>
        <ul className="space-y-2 text-gray-700">
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Graceful error recovery with user-friendly messages</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Error reporting and logging integration</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Fallback UI components for broken sections</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Network error handling with retry mechanisms</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>AI service failure handling with degraded modes</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Custom fallback components support</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Error count tracking to prevent infinite error loops</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="text-green-600 mt-1">✓</span>
            <span>Development mode error details with stack traces</span>
          </li>
        </ul>
      </div>
    </div>
  )
}
