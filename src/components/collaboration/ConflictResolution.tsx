import React, { useState, useCallback, useMemo } from 'react'
import { AlertTriangle, CheckCircle, XCircle, Info, RefreshCw } from 'lucide-react'

export interface ConflictChange {
  id: string
  type: 'insert' | 'delete' | 'replace'
  position: number
  content: string
  userId: string
  userName: string
  userColor: string
  timestamp: number
}

export interface Conflict {
  id: string
  position: number
  localChange: ConflictChange
  remoteChanges: ConflictChange[]
  status: 'pending' | 'resolved' | 'accepted' | 'rejected'
  resolvedAt?: number
  resolvedBy?: string
}

export interface ConflictResolutionProps {
  conflicts: Conflict[]
  onResolveConflict: (conflictId: string, resolution: 'accept-local' | 'accept-remote' | 'merge') => void
  onDismissConflict: (conflictId: string) => void
  onRefreshConflicts: () => void
  className?: string
}

export const ConflictResolution: React.FC<ConflictResolutionProps> = ({
  conflicts,
  onResolveConflict,
  onDismissConflict,
  onRefreshConflicts,
  className = '',
}) => {
  const [expandedConflicts, setExpandedConflicts] = useState<Set<string>>(new Set())

  const pendingConflicts = useMemo(
    () => conflicts.filter(c => c.status === 'pending'),
    [conflicts]
  )

  const resolvedConflicts = useMemo(
    () => conflicts.filter(c => c.status !== 'pending'),
    [conflicts]
  )

  const toggleExpanded = useCallback((conflictId: string) => {
    setExpandedConflicts(prev => {
      const next = new Set(prev)
      if (next.has(conflictId)) {
        next.delete(conflictId)
      } else {
        next.add(conflictId)
      }
      return next
    })
  }, [])

  const handleResolve = useCallback(
    (conflictId: string, resolution: 'accept-local' | 'accept-remote' | 'merge') => {
      onResolveConflict(conflictId, resolution)
      setExpandedConflicts(prev => {
        const next = new Set(prev)
        next.delete(conflictId)
        return next
      })
    },
    [onResolveConflict]
  )

  if (conflicts.length === 0) {
    return null
  }

  return (
    <div className={`conflict-resolution ${className}`}>
      {/* Header */}
      <div className="conflict-resolution-header">
        <div className="conflict-resolution-title">
          <AlertTriangle className="conflict-icon" size={20} />
          <span>
            {pendingConflicts.length > 0
              ? `${pendingConflicts.length} Conflict${pendingConflicts.length > 1 ? 's' : ''} Detected`
              : 'All Conflicts Resolved'}
          </span>
        </div>
        <button
          className="conflict-refresh-button"
          onClick={onRefreshConflicts}
          title="Refresh conflicts"
        >
          <RefreshCw size={16} />
        </button>
      </div>

      {/* Pending Conflicts */}
      {pendingConflicts.length > 0 && (
        <div className="conflict-section">
          <div className="conflict-section-title">Pending Resolution</div>
          {pendingConflicts.map(conflict => (
            <ConflictCard
              key={conflict.id}
              conflict={conflict}
              isExpanded={expandedConflicts.has(conflict.id)}
              onToggleExpanded={() => toggleExpanded(conflict.id)}
              onResolve={handleResolve}
              onDismiss={onDismissConflict}
            />
          ))}
        </div>
      )}

      {/* Resolved Conflicts */}
      {resolvedConflicts.length > 0 && (
        <div className="conflict-section">
          <div className="conflict-section-title">Recently Resolved</div>
          {resolvedConflicts.slice(0, 5).map(conflict => (
            <ResolvedConflictCard
              key={conflict.id}
              conflict={conflict}
              onDismiss={onDismissConflict}
            />
          ))}
        </div>
      )}

      <style>{`
        .conflict-resolution {
          background: #fff;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 16px;
          margin: 16px 0;
        }

        .conflict-resolution-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 16px;
        }

        .conflict-resolution-title {
          display: flex;
          align-items: center;
          gap: 8px;
          font-weight: 600;
          font-size: 16px;
          color: #111827;
        }

        .conflict-icon {
          color: #f59e0b;
        }

        .conflict-refresh-button {
          background: transparent;
          border: 1px solid #e5e7eb;
          border-radius: 4px;
          padding: 6px;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
        }

        .conflict-refresh-button:hover {
          background: #f9fafb;
          border-color: #d1d5db;
          color: #111827;
        }

        .conflict-section {
          margin-bottom: 16px;
        }

        .conflict-section:last-child {
          margin-bottom: 0;
        }

        .conflict-section-title {
          font-size: 14px;
          font-weight: 600;
          color: #6b7280;
          margin-bottom: 12px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
      `}</style>
    </div>
  )
}

