/**
 * Proxemic Design Token System
 *
 * Based on the refined visual architecture with "Conversational Intelligence with Enterprise Structure"
 * Color philosophy: Near-black surfaces with electric cyan accents for AI interactions
 */

// Core Color Palette - Refined
export const colors = {
  // Primary Surface System
  surface: {
    primary: '#0a0a0b', // Near-black with subtle blue undertones
    secondary: '#16161a', // Dark charcoal for panels
    tertiary: '#1f1f23', // Slightly lighter for elevated surfaces
    quaternary: '#28282d', // Hover states and active elements
  },

  // Background System
  background: {
    canvas: '#fafaf9', // Off-white for document viewing comfort
    paper: '#ffffff', // Pure white for document content
    overlay: 'rgba(10, 10, 11, 0.95)', // Modal overlay with transparency
  },

  // Accent Colors - Semantic
  accent: {
    ai: '#00d4ff', // Electric cyan for AI interactions
    success: '#00ff88', // Phosphorescent green for confirmations
    semantic: '#ffb700', // Golden amber for connections and relationships
    alert: '#ff5555', // Coral red for critical actions
    warning: '#ff8c00', // Orange for warnings
    info: '#4f9cff', // Bright blue for information
  },

  // Text Hierarchy
  text: {
    primary: '#ffffff', // Primary text on dark surfaces
    secondary: '#a8a8a8', // Secondary text, lower hierarchy
    tertiary: '#6b6b6b', // Tertiary text, subtle information
    inverse: '#0a0a0b', // Text on light backgrounds
    muted: '#4a4a4a', // Muted text for less important content
  },

  // Border System
  border: {
    subtle: '#2a2a2f', // Subtle borders on dark surfaces
    medium: '#3a3a3f', // Medium emphasis borders
    strong: '#4a4a4f', // Strong borders for definition
    accent: '#00d4ff', // Accent borders for focus states
  },

  // Glass Effects
  glass: {
    white10: 'rgba(255, 255, 255, 0.1)', // 10% white for glass elements
    white5: 'rgba(255, 255, 255, 0.05)', // 5% white for subtle glass
    backdrop: 'blur(10px)', // Backdrop filter for glass effect
  },

  // State Colors
  state: {
    hover: 'rgba(255, 255, 255, 0.05)', // Hover state overlay
    active: 'rgba(255, 255, 255, 0.1)', // Active state overlay
    focus: '#00d4ff', // Focus ring color
    disabled: '#2a2a2f', // Disabled state color
  },

  // Confidence Indicators
  confidence: {
    high: '#00ff88', // High confidence (green)
    medium: '#ffb700', // Medium confidence (amber)
    low: '#ff8c00', // Low confidence (orange)
    critical: '#ff5555', // Critical/error (red)
  },
} as const

// Typography Scale - Optimized for Reading
export const typography = {
  // Font Families
  fonts: {
    sans: [
      'Inter',
      '-apple-system',
      'BlinkMacSystemFont',
      'Segoe UI',
      'Roboto',
      'Helvetica Neue',
      'Arial',
      'sans-serif',
    ],
    mono: ['JetBrains Mono', 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'monospace'],
    display: ['Inter', 'system-ui', 'sans-serif'],
  },

  // Font Sizes - Harmonious Scale
  fontSize: {
    xs: '0.75rem', // 12px - Captions, badges
    sm: '0.875rem', // 14px - Small text, labels
    base: '1rem', // 16px - Body text, base size
    lg: '1.125rem', // 18px - Large body text
    xl: '1.25rem', // 20px - Headings, titles
    '2xl': '1.5rem', // 24px - Large headings
    '3xl': '1.875rem', // 30px - Display text
    '4xl': '2.25rem', // 36px - Large display
    '5xl': '3rem', // 48px - Hero text
  },

  // Font Weights
  fontWeight: {
    thin: '100',
    light: '300',
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
    extrabold: '800',
    black: '900',
  },

  // Line Heights - Optimal for Reading
  lineHeight: {
    tight: '1.25', // Headings
    snug: '1.375', // Subheadings
    normal: '1.5', // Body text
    relaxed: '1.625', // Large text
    loose: '2', // Spacious text
  },

  // Letter Spacing
  letterSpacing: {
    tighter: '-0.05em',
    tight: '-0.025em',
    normal: '0',
    wide: '0.025em',
    wider: '0.05em',
    widest: '0.1em',
  },
} as const

