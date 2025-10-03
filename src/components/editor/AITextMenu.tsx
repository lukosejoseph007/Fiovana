// src/components/editor/AITextMenu.tsx
import React, { useState, useRef, useEffect } from 'react'
import {
  Lightbulb,
  BookOpen,
  Sparkles,
  ZapIcon,
  PenLine,
  CheckCircle,
  FileText,
  Languages,
  Link2,
  MessageSquare,
  X,
} from 'lucide-react'
import { TextOperation, TextOperations } from '../../services/textOperationService'

interface AITextMenuProps {
  position: { x: number; y: number }
  selectedText: string
  onOperationSelect: (operation: TextOperation) => void
  onClose: () => void
}

interface AIOperation {
  id: string
  label: string
  icon: React.ReactNode
  operation: () => TextOperation
  description: string
  requiresParams?: boolean
}

const AI_OPERATIONS: AIOperation[] = [
  {
    id: 'define',
    label: 'Define',
    icon: <BookOpen className="h-4 w-4" />,
    operation: TextOperations.define,
    description: 'Show definition from context',
  },
  {
    id: 'explain',
    label: 'Explain',
    icon: <Lightbulb className="h-4 w-4" />,
    operation: TextOperations.explain,
    description: 'Explain concept in simpler terms',
  },
  {
    id: 'expand',
    label: 'Expand',
    icon: <Sparkles className="h-4 w-4" />,
    operation: TextOperations.expand,
    description: 'Add more details/examples',
  },
  {
    id: 'simplify',
    label: 'Simplify',
    icon: <ZapIcon className="h-4 w-4" />,
    operation: TextOperations.simplify,
    description: 'Make it easier to understand',
  },
  {
    id: 'rewrite',
    label: 'Rewrite',
    icon: <PenLine className="h-4 w-4" />,
    operation: () => TextOperations.rewrite(),
    description: 'Rewrite in different tone/style',
  },
  {
    id: 'improve',
    label: 'Improve',
    icon: <CheckCircle className="h-4 w-4" />,
    operation: TextOperations.improve,
    description: 'Grammar and clarity improvements',
  },
  {
    id: 'summarize',
    label: 'Summarize',
    icon: <FileText className="h-4 w-4" />,
    operation: () => TextOperations.summarize('medium'),
    description: 'Create brief summary',
  },
  {
    id: 'translate',
    label: 'Translate',
    icon: <Languages className="h-4 w-4" />,
    operation: () => TextOperations.translate('Spanish'),
    description: 'Translate to another language',
    requiresParams: true,
  },
  {
    id: 'related',
    label: 'Find Related',
    icon: <Link2 className="h-4 w-4" />,
    operation: TextOperations.findRelated,
    description: 'Find related content in workspace',
  },
  {
    id: 'custom',
    label: 'Ask AI',
    icon: <MessageSquare className="h-4 w-4" />,
    operation: () => TextOperations.custom(''),
    description: 'Custom question about selected text',
    requiresParams: true,
  },
]

