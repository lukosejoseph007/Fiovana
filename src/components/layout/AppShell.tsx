import React, { useState, useEffect } from 'react';
import { designTokens } from '../../styles/tokens';
import { LayoutContext } from './useLayoutContext';

export interface AppShellProps {
  children: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export interface LayoutContextType {
  navigationCollapsed: boolean;
  intelligenceCollapsed: boolean;
  toggleNavigation: () => void;
  toggleIntelligence: () => void;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
}

const AppShell: React.FC<AppShellProps> = ({ children, className = '', style }) => {
  const [navigationCollapsed, setNavigationCollapsed] = useState(false);
  const [intelligenceCollapsed, setIntelligenceCollapsed] = useState(false);
  const [viewport, setViewport] = useState({ width: 0, height: 0 });

  // Responsive breakpoint detection
  useEffect(() => {
    const updateViewport = () => {
      setViewport({ width: window.innerWidth, height: window.innerHeight });
    };

    updateViewport();
    window.addEventListener('resize', updateViewport);
    return () => window.removeEventListener('resize', updateViewport);
  }, []);

  const isMobile = viewport.width < parseInt(designTokens.breakpoints.mobile);
  const isTablet = viewport.width >= parseInt(designTokens.breakpoints.mobile) &&
                   viewport.width < parseInt(designTokens.breakpoints.desktop);
  const isDesktop = viewport.width >= parseInt(designTokens.breakpoints.desktop);

  // Auto-collapse panels on smaller screens
  useEffect(() => {
    if (isMobile) {
      setNavigationCollapsed(true);
      setIntelligenceCollapsed(true);
    } else if (isTablet) {
      setIntelligenceCollapsed(true);
    }
  }, [isMobile, isTablet]);

  const toggleNavigation = () => setNavigationCollapsed(!navigationCollapsed);
  const toggleIntelligence = () => setIntelligenceCollapsed(!intelligenceCollapsed);

  const layoutContextValue: LayoutContextType = {
    navigationCollapsed,
    intelligenceCollapsed,
    toggleNavigation,
    toggleIntelligence,
    isMobile,
    isTablet,
    isDesktop,
  };

  const containerStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100vh',
    backgroundColor: designTokens.colors.surface.primary,
    color: designTokens.colors.text.primary,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
    overflow: 'hidden',
    ...style,
  };


  return (
    <LayoutContext.Provider value={layoutContextValue}>
      <div className={`proxemic-app-shell ${className}`} style={containerStyles}>
        {children}
      </div>
    </LayoutContext.Provider>
  );
};

// Header Component
export interface HeaderProps {
  children?: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export const Header: React.FC<HeaderProps> = ({ children, className = '', style }) => {
  const { isMobile, toggleNavigation, toggleIntelligence } = useLayout();

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
  };

  return (
    <header className={`proxemic-header ${className}`} style={headerStyles}>
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
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
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
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="3" />
              <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1" />
            </svg>
          </button>
        </div>
      )}
    </header>
  );
};

// Main Layout Container
export interface MainProps {
  children: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export const Main: React.FC<MainProps> = ({ children, className = '', style }) => {
  const mainStyles = {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
    position: 'relative' as const,
    ...style,
  };

  return (
    <main className={`proxemic-main ${className}`} style={mainStyles}>
      {children}
    </main>
  );
};

// Navigation Panel
export interface NavigationProps {
  children: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export const Navigation: React.FC<NavigationProps> = ({ children, className = '', style }) => {
  const { navigationCollapsed, isMobile, isDesktop } = useLayout();

  const shouldShow = isDesktop || (!navigationCollapsed && !isDesktop);
  const width = navigationCollapsed && isDesktop
    ? designTokens.layout.navigation.collapsedWidth
    : designTokens.layout.navigation.width;

  const navigationStyles = {
    width: shouldShow ? width : '0',
    minWidth: shouldShow ? width : '0',
    backgroundColor: designTokens.colors.surface.secondary,
    borderRight: shouldShow ? `1px solid ${designTokens.colors.border.subtle}` : 'none',
    overflow: 'hidden' as const,
    transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
    position: isMobile ? 'absolute' as const : 'relative' as const,
    top: isMobile ? 0 : 'auto',
    left: isMobile ? 0 : 'auto',
    height: isMobile ? '100%' : 'auto',
    zIndex: isMobile ? designTokens.zIndex.overlay : 'auto',
    flexShrink: 0,
    ...style,
  };

  const contentStyles = {
    width: designTokens.layout.navigation.width,
    height: '100%',
    overflow: 'auto' as const,
    padding: designTokens.spacing[4],
  };

  if (!shouldShow) return null;

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

      <nav className={`proxemic-navigation ${className}`} style={navigationStyles}>
        <div style={contentStyles}>
          {children}
        </div>
      </nav>
    </>
  );
};

// Document Canvas
export interface CanvasProps {
  children: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export const Canvas: React.FC<CanvasProps> = ({ children, className = '', style }) => {
  const canvasStyles = {
    flex: 1,
    backgroundColor: designTokens.colors.background.canvas,
    overflow: 'auto' as const,
    position: 'relative' as const,
    minWidth: designTokens.layout.canvas.minWidth,
    padding: designTokens.layout.canvas.padding,
    ...style,
  };

  return (
    <div className={`proxemic-canvas ${className}`} style={canvasStyles}>
      {children}
    </div>
  );
};

// Intelligence Panel
export interface IntelligenceProps {
  children: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export const Intelligence: React.FC<IntelligenceProps> = ({ children, className = '', style }) => {
  const { intelligenceCollapsed, isMobile, isDesktop } = useLayout();

  const shouldShow = isDesktop && !intelligenceCollapsed;

  const intelligenceStyles = {
    width: shouldShow ? designTokens.layout.intelligence.width : '0',
    minWidth: shouldShow ? designTokens.layout.intelligence.minWidth : '0',
    maxWidth: designTokens.layout.intelligence.maxWidth,
    backgroundColor: designTokens.colors.surface.secondary,
    borderLeft: shouldShow ? `1px solid ${designTokens.colors.border.subtle}` : 'none',
    overflow: 'hidden' as const,
    transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
    position: isMobile ? 'absolute' as const : 'relative' as const,
    top: isMobile ? 0 : 'auto',
    right: isMobile ? 0 : 'auto',
    height: isMobile ? '100%' : 'auto',
    zIndex: isMobile ? designTokens.zIndex.overlay : 'auto',
    flexShrink: 0,
    ...style,
  };

  const contentStyles = {
    width: designTokens.layout.intelligence.width,
    height: '100%',
    overflow: 'auto' as const,
    padding: designTokens.spacing[4],
  };

  if (!shouldShow) return null;

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

      <aside className={`proxemic-intelligence ${className}`} style={intelligenceStyles}>
        <div style={contentStyles}>
          {children}
        </div>
      </aside>
    </>
  );
};

// Compound component pattern
AppShell.Header = Header;
AppShell.Main = Main;
AppShell.Navigation = Navigation;
AppShell.Canvas = Canvas;
AppShell.Intelligence = Intelligence;

export default AppShell;