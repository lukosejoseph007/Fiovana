// src/components/collaboration/ActiveUsers.tsx
import React from 'react'
import { Users } from 'lucide-react'

export interface ActiveUser {
  id: string
  name: string
  color: string
  isTyping?: boolean
  lastSeen?: number
}

export interface ActiveUsersProps {
  users: ActiveUser[]
  currentUserId: string
  maxVisibleAvatars?: number
  size?: 'small' | 'medium' | 'large'
  showTooltip?: boolean
  onClick?: () => void
}

export const ActiveUsers: React.FC<ActiveUsersProps> = ({
  users,
  currentUserId,
  maxVisibleAvatars = 5,
  size = 'medium',
  showTooltip = true,
  onClick,
}) => {
  const activeUsers = users.filter(u => u.id !== currentUserId)
  const visibleUsers = activeUsers.slice(0, maxVisibleAvatars)
  const extraCount = Math.max(0, activeUsers.length - maxVisibleAvatars)

  const sizeMap = {
    small: { avatar: '1.5rem', fontSize: '0.625rem' },
    medium: { avatar: '2rem', fontSize: '0.75rem' },
    large: { avatar: '2.5rem', fontSize: '0.875rem' },
  }

  const dimensions = sizeMap[size]

  return (
    <div
      className={`active-users ${onClick ? 'clickable' : ''}`}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
    >
      <div className="active-users-badge">
        <Users size={size === 'small' ? 12 : size === 'medium' ? 14 : 16} />
        <span className="active-users-count">{activeUsers.length + 1}</span>
      </div>

      <div className="active-users-avatars">
        {visibleUsers.map((user, index) => (
          <div
            key={user.id}
            className={`user-avatar ${user.isTyping ? 'typing' : ''}`}
            style={{
              backgroundColor: user.color,
              width: dimensions.avatar,
              height: dimensions.avatar,
              fontSize: dimensions.fontSize,
              marginLeft: index > 0 ? '-0.5rem' : '0',
              zIndex: visibleUsers.length - index,
            }}
            title={showTooltip ? `${user.name}${user.isTyping ? ' (typing...)' : ''}` : undefined}
          >
            {getInitials(user.name)}
            {user.isTyping && (
              <div className="typing-indicator">
                <span />
                <span />
                <span />
              </div>
            )}
          </div>
        ))}

        {extraCount > 0 && (
          <div
            className="user-avatar extra-count"
            style={{
              width: dimensions.avatar,
              height: dimensions.avatar,
              fontSize: dimensions.fontSize,
              marginLeft: '-0.5rem',
              zIndex: 0,
            }}
            title={
              showTooltip ? `${extraCount} more ${extraCount === 1 ? 'user' : 'users'}` : undefined
            }
          >
            +{extraCount}
          </div>
        )}
      </div>

      <style>{`
        .active-users {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          padding: 0.5rem;
          border-radius: 0.5rem;
          background: var(--bg-secondary, #f9fafb);
          border: 1px solid var(--border-color, #e5e7eb);
        }

        .active-users.clickable {
          cursor: pointer;
          transition: all 0.15s ease;
        }

        .active-users.clickable:hover {
          background: var(--bg-hover, #f3f4f6);
          border-color: var(--border-hover, #d1d5db);
        }

        .active-users-badge {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.25rem 0.5rem;
          background: var(--primary-color, #3b82f6);
          color: white;
          border-radius: 0.375rem;
          font-size: 0.75rem;
          font-weight: 600;
        }

        .active-users-count {
          line-height: 1;
        }

        .active-users-avatars {
          display: flex;
          align-items: center;
        }

        .user-avatar {
          position: relative;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: 600;
          color: white;
          border: 2px solid var(--bg-primary, #ffffff);
          flex-shrink: 0;
          transition: transform 0.15s ease;
        }

        .user-avatar:hover {
          transform: translateY(-2px);
        }

        .user-avatar.typing {
          animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        }

        .user-avatar.extra-count {
          background: var(--text-secondary, #6b7280);
          font-size: 0.75rem;
        }

        .typing-indicator {
          position: absolute;
          bottom: -2px;
          right: -2px;
          display: flex;
          gap: 2px;
          background: white;
          border-radius: 0.5rem;
          padding: 2px 4px;
        }

        .typing-indicator span {
          width: 3px;
          height: 3px;
          background: var(--primary-color, #3b82f6);
          border-radius: 50%;
          animation: typing 1.4s ease-in-out infinite;
        }

        .typing-indicator span:nth-child(2) {
          animation-delay: 0.2s;
        }

        .typing-indicator span:nth-child(3) {
          animation-delay: 0.4s;
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.7;
          }
        }

        @keyframes typing {
          0%, 60%, 100% {
            transform: translateY(0);
          }
          30% {
            transform: translateY(-4px);
          }
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