export const AITextMenu: React.FC<AITextMenuProps> = ({
  position,
  selectedText,
  onOperationSelect,
  onClose,
}) => {
  const menuRef = useRef<HTMLDivElement>(null)
  const [showCustomInput, setShowCustomInput] = useState(false)
  const [customPrompt, setCustomPrompt] = useState('')
  const [showTranslateInput, setShowTranslateInput] = useState(false)
  const [targetLanguage, setTargetLanguage] = useState('Spanish')
  const [isDragging, setIsDragging] = useState(false)
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 })
  const [menuPosition, setMenuPosition] = useState(position)

  // Update menu position when prop changes and ensure it stays within viewport
  useEffect(() => {
    // Use the position as-is since it's already calculated to be within viewport bounds
    setMenuPosition(position)
  }, [position])

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [onClose])

  // Close menu on Escape key
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
      }
    }

    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [onClose])

  // Handle drag functionality
  useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      setMenuPosition({
        x: e.clientX - dragOffset.x,
        y: e.clientY - dragOffset.y,
      })
    }

    const handleMouseUp = () => {
      setIsDragging(false)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, dragOffset])

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!menuRef.current) return

    const rect = menuRef.current.getBoundingClientRect()
    setDragOffset({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    })
    setIsDragging(true)
  }

  const handleOperationClick = (operation: AIOperation) => {
    if (operation.id === 'custom') {
      setShowCustomInput(true)
      return
    }

    if (operation.id === 'translate') {
      setShowTranslateInput(true)
      return
    }

    onOperationSelect(operation.operation())
  }

  const handleCustomSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (customPrompt.trim()) {
      onOperationSelect(TextOperations.custom(customPrompt))
      setShowCustomInput(false)
      setCustomPrompt('')
    }
  }

  const handleTranslateSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (targetLanguage.trim()) {
      onOperationSelect(TextOperations.translate(targetLanguage))
      setShowTranslateInput(false)
    }
  }

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 min-w-[280px] max-w-[320px]"
      style={{
        left: `${menuPosition.x}px`,
        top: `${menuPosition.y}px`,
        transform: 'translate(-50%, 0)',
        cursor: isDragging ? 'grabbing' : 'default',
      }}
    >
      {/* Header */}
      <div
        className="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700 cursor-grab active:cursor-grabbing"
        onMouseDown={handleMouseDown}
      >
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-blue-500" />
          <span className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            AI Text Assistant
          </span>
        </div>
        <button
          onClick={onClose}
          onMouseDown={e => e.stopPropagation()}
          className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
          aria-label="Close menu"
        >
          <X className="h-4 w-4 text-gray-500 dark:text-gray-400" />
        </button>
      </div>

      {/* Selected text preview */}
      <div className="p-3 bg-gray-50 dark:bg-gray-900/50 border-b border-gray-200 dark:border-gray-700">
        <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Selected text:</p>
        <p className="text-sm text-gray-700 dark:text-gray-300 line-clamp-2">{selectedText}</p>
      </div>

      {/* Custom prompt input */}
      {showCustomInput && (
        <div className="p-3 border-b border-gray-200 dark:border-gray-700">
          <form onSubmit={handleCustomSubmit}>
            <label className="block text-xs text-gray-500 dark:text-gray-400 mb-2">
              Ask AI about this text:
            </label>
            <textarea
              value={customPrompt}
              onChange={e => setCustomPrompt(e.target.value)}
              placeholder="What would you like to know?"
              className="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
              rows={3}
              autoFocus
            />
            <div className="flex gap-2 mt-2">
              <button
                type="submit"
                className="flex-1 px-3 py-1.5 text-xs bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
              >
                Ask AI
              </button>
              <button
                type="button"
                onClick={() => {
                  setShowCustomInput(false)
                  setCustomPrompt('')
                }}
                className="px-3 py-1.5 text-xs bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Translate language input */}
      {showTranslateInput && (
        <div className="p-3 border-b border-gray-200 dark:border-gray-700">
          <form onSubmit={handleTranslateSubmit}>
            <label className="block text-xs text-gray-500 dark:text-gray-400 mb-2">
              Target language:
            </label>
            <input
              type="text"
              value={targetLanguage}
              onChange={e => setTargetLanguage(e.target.value)}
              placeholder="e.g., Spanish, French, German"
              className="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              autoFocus
            />
            <div className="flex gap-2 mt-2">
              <button
                type="submit"
                className="flex-1 px-3 py-1.5 text-xs bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
              >
                Translate
              </button>
              <button
                type="button"
                onClick={() => setShowTranslateInput(false)}
                className="px-3 py-1.5 text-xs bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Operations list */}
      <div className="py-1 max-h-[400px] overflow-y-auto">
        {AI_OPERATIONS.map(operation => (
          <button
            key={operation.id}
            onClick={() => handleOperationClick(operation)}
            className="w-full px-3 py-2 flex items-start gap-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left"
          >
            <div className="mt-0.5 text-blue-500">{operation.icon}</div>
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                {operation.label}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {operation.description}
              </div>
            </div>
          </button>
        ))}
      </div>

      {/* Footer hint */}
      <div className="p-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
        <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
          Press{' '}
          <kbd className="px-1 py-0.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs">
            Esc
          </kbd>{' '}
          to close
        </p>
      </div>
    </div>
  )
}
