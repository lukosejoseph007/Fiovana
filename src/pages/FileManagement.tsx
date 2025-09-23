import React, { useEffect, useCallback, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { useAppState } from '../context/AppStateContext'
import type { FileInfo } from '../context/types'

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

interface SearchResult {
  document: DocumentIndexEntry
  score: number
}

interface SearchResponse {
  success: boolean
  results: SearchResult[]
  total_found: number
  error?: string
}

interface IndexStats {
  total_documents: number
  total_keywords: number
  total_content_size: number
  index_version: number
  error?: string
}

interface DocumentIndexEntry {
  title: string
  path: string
  summary?: string
  content: string
  metadata: object
  structure: DocumentStructure
  keywords: string[]
  content_hash: string
  indexed_at: {
    secs_since_epoch: number
    nanos_since_epoch: number
  }
  index_version: number
}

interface DocumentStructure {
  document_type: string
  sections: DocumentSection[]
  toc?: TocEntry[]
  page_count?: number
  has_images: boolean
  has_tables: boolean
  has_code: boolean
}

interface DocumentSection {
  title: string
  level: number
  content?: string
}

interface TocEntry {
  title: string
  level: number
  page?: number
}

const FileManagement: React.FC = () => {
  const { state, dispatch } = useAppState()
  const { droppedFiles, isDragOver } = state.fileManagement
  const [isHovered, setIsHovered] = React.useState(false)
  const [searchQuery, setSearchQuery] = React.useState('')
  const [filterType, setFilterType] = React.useState('all')
  const [searchResults, setSearchResults] = React.useState<SearchResult[]>([])
  const [isSearching, setIsSearching] = React.useState(false)
  const [indexStats, setIndexStats] = React.useState<IndexStats | null>(null)
  const [selectedDocument, setSelectedDocument] = React.useState<DocumentIndexEntry | null>(null)
  const [showDocumentModal, setShowDocumentModal] = React.useState(false)
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
            dispatch({ type: 'FILE_MANAGEMENT_SET_DRAG_OVER', payload: true })
          } else if (event.payload.type === 'leave') {
            dispatch({ type: 'FILE_MANAGEMENT_SET_DRAG_OVER', payload: false })
          } else if (event.payload.type === 'drop') {
            dispatch({ type: 'FILE_MANAGEMENT_SET_DRAG_OVER', payload: false })

            if ('paths' in event.payload) {
              const currentTime = Date.now()
              // Deduplicate rapid successive drop events (within 100ms)
              if (currentTime - lastDropTimeRef.current < 100) {
                return
              }
              lastDropTimeRef.current = currentTime

              // Process files using the backend pipeline
              dispatch({ type: 'FILE_MANAGEMENT_SET_PROCESSING', payload: true })
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
                    // Additional deduplication: check if files already exist
                    const currentDroppedFiles = droppedFiles
                    const newFiles = processedFiles.filter(
                      newFile =>
                        !currentDroppedFiles.some(
                          existingFile =>
                            existingFile.name === newFile.name &&
                            Math.abs(existingFile.lastModified - newFile.lastModified) < 1000
                        )
                    )

                    if (newFiles.length > 0) {
                      dispatch({ type: 'FILE_MANAGEMENT_ADD_FILES', payload: newFiles })
                    }
                  }
                })
                .catch(error => {
                  console.error('Error processing dropped files:', error)
                })
                .finally(() => {
                  dispatch({ type: 'FILE_MANAGEMENT_SET_PROCESSING', payload: false })
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
  }, [dispatch, droppedFiles]) // Include dependencies

  // Handle click to browse files using Tauri file dialog
  const handleBrowseFiles = useCallback(async () => {
    try {
      // Use Tauri's file dialog to get actual file paths
      const selectedFiles = await invoke('open_file_dialog')

      if (selectedFiles && Array.isArray(selectedFiles) && selectedFiles.length > 0) {
        // Use the same backend processing pipeline as drag-and-drop
        dispatch({ type: 'FILE_MANAGEMENT_SET_PROCESSING', payload: true })
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
              // Additional deduplication: check if files already exist
              const currentDroppedFiles = droppedFiles
              const newFiles = processedFiles.filter(
                newFile =>
                  !currentDroppedFiles.some(
                    existingFile =>
                      existingFile.name === newFile.name &&
                      Math.abs(existingFile.lastModified - newFile.lastModified) < 1000
                  )
              )

              if (newFiles.length > 0) {
                dispatch({ type: 'FILE_MANAGEMENT_ADD_FILES', payload: newFiles })
              }
            }
          })
          .catch(error => {
            console.error('Error processing browsed files:', error)
          })
          .finally(() => {
            dispatch({ type: 'FILE_MANAGEMENT_SET_PROCESSING', payload: false })
          })
      }
    } catch (error) {
      console.error('Error opening file dialog:', error)
    }
  }, [dispatch, droppedFiles])

  // Handle file input change (fallback for non-Tauri environments)
  const handleFileInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
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

        const currentDroppedFiles = droppedFiles
        const newFiles = basicFileInfos.filter(
          newFile =>
            !currentDroppedFiles.some(
              existingFile =>
                existingFile.name === newFile.name &&
                Math.abs(existingFile.lastModified - newFile.lastModified) < 1000
            )
        )

        if (newFiles.length > 0) {
          dispatch({ type: 'FILE_MANAGEMENT_ADD_FILES', payload: newFiles })
        }
      }

      // Reset input
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }
    },
    [dispatch, droppedFiles]
  )

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
    dispatch({ type: 'FILE_MANAGEMENT_CLEAR_FILES' })
  }

  // Initialize document indexer
  useEffect(() => {
    const initDocumentIndexer = async () => {
      try {
        await invoke('init_document_indexer', { indexDir: null })
        console.log('Document indexer initialized')
      } catch (error) {
        console.error('Failed to initialize document indexer:', error)
      }
    }
    initDocumentIndexer()
  }, [])

  // Auto-index files when they are uploaded
  useEffect(() => {
    const indexUploadedFiles = async () => {
      console.log(`Auto-indexing ${droppedFiles.length} files...`)
      for (const file of droppedFiles) {
        if (file.path && file.validation?.is_valid && !file.error) {
          try {
            console.log(`Indexing document: ${file.name} at path: ${file.path}`)
            const result = await invoke('index_document', {
              request: { file_path: file.path },
            })
            console.log(`Successfully indexed document: ${file.name}`, result)
          } catch (error) {
            console.error(`Failed to index document ${file.name}:`, error)
          }
        } else {
          console.log(
            `Skipping indexing for ${file.name}: path=${file.path}, valid=${file.validation?.is_valid}, error=${file.error}`
          )
        }
      }
    }
    if (droppedFiles.length > 0) {
      indexUploadedFiles()
    }
  }, [droppedFiles])

  // Search documents
  const searchDocuments = async () => {
    if (!searchQuery.trim()) {
      setSearchResults([])
      return
    }

    setIsSearching(true)
    try {
      const response = await invoke('search_documents', {
        request: {
          query: searchQuery,
          filter: filterType === 'all' ? null : { extensions: [filterType] },
          limit: 20,
        },
      })

      if (response && typeof response === 'object' && 'results' in response) {
        setSearchResults((response as SearchResponse).results || [])
      }
    } catch (error) {
      console.error('Search failed:', error)
      setSearchResults([])
    } finally {
      setIsSearching(false)
    }
  }

  // Check index stats
  const checkIndexStats = async () => {
    try {
      const stats = await invoke<IndexStats>('get_index_stats')
      console.log('Index stats:', stats)
      setIndexStats(stats)
    } catch (error) {
      console.error('Failed to get index stats:', error)
      setIndexStats({
        total_documents: 0,
        total_keywords: 0,
        total_content_size: 0,
        index_version: 0,
        error: String(error),
      })
    }
  }

  // View document details
  const viewDocumentDetails = async (filePath: string) => {
    try {
      console.log('Getting document details for:', filePath)
      const details = await invoke<DocumentIndexEntry | null>('get_document_details', { filePath })
      console.log('Document details:', details)
      setSelectedDocument(details)
      setShowDocumentModal(true)
    } catch (error) {
      console.error('Failed to get document details:', error)
    }
  }

  // Filter files locally
  const filteredFiles = React.useMemo(() => {
    if (filterType === 'all') return droppedFiles
    return droppedFiles.filter(file => {
      const fileExtension = file.name.split('.').pop()?.toLowerCase()
      switch (filterType) {
        case 'pdf':
          return fileExtension === 'pdf'
        case 'docx':
          return fileExtension === 'docx'
        case 'txt':
          return fileExtension === 'txt'
        case 'md':
          return fileExtension === 'md'
        default:
          return true
      }
    })
  }, [droppedFiles, filterType])

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">File Management</h1>
        <p className="text-gray-600 dark:text-gray-400">
          Upload and analyze documents with AI-powered intelligence
        </p>
      </div>

      {/* Search and Filter Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Search Documents
        </h2>

        <div className="space-y-4">
          {/* Search Input */}
          <div className="flex space-x-4">
            <div className="flex-1">
              <input
                type="text"
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                onKeyPress={e => e.key === 'Enter' && searchDocuments()}
                placeholder="Search document content..."
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                         bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                         placeholder-gray-500 dark:placeholder-gray-400
                         focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
            <button
              onClick={searchDocuments}
              disabled={isSearching || !searchQuery.trim()}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700
                       disabled:opacity-50 disabled:cursor-not-allowed
                       transition-colors duration-200"
            >
              {isSearching ? (
                <div className="flex items-center space-x-2">
                  <svg className="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  <span>Searching...</span>
                </div>
              ) : (
                'Search'
              )}
            </button>
          </div>

          {/* Filter Options and Index Stats */}
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Filter by type:
              </span>
              <select
                value={filterType}
                onChange={e => setFilterType(e.target.value)}
                className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md
                         bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                         focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                <option value="all">All Files</option>
                <option value="pdf">PDF Documents</option>
                <option value="docx">Word Documents</option>
                <option value="txt">Text Files</option>
                <option value="md">Markdown Files</option>
              </select>
            </div>

            <div className="flex items-center space-x-4">
              <button
                onClick={checkIndexStats}
                className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 transition-colors"
              >
                Check Index Stats
              </button>
              {indexStats && (
                <div className="text-sm text-gray-600 dark:text-gray-400">
                  {indexStats.error ? (
                    <span className="text-red-600">Error: {indexStats.error}</span>
                  ) : (
                    <span>
                      📊 {indexStats.total_documents} docs, {indexStats.total_keywords} keywords
                    </span>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* Search Results */}
          {searchResults.length > 0 && (
            <div className="mt-4">
              <h3 className="text-md font-medium text-gray-900 dark:text-white mb-3">
                Search Results ({searchResults.length})
              </h3>
              <div className="space-y-3">
                {searchResults.map((result, index) => (
                  <div key={index} className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <h4 className="font-medium text-gray-900 dark:text-white">
                          {result.document?.title || result.document?.path || 'Unknown Document'}
                        </h4>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                          {result.document?.path}
                        </p>
                        {result.document?.content && (
                          <p className="text-sm text-gray-700 dark:text-gray-300 mt-2">
                            {result.document.content.substring(0, 200)}...
                          </p>
                        )}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">
                        Score: {result.score?.toFixed(2) || 'N/A'}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
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
              Uploaded Files ({filteredFiles.length} of {droppedFiles.length} shown)
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
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {filteredFiles.map((file, index) => (
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
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      {file.path && file.validation?.is_valid && !file.error && (
                        <button
                          onClick={() => viewDocumentDetails(file.path!)}
                          className="px-3 py-1 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors text-xs"
                        >
                          View Details
                        </button>
                      )}
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

      {/* Document Details Modal */}
      {showDocumentModal && selectedDocument && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
            <div className="flex justify-between items-center p-6 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                Document Details
              </h2>
              <button
                onClick={() => setShowDocumentModal(false)}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>

            <div className="p-6 space-y-6">
              {/* Basic Info */}
              <div>
                <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                  Basic Information
                </h3>
                <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 space-y-2">
                  <div>
                    <span className="font-medium">Title:</span> {selectedDocument.title}
                  </div>
                  <div>
                    <span className="font-medium">Path:</span> {selectedDocument.path}
                  </div>
                  <div>
                    <span className="font-medium">Content Hash:</span>{' '}
                    {selectedDocument.content_hash?.substring(0, 16)}...
                  </div>
                  <div>
                    <span className="font-medium">Indexed At:</span>{' '}
                    {new Date(
                      selectedDocument.indexed_at?.secs_since_epoch * 1000
                    ).toLocaleString()}
                  </div>
                </div>
              </div>

              {/* Document Structure */}
              {selectedDocument.structure && (
                <div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Document Structure
                  </h3>
                  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 space-y-2">
                    <div>
                      <span className="font-medium">Type:</span>{' '}
                      {selectedDocument.structure.document_type}
                    </div>
                    {selectedDocument.structure.page_count && (
                      <div>
                        <span className="font-medium">Pages:</span>{' '}
                        {selectedDocument.structure.page_count}
                      </div>
                    )}
                    <div className="flex flex-wrap gap-2 mt-2">
                      {selectedDocument.structure.has_images && (
                        <span className="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-xs">
                          📷 Has Images
                        </span>
                      )}
                      {selectedDocument.structure.has_tables && (
                        <span className="px-2 py-1 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 rounded text-xs">
                          📊 Has Tables
                        </span>
                      )}
                      {selectedDocument.structure.has_code && (
                        <span className="px-2 py-1 bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-200 rounded text-xs">
                          💻 Has Code
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              )}

              {/* Table of Contents */}
              {selectedDocument.structure?.toc && selectedDocument.structure.toc.length > 0 && (
                <div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Table of Contents
                  </h3>
                  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <ul className="space-y-1">
                      {selectedDocument.structure.toc.map((entry: TocEntry, index: number) => (
                        <li
                          key={index}
                          className="text-sm"
                          style={{ marginLeft: `${entry.level * 20}px` }}
                        >
                          <span className="font-medium">{entry.title}</span>
                          {entry.page && (
                            <span className="text-gray-500 ml-2">p. {entry.page}</span>
                          )}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              )}

              {/* Sections */}
              {selectedDocument.structure?.sections &&
                selectedDocument.structure.sections.length > 0 && (
                  <div>
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                      Document Sections
                    </h3>
                    <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 space-y-3">
                      {selectedDocument.structure.sections.map(
                        (section: DocumentSection, index: number) => (
                          <div key={index} className="border-l-2 border-blue-300 pl-3">
                            <div className="font-medium text-sm">{section.title}</div>
                            <div className="text-xs text-gray-600 dark:text-gray-400">
                              Level {section.level} • {section.content?.length || 0} characters
                            </div>
                            {section.content && (
                              <div className="text-sm text-gray-700 dark:text-gray-300 mt-1">
                                {section.content.substring(0, 150)}...
                              </div>
                            )}
                          </div>
                        )
                      )}
                    </div>
                  </div>
                )}

              {/* Keywords */}
              {selectedDocument.keywords && selectedDocument.keywords.length > 0 && (
                <div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Keywords
                  </h3>
                  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <div className="flex flex-wrap gap-2">
                      {selectedDocument.keywords.map((keyword: string, index: number) => (
                        <span
                          key={index}
                          className="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-sm"
                        >
                          {keyword}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
              )}

              {/* Content Preview */}
              {selectedDocument.content && (
                <div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Content Preview
                  </h3>
                  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <div className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap max-h-60 overflow-y-auto">
                      {selectedDocument.content.substring(0, 1000)}
                      {selectedDocument.content.length > 1000 && '...'}
                    </div>
                  </div>
                </div>
              )}

              {/* Summary */}
              {selectedDocument.summary && (
                <div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Summary
                  </h3>
                  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <div className="text-sm text-gray-700 dark:text-gray-300">
                      {selectedDocument.summary}
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default FileManagement
