import React, { useEffect, useRef } from 'react';
import { designTokens } from '../../styles/tokens';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  variant?: 'default' | 'glass';
  closeOnOverlayClick?: boolean;
  closeOnEscape?: boolean;
  showCloseButton?: boolean;
  title?: string;
  children: React.ReactNode;
  className?: string;
  overlayClassName?: string;
}

const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  size = 'md',
  variant = 'default',
  closeOnOverlayClick = true,
  closeOnEscape = true,
  showCloseButton = true,
  title,
  children,
  className = '',
  overlayClassName = '',
}) => {
  const modalRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  // Handle escape key
  useEffect(() => {
    if (!closeOnEscape) return;

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, onClose, closeOnEscape]);

  // Focus management
  useEffect(() => {
    if (isOpen) {
      // Store the previously focused element
      previousFocusRef.current = document.activeElement as HTMLElement;

      // Focus the modal
      if (modalRef.current) {
        modalRef.current.focus();
      }

      // Prevent body scroll
      document.body.style.overflow = 'hidden';
    } else {
      // Restore focus
      if (previousFocusRef.current) {
        previousFocusRef.current.focus();
      }

      // Restore body scroll
      document.body.style.overflow = 'unset';
    }

    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  // Handle overlay click
  const handleOverlayClick = (event: React.MouseEvent) => {
    if (closeOnOverlayClick && event.target === event.currentTarget) {
      onClose();
    }
  };

  if (!isOpen) return null;

  const overlayStyles = {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: designTokens.colors.background.overlay,
    backdropFilter: 'blur(10px)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: designTokens.spacing[4],
    zIndex: designTokens.zIndex.modal,
    animation: 'fadeIn 0.2s ease-out',
  };

  // Size variants
  const sizeStyles = {
    sm: {
      maxWidth: designTokens.layout.modal.maxWidthSmall,
      width: '100%',
    },
    md: {
      maxWidth: designTokens.layout.modal.maxWidth,
      width: '100%',
    },
    lg: {
      maxWidth: designTokens.layout.modal.maxWidthLarge,
      width: '100%',
    },
    xl: {
      maxWidth: '1200px',
      width: '100%',
    },
    full: {
      width: '95vw',
      height: '95vh',
      maxWidth: 'none',
      maxHeight: 'none',
    },
  };

  const modalStyles = {
    backgroundColor: variant === 'glass'
      ? designTokens.colors.glass.white10
      : designTokens.colors.surface.secondary,
    backdropFilter: variant === 'glass' ? designTokens.colors.glass.backdrop : 'none',
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.xl,
    boxShadow: variant === 'glass'
      ? designTokens.shadows.glassStrong
      : designTokens.shadows['2xl'],
    outline: 'none',
    overflow: 'hidden',
    animation: 'scaleIn 0.2s ease-out',
    ...sizeStyles[size],
  };

  const headerStyles = {
    padding: designTokens.layout.modal.padding,
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  };

  const titleStyles = {
    fontSize: designTokens.typography.fontSize.xl,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    margin: 0,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
  };

  const closeButtonStyles = {
    background: 'none',
    border: 'none',
    color: designTokens.colors.text.secondary,
    cursor: 'pointer',
    padding: designTokens.spacing[2],
    borderRadius: designTokens.borderRadius.md,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '32px',
    height: '32px',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
  };

  const contentStyles = {
    padding: designTokens.layout.modal.padding,
    maxHeight: size === 'full' ? 'calc(95vh - 120px)' : '70vh',
    overflowY: 'auto' as const,
    color: designTokens.colors.text.primary,
  };

  return (
    <>
      <style>
        {`
          @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
          }

          @keyframes scaleIn {
            from {
              opacity: 0;
              transform: scale(0.95) translateY(10px);
            }
            to {
              opacity: 1;
              transform: scale(1) translateY(0);
            }
          }

          .proxemic-modal-close:hover {
            background-color: ${designTokens.colors.state.hover};
            color: ${designTokens.colors.text.primary};
          }

          .proxemic-modal-close:focus {
            outline: none;
            background-color: ${designTokens.colors.state.hover};
            box-shadow: 0 0 0 2px ${designTokens.colors.state.focus}40;
          }

          .proxemic-modal-content::-webkit-scrollbar {
            width: 6px;
          }

          .proxemic-modal-content::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .proxemic-modal-content::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }

          .proxemic-modal-content::-webkit-scrollbar-thumb:hover {
            background: ${designTokens.colors.border.strong};
          }
        `}
      </style>

      <div
        className={`proxemic-modal-overlay ${overlayClassName}`}
        style={overlayStyles}
        onClick={handleOverlayClick}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? 'modal-title' : undefined}
      >
        <div
          ref={modalRef}
          className={`proxemic-modal ${className}`}
          style={modalStyles}
          tabIndex={-1}
        >
          {(title || showCloseButton) && (
            <div style={headerStyles}>
              {title && (
                <h2 id="modal-title" style={titleStyles}>
                  {title}
                </h2>
              )}
              {showCloseButton && (
                <button
                  className="proxemic-modal-close"
                  style={closeButtonStyles}
                  onClick={onClose}
                  aria-label="Close modal"
                >
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              )}
            </div>
          )}

          <div
            className="proxemic-modal-content"
            style={contentStyles}
          >
            {children}
          </div>
        </div>
      </div>
    </>
  );
};

export default Modal;