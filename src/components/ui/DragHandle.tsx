import React from 'react'
import { designTokens } from '../../styles/tokens'

export interface DragHandleProps {
  position: 'left' | 'right'
  isActive?: boolean
  onMouseDown: (e: React.MouseEvent) => void
  className?: string
  style?: React.CSSProperties
}

const DragHandle: React.FC<DragHandleProps> = ({
  position,
  isActive = false,
  onMouseDown,
  className = '',
  style,
}) => {
  const handleStyles: React.CSSProperties = {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '4px',
    cursor: 'col-resize',
    backgroundColor: isActive ? designTokens.colors.accent.ai : 'transparent',
    borderRadius: designTokens.borderRadius.sm,
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    zIndex: designTokens.zIndex.docked,
    [position === 'right' ? 'right' : 'left']: '-2px',
    ...style,
  }

  const innerHandleStyles: React.CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    width: '2px',
    height: '40px',
    backgroundColor: isActive ? designTokens.colors.accent.ai : designTokens.colors.border.medium,
    borderRadius: designTokens.borderRadius.sm,
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    opacity: isActive ? 1 : 0.6,
  }

  return (
    <>
      <div
        className={`proxemic-drag-handle ${className}`}
        style={handleStyles}
        onMouseDown={onMouseDown}
        data-testid={`drag-handle-${position}`}
      >
        <div style={innerHandleStyles} />
      </div>

      {/* Hover and active styles */}
      <style>
        {`
          .proxemic-drag-handle:hover {
            background-color: ${designTokens.colors.accent.ai}40 !important;
          }

          .proxemic-drag-handle:hover > div {
            opacity: 1 !important;
            background-color: ${designTokens.colors.accent.ai} !important;
          }

          .proxemic-drag-handle:active {
            background-color: ${designTokens.colors.accent.ai}60 !important;
          }

          .proxemic-drag-handle:active > div {
            background-color: ${designTokens.colors.accent.ai} !important;
            height: 60px !important;
          }
        `}
      </style>
    </>
  )
}

export default DragHandle