// Spacing System - 8px Grid
export const spacing = {
  px: '1px',
  0: '0',
  0.5: '0.125rem', // 2px
  1: '0.25rem', // 4px
  1.5: '0.375rem', // 6px
  2: '0.5rem', // 8px
  2.5: '0.625rem', // 10px
  3: '0.75rem', // 12px
  3.5: '0.875rem', // 14px
  4: '1rem', // 16px
  5: '1.25rem', // 20px
  6: '1.5rem', // 24px
  7: '1.75rem', // 28px
  8: '2rem', // 32px
  9: '2.25rem', // 36px
  10: '2.5rem', // 40px
  11: '2.75rem', // 44px
  12: '3rem', // 48px
  14: '3.5rem', // 56px
  16: '4rem', // 64px
  20: '5rem', // 80px
  24: '6rem', // 96px
  28: '7rem', // 112px
  32: '8rem', // 128px
  36: '9rem', // 144px
  40: '10rem', // 160px
  44: '11rem', // 176px
  48: '12rem', // 192px
  52: '13rem', // 208px
  56: '14rem', // 224px
  60: '15rem', // 240px
  64: '16rem', // 256px
  72: '18rem', // 288px
  80: '20rem', // 320px
  96: '24rem', // 384px
} as const

// Shadow System - Subtle Elevation
export const shadows = {
  none: 'none',
  sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  base: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
  md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
  lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
  xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
  '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
  inner: 'inset 0 2px 4px 0 rgba(0, 0, 0, 0.06)',

  // Glass effects with glow
  glassSubtle: '0 1px 3px 0 rgba(0, 212, 255, 0.1), 0 1px 2px 0 rgba(0, 212, 255, 0.06)',
  glassMedium: '0 4px 6px -1px rgba(0, 212, 255, 0.1), 0 2px 4px -1px rgba(0, 212, 255, 0.06)',
  glassStrong: '0 10px 15px -3px rgba(0, 212, 255, 0.15), 0 4px 6px -2px rgba(0, 212, 255, 0.1)',
} as const

// Border Radius System
export const borderRadius = {
  none: '0',
  sm: '0.125rem', // 2px
  base: '0.25rem', // 4px
  md: '0.375rem', // 6px
  lg: '0.5rem', // 8px
  xl: '0.75rem', // 12px
  '2xl': '1rem', // 16px
  '3xl': '1.5rem', // 24px
  full: '9999px', // Circular
} as const

// Animation System - Smooth & Purposeful
export const animation = {
  // Easing Curves
  easing: {
    linear: 'linear',
    ease: 'ease',
    easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
    easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
    easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    // Custom Proxemic curves
    smooth: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)',
    snappy: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
    gentle: 'cubic-bezier(0.25, 0.1, 0.25, 1)',
  },

  // Duration Tokens
  duration: {
    instant: '0ms',
    fast: '150ms', // Quick hover effects
    normal: '200ms', // Standard transitions
    slow: '300ms', // Panel slides
    slower: '500ms', // Complex animations
    slowest: '1000ms', // Page transitions
  },

  // Common Animations
  keyframes: {
    fadeIn: {
      from: { opacity: 0 },
      to: { opacity: 1 },
    },
    fadeInUp: {
      from: { opacity: 0, transform: 'translateY(20px)' },
      to: { opacity: 1, transform: 'translateY(0)' },
    },
    fadeOutDown: {
      from: { opacity: 1, transform: 'translateY(0)' },
      to: { opacity: 0, transform: 'translateY(20px)' },
    },
    slideUp: {
      from: { transform: 'translateY(10px)', opacity: 0 },
      to: { transform: 'translateY(0)', opacity: 1 },
    },
    slideDown: {
      from: { transform: 'translateY(-10px)', opacity: 0 },
      to: { transform: 'translateY(0)', opacity: 1 },
    },
    slideInRight: {
      from: { transform: 'translateX(100%)', opacity: 0 },
      to: { transform: 'translateX(0)', opacity: 1 },
    },
    slideInLeft: {
      from: { transform: 'translateX(-100%)', opacity: 0 },
      to: { transform: 'translateX(0)', opacity: 1 },
    },
    scaleIn: {
      from: { transform: 'scale(0.95)', opacity: 0 },
      to: { transform: 'scale(1)', opacity: 1 },
    },
    pulse: {
      '0%, 100%': { opacity: 1 },
      '50%': { opacity: 0.5 },
    },
    glow: {
      '0%, 100%': { boxShadow: '0 0 5px rgba(0, 212, 255, 0.5)' },
      '50%': { boxShadow: '0 0 20px rgba(0, 212, 255, 0.8)' },
    },
    spin: {
      from: { transform: 'rotate(0deg)' },
      to: { transform: 'rotate(360deg)' },
    },
    bounce: {
      '0%, 100%': { transform: 'translateY(0)' },
      '50%': { transform: 'translateY(-10px)' },
    },
  },

  // Pre-defined animation strings
  animations: {
    documentEnter: 'fadeInUp 0.3s ease-out',
    documentExit: 'fadeOutDown 0.2s ease-in',
    operationProgress: 'slideInRight 0.4s ease-out',
    aiThinking: 'pulse 2s ease-in-out infinite',
    messageSlideIn: 'slideInLeft 0.3s ease-out',
    cardHover: 'scaleIn 0.2s ease-out',
  },
} as const