interface ConflictCardProps {
  conflict: Conflict
  isExpanded: boolean
  onToggleExpanded: () => void
  onResolve: (conflictId: string, resolution: 'accept-local' | 'accept-remote' | 'merge') => void
  onDismiss: (conflictId: string) => void
}

const ConflictCard: React.FC<ConflictCardProps> = ({
  conflict,
  isExpanded,
  onToggleExpanded,
  onResolve,
  onDismiss,
}) => {
  return (
    <div className="conflict-card">
      {/* Card Header */}
      <div className="conflict-card-header" onClick={onToggleExpanded}>
        <div className="conflict-card-title">
          <AlertTriangle size={16} color="#f59e0b" />
          <span>Conflict at position {conflict.position}</span>
        </div>
        <div className="conflict-card-meta">
          <span className="conflict-users">
            {conflict.remoteChanges.length + 1} user{conflict.remoteChanges.length > 0 ? 's' : ''}
          </span>
          <button className="conflict-expand-button">{isExpanded ? '▼' : '▶'}</button>
        </div>
      </div>

      {/* Expanded Details */}
      {isExpanded && (
        <div className="conflict-card-body">
          {/* Local Change */}
          <div className="conflict-change local-change">
            <div className="conflict-change-header">
              <span className="conflict-change-label">Your Change</span>
              <span
                className="conflict-user-badge"
                style={{ backgroundColor: conflict.localChange.userColor }}
              >
                {conflict.localChange.userName}
              </span>
            </div>
            <div className="conflict-change-content">
              <code>{conflict.localChange.content}</code>
            </div>
            <div className="conflict-change-type">{conflict.localChange.type.toUpperCase()}</div>
          </div>

          {/* Remote Changes */}
          {conflict.remoteChanges.map((remoteChange, index) => (
            <div key={remoteChange.id} className="conflict-change remote-change">
              <div className="conflict-change-header">
                <span className="conflict-change-label">Remote Change #{index + 1}</span>
                <span
                  className="conflict-user-badge"
                  style={{ backgroundColor: remoteChange.userColor }}
                >
                  {remoteChange.userName}
                </span>
              </div>
              <div className="conflict-change-content">
                <code>{remoteChange.content}</code>
              </div>
              <div className="conflict-change-type">{remoteChange.type.toUpperCase()}</div>
            </div>
          ))}

          {/* Resolution Actions */}
          <div className="conflict-actions">
            <button
              className="conflict-action-button accept-local"
              onClick={() => onResolve(conflict.id, 'accept-local')}
              title="Accept your changes and discard remote changes"
            >
              <CheckCircle size={16} />
              Accept Local
            </button>
            <button
              className="conflict-action-button accept-remote"
              onClick={() => onResolve(conflict.id, 'accept-remote')}
              title="Accept remote changes and discard your changes"
            >
              <CheckCircle size={16} />
              Accept Remote
            </button>
            <button
              className="conflict-action-button merge"
              onClick={() => onResolve(conflict.id, 'merge')}
              title="Attempt automatic merge (Yjs CRDT)"
            >
              <RefreshCw size={16} />
              Auto-Merge
            </button>
            <button
              className="conflict-action-button dismiss"
              onClick={() => onDismiss(conflict.id)}
              title="Dismiss this notification (conflict remains)"
            >
              <XCircle size={16} />
              Dismiss
            </button>
          </div>

          {/* Info Message */}
          <div className="conflict-info">
            <Info size={14} />
            <span>
              Yjs uses CRDTs for automatic conflict-free merging. Manual resolution is rarely needed.
            </span>
          </div>
        </div>
      )}

      <style>{`
        .conflict-card {
          background: #fffbeb;
          border: 1px solid #fbbf24;
          border-radius: 6px;
          margin-bottom: 12px;
          overflow: hidden;
        }

        .conflict-card-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 12px 16px;
          cursor: pointer;
          transition: background 0.2s;
        }

        .conflict-card-header:hover {
          background: #fef3c7;
        }

        .conflict-card-title {
          display: flex;
          align-items: center;
          gap: 8px;
          font-weight: 500;
          font-size: 14px;
          color: #92400e;
        }

        .conflict-card-meta {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .conflict-users {
          font-size: 12px;
          color: #78350f;
        }

        .conflict-expand-button {
          background: transparent;
          border: none;
          color: #78350f;
          cursor: pointer;
          font-size: 12px;
        }

        .conflict-card-body {
          padding: 16px;
          border-top: 1px solid #fbbf24;
          background: #fef3c7;
        }

        .conflict-change {
          background: #fff;
          border-radius: 4px;
          padding: 12px;
          margin-bottom: 12px;
        }

        .conflict-change:last-of-type {
          margin-bottom: 16px;
        }

        .local-change {
          border-left: 3px solid #10b981;
        }

        .remote-change {
          border-left: 3px solid #3b82f6;
        }

        .conflict-change-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 8px;
        }

        .conflict-change-label {
          font-size: 12px;
          font-weight: 600;
          color: #6b7280;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .conflict-user-badge {
          font-size: 11px;
          font-weight: 500;
          color: #fff;
          padding: 2px 8px;
          border-radius: 12px;
        }

        .conflict-change-content {
          background: #f9fafb;
          border-radius: 4px;
          padding: 8px;
          margin-bottom: 8px;
          font-family: 'Courier New', monospace;
          font-size: 13px;
          color: #111827;
          overflow-x: auto;
        }

        .conflict-change-type {
          font-size: 11px;
          font-weight: 600;
          color: #9ca3af;
          letter-spacing: 0.5px;
        }

        .conflict-actions {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
          margin-bottom: 12px;
        }

        .conflict-action-button {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 8px 12px;
          border: 1px solid #e5e7eb;
          border-radius: 4px;
          background: #fff;
          font-size: 13px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .conflict-action-button:hover {
          background: #f9fafb;
          border-color: #d1d5db;
        }

        .accept-local {
          color: #10b981;
          border-color: #10b981;
        }

        .accept-local:hover {
          background: #ecfdf5;
        }

        .accept-remote {
          color: #3b82f6;
          border-color: #3b82f6;
        }

        .accept-remote:hover {
          background: #eff6ff;
        }

        .merge {
          color: #8b5cf6;
          border-color: #8b5cf6;
        }

        .merge:hover {
          background: #f5f3ff;
        }

        .dismiss {
          color: #6b7280;
          border-color: #d1d5db;
        }

        .conflict-info {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px;
          background: #dbeafe;
          border-radius: 4px;
          font-size: 12px;
          color: #1e40af;
        }
      `}</style>
    </div>
  )
}

