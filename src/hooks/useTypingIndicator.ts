// src/hooks/useTypingIndicator.ts
import { useEffect, useState, useCallback, useRef } from 'react'

export interface TypingState {
  isTyping: boolean
  lastTypingTime: number
}

export interface UseTypingIndicatorOptions {
  debounceMs?: number
  typingTimeoutMs?: number
  onTypingChange?: (isTyping: boolean) => void
}

export function useTypingIndicator(options: UseTypingIndicatorOptions = {}) {
  const { debounceMs = 300, typingTimeoutMs = 3000, onTypingChange } = options

  const [isTyping, setIsTyping] = useState(false)
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const debounceTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  const clearTypingTimeout = useCallback(() => {
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current)
      typingTimeoutRef.current = null
    }
  }, [])

  const clearDebounceTimeout = useCallback(() => {
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current)
      debounceTimeoutRef.current = null
    }
  }, [])

  const startTyping = useCallback(() => {
    clearDebounceTimeout()

    // Set typing state
    if (!isTyping) {
      setIsTyping(true)
      onTypingChange?.(true)
    }

    // Clear existing timeout
    clearTypingTimeout()

    // Set new timeout to stop typing
    typingTimeoutRef.current = setTimeout(() => {
      setIsTyping(false)
      onTypingChange?.(false)
    }, typingTimeoutMs)
  }, [isTyping, typingTimeoutMs, onTypingChange, clearTypingTimeout, clearDebounceTimeout])

  const stopTyping = useCallback(() => {
    clearDebounceTimeout()

    // Debounce the stop action
    debounceTimeoutRef.current = setTimeout(() => {
      clearTypingTimeout()
      setIsTyping(false)
      onTypingChange?.(false)
    }, debounceMs)
  }, [debounceMs, onTypingChange, clearTypingTimeout, clearDebounceTimeout])

  const handleTyping = useCallback(() => {
    startTyping()
  }, [startTyping])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      clearTypingTimeout()
      clearDebounceTimeout()
    }
  }, [clearTypingTimeout, clearDebounceTimeout])

  return {
    isTyping,
    startTyping,
    stopTyping,
    handleTyping,
  }
}

// Hook to track remote users' typing states
export function useRemoteTyping() {
  const [typingUsers, setTypingUsers] = useState<Map<string, TypingState>>(new Map())

  const updateUserTyping = useCallback((userId: string, isTyping: boolean) => {
    setTypingUsers(prev => {
      const next = new Map(prev)
      if (isTyping) {
        next.set(userId, {
          isTyping: true,
          lastTypingTime: Date.now(),
        })
      } else {
        next.delete(userId)
      }
      return next
    })
  }, [])

  const isUserTyping = useCallback(
    (userId: string): boolean => {
      return typingUsers.has(userId)
    },
    [typingUsers]
  )

  const getTypingUsers = useCallback((): string[] => {
    return Array.from(typingUsers.keys())
  }, [typingUsers])

  // Clean up stale typing states
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now()
      const staleTimeout = 5000 // 5 seconds

      setTypingUsers(prev => {
        const next = new Map(prev)
        let hasChanges = false

        for (const [userId, state] of next.entries()) {
          if (now - state.lastTypingTime > staleTimeout) {
            next.delete(userId)
            hasChanges = true
          }
        }

        return hasChanges ? next : prev
      })
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  return {
    typingUsers,
    updateUserTyping,
    isUserTyping,
    getTypingUsers,
  }
}
