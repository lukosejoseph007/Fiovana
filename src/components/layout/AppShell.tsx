import React, { useState, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import { LayoutContext, useLayout } from './useLayoutContext'
import { useResizablePanels } from '../../hooks/useResizablePanels'
import DragHandle from '../ui/DragHandle'

export interface AppShellProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export interface LayoutContextType {
  navigationCollapsed: boolean
  intelligenceCollapsed: boolean
  toggleNavigation: () => void
  toggleIntelligence: () => void
  isMobile: boolean
  isTablet: boolean
  isDesktop: boolean
  navigationWidth: number
  intelligenceWidth: number
  isNavigationResizing: boolean
  isIntelligenceResizing: boolean
  startNavigationResize: (e: React.MouseEvent) => void
  startIntelligenceResize: (e: React.MouseEvent) => void
}

const AppShell: React.FC<AppShellProps> = ({ children, className = '', style }) => {
  const [navigationCollapsed, setNavigationCollapsed] = useState(false) // Always start expanded
  const [intelligenceCollapsed, setIntelligenceCollapsed] = useState(false) // Always start expanded
  const [viewport, setViewport] = useState({
    width: window.innerWidth || 1024,
    height: window.innerHeight || 768,
  })

  // Initialize resizable panels
  const {
    navigationWidth,
    intelligenceWidth,
    isNavigationResizing,
    isIntelligenceResizing,
    startNavigationResize,
    startIntelligenceResize,
  } = useResizablePanels()

  // Responsive breakpoint detection
  useEffect(() => {
    const updateViewport = () => {
      setViewport({ width: window.innerWidth, height: window.innerHeight })
    }

    updateViewport()
    window.addEventListener('resize', updateViewport)
    return () => window.removeEventListener('resize', updateViewport)
  }, [])

  const isMobile = viewport.width < parseInt(designTokens.breakpoints.mobile)
  const isTablet =
    viewport.width >= parseInt(designTokens.breakpoints.mobile) &&
    viewport.width < parseInt(designTokens.breakpoints.desktop)
  const isDesktop = viewport.width >= parseInt(designTokens.breakpoints.desktop)

  // Auto-collapse panels on smaller screens (less aggressive)
  useEffect(() => {
    // Only auto-collapse if we have a valid viewport measurement and are actually on mobile
    if (
      isMobile &&
      viewport.width > 0 &&
      viewport.width < parseInt(designTokens.breakpoints.mobile)
    ) {
      setNavigationCollapsed(true)
      setIntelligenceCollapsed(true)
    }
    // Don't auto-collapse intelligence panel on tablet - let user decide
  }, [isMobile, viewport.width])

  const toggleNavigation = () => setNavigationCollapsed(!navigationCollapsed)
  const toggleIntelligence = () => setIntelligenceCollapsed(!intelligenceCollapsed)

  const layoutContextValue: LayoutContextType = {
    navigationCollapsed,
    intelligenceCollapsed,
    toggleNavigation,
    toggleIntelligence,
    isMobile,
    isTablet,
    isDesktop,
    navigationWidth,
    intelligenceWidth,
    isNavigationResizing,
    isIntelligenceResizing,
    startNavigationResize,
    startIntelligenceResize,
  }

  const containerStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100vh',
    backgroundColor: designTokens.colors.surface.primary,
    color: designTokens.colors.text.primary,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
    overflow: 'hidden',
    ...style,
  }

  return (
    <LayoutContext.Provider value={layoutContextValue}>
      <div className={`fiovana-app-shell ${className}`} style={containerStyles}>
        {children}
      </div>
    </LayoutContext.Provider>
  )
}