interface ResolvedConflictCardProps {
  conflict: Conflict
  onDismiss: (conflictId: string) => void
}

const ResolvedConflictCard: React.FC<ResolvedConflictCardProps> = ({ conflict, onDismiss }) => {
  const statusColor =
    conflict.status === 'accepted' ? '#10b981' : conflict.status === 'rejected' ? '#ef4444' : '#8b5cf6'

  const StatusIcon =
    conflict.status === 'accepted' ? CheckCircle : conflict.status === 'rejected' ? XCircle : RefreshCw

  return (
    <div className="resolved-conflict-card">
      <div className="resolved-conflict-content">
        <StatusIcon size={16} color={statusColor} />
        <div className="resolved-conflict-text">
          <div className="resolved-conflict-title">
            Conflict at position {conflict.position} - {conflict.status.toUpperCase()}
          </div>
          {conflict.resolvedAt && (
            <div className="resolved-conflict-time">
              Resolved {new Date(conflict.resolvedAt).toLocaleTimeString()}
            </div>
          )}
        </div>
      </div>
      <button className="resolved-conflict-dismiss" onClick={() => onDismiss(conflict.id)}>
        <XCircle size={14} />
      </button>

      <style>{`
        .resolved-conflict-card {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 12px;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          margin-bottom: 8px;
        }

        .resolved-conflict-content {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .resolved-conflict-text {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .resolved-conflict-title {
          font-size: 13px;
          font-weight: 500;
          color: #374151;
        }

        .resolved-conflict-time {
          font-size: 11px;
          color: #9ca3af;
        }

        .resolved-conflict-dismiss {
          background: transparent;
          border: none;
          color: #9ca3af;
          cursor: pointer;
          padding: 4px;
          transition: color 0.2s;
        }

        .resolved-conflict-dismiss:hover {
          color: #ef4444;
        }
      `}</style>
    </div>
  )
}

export default ConflictResolution
