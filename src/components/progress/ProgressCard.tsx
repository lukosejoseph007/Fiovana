// src/components/progress/ProgressCard.tsx
// Individual progress operation card component

import React from 'react'
import {
  X,
  Clock,
  FileText,
  CheckCircle,
  XCircle,
  AlertCircle,
  Pause,
  Play,
  RotateCcw,
} from 'lucide-react'
import type { ProgressCardProps } from '../../types/progress'
import { progressService } from '../../services/progressService'

const ProgressCard: React.FC<ProgressCardProps> = ({
  progress,
  onCancel,
  onRetry,
  compact = false,
}) => {
  const getStatusIcon = () => {
    switch (progress.status) {
      case 'Running':
        return <Play className="h-4 w-4 text-blue-500 animate-spin" />
      case 'Completed':
        return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'Failed':
        return <XCircle className="h-4 w-4 text-red-500" />
      case 'Cancelled':
        return <XCircle className="h-4 w-4 text-orange-500" />
      case 'Pending':
        return <Clock className="h-4 w-4 text-yellow-500" />
      case 'Paused':
        return <Pause className="h-4 w-4 text-gray-500" />
      default:
        return <AlertCircle className="h-4 w-4 text-gray-400" />
    }
  }

  const getStatusColor = () => {
    switch (progress.status) {
      case 'Running':
        return 'border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-900/20'
      case 'Completed':
        return 'border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-900/20'
      case 'Failed':
        return 'border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20'
      case 'Cancelled':
        return 'border-orange-200 bg-orange-50 dark:border-orange-800 dark:bg-orange-900/20'
      case 'Pending':
        return 'border-yellow-200 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-900/20'
      case 'Paused':
        return 'border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800'
      default:
        return 'border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800'
    }
  }

  const handleCancel = () => {
    if (onCancel && progress.cancellable) {
      onCancel(progress.operation_id)
    }
  }

  const handleRetry = () => {
    if (onRetry && (progress.status === 'Failed' || progress.status === 'Cancelled')) {
      onRetry(progress.operation_id)
    }
  }

  const formatETA = (etaSeconds?: number) => {
    return progressService.formatETA(etaSeconds)
  }

  const formatDuration = (startTime: string) => {
    return progressService.formatDuration(startTime)
  }

  if (compact) {
    return (
      <div className={`border rounded-lg p-3 ${getStatusColor()}`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2 flex-1 min-w-0">
            {getStatusIcon()}
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-gray-900 dark:text-white truncate">
                {progress.current_step}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {progress.files_processed}/{progress.total_files} files
              </div>
            </div>
          </div>

          <div className="flex items-center space-x-2 ml-2">
            <div className="text-right">
              <div className="text-sm font-medium text-gray-900 dark:text-white">
                {progress.progress_percentage.toFixed(1)}%
              </div>
            </div>

            {progress.cancellable && onCancel && (
              <button
                onClick={handleCancel}
                className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                title="Cancel operation"
              >
                <X className="h-4 w-4" />
              </button>
            )}
          </div>
        </div>

        {/* Progress bar */}
        <div className="mt-2">
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
            <div
              className="bg-blue-500 h-1.5 rounded-full transition-all duration-300"
              style={{ width: `${progress.progress_percentage}%` }}
            />
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={`border rounded-lg p-4 ${getStatusColor()}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          {getStatusIcon()}
          <div>
            <h3 className="text-sm font-medium text-gray-900 dark:text-white">
              Operation {progress.operation_id.slice(0, 8)}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Started {formatDuration(progress.started_at)} ago
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {onRetry && (progress.status === 'Failed' || progress.status === 'Cancelled') && (
            <button
              onClick={handleRetry}
              className="p-2 text-gray-400 hover:text-blue-500 transition-colors"
              title="Retry operation"
            >
              <RotateCcw className="h-4 w-4" />
            </button>
          )}

          {progress.cancellable && onCancel && (
            <button
              onClick={handleCancel}
              className="p-2 text-gray-400 hover:text-red-500 transition-colors"
              title="Cancel operation"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>

      {/* Current step */}
      <div className="mb-3">
        <div className="flex items-center justify-between mb-1">
          <span className="text-sm font-medium text-gray-900 dark:text-white">
            {progress.current_step}
          </span>
          <span className="text-sm text-gray-500 dark:text-gray-400">
            {progress.progress_percentage.toFixed(1)}%
          </span>
        </div>

        {/* Progress bar */}
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
          <div
            className="bg-blue-500 h-2 rounded-full transition-all duration-300"
            style={{ width: `${progress.progress_percentage}%` }}
          />
        </div>
      </div>

      {/* File progress */}
      <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-300 mb-3">
        <div className="flex items-center space-x-1">
          <FileText className="h-4 w-4" />
          <span>
            {progress.files_processed} of {progress.total_files} files
          </span>
        </div>

        {progress.eta_seconds && (
          <div className="flex items-center space-x-1">
            <Clock className="h-4 w-4" />
            <span>{formatETA(progress.eta_seconds)} remaining</span>
          </div>
        )}
      </div>

      {/* Current file */}
      {progress.current_file && (
        <div className="mb-3">
          <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">Currently processing:</div>
          <div className="text-sm text-gray-900 dark:text-white truncate">
            {progress.current_file}
          </div>
        </div>
      )}

      {/* Steps progress */}
      {progress.steps && progress.steps.length > 0 && (
        <div className="mb-3">
          <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">Steps:</div>
          <div className="space-y-1">
            {progress.steps.map((step, index) => (
              <div key={index} className="flex items-center justify-between text-xs">
                <span
                  className={`${
                    step.status === 'Completed'
                      ? 'text-green-600 dark:text-green-400'
                      : step.status === 'Running'
                        ? 'text-blue-600 dark:text-blue-400'
                        : step.status === 'Failed'
                          ? 'text-red-600 dark:text-red-400'
                          : 'text-gray-500 dark:text-gray-400'
                  }`}
                >
                  {step.name}
                </span>
                <span className="text-gray-500 dark:text-gray-400">
                  {step.progress.toFixed(0)}%
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Errors and warnings */}
      {progress.errors.length > 0 && (
        <div className="mb-2">
          <div className="text-xs text-red-600 dark:text-red-400 mb-1">
            Errors ({progress.errors.length}):
          </div>
          <div className="max-h-20 overflow-y-auto">
            {progress.errors.map((error, index) => (
              <div key={index} className="text-xs text-red-600 dark:text-red-400 mb-1">
                • {error}
              </div>
            ))}
          </div>
        </div>
      )}

      {progress.warnings.length > 0 && (
        <div>
          <div className="text-xs text-yellow-600 dark:text-yellow-400 mb-1">
            Warnings ({progress.warnings.length}):
          </div>
          <div className="max-h-20 overflow-y-auto">
            {progress.warnings.map((warning, index) => (
              <div key={index} className="text-xs text-yellow-600 dark:text-yellow-400 mb-1">
                • {warning}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

export default ProgressCard
