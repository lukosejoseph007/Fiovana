// src/components/editor/AIOperationResult.tsx
import React from 'react'
import { CheckCircle, XCircle, Copy, RotateCcw, Sparkles, AlertCircle } from 'lucide-react'
import ReactMarkdown from 'react-markdown'
import { TextOperationResult } from '../../services/textOperationService'

interface AIOperationResultProps {
  result: TextOperationResult
  onAccept: () => void
  onReject: () => void
  onCopy?: () => void
  onRetry?: () => void
  isAccepting?: boolean
  isRejecting?: boolean
}

export const AIOperationResult: React.FC<AIOperationResultProps> = ({
  result,
  onAccept,
  onReject,
  onCopy,
  onRetry,
  isAccepting = false,
  isRejecting = false,
}) => {
  const handleCopy = () => {
    navigator.clipboard.writeText(result.result)
    onCopy?.()
  }

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'text-green-600 dark:text-green-400'
    if (confidence >= 0.6) return 'text-yellow-600 dark:text-yellow-400'
    return 'text-red-600 dark:text-red-400'
  }

  const getConfidenceBadge = (confidence: number) => {
    if (confidence >= 0.8) return 'High Confidence'
    if (confidence >= 0.6) return 'Medium Confidence'
    return 'Low Confidence'
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-blue-500" />
            <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
              AI Result: {result.operation}
            </h3>
          </div>
          <div className="flex items-center gap-2">
            <span className={`text-xs font-medium ${getConfidenceColor(result.confidence)}`}>
              {getConfidenceBadge(result.confidence)}
            </span>
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {Math.round(result.confidence * 100)}%
            </span>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="p-4 space-y-4">
        {/* Original text */}
        <div>
          <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
            Original:
          </label>
          <div className="p-3 bg-gray-50 dark:bg-gray-900/50 rounded-md border border-gray-200 dark:border-gray-700">
            <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
              {result.original}
            </p>
          </div>
        </div>

        {/* AI result */}
        <div>
          <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
            AI Suggestion:
          </label>
          <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-md border border-blue-200 dark:border-blue-700 prose prose-sm dark:prose-invert max-w-none">
            <ReactMarkdown
              components={{
                // Style headings
                h1: ({ ...props }) => (
                  <h1
                    className="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100"
                    {...props}
                  />
                ),
                h2: ({ ...props }) => (
                  <h2
                    className="text-base font-bold mb-2 text-gray-900 dark:text-gray-100"
                    {...props}
                  />
                ),
                h3: ({ ...props }) => (
                  <h3
                    className="text-sm font-bold mb-1 text-gray-900 dark:text-gray-100"
                    {...props}
                  />
                ),
                // Style paragraphs
                p: ({ ...props }) => (
                  <p className="mb-2 text-gray-900 dark:text-gray-100" {...props} />
                ),
                // Style lists
                ul: ({ ...props }) => (
                  <ul
                    className="list-disc list-inside mb-2 text-gray-900 dark:text-gray-100"
                    {...props}
                  />
                ),
                ol: ({ ...props }) => (
                  <ol
                    className="list-decimal list-inside mb-2 text-gray-900 dark:text-gray-100"
                    {...props}
                  />
                ),
                li: ({ ...props }) => (
                  <li className="mb-1 text-gray-900 dark:text-gray-100" {...props} />
                ),
                // Style code
                code: (props: React.HTMLAttributes<HTMLElement> & { inline?: boolean }) => {
                  const { inline, ...rest } = props
                  return inline ? (
                    <code
                      className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs text-gray-900 dark:text-gray-100"
                      {...rest}
                    />
                  ) : (
                    <code
                      className="block p-2 bg-gray-200 dark:bg-gray-700 rounded text-xs overflow-x-auto text-gray-900 dark:text-gray-100"
                      {...rest}
                    />
                  )
                },
                // Style emphasis
                strong: ({ ...props }) => (
                  <strong className="font-bold text-gray-900 dark:text-gray-100" {...props} />
                ),
                em: ({ ...props }) => (
                  <em className="italic text-gray-900 dark:text-gray-100" {...props} />
                ),
              }}
            >
              {result.result}
            </ReactMarkdown>
          </div>
        </div>

        {/* Reasoning */}
        {result.reasoning && (
          <div>
            <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
              AI Reasoning:
            </label>
            <div className="p-3 bg-purple-50 dark:bg-purple-900/20 rounded-md border border-purple-200 dark:border-purple-700">
              <p className="text-xs text-gray-700 dark:text-gray-300 italic">{result.reasoning}</p>
            </div>
          </div>
        )}

        {/* Suggestions */}
        {result.suggestions && result.suggestions.length > 0 && (
          <div>
            <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
              Additional Suggestions:
            </label>
            <ul className="space-y-1">
              {result.suggestions.map((suggestion, index) => (
                <li
                  key={index}
                  className="text-xs text-gray-600 dark:text-gray-400 flex items-start gap-2"
                >
                  <span className="text-blue-500 mt-1">•</span>
                  <span>{suggestion}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Alternative results */}
        {result.alternative_results && result.alternative_results.length > 0 && (
          <div>
            <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
              Alternative Versions:
            </label>
            <div className="space-y-2">
              {result.alternative_results.map((alternative, index) => (
                <div
                  key={index}
                  className="p-2 bg-gray-50 dark:bg-gray-900/50 rounded border border-gray-200 dark:border-gray-700"
                >
                  <p className="text-xs text-gray-700 dark:text-gray-300">{alternative}</p>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Low confidence warning */}
        {result.confidence < 0.6 && (
          <div className="flex items-start gap-2 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-700 rounded-md">
            <AlertCircle className="h-4 w-4 text-yellow-600 dark:text-yellow-400 mt-0.5" />
            <div className="flex-1">
              <p className="text-xs text-yellow-800 dark:text-yellow-200 font-medium">
                Low Confidence Result
              </p>
              <p className="text-xs text-yellow-700 dark:text-yellow-300 mt-1">
                This result may need manual review. Consider trying a different operation or
                providing more context.
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="px-4 py-3 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between gap-2">
          <div className="flex gap-2">
            <button
              onClick={handleCopy}
              className="px-3 py-1.5 text-xs flex items-center gap-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              <Copy className="h-3.5 w-3.5" />
              Copy
            </button>
            {onRetry && (
              <button
                onClick={onRetry}
                className="px-3 py-1.5 text-xs flex items-center gap-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
              >
                <RotateCcw className="h-3.5 w-3.5" />
                Retry
              </button>
            )}
          </div>
          <div className="flex gap-2">
            <button
              onClick={onReject}
              disabled={isRejecting}
              className="px-3 py-1.5 text-xs flex items-center gap-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <XCircle className="h-3.5 w-3.5" />
              {isRejecting ? 'Rejecting...' : 'Reject'}
            </button>
            <button
              onClick={onAccept}
              disabled={isAccepting}
              className="px-3 py-1.5 text-xs flex items-center gap-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <CheckCircle className="h-3.5 w-3.5" />
              {isAccepting ? 'Accepting...' : 'Accept & Replace'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
