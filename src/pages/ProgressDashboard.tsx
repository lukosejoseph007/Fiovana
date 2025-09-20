// src/pages/ProgressDashboard.tsx
// Comprehensive progress tracking dashboard

import React, { useState, useEffect } from 'react'
import { AlertCircle, Download, History } from 'lucide-react'
import { useProgress } from '../hooks/useProgress'
import { progressService } from '../services/progressService'
import ProgressSummary from '../components/progress/ProgressSummary'
import ProgressList from '../components/progress/ProgressList'
import type { ImportProgress } from '../types/progress'

const ProgressDashboard: React.FC = () => {
  const { operations, summary, loading, error, refresh, cancelOperation, cleanupCompleted } =
    useProgress()

  const [estimatedTime, setEstimatedTime] = useState<number | null>(null)
  const [history, setHistory] = useState<ImportProgress[]>([])
  const [showHistory, setShowHistory] = useState(false)
  const [historyLoading, setHistoryLoading] = useState(false)

  // Load estimated completion time
  useEffect(() => {
    const loadEstimatedTime = async () => {
      try {
        const eta = await progressService.getEstimatedCompletionTime()
        setEstimatedTime(eta)
      } catch (error) {
        console.error('Failed to get estimated completion time:', error)
      }
    }

    if (summary.active_operations > 0) {
      loadEstimatedTime()
      // Update every 30 seconds while operations are active
      const interval = setInterval(loadEstimatedTime, 30000)
      return () => clearInterval(interval)
    } else {
      setEstimatedTime(null)
      return undefined
    }
  }, [summary.active_operations])

  const handleRetryOperation = async (operationId: string) => {
    // Note: This would need to be implemented based on your retry logic
    console.log('Retry operation:', operationId)
    // For now, just refresh the data
    await refresh()
  }

  const handleLoadHistory = async () => {
    setHistoryLoading(true)
    try {
      const historyData = await progressService.getOperationHistory(100)
      setHistory(historyData)
      setShowHistory(true)
    } catch (error) {
      console.error('Failed to load history:', error)
    } finally {
      setHistoryLoading(false)
    }
  }

  const handleExportData = () => {
    // Export current operations and summary as JSON
    const exportData = {
      timestamp: new Date().toISOString(),
      summary,
      operations,
      history,
    }

    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: 'application/json',
    })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `progress-export-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
              Progress Dashboard
            </h1>
            <p className="text-gray-600 dark:text-gray-400">
              Monitor and manage import operations in real-time
            </p>
          </div>

          <div className="flex items-center space-x-3">
            <button
              onClick={handleLoadHistory}
              disabled={historyLoading}
              className="px-4 py-2 bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center space-x-2 disabled:opacity-50"
            >
              <History className={`h-4 w-4 ${historyLoading ? 'animate-spin' : ''}`} />
              <span>History</span>
            </button>

            <button
              onClick={handleExportData}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors flex items-center space-x-2"
            >
              <Download className="h-4 w-4" />
              <span>Export</span>
            </button>
          </div>
        </div>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <AlertCircle className="h-5 w-5 text-red-500" />
            <div>
              <h3 className="text-sm font-medium text-red-800 dark:text-red-200">
                Error Loading Progress Data
              </h3>
              <p className="text-sm text-red-700 dark:text-red-300 mt-1">{error}</p>
            </div>
          </div>
        </div>
      )}

      {/* Progress Summary */}
      <ProgressSummary
        summary={summary}
        onRefresh={refresh}
        onCleanup={cleanupCompleted}
        loading={loading}
        estimatedTime={estimatedTime}
      />

      {/* Active and Recent Operations */}
      <ProgressList
        operations={operations}
        onCancel={cancelOperation}
        onRetry={handleRetryOperation}
        onRefresh={refresh}
        onCleanup={cleanupCompleted}
        showCompleted={true}
        maxItems={20}
        loading={loading}
      />

      {/* History Modal/Panel */}
      {showHistory && (
        <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[80vh] overflow-hidden">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Operation History
                </h2>
                <button
                  onClick={() => setShowHistory(false)}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                >
                  <span className="sr-only">Close</span>✕
                </button>
              </div>
            </div>

            <div className="p-4 overflow-y-auto max-h-[calc(80vh-120px)]">
              {history.length === 0 ? (
                <div className="text-center py-8">
                  <History className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                  <p className="text-gray-500 dark:text-gray-400">No operation history available</p>
                </div>
              ) : (
                <ProgressList
                  operations={history}
                  onCancel={cancelOperation}
                  onRetry={handleRetryOperation}
                  showCompleted={true}
                  maxItems={100}
                />
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default ProgressDashboard
