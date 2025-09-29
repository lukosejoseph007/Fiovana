import { useState, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'

export interface ResponsiveState {
  width: number
  height: number
  isMobile: boolean
  isTablet: boolean
  isDesktop: boolean
  breakpoint: 'mobile' | 'tablet' | 'desktop'
}

export const useResponsive = (): ResponsiveState => {
  const [viewport, setViewport] = useState({ width: 0, height: 0 })

  useEffect(() => {
    const updateViewport = () => {
      setViewport({
        width: window.innerWidth,
        height: window.innerHeight,
      })
    }

    // Set initial values
    updateViewport()

    // Listen for resize events
    window.addEventListener('resize', updateViewport)

    // Cleanup
    return () => window.removeEventListener('resize', updateViewport)
  }, [])

  const isMobile = viewport.width < parseInt(designTokens.breakpoints.mobile)
  const isTablet =
    viewport.width >= parseInt(designTokens.breakpoints.mobile) &&
    viewport.width < parseInt(designTokens.breakpoints.desktop)
  const isDesktop = viewport.width >= parseInt(designTokens.breakpoints.desktop)

  const getBreakpoint = (): 'mobile' | 'tablet' | 'desktop' => {
    if (isMobile) return 'mobile'
    if (isTablet) return 'tablet'
    return 'desktop'
  }

  return {
    width: viewport.width,
    height: viewport.height,
    isMobile,
    isTablet,
    isDesktop,
    breakpoint: getBreakpoint(),
  }
}

export default useResponsive
