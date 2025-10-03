// src/components/collaboration/UserPresence.tsx
import React from 'react'
import { Users, Wifi, WifiOff } from 'lucide-react'

export interface User {
  id: string
  name: string
  color: string
  cursor?: {
    x: number
    y: number
  }
  isActive: boolean
}

export interface UserPresenceProps {
  users: User[]
  currentUserId: string
  isConnected: boolean
  onUserClick?: (userId: string) => void
}

export const UserPresence: React.FC<UserPresenceProps> = ({
  users,
  currentUserId,
  isConnected,
  onUserClick,
}) => {
  const activeUsers = users.filter(u => u.isActive && u.id !== currentUserId)
  const totalUsers = activeUsers.length + 1 // +1 for current user

  return (
    <div className="user-presence">
      <div className="user-presence-header">
        <div className="user-presence-status">
          {isConnected ? (
            <Wifi size={16} className="status-icon connected" />
          ) : (
            <WifiOff size={16} className="status-icon disconnected" />
          )}
          <span className="status-text">{isConnected ? 'Connected' : 'Disconnected'}</span>
        </div>
        <div className="user-presence-count">
          <Users size={16} />
          <span>
            {totalUsers} {totalUsers === 1 ? 'user' : 'users'}
          </span>
        </div>
      </div>

      <div className="user-presence-list">
        {/* Current user */}
        <div className="user-presence-item current-user">
          <div className="user-avatar" style={{ backgroundColor: '#3b82f6' }}>
            {getInitials('You')}
          </div>
          <span className="user-name">You</span>
          <span className="user-badge">You</span>
        </div>

        {/* Other active users */}
        {activeUsers.map(user => (
          <div
            key={user.id}
            className="user-presence-item"
            onClick={() => onUserClick?.(user.id)}
            style={{ cursor: onUserClick ? 'pointer' : 'default' }}
          >
            <div className="user-avatar" style={{ backgroundColor: user.color }}>
              {getInitials(user.name)}
            </div>
            <span className="user-name">{user.name}</span>
            <div className="user-status-indicator active" />
          </div>
        ))}

        {activeUsers.length === 0 && isConnected && (
          <div className="user-presence-empty">
            <p>No other users connected</p>
          </div>
        )}
      </div>

      <style>{`
        .user-presence {
          padding: 1rem;
          background: var(--bg-primary, #ffffff);
          border-radius: 0.5rem;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .user-presence-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
          padding-bottom: 0.75rem;
          border-bottom: 1px solid var(--border-color, #e5e7eb);
        }

        .user-presence-status {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .status-icon {
          flex-shrink: 0;
        }

        .status-icon.connected {
          color: var(--success-color, #10b981);
        }

        .status-icon.disconnected {
          color: var(--error-color, #ef4444);
        }

        .status-text {
          font-size: 0.875rem;
          font-weight: 500;
          color: var(--text-secondary, #6b7280);
        }

        .user-presence-count {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-size: 0.875rem;
          color: var(--text-secondary, #6b7280);
        }

        .user-presence-list {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .user-presence-item {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          padding: 0.5rem;
          border-radius: 0.375rem;
          transition: background-color 0.15s ease;
        }

        .user-presence-item:hover {
          background: var(--bg-hover, #f3f4f6);
        }

        .user-presence-item.current-user {
          background: var(--bg-selected, #eff6ff);
        }

        .user-avatar {
          width: 2rem;
          height: 2rem;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 0.875rem;
          font-weight: 600;
          color: white;
          flex-shrink: 0;
        }

        .user-name {
          flex: 1;
          font-size: 0.875rem;
          font-weight: 500;
          color: var(--text-primary, #111827);
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .user-badge {
          font-size: 0.75rem;
          padding: 0.125rem 0.5rem;
          background: var(--primary-color, #3b82f6);
          color: white;
          border-radius: 0.25rem;
          font-weight: 500;
        }

        .user-status-indicator {
          width: 0.5rem;
          height: 0.5rem;
          border-radius: 50%;
          flex-shrink: 0;
        }

        .user-status-indicator.active {
          background: var(--success-color, #10b981);
          box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.2);
        }

        .user-presence-empty {
          padding: 2rem 1rem;
          text-align: center;
        }

        .user-presence-empty p {
          font-size: 0.875rem;
          color: var(--text-secondary, #6b7280);
          margin: 0;
        }
      `}</style>
    </div>
  )
}

function getInitials(name: string): string {
  return name
    .split(' ')
    .map(word => word[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
}