// Header Component
export interface HeaderProps {
  children?: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export const Header: React.FC<HeaderProps> = ({ children, className = '', style }) => {
  const { isMobile, toggleNavigation, toggleIntelligence } = useLayout()

  const headerStyles = {
    height: designTokens.layout.header.height,
    backgroundColor: designTokens.colors.surface.primary,
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: `0 ${designTokens.spacing[4]}`,
    position: 'relative' as const,
    zIndex: designTokens.zIndex.sticky,
    flexShrink: 0,
    ...style,
  }

  return (
    <header className={`fiovana-header ${className}`} style={headerStyles}>
      {children}

      {/* Mobile menu controls */}
      {isMobile && (
        <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
          <button
            onClick={toggleNavigation}
            style={{
              background: 'none',
              border: 'none',
              color: designTokens.colors.text.secondary,
              cursor: 'pointer',
              padding: designTokens.spacing[2],
              borderRadius: designTokens.borderRadius.md,
              transition: `color ${designTokens.animation.duration.fast}`,
            }}
            aria-label="Toggle navigation"
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </button>

          <button
            onClick={toggleIntelligence}
            style={{
              background: 'none',
              border: 'none',
              color: designTokens.colors.text.secondary,
              cursor: 'pointer',
              padding: designTokens.spacing[2],
              borderRadius: designTokens.borderRadius.md,
              transition: `color ${designTokens.animation.duration.fast}`,
            }}
            aria-label="Toggle intelligence panel"
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <circle cx="12" cy="12" r="3" />
              <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1" />
            </svg>
          </button>
        </div>
      )}
    </header>
  )
}

// Main Layout Container
export interface MainProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export const Main: React.FC<MainProps> = ({ children, className = '', style }) => {
  const mainStyles = {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
    position: 'relative' as const,
    ...style,
  }

  return (
    <main className={`fiovana-main ${className}`} style={mainStyles}>
      {children}
    </main>
  )
}

// Navigation Panel
export interface NavigationProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export const Navigation: React.FC<NavigationProps> = ({ children, className = '', style }) => {
  const {
    navigationCollapsed,
    isMobile,
    isDesktop,
    navigationWidth,
    isNavigationResizing,
    startNavigationResize,
  } = useLayout()

  const shouldShow = isDesktop || (!navigationCollapsed && !isDesktop)
  const width =
    navigationCollapsed && isDesktop
      ? designTokens.layout.navigation.collapsedWidth
      : `${navigationWidth}px`

  const navigationStyles = {
    width: shouldShow ? width : '0',
    minWidth: shouldShow ? width : '0',
    backgroundColor: designTokens.colors.surface.secondary,
    borderRight: shouldShow ? `1px solid ${designTokens.colors.border.subtle}` : 'none',
    overflow: 'hidden' as const,
    transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
    position: isMobile ? ('absolute' as const) : ('relative' as const),
    top: isMobile ? 0 : 'auto',
    left: isMobile ? 0 : 'auto',
    height: isMobile ? '100%' : 'auto',
    zIndex: isMobile ? designTokens.zIndex.overlay : 'auto',
    flexShrink: 0,
    ...style,
  }

  const navigationContentStyles = {
    width:
      navigationCollapsed && isDesktop
        ? designTokens.layout.navigation.collapsedWidth
        : `${navigationWidth}px`,
    height: '100%',
    overflow: 'auto' as const,
    padding: '0.2rem',
  }

  if (!shouldShow) return null

  return (
    <>
      {/* Mobile overlay */}
      {isMobile && !navigationCollapsed && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            zIndex: designTokens.zIndex.overlay - 1,
          }}
        />
      )}

      <nav className={`fiovana-navigation ${className}`} style={navigationStyles}>
        <div className="navigation-content" style={navigationContentStyles}>
          {children}
        </div>
        {/* Drag handle for resizing */}
        {isDesktop && !navigationCollapsed && (
          <DragHandle
            position="right"
            isActive={isNavigationResizing}
            onMouseDown={startNavigationResize}
          />
        )}
      </nav>

      {/* Custom scrollbar styles for navigation panel */}
      <style>
        {`
          .fiovana-navigation .navigation-content::-webkit-scrollbar {
            width: 6px;
          }

          .fiovana-navigation .navigation-content::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .fiovana-navigation .navigation-content::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }

          .fiovana-navigation .navigation-content::-webkit-scrollbar-thumb:hover {
            background: ${designTokens.colors.border.strong};
          }
        `}
      </style>
    </>
  )
}

