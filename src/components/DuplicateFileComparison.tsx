import React, { useState, useEffect, useCallback } from 'react'
import { clsx } from 'clsx'
import { DuplicateFile } from '../types/deduplication'
import { DeduplicationService } from '../services/deduplicationService'

interface FileMetadata {
  permissions: string
  created: Date
  modified: Date
  accessed: Date
  size: number
  checksum: string
}

interface DuplicateFileComparisonProps {
  originalFile: DuplicateFile
  duplicateFile: DuplicateFile
  onResolve?: (keepOriginal: boolean) => void
  showResolveButtons?: boolean
  className?: string
}

const DuplicateFileComparison: React.FC<DuplicateFileComparisonProps> = ({
  originalFile,
  duplicateFile,
  onResolve,
  showResolveButtons = false,
  className,
}) => {
  const [originalMetadata, setOriginalMetadata] = useState<FileMetadata | null>(null)
  const [duplicateMetadata, setDuplicateMetadata] = useState<FileMetadata | null>(null)
  const [isLoadingMetadata, setIsLoadingMetadata] = useState(false)
  const [showDifferences, setShowDifferences] = useState(false)

  const loadFileMetadata = useCallback(async () => {
    setIsLoadingMetadata(true)
    try {
      // In a real implementation, you would call Tauri commands to get detailed file metadata
      // For now, we'll simulate metadata based on the file information we have
      const originalMeta: FileMetadata = {
        permissions: 'rw-r--r--',
        created: new Date(originalFile.lastModified.getTime() - 86400000), // 1 day before modified
        modified: originalFile.lastModified,
        accessed: new Date(),
        size: originalFile.size,
        checksum: originalFile.hash,
      }

      const duplicateMeta: FileMetadata = {
        permissions: 'rw-r--r--',
        created: new Date(duplicateFile.lastModified.getTime() - 86400000),
        modified: duplicateFile.lastModified,
        accessed: new Date(),
        size: duplicateFile.size,
        checksum: duplicateFile.hash,
      }

      setOriginalMetadata(originalMeta)
      setDuplicateMetadata(duplicateMeta)
    } catch (error) {
      console.error('Failed to load file metadata:', error)
    } finally {
      setIsLoadingMetadata(false)
    }
  }, [
    originalFile.lastModified,
    duplicateFile.lastModified,
    originalFile.size,
    duplicateFile.size,
    originalFile.hash,
    duplicateFile.hash,
  ])

  useEffect(() => {
    if (showDifferences) {
      loadFileMetadata()
    }
  }, [showDifferences, loadFileMetadata])

  const getDifferences = () => {
    if (!originalMetadata || !duplicateMetadata) return []

    const differences = []

    if (originalMetadata.size !== duplicateMetadata.size) {
      differences.push('Size')
    }
    if (originalMetadata.modified.getTime() !== duplicateMetadata.modified.getTime()) {
      differences.push('Modified Date')
    }
    if (originalMetadata.permissions !== duplicateMetadata.permissions) {
      differences.push('Permissions')
    }

    return differences
  }

  const isNewer = (file1: DuplicateFile, file2: DuplicateFile) => {
    return file1.lastModified > file2.lastModified
  }

  const isLarger = (file1: DuplicateFile, file2: DuplicateFile) => {
    return file1.size > file2.size
  }

  return (
    <div className={clsx('bg-white border border-gray-200 rounded-lg overflow-hidden', className)}>
      {/* Header */}
      <div className="bg-gray-50 px-4 py-3 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-medium text-gray-900">File Comparison</h3>
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setShowDifferences(!showDifferences)}
              className="text-sm text-blue-600 hover:text-blue-800 transition-colors"
            >
              {showDifferences ? 'Hide Details' : 'Show Details'}
            </button>
          </div>
        </div>
      </div>

      {/* Comparison Grid */}
      <div className="grid grid-cols-2 divide-x divide-gray-200">
        {/* Original File Column */}
        <div className="p-4">
          <div className="flex items-center space-x-2 mb-3">
            <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
            <span className="font-medium text-gray-900">Original File</span>
            {isNewer(originalFile, duplicateFile) && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                Newer
              </span>
            )}
            {isLarger(originalFile, duplicateFile) && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                Larger
              </span>
            )}
          </div>

          <FileDetails
            file={originalFile}
            metadata={originalMetadata}
            showDetails={showDifferences}
          />
        </div>

        {/* Duplicate File Column */}
        <div className="p-4">
          <div className="flex items-center space-x-2 mb-3">
            <div className="w-3 h-3 bg-orange-500 rounded-full"></div>
            <span className="font-medium text-gray-900">Duplicate File</span>
            {isNewer(duplicateFile, originalFile) && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                Newer
              </span>
            )}
            {isLarger(duplicateFile, originalFile) && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                Larger
              </span>
            )}
          </div>

          <FileDetails
            file={duplicateFile}
            metadata={duplicateMetadata}
            showDetails={showDifferences}
          />
        </div>
      </div>

      {/* Differences Summary */}
      {showDifferences && originalMetadata && duplicateMetadata && (
        <div className="border-t border-gray-200 px-4 py-3 bg-amber-50">
          <div className="flex items-start space-x-2">
            <svg
              className="w-5 h-5 text-amber-600 mt-0.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <div>
              <h4 className="font-medium text-amber-800">Differences Detected</h4>
              <div className="text-sm text-amber-700 mt-1">
                {getDifferences().length > 0 ? (
                  <span>Different: {getDifferences().join(', ')}</span>
                ) : (
                  <span>Files are identical except for location</span>
                )}
              </div>
              {isLoadingMetadata && (
                <div className="flex items-center space-x-2 mt-2">
                  <div className="animate-spin w-4 h-4 border-2 border-amber-600 border-t-transparent rounded-full"></div>
                  <span className="text-sm text-amber-600">Loading detailed metadata...</span>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Action Buttons */}
      {showResolveButtons && onResolve && (
        <div className="border-t border-gray-200 px-4 py-3 bg-gray-50">
          <div className="flex items-center justify-between">
            <div className="text-sm text-gray-600">Choose which file to keep:</div>
            <div className="flex space-x-3">
              <button
                onClick={() => onResolve(true)}
                className="px-3 py-2 border border-blue-300 text-blue-700 rounded hover:bg-blue-50 transition-colors text-sm font-medium"
              >
                Keep Original
              </button>
              <button
                onClick={() => onResolve(false)}
                className="px-3 py-2 border border-orange-300 text-orange-700 rounded hover:bg-orange-50 transition-colors text-sm font-medium"
              >
                Keep Duplicate
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

interface FileDetailsProps {
  file: DuplicateFile
  metadata: FileMetadata | null
  showDetails: boolean
}

const FileDetails: React.FC<FileDetailsProps> = ({ file, metadata, showDetails }) => {
  const formatDate = (date: Date) => {
    return date.toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  return (
    <div className="space-y-3">
      {/* Basic Info */}
      <div>
        <h4 className="font-medium text-gray-900 truncate" title={file.name}>
          {file.name}
        </h4>
        <p className="text-sm text-gray-500 truncate" title={file.path}>
          {file.path}
        </p>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 gap-2 text-sm">
        <div className="flex justify-between">
          <span className="text-gray-600">Size:</span>
          <span className="font-medium">{DeduplicationService.formatFileSize(file.size)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Modified:</span>
          <span className="font-medium">{formatDate(file.lastModified)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Hash:</span>
          <span className="font-mono text-xs text-gray-500 truncate" title={file.hash}>
            {file.hash.substring(0, 8)}...
          </span>
        </div>
      </div>

      {/* Detailed Metadata */}
      {showDetails && metadata && (
        <div className="pt-3 border-t border-gray-200">
          <div className="grid grid-cols-1 gap-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600">Created:</span>
              <span className="text-gray-800">{formatDate(metadata.created)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Accessed:</span>
              <span className="text-gray-800">{formatDate(metadata.accessed)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Permissions:</span>
              <span className="font-mono text-gray-800">{metadata.permissions}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Full Hash:</span>
              <span className="font-mono text-xs text-gray-500 break-all">{metadata.checksum}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default DuplicateFileComparison
