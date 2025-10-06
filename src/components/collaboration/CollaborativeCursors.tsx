import React, { useEffect, useState } from 'react'
import type { Awareness } from 'y-protocols/awareness'

export interface CursorPosition {
  x: number
  y: number
}

export interface RemoteCursor {
  clientID: number
  name: string
  color: string
  cursor?: CursorPosition
}

export interface CollaborativeCursorsProps {
  awareness: Awareness
}

const CollaborativeCursors: React.FC<CollaborativeCursorsProps> = ({ awareness }) => {
  const [remoteCursors, setRemoteCursors] = useState<RemoteCursor[]>([])

  useEffect(() => {
    const updateCursors = () => {
      const states = awareness.getStates()
      const cursors: RemoteCursor[] = []
      const localClientID = awareness.clientID

      states.forEach((state, clientID) => {
        // Skip local client
        if (clientID === localClientID) return

        // Extract user info
        if (state && typeof state === 'object') {
          const user = 'user' in state ? (state.user as Record<string, unknown>) : state
          const name = 'name' in user ? String(user.name) : 'Anonymous'
          const color = 'color' in user ? String(user.color) : '#808080'
          const cursor =
            'cursor' in state ? (state.cursor as CursorPosition | undefined) : undefined

          cursors.push({
            clientID,
            name,
            color,
            cursor,
          })
        }
      })

      setRemoteCursors(cursors)
    }

    // Initial update
    updateCursors()

    // Listen for awareness changes
    awareness.on('change', updateCursors)

    return () => {
      awareness.off('change', updateCursors)
    }
  }, [awareness])

  return (
    <>
      {remoteCursors.map(cursor => {
        if (!cursor.cursor) return null

        return (
          <div
            key={cursor.clientID}
            className="pointer-events-none fixed z-50 transition-all duration-100"
            style={{
              left: `${cursor.cursor.x}px`,
              top: `${cursor.cursor.y}px`,
              transform: 'translate(-50%, -50%)',
            }}
          >
            {/* Cursor icon */}
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              style={{ color: cursor.color }}
            >
              <path
                d="M5.65376 12.3673L10.156 21.5L11.6435 17.5L15.6435 17.5L5.65376 12.3673Z"
                fill="currentColor"
              />
              <path
                d="M5.65376 12.3673L10.156 21.5L11.6435 17.5L15.6435 17.5L5.65376 12.3673Z"
                stroke="white"
                strokeWidth="1.5"
                strokeLinejoin="round"
              />
            </svg>

            {/* User label */}
            <div
              className="absolute top-full left-4 mt-1 rounded px-2 py-1 text-xs font-medium text-white whitespace-nowrap shadow-md"
              style={{ backgroundColor: cursor.color }}
            >
              {cursor.name}
            </div>
          </div>
        )
      })}
    </>
  )
}

export default CollaborativeCursors
