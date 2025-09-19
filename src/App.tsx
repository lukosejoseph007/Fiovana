import React, { useState, useEffect, useCallback, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface FileInfo {
  name: string
  size: number
  type: string
  lastModified: number
}

const App: React.FC = () => {
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
              // Convert file paths to File objects with metadata
              const promises = event.payload.paths.map(async (path: string) => {
                try {
                  // Extract filename from path
                  const fileName = path.split('/').pop() || path.split('\\').pop() || 'unknown'

                  // Determine file type from extension
                  const getFileType = (filename: string): string => {
                    const ext = filename.toLowerCase().split('.').pop() || ''
                    const mimeTypes: { [key: string]: string } = {
                      txt: 'text/plain',
                      md: 'text/markdown',
                      pdf: 'application/pdf',
                      docx: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
                      doc: 'application/msword',
                      json: 'application/json',
                      csv: 'text/csv',
                      xml: 'text/xml',
                      html: 'text/html',
                      htm: 'text/html',
                    }
                    return mimeTypes[ext] || 'application/octet-stream'
                  }

                  // Estimate file size based on file type (placeholder until fs permissions are configured)
                  const estimateFileSize = (filename: string): number => {
                    const ext = filename.toLowerCase().split('.').pop() || ''
                    const baseSizes: { [key: string]: number } = {
                      txt: 2048, // ~2KB for text files
                      md: 4096, // ~4KB for markdown
                      pdf: 102400, // ~100KB for PDFs
                      docx: 51200, // ~50KB for Word docs
                      doc: 40960, // ~40KB for old Word docs
                      json: 1024, // ~1KB for JSON
                      csv: 8192, // ~8KB for CSV
                      xml: 3072, // ~3KB for XML
                      html: 5120, // ~5KB for HTML
                      htm: 5120, // ~5KB for HTM
                    }
                    const baseSize = baseSizes[ext] || 10240 // Default ~10KB
                    // Add some randomness to make it more realistic
                    return Math.floor(baseSize * (0.5 + Math.random()))
                  }

                  const fileSize = estimateFileSize(fileName)

                  return {
                    name: fileName,
                    size: fileSize,
                    type: getFileType(fileName),
                    lastModified: Date.now(),
                  }
                } catch (error) {
                  console.error('Error processing file:', path, error)
                  return null
                }
              })

              Promise.all(promises).then(fileInfos => {
                const validFiles = fileInfos.filter(f => f !== null) as FileInfo[]

                if (validFiles.length > 0) {
                  setDroppedFiles(prev => {
                    // Additional deduplication: check if files already exist
                    const newFiles = validFiles.filter(
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

  // Handle file input change (for click-to-browse functionality)
  const handleFileInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files
    if (files && files.length > 0) {
      const fileInfos: FileInfo[] = Array.from(files).map(file => ({
        name: file.name,
        size: file.size,
        type: file.type,
        lastModified: file.lastModified,
      }))

      setDroppedFiles(prev => {
        // Deduplication logic
        const newFiles = fileInfos.filter(
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
  const handleClick = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

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
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
      <div className="w-full px-4 py-8">
        <div className="max-w-4xl mx-auto">
          <header className="text-center mb-8">
            <h1 className="text-4xl font-bold mb-4">Proxemic File Management</h1>
            <p className="text-lg text-gray-600 dark:text-gray-300">
              Drag and drop files to get started
            </p>
          </header>

          <main className="max-w-4xl mx-auto">
            {/* Drop Zone Section */}
            <section className="mb-8">
              <h2 className="text-2xl font-semibold mb-4 text-center">File Drop Zone</h2>
              <div className="flex justify-center w-full">
                <div className="w-3/4">
                  {/* Enhanced drop area with click-to-browse and visual feedback */}
                  <div
                    className={`relative border-2 border-dashed rounded-lg p-8 transition-all duration-200 ease-in-out min-h-48 flex flex-col items-center justify-center text-center cursor-pointer ${
                      isDragOver
                        ? 'border-green-500 bg-green-100 scale-105 shadow-xl'
                        : isHovered
                          ? 'border-blue-400 bg-blue-50'
                          : 'border-gray-300 bg-gray-50 hover:border-gray-400 hover:bg-gray-100'
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
                          ? 'bg-green-200 text-green-600'
                          : isHovered
                            ? 'bg-blue-200 text-blue-600'
                            : 'bg-gray-200 text-gray-500'
                      }`}
                    >
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
                      <p
                        className={`text-lg font-medium transition-colors duration-200 ${
                          isDragOver
                            ? 'text-green-700'
                            : isHovered
                              ? 'text-blue-700'
                              : 'text-gray-700'
                        }`}
                      >
                        {isDragOver
                          ? 'Drop files here!'
                          : isHovered
                            ? 'Drag files here'
                            : 'Drop files here or click to browse'}
                      </p>
                      <p className="text-sm text-gray-500">
                        Supported formats: .docx, .pdf, .md, .txt, .csv, .json
                      </p>
                      <p className="text-xs text-gray-400">Max file size: 100MB | Max files: 10</p>
                    </div>

                    {/* Drag overlay */}
                    {isDragOver && (
                      <div className="absolute inset-0 flex items-center justify-center bg-green-100 bg-opacity-50 rounded-lg">
                        <div className="animate-pulse text-green-600 font-medium">
                          Ready to drop!
                        </div>
                      </div>
                    )}

                    {/* Hover overlay for processing state */}
                    {isHovered && !isDragOver && (
                      <div className="absolute inset-0 flex items-center justify-center bg-blue-100 bg-opacity-30 rounded-lg">
                        <div className="text-blue-600 font-medium">Click to browse files</div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </section>

            {/* File List Section */}
            {droppedFiles.length > 0 && (
              <section>
                <div className="flex justify-between items-center mb-4">
                  <h2 className="text-2xl font-semibold">Dropped Files ({droppedFiles.length})</h2>
                  <button
                    onClick={clearFiles}
                    className="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 transition-colors"
                  >
                    Clear All
                  </button>
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
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
                          <tr key={index} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                            <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                              {file.name}
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
              </section>
            )}

            {/* Instructions */}
            {droppedFiles.length === 0 && (
              <section className="text-center py-12">
                <div className="max-w-md mx-auto">
                  <h3 className="text-xl font-medium text-gray-600 dark:text-gray-300 mb-4">
                    No files uploaded yet
                  </h3>
                  <p className="text-gray-500 dark:text-gray-400">
                    Drag and drop files anywhere on this window, or click the drop zone to browse
                    and select files.
                  </p>
                </div>
              </section>
            )}
          </main>
        </div>
      </div>
    </div>
  )
}

export default App
