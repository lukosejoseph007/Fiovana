import React, { useState, useCallback, useMemo } from 'react'
import { MessageSquare, Check, X, Eye, EyeOff, Filter, ChevronDown, ChevronUp } from 'lucide-react'
import Button from '../ui/Button'

export interface Change {
  id: string
  type: 'insertion' | 'deletion' | 'formatting'
  content: string
  originalContent?: string // For deletions and formatting changes
  author: string
  timestamp: Date
  position: {
    start: number
    end: number
  }
  accepted?: boolean
  rejected?: boolean
  metadata?: {
    formattingType?: string // e.g., 'bold', 'italic', 'heading'
    before?: string
    after?: string
  }
}

export interface TrackChangesProps {
  changes: Change[]
  onAcceptChange: (changeId: string) => void
  onRejectChange: (changeId: string) => void
  onAcceptAll: () => void
  onRejectAll: () => void
  isTrackingEnabled: boolean
  onToggleTracking: () => void
  showChanges: boolean
  onToggleShowChanges: () => void
  currentUser?: string
}

export const TrackChanges: React.FC<TrackChangesProps> = ({
  changes,
  onAcceptChange,
  onRejectChange,
  onAcceptAll,
  onRejectAll,
  isTrackingEnabled,
  onToggleTracking,
  showChanges,
  onToggleShowChanges,
  currentUser = 'You',
}) => {
  const [filterType, setFilterType] = useState<'all' | 'insertion' | 'deletion' | 'formatting'>(
    'all'
  )
  const [filterAuthor, setFilterAuthor] = useState<string>('all')
  const [isExpanded, setIsExpanded] = useState(true)

  // Get unique authors
  const authors = useMemo(() => {
    const uniqueAuthors = new Set(changes.map(c => c.author))
    return Array.from(uniqueAuthors)
  }, [changes])

  // Filter changes
  const filteredChanges = useMemo(() => {
    return changes.filter(change => {
      // Skip accepted/rejected changes
      if (change.accepted || change.rejected) {
        return false
      }

      // Filter by type
      if (filterType !== 'all' && change.type !== filterType) {
        return false
      }

      // Filter by author
      if (filterAuthor !== 'all' && change.author !== filterAuthor) {
        return false
      }

      return true
    })
  }, [changes, filterType, filterAuthor])

  // Statistics
  const stats = useMemo(() => {
    const pending = changes.filter(c => !c.accepted && !c.rejected)
    const accepted = changes.filter(c => c.accepted)
    const rejected = changes.filter(c => c.rejected)

    return {
      total: changes.length,
      pending: pending.length,
      accepted: accepted.length,
      rejected: rejected.length,
      insertions: pending.filter(c => c.type === 'insertion').length,
      deletions: pending.filter(c => c.type === 'deletion').length,
      formatting: pending.filter(c => c.type === 'formatting').length,
    }
  }, [changes])

  const formatTimestamp = useCallback((date: Date) => {
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)
    const diffDays = Math.floor(diffMs / 86400000)

    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    if (diffDays < 7) return `${diffDays}d ago`

    return date.toLocaleDateString()
  }, [])

  const getChangeIcon = useCallback((type: Change['type']) => {
    switch (type) {
      case 'insertion':
        return <span className="text-green-500 font-bold">+</span>
      case 'deletion':
        return <span className="text-red-500 font-bold">-</span>
      case 'formatting':
        return <span className="text-blue-500 font-bold">F</span>
      default:
        return null
    }
  }, [])

  const getChangeColor = useCallback((type: Change['type']) => {
    switch (type) {
      case 'insertion':
        return 'bg-green-100 dark:bg-green-900/30 border-green-300 dark:border-green-700'
      case 'deletion':
        return 'bg-red-100 dark:bg-red-900/30 border-red-300 dark:border-red-700'
      case 'formatting':
        return 'bg-blue-100 dark:bg-blue-900/30 border-blue-300 dark:border-blue-700'
      default:
        return 'bg-gray-100 dark:bg-gray-800 border-gray-300 dark:border-gray-700'
    }
  }, [])

  return (
    <div className="track-changes-panel bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-700 w-96 flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <MessageSquare className="w-5 h-5 text-gray-700 dark:text-gray-300" />
            <h3 className="font-semibold text-gray-900 dark:text-white">Track Changes</h3>
          </div>
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
            aria-label={isExpanded ? 'Collapse' : 'Expand'}
          >
            {isExpanded ? (
              <ChevronUp className="w-4 h-4 text-gray-500" />
            ) : (
              <ChevronDown className="w-4 h-4 text-gray-500" />
            )}
          </button>
        </div>

        {isExpanded && (
          <>
            {/* Controls */}
            <div className="flex gap-2 mb-3">
              <Button
                onClick={onToggleTracking}
                variant={isTrackingEnabled ? 'primary' : 'secondary'}
                size="sm"
                className="flex-1"
              >
                {isTrackingEnabled ? 'Tracking On' : 'Tracking Off'}
              </Button>
              <Button
                onClick={onToggleShowChanges}
                variant={showChanges ? 'primary' : 'secondary'}
                size="sm"
                className="flex-1"
              >
                {showChanges ? <Eye className="w-4 h-4" /> : <EyeOff className="w-4 h-4" />}
                <span className="ml-1">{showChanges ? 'Hide' : 'Show'}</span>
              </Button>
            </div>

            {/* Statistics */}
            <div className="grid grid-cols-4 gap-2 text-xs text-center mb-3">
              <div className="bg-gray-50 dark:bg-gray-800 p-2 rounded">
                <div className="font-semibold text-gray-900 dark:text-white">{stats.pending}</div>
                <div className="text-gray-500">Pending</div>
              </div>
              <div className="bg-green-50 dark:bg-green-900/20 p-2 rounded">
                <div className="font-semibold text-green-700 dark:text-green-400">
                  {stats.insertions}
                </div>
                <div className="text-gray-500">Added</div>
              </div>
              <div className="bg-red-50 dark:bg-red-900/20 p-2 rounded">
                <div className="font-semibold text-red-700 dark:text-red-400">
                  {stats.deletions}
                </div>
                <div className="text-gray-500">Deleted</div>
              </div>
              <div className="bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
                <div className="font-semibold text-blue-700 dark:text-blue-400">
                  {stats.formatting}
                </div>
                <div className="text-gray-500">Format</div>
              </div>
            </div>

            {/* Filters */}
            <div className="flex gap-2 mb-3">
              <div className="flex-1">
                <label className="text-xs text-gray-500 mb-1 block">Type</label>
                <select
                  value={filterType}
                  onChange={e => setFilterType(e.target.value as typeof filterType)}
                  className="w-full text-sm px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="all">All Types</option>
                  <option value="insertion">Insertions</option>
                  <option value="deletion">Deletions</option>
                  <option value="formatting">Formatting</option>
                </select>
              </div>
              <div className="flex-1">
                <label className="text-xs text-gray-500 mb-1 block">Author</label>
                <select
                  value={filterAuthor}
                  onChange={e => setFilterAuthor(e.target.value)}
                  className="w-full text-sm px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="all">All Authors</option>
                  {authors.map(author => (
                    <option key={author} value={author}>
                      {author}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Bulk Actions */}
            {stats.pending > 0 && (
              <div className="flex gap-2">
                <Button onClick={onAcceptAll} variant="success" size="sm" className="flex-1">
                  <Check className="w-3 h-3 mr-1" />
                  Accept All
                </Button>
                <Button onClick={onRejectAll} variant="danger" size="sm" className="flex-1">
                  <X className="w-3 h-3 mr-1" />
                  Reject All
                </Button>
              </div>
            )}
          </>
        )}
      </div>

      {/* Changes List */}
      <div className="flex-1 overflow-y-auto">
        {filteredChanges.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <Filter className="w-12 h-12 mx-auto mb-3 opacity-30" />
            <p className="text-sm">No pending changes</p>
            {stats.total > 0 && (
              <p className="text-xs mt-1">
                {stats.accepted} accepted, {stats.rejected} rejected
              </p>
            )}
          </div>
        ) : (
          <div className="space-y-2 p-3">
            {filteredChanges.map(change => (
              <ChangeCard
                key={change.id}
                change={change}
                onAccept={() => onAcceptChange(change.id)}
                onReject={() => onRejectChange(change.id)}
                getChangeIcon={getChangeIcon}
                getChangeColor={getChangeColor}
                formatTimestamp={formatTimestamp}
                currentUser={currentUser}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

interface ChangeCardProps {
  change: Change
  onAccept: () => void
  onReject: () => void
  getChangeIcon: (type: Change['type']) => React.ReactNode
  getChangeColor: (type: Change['type']) => string
  formatTimestamp: (date: Date) => string
  currentUser: string
}

const ChangeCard: React.FC<ChangeCardProps> = ({
  change,
  onAccept,
  onReject,
  getChangeIcon,
  getChangeColor,
  formatTimestamp,
  currentUser,
}) => {
  const [isExpanded, setIsExpanded] = useState(false)

  const renderChangeContent = () => {
    switch (change.type) {
      case 'insertion':
        return (
          <div className="text-sm">
            <span className="text-green-700 dark:text-green-400 bg-green-100 dark:bg-green-900/30 px-1 rounded">
              {change.content}
            </span>
          </div>
        )
      case 'deletion':
        return (
          <div className="text-sm">
            <span className="text-red-700 dark:text-red-400 line-through bg-red-100 dark:bg-red-900/30 px-1 rounded">
              {change.content}
            </span>
          </div>
        )
      case 'formatting':
        return (
          <div className="text-sm space-y-1">
            <div className="text-gray-500 text-xs">
              {change.metadata?.formattingType || 'Formatting change'}
            </div>
            {change.metadata?.before && (
              <div className="text-red-600 dark:text-red-400 line-through">
                {change.metadata.before}
              </div>
            )}
            {change.metadata?.after && (
              <div className="text-green-600 dark:text-green-400">{change.metadata.after}</div>
            )}
          </div>
        )
      default:
        return null
    }
  }

  return (
    <div className={`border rounded-lg p-3 ${getChangeColor(change.type)} transition-all`}>
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2 flex-1">
          <div className="flex-shrink-0">{getChangeIcon(change.type)}</div>
          <div className="flex-1 min-w-0">
            <div className="text-xs font-medium text-gray-700 dark:text-gray-300">
              {change.author === currentUser ? 'You' : change.author}
            </div>
            <div className="text-xs text-gray-500">{formatTimestamp(change.timestamp)}</div>
          </div>
        </div>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="p-1 hover:bg-white/50 dark:hover:bg-black/20 rounded transition-colors"
          aria-label={isExpanded ? 'Collapse' : 'Expand'}
        >
          {isExpanded ? <ChevronUp className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
        </button>
      </div>

      {isExpanded && (
        <>
          <div className="mb-3 max-w-full overflow-hidden">{renderChangeContent()}</div>

          {/* Actions */}
          <div className="flex gap-2">
            <button
              onClick={onAccept}
              className="flex-1 px-3 py-1.5 bg-green-600 hover:bg-green-700 text-white text-xs font-medium rounded transition-colors flex items-center justify-center gap-1"
            >
              <Check className="w-3 h-3" />
              Accept
            </button>
            <button
              onClick={onReject}
              className="flex-1 px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-xs font-medium rounded transition-colors flex items-center justify-center gap-1"
            >
              <X className="w-3 h-3" />
              Reject
            </button>
          </div>
        </>
      )}

      {!isExpanded && (
        <div className="text-xs text-gray-600 dark:text-gray-400 truncate">
          {change.content.substring(0, 50)}
          {change.content.length > 50 ? '...' : ''}
        </div>
      )}
    </div>
  )
}
