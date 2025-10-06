import React from 'react'
import { ProviderStatus } from '../../collaboration/YjsProvider'

export interface ConnectionStatusProps {
  status: ProviderStatus
  onRetry?: () => void
}

const ConnectionStatus: React.FC<ConnectionStatusProps> = ({ status, onRetry }) => {
  const { connectionState, connected, synced, error, reconnectAttempts } = status

  // Don't show anything if connected and synced
  if (connectionState === 'connected' && connected && synced) {
    return null
  }

  const getStatusColor = () => {
    switch (connectionState) {
      case 'connected':
        return 'bg-green-100 border-green-400 text-green-700'
      case 'connecting':
      case 'reconnecting':
        return 'bg-yellow-100 border-yellow-400 text-yellow-700'
      case 'disconnected':
        return 'bg-red-100 border-red-400 text-red-700'
      default:
        return 'bg-gray-100 border-gray-400 text-gray-700'
    }
  }

  const getStatusIcon = () => {
    switch (connectionState) {
      case 'connected':
        return (
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clipRule="evenodd"
            />
          </svg>
        )
      case 'connecting':
      case 'reconnecting':
        return (
          <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        )
      case 'disconnected':
        return (
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
              clipRule="evenodd"
            />
          </svg>
        )
      default:
        return null
    }
  }

  const getStatusMessage = () => {
    switch (connectionState) {
      case 'connected':
        if (!synced) return 'Connected - Syncing...'
        return 'Connected'
      case 'connecting':
        return 'Connecting to collaboration server...'
      case 'reconnecting':
        return `Reconnecting... (attempt ${reconnectAttempts})`
      case 'disconnected':
        return 'Disconnected from collaboration server'
      default:
        return 'Unknown connection status'
    }
  }

  return (
    <div
      className={`fixed top-4 right-4 z-50 max-w-md rounded-lg border-2 p-4 shadow-lg ${getStatusColor()}`}
    >
      <div className="flex items-start gap-3">
        <div className="flex-shrink-0 mt-0.5">{getStatusIcon()}</div>

        <div className="flex-1">
          <div className="font-semibold">{getStatusMessage()}</div>

          {error && <div className="mt-1 text-sm opacity-90">Error: {error.message}</div>}

          {connectionState === 'disconnected' && onRetry && (
            <button
              onClick={onRetry}
              className="mt-2 rounded bg-white px-3 py-1 text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2"
            >
              Retry Connection
            </button>
          )}

          {connectionState === 'reconnecting' && (
            <div className="mt-2 text-sm">
              <div className="mb-1">Reconnecting with exponential backoff...</div>
              <div className="w-full bg-white bg-opacity-30 rounded-full h-2">
                <div
                  className="h-2 rounded-full bg-current transition-all duration-300"
                  style={{ width: `${(reconnectAttempts / 10) * 100}%` }}
                />
              </div>
            </div>
          )}
        </div>

        {connectionState !== 'reconnecting' && connectionState !== 'connecting' && (
          <button
            onClick={() => {
              /* Close notification */
            }}
            className="flex-shrink-0 ml-2 -mt-1 -mr-1"
          >
            <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
        )}
      </div>
    </div>
  )
}

export default ConnectionStatus