// Document Canvas
export interface CanvasProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export const Canvas: React.FC<CanvasProps> = ({ children, className = '', style }) => {
  const canvasStyles = {
    flex: 1,
    backgroundColor: designTokens.colors.background.canvas,
    overflow: 'auto' as const,
    position: 'relative' as const,
    minWidth: designTokens.layout.canvas.minWidth,
    padding: 0,
    ...style,
  }

  return (
    <div className={`fiovana-canvas ${className}`} style={canvasStyles}>
      {children}
    </div>
  )
}

// Intelligence Panel
export interface IntelligenceProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
}

export const Intelligence: React.FC<IntelligenceProps> = ({ children, className = '', style }) => {
  const {
    intelligenceCollapsed,
    isMobile,
    isDesktop,
    intelligenceWidth,
    isIntelligenceResizing,
    startIntelligenceResize,
  } = useLayout()

  const shouldShow = isDesktop && !intelligenceCollapsed

  const intelligenceStyles = {
    width: shouldShow ? `${intelligenceWidth}px` : '0',
    minWidth: shouldShow ? `${intelligenceWidth}px` : '0',
    maxWidth: designTokens.layout.intelligence.maxWidth,
    backgroundColor: designTokens.colors.surface.secondary,
    borderLeft: shouldShow ? `1px solid ${designTokens.colors.border.subtle}` : 'none',
    overflow: 'hidden' as const,
    transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
    position: isMobile ? ('absolute' as const) : ('relative' as const),
    top: isMobile ? 0 : 'auto',
    right: isMobile ? 0 : 'auto',
    height: isMobile ? '100%' : 'auto',
    zIndex: isMobile ? designTokens.zIndex.overlay : 'auto',
    flexShrink: 0,
    ...style,
  }

  const contentStyles = {
    width: `${intelligenceWidth}px`,
    height: '100%',
    overflow: 'auto' as const,
    padding: designTokens.spacing[4],
  }

  if (!shouldShow) return null

  return (
    <>
      {/* Mobile overlay for intelligence panel */}
      {isMobile && !intelligenceCollapsed && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            zIndex: designTokens.zIndex.overlay - 1,
          }}
        />
      )}

      <aside className={`fiovana-intelligence ${className}`} style={intelligenceStyles}>
        {/* Drag handle for resizing */}
        <DragHandle
          position="left"
          isActive={isIntelligenceResizing}
          onMouseDown={startIntelligenceResize}
        />
        <div className="intelligence-content" style={contentStyles}>
          {children}
        </div>
      </aside>

      {/* Custom scrollbar styles for intelligence panel */}
      <style>
        {`
          .fiovana-intelligence .intelligence-content::-webkit-scrollbar {
            width: 6px;
          }

          .fiovana-intelligence .intelligence-content::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .fiovana-intelligence .intelligence-content::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }

          .fiovana-intelligence .intelligence-content::-webkit-scrollbar-thumb:hover {
            background: ${designTokens.colors.border.strong};
          }
        `}
      </style>
    </>
  )
}

// Compound component pattern with proper typing
interface AppShellComponent extends React.FC<AppShellProps> {
  Header: typeof Header
  Main: typeof Main
  Navigation: typeof Navigation
  Canvas: typeof Canvas
  Intelligence: typeof Intelligence
}

const AppShellWithSubComponents = AppShell as AppShellComponent
AppShellWithSubComponents.Header = Header
AppShellWithSubComponents.Main = Main
AppShellWithSubComponents.Navigation = Navigation
AppShellWithSubComponents.Canvas = Canvas
AppShellWithSubComponents.Intelligence = Intelligence

export default AppShellWithSubComponents
