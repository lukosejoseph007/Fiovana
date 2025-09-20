// src/components/progress/ProgressList.tsx
// List component for displaying multiple progress operations

import React, { useState, useMemo } from 'react'
import { Filter, Trash2, RefreshCw } from 'lucide-react'
import ProgressCard from './ProgressCard'
import type { ProgressListProps } from '../../types/progress'
import { OperationStatus } from '../../types/progress'

const ProgressList: React.FC<
  ProgressListProps & {
    onRefresh?: () => void
    onCleanup?: () => void
    loading?: boolean
  }
> = ({
  operations,
  onCancel,
  onRetry,
  onRefresh,
  onCleanup,
  showCompleted = true,
  maxItems = 50,
  loading = false,
}) => {
  const [filter, setFilter] = useState<'all' | 'active' | 'completed' | 'failed'>('all')
  const [compactMode, setCompactMode] = useState(false)

  const filteredOperations = useMemo(() => {
    let filtered = operations

    // Apply status filter
    switch (filter) {
      case 'active':
        filtered = operations.filter(
          op =>
            op.status === OperationStatus.Running ||
            op.status === OperationStatus.Pending ||
            op.status === OperationStatus.Paused
        )
        break
      case 'completed':
        filtered = operations.filter(op => op.status === OperationStatus.Completed)
        break
      case 'failed':
        filtered = operations.filter(
          op => op.status === OperationStatus.Failed || op.status === OperationStatus.Cancelled
        )
        break
      case 'all':
      default:
        filtered = showCompleted
          ? operations
          : operations.filter(
              op =>
                op.status !== OperationStatus.Completed &&
                op.status !== OperationStatus.Failed &&
                op.status !== OperationStatus.Cancelled
            )
        break
    }

    // Sort by status (active first) and then by start time (most recent first)
    filtered.sort((a, b) => {
      // Active operations first
      const aActive = a.status === OperationStatus.Running || a.status === OperationStatus.Pending
      const bActive = b.status === OperationStatus.Running || b.status === OperationStatus.Pending

      if (aActive && !bActive) return -1
      if (!aActive && bActive) return 1

      // Then by start time (most recent first)
      return new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
    })

    // Apply max items limit
    return filtered.slice(0, maxItems)
  }, [operations, filter, showCompleted, maxItems])

  const getFilterCounts = () => {
    const counts = {
      all: operations.length,
      active: operations.filter(
        op =>
          op.status === OperationStatus.Running ||
          op.status === OperationStatus.Pending ||
          op.status === OperationStatus.Paused
      ).length,
      completed: operations.filter(op => op.status === OperationStatus.Completed).length,
      failed: operations.filter(
        op => op.status === OperationStatus.Failed || op.status === OperationStatus.Cancelled
      ).length,
    }
    return counts
  }

  const counts = getFilterCounts()

  if (operations.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-8">
        <div className="text-center">
          <div className="w-16 h-16 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center mx-auto mb-4">
            <RefreshCw className="h-8 w-8 text-gray-400" />
          </div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No Operations</h3>
          <p className="text-gray-500 dark:text-gray-400">
            There are no import operations to display. Operations will appear here when files are
            processed.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Import Operations</h2>

          <div className="flex items-center space-x-2">
            {/* Compact mode toggle */}
            <button
              onClick={() => setCompactMode(!compactMode)}
              className={`px-3 py-1 text-xs rounded transition-colors ${
                compactMode
                  ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300'
                  : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
              }`}
            >
              Compact
            </button>

            {/* Cleanup button */}
            {onCleanup && (counts.completed > 0 || counts.failed > 0) && (
              <button
                onClick={onCleanup}
                className="px-3 py-1 text-xs bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300 rounded hover:bg-red-200 dark:hover:bg-red-900/30 transition-colors flex items-center space-x-1"
                title="Clean up completed and failed operations"
              >
                <Trash2 className="h-3 w-3" />
                <span>Cleanup</span>
              </button>
            )}

            {/* Refresh button */}
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

        {/* Filter tabs */}
        <div className="flex items-center space-x-1 mt-3">
          <Filter className="h-4 w-4 text-gray-400" />
          <div className="flex space-x-1">
            {[
              { key: 'all', label: 'All', count: counts.all },
              { key: 'active', label: 'Active', count: counts.active },
              { key: 'completed', label: 'Completed', count: counts.completed },
              { key: 'failed', label: 'Failed', count: counts.failed },
            ].map(({ key, label, count }) => (
              <button
                key={key}
                onClick={() => setFilter(key as 'all' | 'active' | 'completed' | 'failed')}
                className={`px-2 py-1 text-xs rounded transition-colors ${
                  filter === key
                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300'
                    : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
                }`}
              >
                {label} ({count})
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Operations list */}
      <div className="p-4">
        {filteredOperations.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-gray-500 dark:text-gray-400">
              No operations match the current filter.
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredOperations.map(operation => (
              <ProgressCard
                key={operation.operation_id}
                progress={operation}
                onCancel={onCancel}
                onRetry={onRetry}
                compact={compactMode}
              />
            ))}

            {filteredOperations.length >= maxItems && (
              <div className="text-center py-2">
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Showing {maxItems} of {operations.length} operations
                </p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

export default ProgressList
