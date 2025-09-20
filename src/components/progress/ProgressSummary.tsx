// src/components/progress/ProgressSummary.tsx
// Summary component for overall progress status

import React from 'react'
import {
  Activity,
  CheckCircle,
  XCircle,
  FileText,
  Clock,
  TrendingUp,
  RefreshCw,
  Trash2,
} from 'lucide-react'
import type { ProgressSummaryProps } from '../../types/progress'

const ProgressSummary: React.FC<
  ProgressSummaryProps & {
    loading?: boolean
    estimatedTime?: number | null
  }
> = ({ summary, onRefresh, onCleanup, loading = false, estimatedTime = null }) => {
  const formatETA = (etaSeconds?: number | null) => {
    if (!etaSeconds) return 'Unknown'

    if (etaSeconds < 60) {
      return `${etaSeconds}s`
    } else if (etaSeconds < 3600) {
      const minutes = Math.floor(etaSeconds / 60)
      const seconds = etaSeconds % 60
      return `${minutes}m ${seconds}s`
    } else {
      const hours = Math.floor(etaSeconds / 3600)
      const minutes = Math.floor((etaSeconds % 3600) / 60)
      return `${hours}h ${minutes}m`
    }
  }

  const stats = [
    {
      name: 'Active Operations',
      value: summary.active_operations,
      icon: Activity,
      color: 'blue',
      description: 'Currently running',
    },
    {
      name: 'Completed',
      value: summary.completed_operations,
      icon: CheckCircle,
      color: 'green',
      description: 'Successfully finished',
    },
    {
      name: 'Failed/Cancelled',
      value: summary.failed_operations,
      icon: XCircle,
      color: 'red',
      description: 'Require attention',
    },
    {
      name: 'Files Processing',
      value: summary.total_files_processing,
      icon: FileText,
      color: 'purple',
      description: 'Total files in queue',
    },
  ]

  const getColorClasses = (color: string) => {
    switch (color) {
      case 'blue':
        return {
          bg: 'bg-blue-50 dark:bg-blue-900/20',
          icon: 'text-blue-600 dark:text-blue-400',
          text: 'text-blue-900 dark:text-blue-100',
        }
      case 'green':
        return {
          bg: 'bg-green-50 dark:bg-green-900/20',
          icon: 'text-green-600 dark:text-green-400',
          text: 'text-green-900 dark:text-green-100',
        }
      case 'red':
        return {
          bg: 'bg-red-50 dark:bg-red-900/20',
          icon: 'text-red-600 dark:text-red-400',
          text: 'text-red-900 dark:text-red-100',
        }
      case 'purple':
        return {
          bg: 'bg-purple-50 dark:bg-purple-900/20',
          icon: 'text-purple-600 dark:text-purple-400',
          text: 'text-purple-900 dark:text-purple-100',
        }
      default:
        return {
          bg: 'bg-gray-50 dark:bg-gray-800',
          icon: 'text-gray-600 dark:text-gray-400',
          text: 'text-gray-900 dark:text-gray-100',
        }
    }
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Progress Overview</h2>

          <div className="flex items-center space-x-2">
            {onCleanup && (summary.completed_operations > 0 || summary.failed_operations > 0) && (
              <button
                onClick={onCleanup}
                className="px-3 py-1 text-xs bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300 rounded hover:bg-red-200 dark:hover:bg-red-900/30 transition-colors flex items-center space-x-1"
                title="Clean up completed and failed operations"
              >
                <Trash2 className="h-3 w-3" />
                <span>Cleanup</span>
              </button>
            )}

            {onRefresh && (
              <button
                onClick={onRefresh}
                disabled={loading}
                className="px-3 py-1 text-xs bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center space-x-1 disabled:opacity-50"
              >
                <RefreshCw className={`h-3 w-3 ${loading ? 'animate-spin' : ''}`} />
                <span>Refresh</span>
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="p-4">
        {/* Stats grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
          {stats.map(stat => {
            const Icon = stat.icon
            const colors = getColorClasses(stat.color)

            return (
              <div
                key={stat.name}
                className={`${colors.bg} rounded-lg p-4 border border-gray-200 dark:border-gray-700`}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                      {stat.name}
                    </p>
                    <p className={`text-2xl font-bold ${colors.text} mt-1`}>
                      {stat.value.toLocaleString()}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {stat.description}
                    </p>
                  </div>
                  <div className={`p-3 rounded-lg bg-white dark:bg-gray-800 shadow-sm`}>
                    <Icon className={`h-6 w-6 ${colors.icon}`} />
                  </div>
                </div>
              </div>
            )
          })}
        </div>

        {/* Overall progress */}
        {summary.total_files_processing > 0 && (
          <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-4">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-2">
                <TrendingUp className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                <span className="text-sm font-medium text-gray-900 dark:text-white">
                  Overall Progress
                </span>
              </div>
              <span className="text-sm text-gray-600 dark:text-gray-300">
                {summary.overall_progress.toFixed(1)}%
              </span>
            </div>

            {/* Progress bar */}
            <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-3 mb-3">
              <div
                className="bg-blue-500 h-3 rounded-full transition-all duration-500"
                style={{ width: `${summary.overall_progress}%` }}
              />
            </div>

            <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-300">
              <span>
                {summary.total_files_completed.toLocaleString()} of{' '}
                {summary.total_files_processing.toLocaleString()} files completed
              </span>

              {estimatedTime && (
                <div className="flex items-center space-x-1">
                  <Clock className="h-4 w-4" />
                  <span>{formatETA(estimatedTime)} remaining</span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Status message */}
        <div className="text-center">
          {summary.active_operations === 0 && summary.total_files_processing === 0 ? (
            <div className="text-gray-500 dark:text-gray-400">
              <FileText className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p className="text-sm">No active operations</p>
              <p className="text-xs">Import operations will appear here when files are processed</p>
            </div>
          ) : summary.active_operations > 0 ? (
            <div className="text-blue-600 dark:text-blue-400">
              <Activity className="h-8 w-8 mx-auto mb-2" />
              <p className="text-sm font-medium">
                {summary.active_operations} operation{summary.active_operations !== 1 ? 's' : ''} in
                progress
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Processing {summary.total_files_processing} files
              </p>
            </div>
          ) : (
            <div className="text-green-600 dark:text-green-400">
              <CheckCircle className="h-8 w-8 mx-auto mb-2" />
              <p className="text-sm font-medium">All operations completed</p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                {summary.completed_operations} successful, {summary.failed_operations} failed
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default ProgressSummary