// Responsive Breakpoints
export const breakpoints = {
  xs: '320px', // Mobile small
  sm: '640px', // Mobile
  md: '768px', // Tablet
  lg: '1024px', // Desktop small
  xl: '1280px', // Desktop
  '2xl': '1536px', // Desktop large

  // Proxemic-specific breakpoints
  mobile: '768px', // Below this is mobile
  tablet: '1024px', // Below this is tablet
  desktop: '1024px', // Above this is desktop
} as const

// Z-Index Scale
export const zIndex = {
  hide: -1,
  auto: 'auto',
  base: 0,
  docked: 10,
  dropdown: 1000,
  sticky: 1020,
  banner: 1030,
  overlay: 1040,
  modal: 1050,
  popover: 1060,
  skipLink: 1070,
  toast: 1080,
  tooltip: 1090,
  max: 2147483647,
} as const

// Layout Dimensions - Proxemic Specific
export const layout = {
  // Main Application Shell
  header: {
    height: '48px', // Ultra-minimal header height
  },

  navigation: {
    width: '240px', // Navigation panel width
    collapsedWidth: '60px', // Collapsed navigation width
  },

  intelligence: {
    width: '320px', // Intelligence panel width
    minWidth: '280px', // Minimum intelligence panel width
    maxWidth: '400px', // Maximum intelligence panel width
  },

  // Content Areas
  canvas: {
    minWidth: '400px', // Minimum canvas width
    maxWidth: '1200px', // Maximum canvas width for readability
    padding: '24px', // Canvas padding
  },

  // Document Typography
  document: {
    lineLength: '65ch', // Optimal line length (65-75 characters)
    lineHeight: '1.6', // Optimal line height for reading
    paragraphSpacing: '1.2em', // Paragraph spacing
  },

  // Modal System
  modal: {
    maxWidth: '600px', // Standard modal width
    maxWidthLarge: '900px', // Large modal width
    maxWidthSmall: '400px', // Small modal width
    padding: '24px', // Modal padding
  },
} as const

// Component Variants
export const variants = {
  // Button Variants
  button: {
    primary: {
      background: colors.accent.ai,
      color: colors.surface.primary,
      hover: 'rgba(0, 212, 255, 0.9)',
    },
    secondary: {
      background: colors.surface.secondary,
      color: colors.text.primary,
      hover: colors.surface.tertiary,
    },
    ghost: {
      background: 'transparent',
      color: colors.text.secondary,
      hover: colors.state.hover,
    },
    minimal: {
      background: 'transparent',
      color: colors.text.primary,
      hover: colors.state.hover,
    },
  },

  // Card Variants
  card: {
    default: {
      background: colors.surface.secondary,
      border: colors.border.subtle,
      shadow: shadows.base,
    },
    glass: {
      background: colors.glass.white10,
      backdropFilter: colors.glass.backdrop,
      border: colors.border.subtle,
      shadow: shadows.glassSubtle,
    },
    elevated: {
      background: colors.surface.tertiary,
      border: colors.border.medium,
      shadow: shadows.md,
    },
  },
} as const

// Export the complete design system
export const designTokens = {
  colors,
  typography,
  spacing,
  shadows,
  borderRadius,
  animation,
  breakpoints,
  zIndex,
  layout,
  variants,
} as const

export default designTokens
