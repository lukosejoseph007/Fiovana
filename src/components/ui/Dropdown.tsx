import React, { useState, useRef, useEffect, useCallback } from 'react';
import { designTokens } from '../../styles/tokens';

export interface DropdownOption {
  value: string;
  label: string;
  disabled?: boolean;
  icon?: React.ReactNode;
  description?: string;
}

export interface DropdownProps {
  options: DropdownOption[];
  value?: string;
  onChange?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  error?: string;
  size?: 'sm' | 'md' | 'lg';
  fullWidth?: boolean;
  searchable?: boolean;
  clearable?: boolean;
  maxHeight?: number;
  className?: string;
  dropdownClassName?: string;
}

const Dropdown: React.FC<DropdownProps> = ({
  options,
  value,
  onChange,
  placeholder = 'Select an option',
  disabled = false,
  error,
  size = 'md',
  fullWidth = false,
  searchable = false,
  clearable = false,
  maxHeight = 200,
  className = '',
  dropdownClassName = '',
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const selectedOption = options.find(option => option.value === value);
  const filteredOptions = searchable
    ? options.filter(option =>
        option.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
        option.value.toLowerCase().includes(searchTerm.toLowerCase())
      )
    : options;

  const handleSelect = useCallback((optionValue: string) => {
    onChange?.(optionValue);
    setIsOpen(false);
    setSearchTerm('');
    setHighlightedIndex(-1);
  }, [onChange]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setSearchTerm('');
        setHighlightedIndex(-1);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!isOpen) return;

      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault();
          setHighlightedIndex(prev =>
            prev < filteredOptions.length - 1 ? prev + 1 : 0
          );
          break;
        case 'ArrowUp':
          event.preventDefault();
          setHighlightedIndex(prev =>
            prev > 0 ? prev - 1 : filteredOptions.length - 1
          );
          break;
        case 'Enter':
          event.preventDefault();
          if (highlightedIndex >= 0 && !filteredOptions[highlightedIndex]?.disabled) {
            handleSelect(filteredOptions[highlightedIndex].value);
          }
          break;
        case 'Escape':
          setIsOpen(false);
          setSearchTerm('');
          setHighlightedIndex(-1);
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, highlightedIndex, filteredOptions, handleSelect]);

  const handleClear = (event: React.MouseEvent) => {
    event.stopPropagation();
    onChange?.('');
    setSearchTerm('');
  };

  const toggleDropdown = () => {
    if (disabled) return;
    setIsOpen(!isOpen);
    if (searchable && !isOpen) {
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  };

  // Size variants
  const sizeStyles = {
    sm: {
      height: '32px',
      padding: `0 ${designTokens.spacing[8]} 0 ${designTokens.spacing[3]}`,
      fontSize: designTokens.typography.fontSize.sm,
    },
    md: {
      height: '40px',
      padding: `0 ${designTokens.spacing[10]} 0 ${designTokens.spacing[4]}`,
      fontSize: designTokens.typography.fontSize.base,
    },
    lg: {
      height: '48px',
      padding: `0 ${designTokens.spacing[12]} 0 ${designTokens.spacing[5]}`,
      fontSize: designTokens.typography.fontSize.lg,
    },
  };

  const triggerStyles = {
    position: 'relative' as const,
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    width: fullWidth ? '100%' : 'auto',
    minWidth: '120px',
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${error ? designTokens.colors.accent.alert : designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    color: designTokens.colors.text.primary,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
    cursor: disabled ? 'not-allowed' : 'pointer',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    opacity: disabled ? 0.5 : 1,
    outline: 'none',
    ...sizeStyles[size],
  };

  const dropdownStyles = {
    position: 'absolute' as const,
    top: '100%',
    left: 0,
    right: 0,
    marginTop: '4px',
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.medium}`,
    borderRadius: designTokens.borderRadius.md,
    boxShadow: designTokens.shadows.lg,
    zIndex: designTokens.zIndex.dropdown,
    maxHeight: `${maxHeight}px`,
    overflowY: 'auto' as const,
    animation: 'slideDown 0.15s ease-out',
  };

  const optionStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    padding: `${designTokens.spacing[2]} ${designTokens.spacing[3]}`,
    cursor: 'pointer',
    fontSize: sizeStyles[size].fontSize,
    color: designTokens.colors.text.primary,
    transition: `background-color ${designTokens.animation.duration.fast}`,
  };

  const searchInputStyles = {
    width: '100%',
    padding: designTokens.spacing[3],
    border: 'none',
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    backgroundColor: 'transparent',
    color: designTokens.colors.text.primary,
    fontSize: sizeStyles[size].fontSize,
    outline: 'none',
    fontFamily: designTokens.typography.fonts.sans.join(', '),
  };

  const chevronStyles = {
    position: 'absolute' as const,
    right: designTokens.spacing[3],
    color: designTokens.colors.text.secondary,
    transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)',
    transition: `transform ${designTokens.animation.duration.fast}`,
  };

  const clearButtonStyles = {
    position: 'absolute' as const,
    right: clearable ? designTokens.spacing[8] : designTokens.spacing[3],
    color: designTokens.colors.text.secondary,
    cursor: 'pointer',
    padding: '2px',
    borderRadius: designTokens.borderRadius.sm,
    transition: `color ${designTokens.animation.duration.fast}`,
  };

  return (
    <>
      <style>
        {`
          @keyframes slideDown {
            from {
              opacity: 0;
              transform: translateY(-8px);
            }
            to {
              opacity: 1;
              transform: translateY(0);
            }
          }

          .proxemic-dropdown-trigger:hover {
            ${!disabled ? `
              border-color: ${designTokens.colors.border.medium};
            ` : ''}
          }

          .proxemic-dropdown-trigger:focus {
            border-color: ${designTokens.colors.state.focus};
            box-shadow: 0 0 0 3px ${designTokens.colors.state.focus}40;
          }

          .proxemic-dropdown-option:hover {
            background-color: ${designTokens.colors.state.hover};
          }

          .proxemic-dropdown-option-highlighted {
            background-color: ${designTokens.colors.state.hover};
          }

          .proxemic-dropdown-option-disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }

          .proxemic-dropdown-clear:hover {
            color: ${designTokens.colors.text.primary};
          }

          .proxemic-dropdown::-webkit-scrollbar {
            width: 6px;
          }

          .proxemic-dropdown::-webkit-scrollbar-track {
            background: ${designTokens.colors.surface.tertiary};
            border-radius: 3px;
          }

          .proxemic-dropdown::-webkit-scrollbar-thumb {
            background: ${designTokens.colors.border.medium};
            border-radius: 3px;
          }
        `}
      </style>

      <div ref={dropdownRef} style={{ position: 'relative', width: fullWidth ? '100%' : 'auto' }}>
        <div
          className={`proxemic-dropdown-trigger ${className}`}
          style={triggerStyles}
          onClick={toggleDropdown}
          tabIndex={disabled ? -1 : 0}
          role="combobox"
          aria-expanded={isOpen}
          aria-haspopup="listbox"
        >
          <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {selectedOption ? (
              <span style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
                {selectedOption.icon}
                {selectedOption.label}
              </span>
            ) : (
              <span style={{ color: designTokens.colors.text.tertiary }}>{placeholder}</span>
            )}
          </span>

          {clearable && selectedOption && (
            <button
              className="proxemic-dropdown-clear"
              style={clearButtonStyles}
              onClick={handleClear}
              aria-label="Clear selection"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          )}

          <div style={chevronStyles}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="6,9 12,15 18,9" />
            </svg>
          </div>
        </div>

        {isOpen && (
          <div
            className={`proxemic-dropdown ${dropdownClassName}`}
            style={dropdownStyles}
            role="listbox"
          >
            {searchable && (
              <input
                ref={inputRef}
                style={searchInputStyles}
                placeholder="Search..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            )}

            {filteredOptions.length === 0 ? (
              <div style={{ ...optionStyles, cursor: 'default', color: designTokens.colors.text.tertiary }}>
                No options found
              </div>
            ) : (
              filteredOptions.map((option, index) => (
                <div
                  key={option.value}
                  className={`proxemic-dropdown-option ${
                    index === highlightedIndex ? 'proxemic-dropdown-option-highlighted' : ''
                  } ${option.disabled ? 'proxemic-dropdown-option-disabled' : ''}`}
                  style={optionStyles}
                  onClick={() => !option.disabled && handleSelect(option.value)}
                  role="option"
                  aria-selected={option.value === value}
                >
                  {option.icon}
                  <div>
                    <div>{option.label}</div>
                    {option.description && (
                      <div style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                        marginTop: '2px',
                      }}>
                        {option.description}
                      </div>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {error && (
          <div style={{
            marginTop: designTokens.spacing[1],
            color: designTokens.colors.accent.alert,
            fontSize: designTokens.typography.fontSize.sm,
          }}>
            {error}
          </div>
        )}
      </div>
    </>
  );
};

export default Dropdown;