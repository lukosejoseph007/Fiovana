import React, { useCallback, useState, useRef } from 'react'
import { clsx } from 'clsx'

export interface DropZoneProps {
  onFileDrop: (files: File[]) => void
  acceptedFileTypes?: string[]
  maxFileSize?: number // in bytes
  maxFiles?: number
  disabled?: boolean
  className?: string
  children?: React.ReactNode
}

export interface DropZoneState {
  isDragOver: boolean
  isDragActive: boolean
  isHovered: boolean
  error?: string
  dragCounter: number
}

const DropZone: React.FC<DropZoneProps> = ({
  onFileDrop,
  acceptedFileTypes = ['.docx', '.pdf', '.md', '.txt', '.csv', '.json'],
  maxFileSize = 100 * 1024 * 1024, // 100MB default
  maxFiles = 10,
  disabled = false,
  className,
  children,
}) => {
  const [state, setState] = useState<DropZoneState>({
    isDragOver: false,
    isDragActive: false,
    isHovered: false,
    dragCounter: 0,
  })

  const dropZoneRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const dragLeaveTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  const resetDragState = useCallback(() => {
    if (dragLeaveTimeoutRef.current) {
      clearTimeout(dragLeaveTimeoutRef.current)
      dragLeaveTimeoutRef.current = null
    }
    setState(prev => ({
      ...prev,
      isDragActive: false,
      isDragOver: false,
      dragCounter: 0
    }))
  }, [])

  const validateFiles = useCallback((files: FileList): { validFiles: File[]; errors: string[] } => {
    const validFiles: File[] = []
    const errors: string[] = []

    if (files.length > maxFiles) {
      errors.push(`Maximum ${maxFiles} files allowed`)
      return { validFiles, errors }
    }

    Array.from(files).forEach((file) => {
      // Check file size
      if (file.size > maxFileSize) {
        errors.push(`File "${file.name}" exceeds maximum size of ${Math.round(maxFileSize / (1024 * 1024))}MB`)
        return
      }

      // Check file type
      const nameParts = file.name.split('.')
      const fileExtension = nameParts.length > 1 ? '.' + nameParts.pop()?.toLowerCase() : ''

      if (acceptedFileTypes.length > 0 && fileExtension && !acceptedFileTypes.includes(fileExtension)) {
        errors.push(`File "${file.name}" type not supported. Allowed: ${acceptedFileTypes.join(', ')}`)
        return
      }

      validFiles.push(file)
    })

    return { validFiles, errors }
  }, [acceptedFileTypes, maxFileSize, maxFiles])

  const handleDragEnter = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()

    if (disabled) return

    if (dragLeaveTimeoutRef.current) {
      clearTimeout(dragLeaveTimeoutRef.current)
      dragLeaveTimeoutRef.current = null
    }

    setState(prev => {
      const newCounter = prev.dragCounter + 1
      return {
        ...prev,
        isDragActive: true,
        isDragOver: true,
        dragCounter: newCounter,
        error: undefined
      }
    })
  }, [disabled])

  const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()

    if (disabled) return

    setState(prev => {
      const newCounter = prev.dragCounter - 1

      if (newCounter <= 0) {
        if (dragLeaveTimeoutRef.current) {
          clearTimeout(dragLeaveTimeoutRef.current)
        }

        dragLeaveTimeoutRef.current = setTimeout(() => {
          setState(current => ({
            ...current,
            isDragActive: false,
            isDragOver: false,
            dragCounter: 0
          }))
          dragLeaveTimeoutRef.current = null
        }, 100)

        return {
          ...prev,
          dragCounter: 0
        }
      }

      return {
        ...prev,
        dragCounter: newCounter
      }
    })
  }, [disabled])

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()

    if (disabled) return

    setState(prev => {
      if (!prev.isDragOver || !prev.isDragActive) {
        return { ...prev, isDragOver: true, isDragActive: true }
      }
      return prev
    })
  }, [disabled])

  const handleDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()

    if (dragLeaveTimeoutRef.current) {
      clearTimeout(dragLeaveTimeoutRef.current)
      dragLeaveTimeoutRef.current = null
    }

    if (disabled) {
      resetDragState()
      return
    }

    resetDragState()

    const files = e.dataTransfer.files

    if (files.length > 0) {
      const { validFiles, errors } = validateFiles(files)

      if (errors.length > 0) {
        setState(prev => ({ ...prev, error: errors.join('; ') }))
        return
      }

      if (validFiles.length > 0) {
        onFileDrop(validFiles)
        setState(prev => ({ ...prev, error: undefined }))
      }
    } else if (e.dataTransfer.items) {
      const itemFiles: File[] = []
      for (let i = 0; i < e.dataTransfer.items.length; i++) {
        const item = e.dataTransfer.items[i]
        if (item && item.kind === 'file') {
          const file = item.getAsFile()
          if (file) {
            itemFiles.push(file)
          }
        }
      }

      if (itemFiles.length > 0) {
        const fileListLike = {
          length: itemFiles.length,
          item: (index: number) => itemFiles[index] || null,
          [Symbol.iterator]: function* () {
            for (let i = 0; i < this.length; i++) {
              yield itemFiles[i]
            }
          }
        } as FileList

        Object.defineProperty(fileListLike, 'length', { value: itemFiles.length, writable: false })
        for (let i = 0; i < itemFiles.length; i++) {
          Object.defineProperty(fileListLike, i, { value: itemFiles[i], writable: false })
        }

        const { validFiles } = validateFiles(fileListLike)

        if (validFiles.length > 0) {
          onFileDrop(validFiles)
          setState(prev => ({ ...prev, error: undefined }))
        }
      }
    }
  }, [disabled, validateFiles, onFileDrop, resetDragState])


  const handleFileInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    if (disabled) return

    const files = e.target.files
    if (files && files.length > 0) {
      const { validFiles, errors } = validateFiles(files)

      if (errors.length > 0) {
        setState(prev => ({ ...prev, error: errors.join('; ') }))
        return
      }

      if (validFiles.length > 0) {
        onFileDrop(validFiles)
        setState(prev => ({ ...prev, error: undefined }))
      }
    }

    // Reset input
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }, [disabled, validateFiles, onFileDrop])

  const handleClick = useCallback(() => {
    if (disabled) return

    // Reset drag state when clicking (in case it's stuck)
    resetDragState()

    fileInputRef.current?.click()
  }, [disabled, resetDragState])

  const handleMouseEnter = useCallback(() => {
    if (disabled) return
    setState(prev => ({ ...prev, isHovered: true }))
  }, [disabled])

  const handleMouseLeave = useCallback(() => {
    if (disabled) return
    setState(prev => ({ ...prev, isHovered: false }))
  }, [disabled])

  const handleDragEnd = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
    resetDragState()
  }, [resetDragState])

  const dropZoneClasses = clsx(
    // Base styles
    'relative border-2 border-dashed rounded-lg p-8 transition-all duration-200 ease-in-out cursor-pointer',
    'flex flex-col items-center justify-center text-center min-h-48',

    // State-based styles
    {
      // Default state
      'border-gray-300 bg-gray-50 hover:border-gray-400 hover:bg-gray-100':
        !state.isDragActive && !state.isDragOver && !disabled && !state.isHovered,

      // Hover state
      'border-blue-400 bg-blue-50':
        state.isHovered && !state.isDragActive && !disabled,

      // Drag active state
      'border-blue-500 bg-blue-100 scale-105 shadow-lg':
        state.isDragActive && !disabled,

      // Drag over state
      'border-green-500 bg-green-100 scale-105 shadow-xl':
        state.isDragOver && !disabled,

      // Disabled state
      'border-gray-200 bg-gray-100 cursor-not-allowed opacity-50':
        disabled,

      // Error state
      'border-red-400 bg-red-50':
        state.error && !state.isDragActive,
    },

    className
  )

  return (
    <div className="w-full">
      <div
        ref={dropZoneRef}
        className={dropZoneClasses}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onDragEnd={handleDragEnd}
        onClick={handleClick}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        role="button"
        tabIndex={disabled ? -1 : 0}
        aria-disabled={disabled}
        aria-label="Drop files here or click to browse"
      >
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept={acceptedFileTypes.join(',')}
          onChange={handleFileInputChange}
          className="hidden"
          disabled={disabled}
        />

        {children ? (
          children
        ) : (
          <>
            {/* Upload Icon */}
            <div className={clsx(
              'mb-4 p-3 rounded-full transition-colors duration-200',
              {
                'bg-blue-200 text-blue-600': state.isDragActive || state.isHovered,
                'bg-green-200 text-green-600': state.isDragOver,
                'bg-gray-200 text-gray-500': !state.isDragActive && !state.isDragOver && !state.isHovered,
                'bg-gray-100 text-gray-400': disabled,
              }
            )}>
              <svg
                className="w-8 h-8"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                />
              </svg>
            </div>

            {/* Text Content */}
            <div className="space-y-2">
              <p className={clsx(
                'text-lg font-medium transition-colors duration-200',
                {
                  'text-blue-700': state.isDragActive || state.isHovered,
                  'text-green-700': state.isDragOver,
                  'text-gray-700': !state.isDragActive && !state.isDragOver && !state.isHovered && !disabled,
                  'text-gray-500': disabled,
                }
              )}>
                {state.isDragOver
                  ? 'Drop files here!'
                  : state.isDragActive
                    ? 'Drag files here'
                    : 'Drop files here or click to browse'
                }
              </p>

              <p className="text-sm text-gray-500">
                Supported formats: {acceptedFileTypes.join(', ')}
              </p>

              <p className="text-xs text-gray-400">
                Max file size: {Math.round(maxFileSize / (1024 * 1024))}MB | Max files: {maxFiles}
              </p>
            </div>
          </>
        )}

        {/* Loading indicator when dragging */}
        {state.isDragActive && (
          <div className="absolute inset-0 flex items-center justify-center bg-blue-100 bg-opacity-50 rounded-lg">
            <div className="animate-pulse text-blue-600 font-medium">
              Processing...
            </div>
          </div>
        )}
      </div>

      {/* Error Display */}
      {state.error && (
        <div className="mt-2 p-3 bg-red-100 border border-red-400 text-red-700 rounded-md">
          <p className="text-sm font-medium">Error:</p>
          <p className="text-sm">{state.error}</p>
        </div>
      )}
    </div>
  )
}

export default DropZone