// Layout System for Fiovana
// Adaptive three-column layout with responsive behavior

export { default as AppShell } from './AppShell'
export { useLayout } from './useLayoutContext'
export type {
  AppShellProps,
  LayoutContextType,
  HeaderProps,
  MainProps,
  NavigationProps,
  CanvasProps,
  IntelligenceProps,
} from './AppShell'

export { default as HeaderBar } from './HeaderBar'
export type { HeaderBarProps } from './HeaderBar'

export { default as useResponsive } from './useResponsive'
export type { ResponsiveState } from './useResponsive'

// Re-export design tokens for layout consistency
export { designTokens } from '../../styles/tokens'
