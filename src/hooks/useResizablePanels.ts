import { useState, useCallback, useRef } from 'react'

interface ResizableState {
  width: number
  isResizing: boolean
}

interface UseResizablePanelsReturn {
  navigationWidth: number
  intelligenceWidth: number
  isNavigationResizing: boolean
  isIntelligenceResizing: boolean
  startNavigationResize: (e: React.MouseEvent) => void
  startIntelligenceResize: (e: React.MouseEvent) => void
}

const PANEL_CONFIGS = {
  navigation: {
    minWidth: 200,
    maxWidth: 400,
    defaultWidth: Math.floor((window.innerWidth || 1024) * 0.15), // 15% of viewport width
    storageKey: 'fiovana-navigation-width',
  },
  intelligence: {
    minWidth: 280,
    maxWidth: 500,
    defaultWidth: Math.floor((window.innerWidth || 1024) * 0.25), // 25% of viewport width
    storageKey: 'fiovana-intelligence-width',
  },
} as const

// Helper function to get initial width with localStorage
const getInitialWidth = (panelType: 'navigation' | 'intelligence'): number => {
  const config = PANEL_CONFIGS[panelType]

  try {
    const savedWidth = localStorage.getItem(config.storageKey)
    if (savedWidth) {
      const width = parseInt(savedWidth)
      if (width >= config.minWidth && width <= config.maxWidth) {
        return width
      }
    }
  } catch (error) {
    // localStorage might not be available (SSR, etc.)
    console.warn(`Failed to load ${panelType} width from localStorage:`, error)
  }

  return config.defaultWidth
}

export const useResizablePanels = (): UseResizablePanelsReturn => {
  const [navigationState, setNavigationState] = useState<ResizableState>({
    width: getInitialWidth('navigation'),
    isResizing: false,
  })

  const [intelligenceState, setIntelligenceState] = useState<ResizableState>({
    width: getInitialWidth('intelligence'),
    isResizing: false,
  })

  const resizeStateRef = useRef<{
    panel: 'navigation' | 'intelligence' | null
    startX: number
    startWidth: number
  }>({
    panel: null,
    startX: 0,
    startWidth: 0,
  })

  // Keep refs for current widths to avoid stale closure issues
  const currentWidthsRef = useRef({
    navigation: navigationState.width,
    intelligence: intelligenceState.width,
  })

  // Update refs when state changes
  currentWidthsRef.current.navigation = navigationState.width
  currentWidthsRef.current.intelligence = intelligenceState.width

  // Save to localStorage when width changes
  const saveWidth = useCallback((panel: 'navigation' | 'intelligence', width: number) => {
    const config = PANEL_CONFIGS[panel]
    localStorage.setItem(config.storageKey, width.toString())
  }, [])

  // Handle mouse move during resize
  const handleMouseMove = useCallback((e: MouseEvent) => {
    const { panel, startX, startWidth } = resizeStateRef.current

    if (!panel) return

    const deltaX = e.clientX - startX
    const config = PANEL_CONFIGS[panel]

    let newWidth: number
    if (panel === 'navigation') {
      newWidth = Math.max(config.minWidth, Math.min(config.maxWidth, startWidth + deltaX))
    } else {
      // For intelligence panel (right side), dragging left decreases width
      newWidth = Math.max(config.minWidth, Math.min(config.maxWidth, startWidth - deltaX))
    }

    if (panel === 'navigation') {
      setNavigationState(prev => ({ ...prev, width: newWidth }))
    } else {
      setIntelligenceState(prev => ({ ...prev, width: newWidth }))
    }
  }, [])

  // Handle mouse up to end resize
  const handleMouseUp = useCallback(() => {
    const { panel } = resizeStateRef.current

    if (!panel) return

    // Save the final width using current ref values
    if (panel === 'navigation') {
      saveWidth('navigation', currentWidthsRef.current.navigation)
      setNavigationState(prev => ({ ...prev, isResizing: false }))
    } else {
      saveWidth('intelligence', currentWidthsRef.current.intelligence)
      setIntelligenceState(prev => ({ ...prev, isResizing: false }))
    }

    resizeStateRef.current = { panel: null, startX: 0, startWidth: 0 }

    // Clean up event listeners
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }, [handleMouseMove, saveWidth])

  // Start resizing navigation panel
  const startNavigationResize = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()

      resizeStateRef.current = {
        panel: 'navigation',
        startX: e.clientX,
        startWidth: navigationState.width,
      }

      setNavigationState(prev => ({ ...prev, isResizing: true }))

      document.addEventListener('mousemove', handleMouseMove)
      document.addEventListener('mouseup', handleMouseUp)
      document.body.style.cursor = 'col-resize'
      document.body.style.userSelect = 'none'
    },
    [navigationState.width, handleMouseMove, handleMouseUp]
  )

  // Start resizing intelligence panel
  const startIntelligenceResize = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()

      resizeStateRef.current = {
        panel: 'intelligence',
        startX: e.clientX,
        startWidth: intelligenceState.width,
      }

      setIntelligenceState(prev => ({ ...prev, isResizing: true }))

      document.addEventListener('mousemove', handleMouseMove)
      document.addEventListener('mouseup', handleMouseUp)
      document.body.style.cursor = 'col-resize'
      document.body.style.userSelect = 'none'
    },
    [intelligenceState.width, handleMouseMove, handleMouseUp]
  )

  return {
    navigationWidth: navigationState.width,
    intelligenceWidth: intelligenceState.width,
    isNavigationResizing: navigationState.isResizing,
    isIntelligenceResizing: intelligenceState.isResizing,
    startNavigationResize,
    startIntelligenceResize,
  }
}
