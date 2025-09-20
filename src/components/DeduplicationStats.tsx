import React, { useState, useEffect, useCallback } from 'react'
import { clsx } from 'clsx'
import { StorageStats, GarbageCollectionResult } from '../types/deduplication'
import { DeduplicationService } from '../services/deduplicationService'

interface DeduplicationStatsProps {
  workspacePath?: string
  className?: string
  showGarbageCollection?: boolean
  onGarbageCollectionComplete?: (result: GarbageCollectionResult) => void
}

const DeduplicationStats: React.FC<DeduplicationStatsProps> = ({
  workspacePath,
  className,
  showGarbageCollection = true,
  onGarbageCollectionComplete,
}) => {
  const [stats, setStats] = useState<StorageStats | null>(null)
  const [allStats, setAllStats] = useState<Record<string, StorageStats> | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isRunningGC, setIsRunningGC] = useState(false)
  const [shouldRunGC, setShouldRunGC] = useState(false)
  const [gcResult, setGcResult] = useState<GarbageCollectionResult | null>(null)

  const loadStats = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      if (workspacePath) {
        // Load stats for specific workspace
        const [workspaceStats, shouldGC] = await Promise.all([
          DeduplicationService.getDeduplicationStats(workspacePath),
          DeduplicationService.shouldRunGarbageCollection(workspacePath),
        ])
        setStats(workspaceStats)
        setShouldRunGC(shouldGC)
      } else {
        // Load stats for all workspaces
        const globalStats = await DeduplicationService.getAllDeduplicationStats()
        setAllStats(globalStats)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load deduplication stats')
    } finally {
      setIsLoading(false)
    }
  }, [workspacePath])

  useEffect(() => {
    loadStats()
  }, [loadStats])

  const handleRunGarbageCollection = async () => {
    if (!workspacePath) return

    setIsRunningGC(true)
    setError(null)

    try {
      const result = await DeduplicationService.runGarbageCollection(workspacePath)
      setGcResult(result)
      setShouldRunGC(false)
      onGarbageCollectionComplete?.(result)

      // Reload stats after GC
      await loadStats()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to run garbage collection')
    } finally {
      setIsRunningGC(false)
    }
  }

  const formatPercentage = (value: number, total: number) => {
    if (total === 0) return '0%'
    return `${((value / total) * 100).toFixed(1)}%`
  }

  const getTotalStats = () => {
    if (stats) return stats

    if (allStats) {
      return Object.values(allStats).reduce(
        (total, stat) => ({
          total_files: total.total_files + stat.total_files,
          total_references: total.total_references + stat.total_references,
          unreferenced_count: total.unreferenced_count + stat.unreferenced_count,
          space_saved: total.space_saved + stat.space_saved,
        }),
        { total_files: 0, total_references: 0, unreferenced_count: 0, space_saved: 0 }
      )
    }

    return null
  }

  const totalStats = getTotalStats()

  if (isLoading) {
    return (
      <div className={clsx('bg-white border border-gray-200 rounded-lg p-6', className)}>
        <div className="animate-pulse">
          <div className="h-6 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="space-y-3">
            <div className="h-4 bg-gray-200 rounded"></div>
            <div className="h-4 bg-gray-200 rounded w-2/3"></div>
            <div className="h-4 bg-gray-200 rounded w-1/2"></div>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className={clsx('bg-white border border-red-200 rounded-lg p-6', className)}>
        <div className="flex items-center space-x-2 text-red-600">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span className="font-medium">Error loading stats</span>
        </div>
        <p className="text-red-600 text-sm mt-2">{error}</p>
        <button
          onClick={loadStats}
          className="mt-3 text-sm text-blue-600 hover:text-blue-800 transition-colors"
        >
          Try again
        </button>
      </div>
    )
  }

  if (!totalStats) {
    return (
      <div className={clsx('bg-white border border-gray-200 rounded-lg p-6', className)}>
        <p className="text-gray-500 text-center">No deduplication data available</p>
      </div>
    )
  }

  return (
    <div className={clsx('bg-white border border-gray-200 rounded-lg overflow-hidden', className)}>
      {/* Header */}
      <div className="bg-blue-50 px-6 py-4 border-b border-blue-200">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-blue-900">
            {workspacePath ? 'Workspace' : 'Global'} Deduplication Stats
          </h3>
          <button
            onClick={loadStats}
            className="text-blue-600 hover:text-blue-800 transition-colors"
            title="Refresh stats"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* Main Stats */}
      <div className="p-6">
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 mb-6">
          <StatCard
            title="Total Files"
            value={totalStats.total_files.toLocaleString()}
            icon="file"
            color="blue"
          />
          <StatCard
            title="References"
            value={totalStats.total_references.toLocaleString()}
            icon="link"
            color="green"
            subtitle={`${formatPercentage(totalStats.total_references - totalStats.total_files, totalStats.total_files)} duplicated`}
          />
          <StatCard
            title="Space Saved"
            value={DeduplicationService.formatFileSize(totalStats.space_saved)}
            icon="save"
            color="purple"
          />
          <StatCard
            title="Cleanup Needed"
            value={totalStats.unreferenced_count.toLocaleString()}
            icon="trash"
            color={totalStats.unreferenced_count > 0 ? 'amber' : 'gray'}
          />
        </div>

        {/* Deduplication Efficiency */}
        <div className="mb-6">
          <h4 className="text-sm font-medium text-gray-700 mb-3">Deduplication Efficiency</h4>
          <div className="bg-gray-100 rounded-lg p-4">
            <div className="flex justify-between items-center mb-2">
              <span className="text-sm text-gray-600">Storage Efficiency</span>
              <span className="text-sm font-medium">
                {formatPercentage(
                  totalStats.space_saved,
                  totalStats.space_saved + totalStats.total_files * 1024 * 1024
                )}
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-green-500 h-2 rounded-full transition-all duration-300"
                style={{
                  width: `${Math.min(100, (totalStats.space_saved / (totalStats.space_saved + totalStats.total_files * 1024 * 1024)) * 100)}%`,
                }}
              ></div>
            </div>
          </div>
        </div>

        {/* Garbage Collection */}
        {showGarbageCollection && workspacePath && (
          <div className="border-t border-gray-200 pt-6">
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-sm font-medium text-gray-700">Garbage Collection</h4>
              {shouldRunGC && (
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-amber-100 text-amber-800">
                  Cleanup Recommended
                </span>
              )}
            </div>

            {totalStats.unreferenced_count > 0 ? (
              <div className="bg-amber-50 border border-amber-200 rounded-lg p-4">
                <div className="flex items-start space-x-3">
                  <svg
                    className="w-5 h-5 text-amber-600 mt-0.5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 15.5c-.77.833.192 2.5 1.732 2.5z"
                    />
                  </svg>
                  <div className="flex-1">
                    <p className="text-sm text-amber-800 font-medium">
                      {totalStats.unreferenced_count} unreferenced file
                      {totalStats.unreferenced_count !== 1 ? 's' : ''} found
                    </p>
                    <p className="text-sm text-amber-700 mt-1">
                      These files can be safely removed to free up space.
                    </p>
                    <button
                      onClick={handleRunGarbageCollection}
                      disabled={isRunningGC}
                      className={clsx(
                        'mt-3 px-3 py-2 rounded text-sm font-medium transition-colors',
                        {
                          'bg-amber-600 text-white hover:bg-amber-700': !isRunningGC,
                          'bg-amber-300 text-amber-800 cursor-not-allowed': isRunningGC,
                        }
                      )}
                    >
                      {isRunningGC ? 'Running Cleanup...' : 'Run Garbage Collection'}
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                <div className="flex items-center space-x-2">
                  <svg
                    className="w-5 h-5 text-green-600"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                  <span className="text-sm text-green-800 font-medium">
                    No cleanup needed - all files are properly referenced
                  </span>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Recent GC Result */}
        {gcResult && (
          <div className="mt-4 bg-blue-50 border border-blue-200 rounded-lg p-4">
            <h5 className="font-medium text-blue-900 mb-2">Last Cleanup Result</h5>
            <div className="text-sm text-blue-800 space-y-1">
              <p>
                • Deleted {gcResult.deleted_files} file{gcResult.deleted_files !== 1 ? 's' : ''}
              </p>
              <p>• Freed {DeduplicationService.formatFileSize(gcResult.space_freed)} of space</p>
              <p>
                • Cleaned {gcResult.cleaned_entries} reference
                {gcResult.cleaned_entries !== 1 ? 's' : ''}
              </p>
              <p>• Completed in {(gcResult.duration / 1000).toFixed(2)} seconds</p>
              {gcResult.errors.length > 0 && (
                <p className="text-red-600">
                  • {gcResult.errors.length} error{gcResult.errors.length !== 1 ? 's' : ''} occurred
                </p>
              )}
            </div>
          </div>
        )}

        {/* Multiple Workspaces View */}
        {allStats && Object.keys(allStats).length > 1 && (
          <div className="border-t border-gray-200 pt-6 mt-6">
            <h4 className="text-sm font-medium text-gray-700 mb-3">Workspace Breakdown</h4>
            <div className="space-y-2">
              {Object.entries(allStats).map(([path, stat]) => (
                <div
                  key={path}
                  className="flex justify-between items-center p-3 bg-gray-50 rounded"
                >
                  <div>
                    <span className="text-sm font-medium text-gray-900 truncate">
                      {path.split('/').pop()}
                    </span>
                    <span className="text-xs text-gray-500 block truncate">{path}</span>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-medium">{stat.total_files} files</div>
                    <div className="text-xs text-gray-500">
                      {DeduplicationService.formatFileSize(stat.space_saved)} saved
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

interface StatCardProps {
  title: string
  value: string
  icon: 'file' | 'link' | 'save' | 'trash'
  color: 'blue' | 'green' | 'purple' | 'amber' | 'gray'
  subtitle?: string
}

const StatCard: React.FC<StatCardProps> = ({ title, value, icon, color, subtitle }) => {
  const IconComponent = () => {
    switch (icon) {
      case 'file':
        return (
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
        )
      case 'link':
        return (
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
            />
          </svg>
        )
      case 'save':
        return (
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"
            />
          </svg>
        )
      case 'trash':
        return (
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
            />
          </svg>
        )
    }
  }

  const colorClasses = {
    blue: 'bg-blue-100 text-blue-600',
    green: 'bg-green-100 text-green-600',
    purple: 'bg-purple-100 text-purple-600',
    amber: 'bg-amber-100 text-amber-600',
    gray: 'bg-gray-100 text-gray-600',
  }

  return (
    <div className="bg-gray-50 rounded-lg p-4">
      <div className="flex items-center space-x-3">
        <div className={clsx('p-2 rounded-lg', colorClasses[color])}>
          <IconComponent />
        </div>
        <div>
          <p className="text-sm text-gray-600">{title}</p>
          <p className="text-xl font-semibold text-gray-900">{value}</p>
          {subtitle && <p className="text-xs text-gray-500">{subtitle}</p>}
        </div>
      </div>
    </div>
  )
}

export default DeduplicationStats
