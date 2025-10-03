// Offline Indicator Component
import React, { useState } from 'react'
import { useOfflineStatus } from '../../hooks/useOfflineStatus'
import { RefreshCw, AlertTriangle, CheckCircle } from 'lucide-react'

export interface OfflineIndicatorProps {
  className?: string
  showDetails?: boolean
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'inline'
  showSyncProgress?: boolean
  syncProgress?: number
  isSyncing?: boolean
  onManualSync?: () => void
}

export const OfflineIndicator: React.FC<OfflineIndicatorProps> = ({
  className = '',
  showDetails = false,
  position = 'inline',
  showSyncProgress = false,
  syncProgress = 0,
  isSyncing = false,
  onManualSync,
}) => {
  const { status, isOnline, processQueue } = useOfflineStatus()
  const [showTooltip, setShowTooltip] = useState(false)

  const positionClasses: Record<string, string> = {
    'top-right': 'fixed top-4 right-4 z-50',
    'top-left': 'fixed top-4 left-4 z-50',
    'bottom-right': 'fixed bottom-4 right-4 z-50',
    'bottom-left': 'fixed bottom-4 left-4 z-50',
    inline: '',
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`
  }

  const handleSync = async (): Promise<void> => {
    if (isOnline && status.queuedOperationsCount > 0) {
      if (onManualSync) {
        onManualSync()
      } else {
        await processQueue()
      }
    }
  }

  return (
    <div className={`relative ${positionClasses[position]} ${className}`}>
      {/* Main Indicator */}
      <div
        className={`flex items-center gap-2 px-3 py-1.5 rounded-lg transition-all cursor-pointer ${
          isOnline
            ? 'bg-green-500/10 text-green-500 hover:bg-green-500/20'
            : 'bg-amber-500/10 text-amber-500 hover:bg-amber-500/20'
        }`}
        onMouseEnter={() => setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
        onClick={() => setShowTooltip(!showTooltip)}
      >
        {/* Status Dot */}
        <div className="relative">
          <div className={`w-2 h-2 rounded-full ${isOnline ? 'bg-green-500' : 'bg-amber-500'}`} />
          {!isOnline && (
            <div className="absolute inset-0 w-2 h-2 rounded-full bg-amber-500 animate-ping opacity-75" />
          )}
        </div>

        {/* Status Text */}
        <span className="text-xs font-medium">{isOnline ? 'Online' : 'Offline'}</span>

        {/* Queue Count Badge */}
        {status.queuedOperationsCount > 0 && (
          <div className="flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full bg-amber-500 text-white text-xs font-bold">
            {status.queuedOperationsCount}
          </div>
        )}
      </div>

      {/* Tooltip/Details Panel */}
      {showTooltip && (
        <div className="absolute top-full mt-2 right-0 w-80 bg-neutral-900 border border-neutral-800 rounded-lg shadow-xl overflow-hidden z-50">
          {/* Header */}
          <div className="px-4 py-3 bg-neutral-800/50 border-b border-neutral-700">
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <div
                className={`w-2 h-2 rounded-full ${isOnline ? 'bg-green-500' : 'bg-amber-500'}`}
              />
              {isOnline ? 'Connected' : 'Offline Mode'}
            </h3>
          </div>

          {/* Content */}
          <div className="p-4 space-y-4">
            {/* Offline Message */}
            {!isOnline && (
              <div className="text-xs text-neutral-400 bg-amber-500/10 border border-amber-500/20 rounded-lg p-3">
                <p className="font-medium text-amber-500 mb-1">Working Offline</p>
                <p>Your changes will be synced when connection is restored.</p>
              </div>
            )}

            {/* Sync Progress */}
            {showSyncProgress && isSyncing && (
              <div className="bg-cyan-500/10 border border-cyan-500/20 rounded-lg p-3 mb-4">
                <div className="flex items-center gap-2 mb-2">
                  <RefreshCw className="w-4 h-4 text-cyan-500 animate-spin" />
                  <span className="text-xs font-medium text-cyan-500">Syncing Changes...</span>
                </div>
                <div className="w-full h-2 bg-neutral-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-cyan-500 rounded-full transition-all duration-300"
                    style={{ width: `${syncProgress}%` }}
                  />
                </div>
                <p className="text-xs text-neutral-400 mt-1">
                  {Math.round(syncProgress)}% complete
                </p>
              </div>
            )}

            {/* Sync Success */}
            {showSyncProgress && !isSyncing && syncProgress === 100 && (
              <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3 mb-4">
                <div className="flex items-center gap-2">
                  <CheckCircle className="w-4 h-4 text-green-500" />
                  <span className="text-xs font-medium text-green-500">All Changes Synced!</span>
                </div>
              </div>
            )}

            {/* Statistics */}
            <div className="space-y-2">
              <StatItem
                label="Queued Operations"
                value={status.queuedOperationsCount}
                color={status.queuedOperationsCount > 0 ? 'text-amber-500' : 'text-neutral-400'}
                icon={
                  status.queuedOperationsCount > 0 ? (
                    <AlertTriangle className="w-3 h-3" />
                  ) : undefined
                }
              />
              <StatItem
                label="Cached Documents"
                value={status.cachedDocumentsCount}
                color="text-neutral-400"
              />
              <StatItem
                label="Cached Conversations"
                value={status.cachedConversationsCount}
                color="text-neutral-400"
              />
              <div className="flex justify-between items-center text-xs">
                <span className="text-neutral-500">Storage Used</span>
                <div className="flex items-center gap-2">
                  <div className="w-24 h-1.5 bg-neutral-800 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-cyan-500 rounded-full transition-all"
                      style={{
                        width: `${Math.min((status.storageUsed / status.storageLimit) * 100, 100)}%`,
                      }}
                    />
                  </div>
                  <span className="text-neutral-400 font-medium">
                    {formatBytes(status.storageUsed)}
                  </span>
                </div>
              </div>
            </div>

            {/* Actions */}
            {isOnline && status.queuedOperationsCount > 0 && (
              <button
                onClick={handleSync}
                className="w-full px-3 py-2 bg-cyan-500 hover:bg-cyan-600 text-white text-xs font-medium rounded-lg transition-colors"
              >
                Sync Queued Operations
              </button>
            )}

            {showDetails && (
              <div className="pt-3 border-t border-neutral-800">
                <p className="text-xs text-neutral-500">
                  Last check: {new Date(status.lastOnlineCheck).toLocaleTimeString()}
                </p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

// Helper component for stat items
const StatItem: React.FC<{
  label: string
  value: number | string
  color?: string
  icon?: React.ReactNode
}> = ({ label, value, color = 'text-neutral-400', icon }) => (
  <div className="flex justify-between items-center text-xs">
    <span className="text-neutral-500">{label}</span>
    <div className="flex items-center gap-1">
      {icon && <span className={color}>{icon}</span>}
      <span className={`font-medium ${color}`}>{value}</span>
    </div>
  </div>
)

export default OfflineIndicator
