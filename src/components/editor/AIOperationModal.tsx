// src/components/editor/AIOperationModal.tsx
import React, { useEffect } from 'react'
import { X, Loader2 } from 'lucide-react'
import { AIOperationResult } from './AIOperationResult'
import { TextOperationResult } from '../../services/textOperationService'

interface AIOperationModalProps {
  isOpen: boolean
  onClose: () => void
  isLoading: boolean
  result: TextOperationResult | null
  error: string | null
  onAccept: () => void
  onReject: () => void
  onRetry?: () => void
}

export const AIOperationModal: React.FC<AIOperationModalProps> = ({
  isOpen,
  onClose,
  isLoading,
  result,
  error,
  onAccept,
  onReject,
  onRetry,
}) => {
  // Close on Escape key
  useEffect(() => {
    if (!isOpen) return

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }

    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [isOpen, onClose])

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />

      {/* Modal */}
      <div className="relative z-10 w-full max-w-3xl mx-4 max-h-[90vh] overflow-hidden">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-2xl">
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              AI Text Operation
            </h2>
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              aria-label="Close modal"
            >
              <X className="h-5 w-5 text-gray-500 dark:text-gray-400" />
            </button>
          </div>

          {/* Content */}
          <div className="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
            {/* Loading state */}
            {isLoading && (
              <div className="flex flex-col items-center justify-center py-12">
                <Loader2 className="h-12 w-12 text-blue-500 animate-spin mb-4" />
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  AI is processing your request...
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-500 mt-2">
                  This may take a few seconds
                </p>
              </div>
            )}

            {/* Error state */}
            {error && !isLoading && (
              <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-700 rounded-lg p-4">
                <h3 className="text-sm font-semibold text-red-800 dark:text-red-200 mb-2">
                  Error Processing Request
                </h3>
                <p className="text-sm text-red-700 dark:text-red-300 mb-4">{error}</p>
                <div className="flex gap-2">
                  {onRetry && (
                    <button
                      onClick={onRetry}
                      className="px-4 py-2 text-sm bg-red-500 text-white rounded hover:bg-red-600 transition-colors"
                    >
                      Try Again
                    </button>
                  )}
                  <button
                    onClick={onClose}
                    className="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                  >
                    Close
                  </button>
                </div>
              </div>
            )}

            {/* Result */}
            {result && !isLoading && !error && (
              <AIOperationResult
                result={result}
                onAccept={onAccept}
                onReject={onReject}
                onRetry={onRetry}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
