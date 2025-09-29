import React, { useState, useRef, useEffect, useCallback } from 'react';
import { designTokens } from '../../styles/tokens';

export interface TooltipProps {
  content: React.ReactNode;
  children: React.ReactNode;
  placement?: 'top' | 'bottom' | 'left' | 'right';
  delay?: number;
  disabled?: boolean;
  className?: string;
  contentClassName?: string;
}

const Tooltip: React.FC<TooltipProps> = ({
  content,
  children,
  placement = 'top',
  delay = 1000,
  disabled = false,
  className = '',
  contentClassName = '',
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const triggerRef = useRef<HTMLDivElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const timeoutRef = useRef<NodeJS.Timeout>();

  const showTooltip = () => {
    if (disabled) return;

    timeoutRef.current = setTimeout(() => {
      setIsVisible(true);
      calculatePosition();
    }, delay);
  };

  const hideTooltip = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  };

  const calculatePosition = useCallback(() => {
    if (!triggerRef.current || !tooltipRef.current) return;

    const triggerRect = triggerRef.current.getBoundingClientRect();
    const tooltipRect = tooltipRef.current.getBoundingClientRect();
    const viewport = {
      width: window.innerWidth,
      height: window.innerHeight,
    };

    let top = 0;
    let left = 0;

    // Calculate base position based on placement
    switch (placement) {
      case 'top':
        top = triggerRect.top - tooltipRect.height - 8;
        left = triggerRect.left + (triggerRect.width / 2) - (tooltipRect.width / 2);
        break;
      case 'bottom':
        top = triggerRect.bottom + 8;
        left = triggerRect.left + (triggerRect.width / 2) - (tooltipRect.width / 2);
        break;
      case 'left':
        top = triggerRect.top + (triggerRect.height / 2) - (tooltipRect.height / 2);
        left = triggerRect.left - tooltipRect.width - 8;
        break;
      case 'right':
        top = triggerRect.top + (triggerRect.height / 2) - (tooltipRect.height / 2);
        left = triggerRect.right + 8;
        break;
    }

    // Adjust for viewport boundaries
    if (left < 8) {
      left = 8;
    } else if (left + tooltipRect.width > viewport.width - 8) {
      left = viewport.width - tooltipRect.width - 8;
    }

    if (top < 8) {
      top = 8;
    } else if (top + tooltipRect.height > viewport.height - 8) {
      top = viewport.height - tooltipRect.height - 8;
    }

    setPosition({ top, left });
  }, [placement]);

  useEffect(() => {
    if (isVisible) {
      calculatePosition();
      window.addEventListener('scroll', calculatePosition);
      window.addEventListener('resize', calculatePosition);

      return () => {
        window.removeEventListener('scroll', calculatePosition);
        window.removeEventListener('resize', calculatePosition);
      };
    }
  }, [isVisible, placement, calculatePosition]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const triggerStyles = {
    display: 'inline-block',
    cursor: 'help',
  };

  const tooltipStyles = {
    position: 'fixed' as const,
    top: position.top,
    left: position.left,
    backgroundColor: designTokens.colors.surface.primary,
    color: designTokens.colors.text.primary,
    padding: `${designTokens.spacing[2]} ${designTokens.spacing[3]}`,
    borderRadius: designTokens.borderRadius.md,
    border: `1px solid ${designTokens.colors.border.medium}`,
    boxShadow: designTokens.shadows.lg,
    fontSize: designTokens.typography.fontSize.sm,
    lineHeight: designTokens.typography.lineHeight.snug,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
    maxWidth: '300px',
    zIndex: designTokens.zIndex.tooltip,
    opacity: isVisible ? 1 : 0,
    visibility: isVisible ? 'visible' as const : 'hidden' as const,
    transition: `opacity ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}, visibility ${designTokens.animation.duration.fast}`,
    pointerEvents: 'none' as const,
    wordWrap: 'break-word' as const,
  };

  // Arrow styles based on placement
  const getArrowStyles = () => {
    const arrowSize = 6;
    const arrowColor = designTokens.colors.surface.primary;
    // const borderColor = designTokens.colors.border.medium; // TODO: Use for arrow border

    const baseArrowStyles = {
      position: 'absolute' as const,
      width: 0,
      height: 0,
      border: `${arrowSize}px solid transparent`,
    };

    switch (placement) {
      case 'top':
        return {
          ...baseArrowStyles,
          bottom: `-${arrowSize * 2}px`,
          left: '50%',
          transform: 'translateX(-50%)',
          borderTopColor: arrowColor,
          borderBottom: 'none',
        };
      case 'bottom':
        return {
          ...baseArrowStyles,
          top: `-${arrowSize * 2}px`,
          left: '50%',
          transform: 'translateX(-50%)',
          borderBottomColor: arrowColor,
          borderTop: 'none',
        };
      case 'left':
        return {
          ...baseArrowStyles,
          right: `-${arrowSize * 2}px`,
          top: '50%',
          transform: 'translateY(-50%)',
          borderLeftColor: arrowColor,
          borderRight: 'none',
        };
      case 'right':
        return {
          ...baseArrowStyles,
          left: `-${arrowSize * 2}px`,
          top: '50%',
          transform: 'translateY(-50%)',
          borderRightColor: arrowColor,
          borderLeft: 'none',
        };
      default:
        return {};
    }
  };

  return (
    <>
      <div
        ref={triggerRef}
        className={`proxemic-tooltip-trigger ${className}`}
        style={triggerStyles}
        onMouseEnter={showTooltip}
        onMouseLeave={hideTooltip}
        onFocus={showTooltip}
        onBlur={hideTooltip}
      >
        {children}
      </div>

      {!disabled && (
        <div
          ref={tooltipRef}
          className={`proxemic-tooltip ${contentClassName}`}
          style={tooltipStyles}
          role="tooltip"
        >
          {content}
          <div style={getArrowStyles()} />
        </div>
      )}
    </>
  );
};

export default Tooltip;