import React, { useState, useEffect } from 'react'
import { clsx } from 'clsx'
import { DuplicateFile, DuplicateAction } from '../types/deduplication'
import { DeduplicationService } from '../services/deduplicationService'

interface DuplicateDetectionDialogProps {
  isOpen: boolean
  duplicateFiles: DuplicateFile[]
  originalFile: DuplicateFile
  workspacePath: string
  onResolve: (action: DuplicateAction) => void
  onCancel: () => void
}

const DuplicateDetectionDialog: React.FC<DuplicateDetectionDialogProps> = ({
  isOpen,
  duplicateFiles,
  originalFile,
  workspacePath: _workspacePath,
  onResolve,
  onCancel,
}) => {
  const [selectedAction, setSelectedAction] = useState<DuplicateAction['type']>('keep_original')
  const [selectedFiles, setSelectedFiles] = useState<string[]>([])
  const [showPreview, setShowPreview] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [previewError, setPreviewError] = useState<string | null>(null)

  const totalDuplicates = duplicateFiles.length
  const potentialSavings = duplicateFiles.reduce((sum, file) => sum + file.size, 0)

  useEffect(() => {
    // Reset selection when dialog opens
    if (isOpen) {
      setSelectedAction('keep_original')
      setSelectedFiles([])
      setShowPreview(false)
      setPreviewError(null)
    }
  }, [isOpen])

  const handleFileSelection = (filePath: string, isSelected: boolean) => {
    setSelectedFiles(prev =>
      isSelected ? [...prev, filePath] : prev.filter(path => path !== filePath)
    )
  }

  const handleActionChange = (action: DuplicateAction['type']) => {
    setSelectedAction(action)
    if (action !== 'keep_selected') {
      setSelectedFiles([])
    }
  }

  const handleResolve = async () => {
    setIsProcessing(true)
    try {
      const action: DuplicateAction = {
        type: selectedAction,
        selectedFiles: selectedAction === 'keep_selected' ? selectedFiles : undefined,
      }

      await onResolve(action)
    } catch (error) {
      console.error('Error resolving duplicates:', error)
      setPreviewError('Failed to resolve duplicates. Please try again.')
    } finally {
      setIsProcessing(false)
    }
  }

  const isResolutionValid = () => {
    if (selectedAction === 'keep_selected') {
      return selectedFiles.length > 0
    }
    return true
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-amber-50 border-b border-amber-200 px-6 py-4">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold text-amber-800 flex items-center">
              <svg className="w-6 h-6 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 15.5c-.77.833.192 2.5 1.732 2.5z"
                />
              </svg>
              Duplicate Files Detected
            </h2>
            <button
              onClick={onCancel}
              className="text-amber-600 hover:text-amber-800 transition-colors"
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

          <div className="mt-2 text-sm text-amber-700">
            Found {totalDuplicates} duplicate{totalDuplicates !== 1 ? 's' : ''} of this file.
            Potential space savings:{' '}
            <span className="font-medium">
              {DeduplicationService.formatFileSize(potentialSavings)}
            </span>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[60vh]">
          {/* Original File */}
          <div className="mb-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Original File</h3>
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
              <FileInfoCard file={originalFile} isOriginal={true} />
            </div>
          </div>

          {/* Duplicate Files */}
          <div className="mb-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">
              Duplicate Files ({duplicateFiles.length})
            </h3>
            <div className="space-y-3 max-h-64 overflow-y-auto">
              {duplicateFiles.map((file, index) => (
                <div key={index} className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <FileInfoCard
                    file={file}
                    isSelectable={selectedAction === 'keep_selected'}
                    isSelected={selectedFiles.includes(file.path)}
                    onSelectionChange={isSelected => handleFileSelection(file.path, isSelected)}
                  />
                </div>
              ))}
            </div>
          </div>

          {/* Resolution Options */}
          <div className="mb-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Resolution Options</h3>
            <div className="space-y-3">
              <ResolutionOption
                id="keep_original"
                title="Keep Original Only"
                description="Remove all duplicate files and keep only the original"
                selected={selectedAction === 'keep_original'}
                onChange={() => handleActionChange('keep_original')}
                icon="single"
              />

              <ResolutionOption
                id="keep_selected"
                title="Keep Selected Files"
                description="Choose which files to keep and remove the rest"
                selected={selectedAction === 'keep_selected'}
                onChange={() => handleActionChange('keep_selected')}
                icon="select"
              />

              <ResolutionOption
                id="deduplicate"
                title="Create Hard Links"
                description="Keep all files but use hard links to save space"
                selected={selectedAction === 'deduplicate'}
                onChange={() => handleActionChange('deduplicate')}
                icon="link"
              />

              <ResolutionOption
                id="keep_all"
                title="Keep All Files"
                description="Keep all files as separate copies (no space savings)"
                selected={selectedAction === 'keep_all'}
                onChange={() => handleActionChange('keep_all')}
                icon="copy"
              />
            </div>
          </div>

          {/* Preview */}
          {showPreview && (
            <div className="mb-6">
              <h3 className="text-lg font-medium text-gray-900 mb-3">Preview Changes</h3>
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                <PreviewChanges
                  action={selectedAction}
                  originalFile={originalFile}
                  duplicateFiles={duplicateFiles}
                  selectedFiles={selectedFiles}
                />
              </div>
            </div>
          )}

          {previewError && (
            <div className="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded-md">
              <p className="text-sm">{previewError}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="bg-gray-50 px-6 py-4 flex items-center justify-between border-t">
          <button
            onClick={() => setShowPreview(!showPreview)}
            className="text-blue-600 hover:text-blue-800 text-sm font-medium transition-colors"
          >
            {showPreview ? 'Hide Preview' : 'Show Preview'}
          </button>

          <div className="flex space-x-3">
            <button
              onClick={onCancel}
              className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 transition-colors"
              disabled={isProcessing}
            >
              Cancel
            </button>
            <button
              onClick={handleResolve}
              disabled={!isResolutionValid() || isProcessing}
              className={clsx('px-4 py-2 rounded-md font-medium transition-colors', {
                'bg-blue-600 text-white hover:bg-blue-700': isResolutionValid() && !isProcessing,
                'bg-gray-300 text-gray-500 cursor-not-allowed':
                  !isResolutionValid() || isProcessing,
              })}
            >
              {isProcessing ? 'Processing...' : 'Apply Resolution'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

// Supporting Components

interface FileInfoCardProps {
  file: DuplicateFile
  isOriginal?: boolean
  isSelectable?: boolean
  isSelected?: boolean
  onSelectionChange?: (isSelected: boolean) => void
}

const FileInfoCard: React.FC<FileInfoCardProps> = ({
  file,
  isOriginal = false,
  isSelectable = false,
  isSelected = false,
  onSelectionChange,
}) => {
  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center space-x-3">
        {isSelectable && (
          <input
            type="checkbox"
            checked={isSelected}
            onChange={e => onSelectionChange?.(e.target.checked)}
            className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
          />
        )}

        <div className="flex-1">
          <div className="flex items-center space-x-2">
            <span className="font-medium text-gray-900">{file.name}</span>
            {isOriginal && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                Original
              </span>
            )}
          </div>
          <div className="text-sm text-gray-500">{file.path}</div>
        </div>
      </div>

      <div className="text-right">
        <div className="text-sm font-medium text-gray-900">
          {DeduplicationService.formatFileSize(file.size)}
        </div>
        <div className="text-xs text-gray-500">
          Modified {file.lastModified.toLocaleDateString()}
        </div>
      </div>
    </div>
  )
}

interface ResolutionOptionProps {
  id: string
  title: string
  description: string
  selected: boolean
  onChange: () => void
  icon: 'single' | 'select' | 'link' | 'copy'
}

const ResolutionOption: React.FC<ResolutionOptionProps> = ({
  id,
  title,
  description,
  selected,
  onChange,
  icon,
}) => {
  const IconComponent = () => {
    switch (icon) {
      case 'single':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        )
      case 'select':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
            />
          </svg>
        )
      case 'link':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
            />
          </svg>
        )
      case 'copy':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
            />
          </svg>
        )
    }
  }

  return (
    <label
      className={clsx(
        'flex items-start space-x-3 p-4 border rounded-lg cursor-pointer transition-colors',
        {
          'border-blue-500 bg-blue-50': selected,
          'border-gray-200 hover:border-gray-300': !selected,
        }
      )}
    >
      <input
        type="radio"
        name="resolution"
        value={id}
        checked={selected}
        onChange={onChange}
        className="mt-1 w-4 h-4 text-blue-600 focus:ring-blue-500"
      />
      <div className="flex-1">
        <div className="flex items-center space-x-2">
          <span className={clsx('text-sm', selected ? 'text-blue-600' : 'text-gray-500')}>
            <IconComponent />
          </span>
          <span className="font-medium text-gray-900">{title}</span>
        </div>
        <p className="text-sm text-gray-600 mt-1">{description}</p>
      </div>
    </label>
  )
}

