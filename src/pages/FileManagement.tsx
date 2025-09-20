import React, { useState, useEffect, useCallback, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'

interface FileInfo {
  name: string
  size: number
  type: string
  lastModified: number
  path?: string
  validation?: {
    is_valid: boolean
    message: string
    warnings: string[]
  }
  hash?: {
    sha256: string
    algorithm: string
  }
  duplicate_check?: {
    is_duplicate: boolean
    existing_paths: string[]
  }
  metadata?: {
    file_type: string
    mime_type: string
    page_count?: number
    word_count?: number
    creation_date?: string
    author?: string
    language?: string
    binary_ratio?: number
    entropy?: number
    text_preview?: string
  }
  error?: string
}

interface BackendFileResult {
  name?: string
  size?: number
  modified?: number
  path?: string
  validation?: {
    is_valid: boolean
    message: string
    warnings: string[]
  }
  hash?: {
    sha256: string
    algorithm: string
  }
  duplicate_check?: {
    is_duplicate: boolean
    existing_paths: string[]
  }
  metadata?: {
    file_type: string
    mime_type: string
    page_count?: number
    word_count?: number
    creation_date?: string
    author?: string
    language?: string
    binary_ratio?: number
    entropy?: number
    text_preview?: string
  }
  error?: string
}

const FileManagement: React.FC = () => {
  const [droppedFiles, setDroppedFiles] = useState<FileInfo[]>([])
  const [isDragOver, setIsDragOver] = useState(false)
  const [isHovered, setIsHovered] = useState(false)
  const isListenerSetupRef = useRef(false)
  const lastDropTimeRef = useRef<number>(0)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Prevent default drag/drop behavior on the whole document (but allow our DropZone)
  useEffect(() => {
    const handleGlobalDrop = (e: DragEvent) => {
      const dropZone = (e.target as Element)?.closest(
        '[aria-label="Drop files here or click to browse"]'
      )

      // Only prevent if not dropping on our DropZone
      if (!dropZone) {
        e.preventDefault()
        e.stopPropagation()
      }
    }

    const preventDefaults = (e: DragEvent) => {
      const dropZone = (e.target as Element)?.closest(
        '[aria-label="Drop files here or click to browse"]'
      )

      // Only prevent if not on our DropZone
      if (!dropZone) {
        e.preventDefault()
        e.stopPropagation()
      }
    }

    document.addEventListener('dragover', preventDefaults)
    document.addEventListener('drop', handleGlobalDrop)

    return () => {
      document.removeEventListener('dragover', preventDefaults)
      document.removeEventListener('drop', handleGlobalDrop)
    }
  }, [])

  // Tauri file drop event listener
  useEffect(() => {
    // Prevent multiple listener setup in React Strict Mode
    if (isListenerSetupRef.current) {
      return
    }

    let unlisten: (() => void) | undefined

    const setupTauriFileDropListener = async () => {
      try {
        const window = getCurrentWindow()
        isListenerSetupRef.current = true

        unlisten = await window.onDragDropEvent(event => {
          // Handle drag state for visual feedback
          if (event.payload.type === 'enter') {
            setIsDragOver(true)
          } else if (event.payload.type === 'leave') {
            setIsDragOver(false)
          } else if (event.payload.type === 'drop') {
            setIsDragOver(false)

            if ('paths' in event.payload) {
              const currentTime = Date.now()
              // Deduplicate rapid successive drop events (within 100ms)
              if (currentTime - lastDropTimeRef.current < 100) {
                return
              }
              lastDropTimeRef.current = currentTime

              // Process files using the backend pipeline
              invoke('process_dropped_files', {
                filePaths: event.payload.paths,
                checkDuplicates: true,
                extractMetadata: true,
              })
                .then((results: unknown) => {
                  const typedResults = results as BackendFileResult[]
                  const processedFiles: FileInfo[] = typedResults.map(
                    (result: BackendFileResult) => {
                      // Convert backend result to FileInfo format
                      const fileInfo: FileInfo = {
                        name: result.name || 'unknown',
                        size: result.size || 0,
                        type: result.metadata?.mime_type || 'application/octet-stream',
                        lastModified: result.modified ? result.modified * 1000 : Date.now(), // Convert to milliseconds
                        path: result.path,
                        validation: result.validation,
                        hash: result.hash,
                        duplicate_check: result.duplicate_check,
                        metadata: result.metadata,
                        error: result.error,
                      }
                      return fileInfo
                    }
                  )

                  if (processedFiles.length > 0) {
                    setDroppedFiles(prev => {
                      // Additional deduplication: check if files already exist
                      const newFiles = processedFiles.filter(
                        newFile =>
                          !prev.some(
                            existingFile =>
                              existingFile.name === newFile.name &&
                              Math.abs(existingFile.lastModified - newFile.lastModified) < 1000
                          )
                      )

                      if (newFiles.length === 0) {
                        return prev
                      }

                      return [...prev, ...newFiles]
                    })
                  }
                })
                .catch(error => {
                  console.error('Error processing dropped files:', error)
                })
            }
          }
        })
      } catch (error) {
        console.error('Error setting up Tauri file drop listener:', error)
      }
    }

    setupTauriFileDropListener()

    return () => {
      isListenerSetupRef.current = false
      if (unlisten) {
        unlisten()
      }
    }
  }, []) // Empty dependency array to run only once

  // Handle click to browse files using Tauri file dialog
  const handleBrowseFiles = useCallback(async () => {
    try {
      // Use Tauri's file dialog to get actual file paths
      const selectedFiles = await invoke('open_file_dialog')

      if (selectedFiles && Array.isArray(selectedFiles) && selectedFiles.length > 0) {
        // Use the same backend processing pipeline as drag-and-drop
        invoke('process_dropped_files', {
          filePaths: selectedFiles,
          checkDuplicates: true,
          extractMetadata: true,
        })
          .then((results: unknown) => {
            const typedResults = results as BackendFileResult[]
            const processedFiles: FileInfo[] = typedResults.map((result: BackendFileResult) => {
              // Convert backend result to FileInfo format
              const fileInfo: FileInfo = {
                name: result.name || 'unknown',
                size: result.size || 0,
                type: result.metadata?.mime_type || 'application/octet-stream',
                lastModified: result.modified ? result.modified * 1000 : Date.now(), // Convert to milliseconds
                path: result.path,
                validation: result.validation,
                hash: result.hash,
                duplicate_check: result.duplicate_check,
                metadata: result.metadata,
                error: result.error,
              }
              return fileInfo
            })

            if (processedFiles.length > 0) {
              setDroppedFiles(prev => {
                // Additional deduplication: check if files already exist
                const newFiles = processedFiles.filter(
                  newFile =>
                    !prev.some(
                      existingFile =>
                        existingFile.name === newFile.name &&
                        Math.abs(existingFile.lastModified - newFile.lastModified) < 1000
                    )
                )

                if (newFiles.length === 0) {
                  return prev
                }

                return [...prev, ...newFiles]
              })
            }
          })
          .catch(error => {
            console.error('Error processing browsed files:', error)
          })
      }
    } catch (error) {
      console.error('Error opening file dialog:', error)
    }
  }, [])

  // Handle file input change (fallback for non-Tauri environments)
  const handleFileInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files
    if (files && files.length > 0) {
      // Fallback to basic file info for browser environments
      const basicFileInfos: FileInfo[] = Array.from(files).map(file => ({
        name: file.name,
        size: file.size,
        type: file.type,
        lastModified: file.lastModified,
        error: 'Limited processing - drag and drop for full features',
      }))

      setDroppedFiles(prev => {
        const newFiles = basicFileInfos.filter(
          newFile =>
            !prev.some(
              existingFile =>
                existingFile.name === newFile.name &&
                Math.abs(existingFile.lastModified - newFile.lastModified) < 1000
            )
        )

        if (newFiles.length === 0) {
          return prev
        }

        return [...prev, ...newFiles]
      })
    }

    // Reset input
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }, [])

  // Handle click to browse files
  const handleClick = useCallback(async () => {
    try {
      // Try to use Tauri file dialog first
      await handleBrowseFiles()
    } catch {
      // Fallback to HTML file input for browser environments
      fileInputRef.current?.click()
    }
  }, [handleBrowseFiles])

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const clearFiles = () => {
    setDroppedFiles([])
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">File Management</h1>
        <p className="text-gray-600 dark:text-gray-400">
          Upload and analyze documents with AI-powered intelligence
        </p>
      </div>

      {/* Drop Zone Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Upload Documents
        </h2>

        <div className="flex justify-center w-full">
          <div className="w-full max-w-2xl">
            {/* Enhanced drop area with click-to-browse and visual feedback */}
            <div
              className={`relative border-2 border-dashed rounded-lg p-8 transition-all duration-200 ease-in-out min-h-48 flex flex-col items-center justify-center text-center cursor-pointer ${
                isDragOver
                  ? 'border-green-500 bg-green-50 dark:bg-green-900/20 scale-105 shadow-xl'
                  : isHovered
                    ? 'border-blue-400 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 hover:border-gray-400 hover:bg-gray-100 dark:hover:bg-gray-600'
              }`}
              onClick={handleClick}
              onMouseEnter={() => setIsHovered(true)}
              onMouseLeave={() => setIsHovered(false)}
              role="button"
              tabIndex={0}
              aria-label="Drop files here or click to browse"
            >
              {/* Hidden file input */}
              <input
                ref={fileInputRef}
                type="file"
                multiple
                accept=".docx,.pdf,.md,.txt,.csv,.json"
                onChange={handleFileInputChange}
                className="hidden"
              />

              {/* Upload Icon */}
              <div
                className={`mb-4 p-3 rounded-full transition-colors duration-200 ${
                  isDragOver
                    ? 'bg-green-200 text-green-600 dark:bg-green-800 dark:text-green-300'
                    : isHovered
                      ? 'bg-blue-200 text-blue-600 dark:bg-blue-800 dark:text-blue-300'
                      : 'bg-gray-200 text-gray-500 dark:bg-gray-600 dark:text-gray-400'
                }`}
              >
                <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                <p
                  className={`text-lg font-medium transition-colors duration-200 ${
                    isDragOver
                      ? 'text-green-700 dark:text-green-300'
                      : isHovered
                        ? 'text-blue-700 dark:text-blue-300'
                        : 'text-gray-700 dark:text-gray-300'
                  }`}
                >
                  {isDragOver
                    ? 'Drop files here!'
                    : isHovered
                      ? 'Drag files here'
                      : 'Drop files here or click to browse'}
                </p>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Supported formats: .docx, .pdf, .md, .txt, .csv, .json
                </p>
                <p className="text-xs text-gray-400 dark:text-gray-500">
                  Max file size: 100MB | Max files: 10
                </p>
              </div>

              {/* Drag overlay */}
              {isDragOver && (
                <div className="absolute inset-0 flex items-center justify-center bg-green-100 dark:bg-green-900/30 bg-opacity-50 rounded-lg">
                  <div className="animate-pulse text-green-600 dark:text-green-300 font-medium">
                    Ready to drop!
                  </div>
                </div>
              )}

              {/* Hover overlay for processing state */}
              {isHovered && !isDragOver && (
                <div className="absolute inset-0 flex items-center justify-center bg-blue-100 dark:bg-blue-900/30 bg-opacity-30 rounded-lg">
                  <div className="text-blue-600 dark:text-blue-300 font-medium">
                    Click to browse files
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* File List Section */}
      {droppedFiles.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <div className="flex justify-between items-center p-6 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Uploaded Files ({droppedFiles.length})
            </h2>
            <button
              onClick={clearFiles}
              className="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 transition-colors"
            >
              Clear All
            </button>
          </div>

          {/* File Processing Summary */}
          <div className="p-6 bg-gray-50 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600">
            <h3 className="text-base font-medium text-gray-900 dark:text-white mb-3">
              Processing Summary
            </h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                  {
                    droppedFiles.filter(
                      f => f.validation?.is_valid && !f.error && !f.duplicate_check?.is_duplicate
                    ).length
                  }
                </div>
                <div className="text-gray-600 dark:text-gray-400">Valid Files</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600 dark:text-red-400">
                  {droppedFiles.filter(f => f.error).length}
                </div>
                <div className="text-gray-600 dark:text-gray-400">Errors</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                  {
                    droppedFiles.filter(f => f.validation && !f.validation.is_valid && !f.error)
                      .length
                  }
                </div>
                <div className="text-gray-600 dark:text-gray-400">Warnings</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
                  {droppedFiles.filter(f => f.duplicate_check?.is_duplicate).length}
                </div>
                <div className="text-gray-600 dark:text-gray-400">Duplicates</div>
              </div>
            </div>
          </div>

          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    File Name
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Size
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Last Modified
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {droppedFiles.map((file, index) => (
                  <tr
                    key={index}
                    className={`hover:bg-gray-50 dark:hover:bg-gray-700 ${
                      file.error
                        ? 'bg-red-50 dark:bg-red-900/20'
                        : file.validation && !file.validation.is_valid
                          ? 'bg-yellow-50 dark:bg-yellow-900/20'
                          : file.duplicate_check?.is_duplicate
                            ? 'bg-orange-50 dark:bg-orange-900/20'
                            : ''
                    }`}
                  >
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                      <div className="flex items-center">
                        {file.error && (
                          <span className="mr-2 text-red-500" title={file.error}>
                            ❌
                          </span>
                        )}
                        {file.validation && !file.validation.is_valid && (
                          <span className="mr-2 text-yellow-500" title={file.validation.message}>
                            ⚠️
                          </span>
                        )}
                        {file.duplicate_check?.is_duplicate && (
                          <span className="mr-2 text-orange-500" title="Duplicate file detected">
                            🔄
                          </span>
                        )}
                        {file.validation?.is_valid &&
                          !file.error &&
                          !file.duplicate_check?.is_duplicate && (
                            <span
                              className="mr-2 text-green-500"
                              title="File validated successfully"
                            >
                              ✅
                            </span>
                          )}
                        {file.name}
                      </div>
                      {file.error && (
                        <div className="text-xs text-red-600 dark:text-red-400 mt-1">
                          Error: {file.error}
                        </div>
                      )}
                      {file.validation && !file.validation.is_valid && (
                        <div className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                          Validation: {file.validation.message}
                        </div>
                      )}
                      {file.duplicate_check?.is_duplicate && (
                        <div className="text-xs text-orange-600 dark:text-orange-400 mt-1">
                          Duplicate detected. Existing at:{' '}
                          {file.duplicate_check.existing_paths.join(', ')}
                        </div>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-300">
                      {formatFileSize(file.size)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-300">
                      {file.type || 'Unknown'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-300">
                      {new Date(file.lastModified).toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Instructions */}
      {droppedFiles.length === 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-12">
          <div className="text-center">
            <div className="max-w-md mx-auto">
              <h3 className="text-xl font-medium text-gray-600 dark:text-gray-300 mb-4">
                No documents uploaded yet
              </h3>
              <p className="text-gray-500 dark:text-gray-400">
                Drag and drop files anywhere on this window, or click the drop zone to browse and
                select files for AI-powered analysis.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default FileManagement
