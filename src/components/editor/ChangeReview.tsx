import React, { useState, useCallback, useMemo } from 'react'
import { FileCheck, AlertCircle, CheckCircle2, XCircle, Eye } from 'lucide-react'
import Button from '../ui/Button'
import type { Change } from './TrackChanges'

export interface ChangeReviewProps {
  changes: Change[]
  documentTitle?: string
  onAcceptChange: (changeId: string) => void
  onRejectChange: (changeId: string) => void
  onAcceptAll: () => void
  onRejectAll: () => void
  onClose?: () => void
  currentUser?: string
}

export const ChangeReview: React.FC<ChangeReviewProps> = ({
  changes,
  documentTitle = 'Untitled Document',
  onAcceptChange,
  onRejectChange,
  onAcceptAll,
  onRejectAll,
  onClose,
  currentUser = 'You',
}) => {
  const [currentIndex, setCurrentIndex] = useState(0)
  const [viewMode, setViewMode] = useState<'pending' | 'accepted' | 'rejected' | 'all'>('pending')

  // Filter changes based on view mode
  const filteredChanges = useMemo(() => {
    switch (viewMode) {
      case 'pending':
        return changes.filter(c => !c.accepted && !c.rejected)
      case 'accepted':
        return changes.filter(c => c.accepted)
      case 'rejected':
        return changes.filter(c => c.rejected)
      case 'all':
      default:
        return changes
    }
  }, [changes, viewMode])

  const currentChange = filteredChanges[currentIndex]

  const stats = useMemo(() => {
    const pending = changes.filter(c => !c.accepted && !c.rejected)
    const accepted = changes.filter(c => c.accepted)
    const rejected = changes.filter(c => c.rejected)

    return {
      total: changes.length,
      pending: pending.length,
      accepted: accepted.length,
      rejected: rejected.length,
      percentComplete:
        changes.length > 0
          ? Math.round(((accepted.length + rejected.length) / changes.length) * 100)
          : 0,
    }
  }, [changes])

  const handleNext = useCallback(() => {
    if (currentIndex < filteredChanges.length - 1) {
      setCurrentIndex(currentIndex + 1)
    }
  }, [currentIndex, filteredChanges.length])

  const handlePrevious = useCallback(() => {
    if (currentIndex > 0) {
      setCurrentIndex(currentIndex - 1)
    }
  }, [currentIndex])

  const handleAccept = useCallback(() => {
    if (currentChange) {
      onAcceptChange(currentChange.id)
      if (currentIndex < filteredChanges.length - 1) {
        handleNext()
      }
    }
  }, [currentChange, onAcceptChange, currentIndex, filteredChanges.length, handleNext])

  const handleReject = useCallback(() => {
    if (currentChange) {
      onRejectChange(currentChange.id)
      if (currentIndex < filteredChanges.length - 1) {
        handleNext()
      }
    }
  }, [currentChange, onRejectChange, currentIndex, filteredChanges.length, handleNext])

  const getChangeTypeLabel = (type: Change['type']) => {
    switch (type) {
      case 'insertion':
        return 'Addition'
      case 'deletion':
        return 'Deletion'
      case 'formatting':
        return 'Formatting'
      default:
        return 'Change'
    }
  }

  const getChangeTypeColor = (type: Change['type']) => {
    switch (type) {
      case 'insertion':
        return 'text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30'
      case 'deletion':
        return 'text-red-600 dark:text-red-400 bg-red-100 dark:bg-red-900/30'
      case 'formatting':
        return 'text-blue-600 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/30'
      default:
        return 'text-gray-600 dark:text-gray-400 bg-gray-100 dark:bg-gray-800'
    }
  }

  const renderChangePreview = (change: Change) => {
    switch (change.type) {
      case 'insertion':
        return (
          <div className="p-4 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 rounded">
            <div className="text-sm text-gray-500 dark:text-gray-400 mb-2">Added text:</div>
            <div className="text-base text-green-700 dark:text-green-300 bg-green-100 dark:bg-green-900/40 px-2 py-1 rounded">
              {change.content}
            </div>
          </div>
        )
      case 'deletion':
        return (
          <div className="p-4 bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 rounded">
            <div className="text-sm text-gray-500 dark:text-gray-400 mb-2">Deleted text:</div>
            <div className="text-base text-red-700 dark:text-red-300 line-through bg-red-100 dark:bg-red-900/40 px-2 py-1 rounded">
              {change.content}
            </div>
          </div>
        )
      case 'formatting':
        return (
          <div className="p-4 bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 rounded">
            <div className="text-sm text-gray-500 dark:text-gray-400 mb-2">
              Formatting: {change.metadata?.formattingType || 'Style change'}
            </div>
            {change.metadata?.before && (
              <div className="mb-2">
                <div className="text-xs text-gray-500 mb-1">Before:</div>
                <div className="text-base text-red-700 dark:text-red-300 line-through bg-red-100 dark:bg-red-900/40 px-2 py-1 rounded">
                  {change.metadata.before}
                </div>
              </div>
            )}
            {change.metadata?.after && (
              <div>
                <div className="text-xs text-gray-500 mb-1">After:</div>
                <div className="text-base text-green-700 dark:text-green-300 bg-green-100 dark:bg-green-900/40 px-2 py-1 rounded">
                  {change.metadata.after}
                </div>
              </div>
            )}
          </div>
        )
      default:
        return null
    }
  }

  if (filteredChanges.length === 0) {
    return (
      <div className="change-review-panel bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg max-w-2xl mx-auto">
        <div className="p-8 text-center">
          {viewMode === 'pending' ? (
            <>
              <CheckCircle2 className="w-16 h-16 text-green-500 mx-auto mb-4" />
              <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                All Changes Reviewed!
              </h3>
              <p className="text-gray-600 dark:text-gray-400 mb-4">
                You've reviewed all {stats.total} changes in this document.
              </p>
              <div className="flex gap-4 justify-center text-sm">
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="w-4 h-4 text-green-500" />
                  <span>{stats.accepted} Accepted</span>
                </div>
                <div className="flex items-center gap-2">
                  <XCircle className="w-4 h-4 text-red-500" />
                  <span>{stats.rejected} Rejected</span>
                </div>
              </div>
            </>
          ) : (
            <>
              <Eye className="w-16 h-16 text-gray-400 mx-auto mb-4" />
              <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                No {viewMode} changes
              </h3>
              <p className="text-gray-600 dark:text-gray-400">
                There are no {viewMode} changes to display.
              </p>
            </>
          )}
          {onClose && (
            <Button onClick={onClose} variant="primary" className="mt-4">
              Close Review
            </Button>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="change-review-panel bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg max-w-4xl mx-auto">
      {/* Header */}
      <div className="p-6 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <FileCheck className="w-6 h-6 text-blue-500" />
            <div>
              <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                Review Changes
              </h2>
              <p className="text-sm text-gray-500">{documentTitle}</p>
            </div>
          </div>
          {onClose && (
            <Button onClick={onClose} variant="ghost" size="sm">
              Close
            </Button>
          )}
        </div>

        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between text-xs text-gray-600 dark:text-gray-400 mb-1">
            <span>Review Progress</span>
            <span>{stats.percentComplete}% Complete</span>
          </div>
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              className="bg-blue-500 h-2 rounded-full transition-all duration-300"
              style={{ width: `${stats.percentComplete}%` }}
            />
          </div>
        </div>

        {/* Statistics */}
        <div className="grid grid-cols-4 gap-3 mb-4">
          <div className="text-center p-2 bg-gray-50 dark:bg-gray-800 rounded">
            <div className="text-lg font-semibold text-gray-900 dark:text-white">{stats.total}</div>
            <div className="text-xs text-gray-500">Total</div>
          </div>
          <div className="text-center p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded">
            <div className="text-lg font-semibold text-yellow-700 dark:text-yellow-400">
              {stats.pending}
            </div>
            <div className="text-xs text-gray-500">Pending</div>
          </div>
          <div className="text-center p-2 bg-green-50 dark:bg-green-900/20 rounded">
            <div className="text-lg font-semibold text-green-700 dark:text-green-400">
              {stats.accepted}
            </div>
            <div className="text-xs text-gray-500">Accepted</div>
          </div>
          <div className="text-center p-2 bg-red-50 dark:bg-red-900/20 rounded">
            <div className="text-lg font-semibold text-red-700 dark:text-red-400">
              {stats.rejected}
            </div>
            <div className="text-xs text-gray-500">Rejected</div>
          </div>
        </div>

        {/* View Mode Selector */}
        <div className="flex gap-2">
          <Button
            onClick={() => {
              setViewMode('pending')
              setCurrentIndex(0)
            }}
            variant={viewMode === 'pending' ? 'primary' : 'secondary'}
            size="sm"
          >
            Pending ({stats.pending})
          </Button>
          <Button
            onClick={() => {
              setViewMode('accepted')
              setCurrentIndex(0)
            }}
            variant={viewMode === 'accepted' ? 'primary' : 'secondary'}
            size="sm"
          >
            Accepted ({stats.accepted})
          </Button>
          <Button
            onClick={() => {
              setViewMode('rejected')
              setCurrentIndex(0)
            }}
            variant={viewMode === 'rejected' ? 'primary' : 'secondary'}
            size="sm"
          >
            Rejected ({stats.rejected})
          </Button>
          <Button
            onClick={() => {
              setViewMode('all')
              setCurrentIndex(0)
            }}
            variant={viewMode === 'all' ? 'primary' : 'secondary'}
            size="sm"
          >
            All ({stats.total})
          </Button>
        </div>
      </div>

      {/* Current Change */}
      {currentChange && (
        <div className="p-6">
          {/* Change Info */}
          <div className="flex items-center justify-between mb-4">
            <div>
              <div className="flex items-center gap-2 mb-1">
                <span
                  className={`px-2 py-1 text-xs font-medium rounded ${getChangeTypeColor(currentChange.type)}`}
                >
                  {getChangeTypeLabel(currentChange.type)}
                </span>
                {currentChange.accepted && (
                  <span className="px-2 py-1 text-xs font-medium text-green-700 bg-green-100 dark:bg-green-900/30 rounded flex items-center gap-1">
                    <CheckCircle2 className="w-3 h-3" />
                    Accepted
                  </span>
                )}
                {currentChange.rejected && (
                  <span className="px-2 py-1 text-xs font-medium text-red-700 bg-red-100 dark:bg-red-900/30 rounded flex items-center gap-1">
                    <XCircle className="w-3 h-3" />
                    Rejected
                  </span>
                )}
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">
                By {currentChange.author === currentUser ? 'You' : currentChange.author} •{' '}
                {currentChange.timestamp.toLocaleString()}
              </div>
            </div>
            <div className="text-sm text-gray-500">
              Change {currentIndex + 1} of {filteredChanges.length}
            </div>
          </div>

          {/* Change Preview */}
          <div className="mb-6">{renderChangePreview(currentChange)}</div>

          {/* Navigation and Actions */}
          <div className="flex items-center justify-between">
            <div className="flex gap-2">
              <Button
                onClick={handlePrevious}
                disabled={currentIndex === 0}
                variant="secondary"
                size="sm"
              >
                ← Previous
              </Button>
              <Button
                onClick={handleNext}
                disabled={currentIndex === filteredChanges.length - 1}
                variant="secondary"
                size="sm"
              >
                Next →
              </Button>
            </div>

            {!currentChange.accepted && !currentChange.rejected && (
              <div className="flex gap-2">
                <Button onClick={handleReject} variant="danger" size="md">
                  <XCircle className="w-4 h-4 mr-1" />
                  Reject
                </Button>
                <Button onClick={handleAccept} variant="success" size="md">
                  <CheckCircle2 className="w-4 h-4 mr-1" />
                  Accept
                </Button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Bulk Actions Footer */}
      {viewMode === 'pending' && stats.pending > 0 && (
        <div className="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <AlertCircle className="w-4 h-4" />
              <span>{stats.pending} pending changes</span>
            </div>
            <div className="flex gap-2">
              <Button onClick={onRejectAll} variant="danger" size="sm">
                <XCircle className="w-4 h-4 mr-1" />
                Reject All
              </Button>
              <Button onClick={onAcceptAll} variant="success" size="sm">
                <CheckCircle2 className="w-4 h-4 mr-1" />
                Accept All
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
