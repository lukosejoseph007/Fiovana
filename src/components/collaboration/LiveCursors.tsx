// src/components/collaboration/LiveCursors.tsx
import React, { useEffect, useState } from 'react'
import { MousePointer2 } from 'lucide-react'

export interface CursorPosition {
  userId: string
  userName: string
  userColor: string
  x: number
  y: number
  lastUpdate: number
}

export interface LiveCursorsProps {
  cursors: CursorPosition[]
  containerRef?: React.RefObject<HTMLElement | HTMLDivElement | null>
  showLabels?: boolean
  fadeTimeout?: number // milliseconds before cursor fades out
}

export const LiveCursors: React.FC<LiveCursorsProps> = ({
  cursors,
  containerRef,
  showLabels = true,
  fadeTimeout = 3000,
}) => {
  const [visibleCursors, setVisibleCursors] = useState<CursorPosition[]>([])

  useEffect(() => {
    // Filter out stale cursors
    const now = Date.now()
    const active = cursors.filter(cursor => now - cursor.lastUpdate < fadeTimeout)
    setVisibleCursors(active)

    // Set up interval to clean up stale cursors
    const interval = setInterval(() => {
      const currentTime = Date.now()
      setVisibleCursors(prev =>
        prev.filter(cursor => currentTime - cursor.lastUpdate < fadeTimeout)
      )
    }, 1000)

    return () => clearInterval(interval)
  }, [cursors, fadeTimeout])

  // Get container bounds for positioning
  const getContainerBounds = () => {
    if (containerRef?.current) {
      return containerRef.current.getBoundingClientRect()
    }
    return { left: 0, top: 0 }
  }

  const containerBounds = getContainerBounds()

  return (
    <>
      {visibleCursors.map(cursor => {
        const opacity = calculateOpacity(cursor.lastUpdate, fadeTimeout)

        return (
          <div
            key={cursor.userId}
            className="live-cursor"
            style={{
              position: 'fixed',
              left: `${cursor.x - containerBounds.left}px`,
              top: `${cursor.y - containerBounds.top}px`,
              opacity,
              pointerEvents: 'none',
              zIndex: 9999,
              transition: 'opacity 0.3s ease, left 0.1s ease, top 0.1s ease',
            }}
          >
            <MousePointer2
              size={20}
              style={{
                color: cursor.userColor,
                filter: 'drop-shadow(0 1px 2px rgba(0, 0, 0, 0.3))',
              }}
              fill={cursor.userColor}
            />
            {showLabels && (
              <div
                className="live-cursor-label"
                style={{
                  backgroundColor: cursor.userColor,
                }}
              >
                {cursor.userName}
              </div>
            )}
          </div>
        )
      })}

      <style>{`
        .live-cursor {
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
        }

        .live-cursor-label {
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-size: 0.75rem;
          font-weight: 500;
          color: white;
          white-space: nowrap;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
          margin-left: 1.25rem;
          margin-top: -0.25rem;
        }
      `}</style>
    </>
  )
}

function calculateOpacity(lastUpdate: number, fadeTimeout: number): number {
  const now = Date.now()
  const age = now - lastUpdate

  if (age > fadeTimeout) return 0
  if (age < fadeTimeout * 0.7) return 1

  // Fade out in the last 30% of the timeout period
  const fadeStart = fadeTimeout * 0.7
  const fadeDuration = fadeTimeout * 0.3
  const fadeProgress = (age - fadeStart) / fadeDuration

  return 1 - fadeProgress
}
