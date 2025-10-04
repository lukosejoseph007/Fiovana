import { useState, useCallback, useRef, useEffect } from 'react'
import type { Change } from '../components/editor/TrackChanges'

export interface UseTrackChangesOptions {
  enabled?: boolean
  currentUser?: string
  onChangeAccepted?: (change: Change) => void
  onChangeRejected?: (change: Change) => void
}

export interface UseTrackChangesReturn {
  changes: Change[]
  isTrackingEnabled: boolean
  showChanges: boolean
  addChange: (change: Omit<Change, 'id' | 'timestamp' | 'author'>) => void
  acceptChange: (changeId: string) => void
  rejectChange: (changeId: string) => void
  acceptAllChanges: () => void
  rejectAllChanges: () => void
  toggleTracking: () => void
  toggleShowChanges: () => void
  clearAcceptedRejected: () => void
}

export const useTrackChanges = (options: UseTrackChangesOptions = {}): UseTrackChangesReturn => {
  const { enabled = false, currentUser = 'You', onChangeAccepted, onChangeRejected } = options

  const [changes, setChanges] = useState<Change[]>([])
  const [isTrackingEnabled, setIsTrackingEnabled] = useState(enabled)
  const [showChanges, setShowChanges] = useState(true)
  const changeIdCounter = useRef(0)

  // Update tracking state when enabled prop changes
  useEffect(() => {
    setIsTrackingEnabled(enabled)
  }, [enabled])

  const generateChangeId = useCallback(() => {
    changeIdCounter.current += 1
    return `change-${Date.now()}-${changeIdCounter.current}`
  }, [])

  const addChange = useCallback(
    (changeData: Omit<Change, 'id' | 'timestamp' | 'author'>) => {
      if (!isTrackingEnabled) {
        return
      }

      const newChange: Change = {
        ...changeData,
        id: generateChangeId(),
        timestamp: new Date(),
        author: currentUser,
      }

      setChanges(prev => [...prev, newChange])
    },
    [isTrackingEnabled, currentUser, generateChangeId]
  )

  const acceptChange = useCallback(
    (changeId: string) => {
      setChanges(prev =>
        prev.map(change => {
          if (change.id === changeId) {
            const updatedChange = { ...change, accepted: true, rejected: false }
            onChangeAccepted?.(updatedChange)
            return updatedChange
          }
          return change
        })
      )
    },
    [onChangeAccepted]
  )

  const rejectChange = useCallback(
    (changeId: string) => {
      setChanges(prev =>
        prev.map(change => {
          if (change.id === changeId) {
            const updatedChange = { ...change, rejected: true, accepted: false }
            onChangeRejected?.(updatedChange)
            return updatedChange
          }
          return change
        })
      )
    },
    [onChangeRejected]
  )

  const acceptAllChanges = useCallback(() => {
    setChanges(prev =>
      prev.map(change => {
        if (!change.accepted && !change.rejected) {
          const updatedChange = { ...change, accepted: true, rejected: false }
          onChangeAccepted?.(updatedChange)
          return updatedChange
        }
        return change
      })
    )
  }, [onChangeAccepted])

  const rejectAllChanges = useCallback(() => {
    setChanges(prev =>
      prev.map(change => {
        if (!change.accepted && !change.rejected) {
          const updatedChange = { ...change, rejected: true, accepted: false }
          onChangeRejected?.(updatedChange)
          return updatedChange
        }
        return change
      })
    )
  }, [onChangeRejected])

  const toggleTracking = useCallback(() => {
    setIsTrackingEnabled(prev => !prev)
  }, [])

  const toggleShowChanges = useCallback(() => {
    setShowChanges(prev => !prev)
  }, [])

  const clearAcceptedRejected = useCallback(() => {
    setChanges(prev => prev.filter(change => !change.accepted && !change.rejected))
  }, [])

  return {
    changes,
    isTrackingEnabled,
    showChanges,
    addChange,
    acceptChange,
    rejectChange,
    acceptAllChanges,
    rejectAllChanges,
    toggleTracking,
    toggleShowChanges,
    clearAcceptedRejected,
  }
}