interface PreviewChangesProps {
  action: DuplicateAction['type']
  originalFile: DuplicateFile
  duplicateFiles: DuplicateFile[]
  selectedFiles: string[]
}

const PreviewChanges: React.FC<PreviewChangesProps> = ({
  action,
  originalFile: _originalFile,
  duplicateFiles,
  selectedFiles,
}) => {
  const getPreviewMessage = () => {
    switch (action) {
      case 'keep_original':
        return {
          title: 'Files to be removed:',
          files: duplicateFiles,
          savings: duplicateFiles.reduce((sum, f) => sum + f.size, 0),
        }
      case 'keep_selected': {
        const filesToRemove = duplicateFiles.filter(f => !selectedFiles.includes(f.path))
        return {
          title: 'Files to be removed:',
          files: filesToRemove,
          savings: filesToRemove.reduce((sum, f) => sum + f.size, 0),
        }
      }
      case 'deduplicate':
        return {
          title: 'Files to be converted to hard links:',
          files: duplicateFiles,
          savings: duplicateFiles.reduce((sum, f) => sum + f.size, 0),
        }
      case 'keep_all':
        return {
          title: 'All files will be kept as separate copies',
          files: [],
          savings: 0,
        }
    }
  }

  const preview = getPreviewMessage()

  return (
    <div>
      <h4 className="font-medium text-gray-900 mb-2">{preview.title}</h4>
      {preview.files.length > 0 ? (
        <div className="space-y-2 mb-3">
          {preview.files.map((file, index) => (
            <div key={index} className="text-sm text-gray-600 flex justify-between">
              <span>{file.name}</span>
              <span>{DeduplicationService.formatFileSize(file.size)}</span>
            </div>
          ))}
        </div>
      ) : action === 'keep_all' ? (
        <p className="text-sm text-gray-600 mb-3">No files will be removed or modified.</p>
      ) : null}

      {preview.savings > 0 && (
        <div className="bg-green-50 border border-green-200 rounded p-2">
          <span className="text-sm font-medium text-green-800">
            Space savings: {DeduplicationService.formatFileSize(preview.savings)}
          </span>
        </div>
      )}
    </div>
  )
}

export default DuplicateDetectionDialog
