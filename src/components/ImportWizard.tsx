import React, { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  ArrowLeft,
  ArrowRight,
  CheckCircle,
  FileText,
  Settings,
  Upload,
  AlertTriangle,
  BookOpen,
  Users,
  Layers,
  Database,
} from 'lucide-react'
import DropZone from './DropZone'

// Types for workspace templates and import settings
type WorkspaceTemplate =
  | 'Basic'
  | 'Research'
  | 'Documentation'
  | 'Collaboration'
  | { Custom: string }

interface ImportSettings {
  allowed_extensions: string[]
  max_file_size: number
  auto_process: boolean
  duplicate_handling: 'Prompt' | 'Skip' | 'Replace' | 'KeepBoth'
}

interface WorkspaceAISettings {
  preferred_local_model?: string
  cloud_fallback: boolean
  privacy_mode: boolean
}

interface ImportPreset {
  name: string
  template: WorkspaceTemplate
  description: string
  icon: React.ReactNode
  import_settings: ImportSettings
  ai_settings: WorkspaceAISettings
}

interface FilePreview {
  name: string
  size: number
  type: string
  path?: string
  validation_status: 'pending' | 'valid' | 'invalid' | 'duplicate'
  validation_message?: string
  estimated_processing_time?: number
}

interface BatchProcessingResult {
  operation_id: string
  total_files: number
  successful_files: number
  failed_files: number
  processing_time: { secs: number; nanos: number }
  files: FileProcessingResult[]
  partial_success: boolean
  can_retry_failures: boolean
}

interface FileProcessingResult {
  file_path: string
  success: boolean
  processing_time: { secs: number; nanos: number }
  file_hash?: string
  validation_result?: unknown
  error_message?: string
  retry_count: number
}

interface ImportWizardProps {
  onImportComplete?: (results: BatchProcessingResult) => void
  onCancel?: () => void
  currentWorkspace?: string
  className?: string
}

const WIZARD_STEPS = [
  {
    id: 'preset',
    title: 'Choose Import Preset',
    description: 'Select how you want to import files',
  },
  { id: 'files', title: 'Select Files', description: 'Choose files to import' },
  { id: 'configure', title: 'Configure Import', description: 'Customize import settings' },
  { id: 'preview', title: 'Preview & Confirm', description: 'Review your import configuration' },
  { id: 'import', title: 'Import Progress', description: 'Processing your files' },
]

const IMPORT_PRESETS: ImportPreset[] = [
  {
    name: 'Basic Import',
    template: 'Basic',
    description: 'Standard document import with basic processing',
    icon: <Layers className="h-6 w-6" />,
    import_settings: {
      allowed_extensions: ['.docx', '.pdf', '.md', '.txt', '.csv', '.json'],
      max_file_size: 100 * 1024 * 1024, // 100MB
      auto_process: true,
      duplicate_handling: 'Prompt',
    },
    ai_settings: {
      preferred_local_model: 'llama3.2-3b',
      cloud_fallback: true,
      privacy_mode: false,
    },
  },
  {
    name: 'Research Import',
    template: 'Research',
    description: 'Import research documents with enhanced analysis',
    icon: <BookOpen className="h-6 w-6" />,
    import_settings: {
      allowed_extensions: [
        '.docx',
        '.pdf',
        '.md',
        '.txt',
        '.csv',
        '.json',
        '.xlsx',
        '.bib',
        '.ris',
        '.tsv',
      ],
      max_file_size: 200 * 1024 * 1024, // 200MB
      auto_process: true,
      duplicate_handling: 'KeepBoth',
    },
    ai_settings: {
      preferred_local_model: 'llama3.1-8b',
      cloud_fallback: true,
      privacy_mode: true,
    },
  },
  {
    name: 'Documentation Import',
    template: 'Documentation',
    description: 'Import documentation files with structure preservation',
    icon: <FileText className="h-6 w-6" />,
    import_settings: {
      allowed_extensions: ['.md', '.txt', '.docx', '.html', '.rst', '.adoc'],
      max_file_size: 50 * 1024 * 1024, // 50MB
      auto_process: true,
      duplicate_handling: 'Replace',
    },
    ai_settings: {
      preferred_local_model: 'llama3.2-3b',
      cloud_fallback: true,
      privacy_mode: false,
    },
  },
  {
    name: 'Collaboration Import',
    template: 'Collaboration',
    description: 'Import files for team collaboration with media support',
    icon: <Users className="h-6 w-6" />,
    import_settings: {
      allowed_extensions: [
        '.docx',
        '.pdf',
        '.md',
        '.txt',
        '.pptx',
        '.xlsx',
        '.png',
        '.jpg',
        '.jpeg',
        '.gif',
      ],
      max_file_size: 150 * 1024 * 1024, // 150MB
      auto_process: true,
      duplicate_handling: 'Prompt',
    },
    ai_settings: {
      preferred_local_model: 'llama3.2-3b',
      cloud_fallback: true,
      privacy_mode: false,
    },
  },
]

const ImportWizard: React.FC<ImportWizardProps> = ({
  onImportComplete,
  onCancel,
  currentWorkspace,
  className = '',
}) => {
  const [currentStep, setCurrentStep] = useState(0)
  const [selectedPreset, setSelectedPreset] = useState<ImportPreset | null>(null)
  const [selectedFiles, setSelectedFiles] = useState<File[]>([])
  const [filePreviews, setFilePreviews] = useState<FilePreview[]>([])
  const [customSettings, setCustomSettings] = useState<ImportSettings | null>(null)
  const [isProcessing, setIsProcessing] = useState(false)
  const [processingProgress, setProcessingProgress] = useState(0)
  const [processingStatus, setProcessingStatus] = useState('')
  const [importResults, setImportResults] = useState<BatchProcessingResult | null>(null)

  // Format file size for display
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  // Get current import settings (custom or preset)
  const getCurrentImportSettings = useCallback((): ImportSettings => {
    if (customSettings) return customSettings
    if (selectedPreset?.import_settings) return selectedPreset.import_settings
    return (
      IMPORT_PRESETS[0]?.import_settings || {
        allowed_extensions: ['.docx', '.pdf', '.md', '.txt', '.csv', '.json'],
        max_file_size: 100 * 1024 * 1024,
        auto_process: true,
        duplicate_handling: 'Prompt',
      }
    )
  }, [customSettings, selectedPreset])

  // Validate files against current settings
  const validateFiles = useCallback(
    async (files: File[]) => {
      const settings = getCurrentImportSettings()
      const previews: FilePreview[] = []

      for (const file of files) {
        const preview: FilePreview = {
          name: file.name,
          size: file.size,
          type: file.type || 'unknown',
          validation_status: 'pending',
          estimated_processing_time: Math.max(1, Math.floor(file.size / (1024 * 1024))), // Rough estimate
        }

        // Check file size
        if (file.size > settings.max_file_size) {
          preview.validation_status = 'invalid'
          preview.validation_message = `File exceeds maximum size of ${formatFileSize(settings.max_file_size)}`
          previews.push(preview)
          continue
        }

        // Check file extension
        const fileExtension = '.' + (file.name.split('.').pop()?.toLowerCase() || '')
        if (fileExtension && !settings.allowed_extensions.includes(fileExtension)) {
          preview.validation_status = 'invalid'
          preview.validation_message = `File type not allowed. Allowed: ${settings.allowed_extensions.join(', ')}`
          previews.push(preview)
          continue
        }

        // TODO: Check for duplicates using backend command
        try {
          // This would be a real duplicate check in the actual implementation
          preview.validation_status = 'valid'
          preview.validation_message = 'File is valid and ready for import'
        } catch {
          preview.validation_status = 'invalid'
          preview.validation_message = 'Failed to validate file'
        }

        previews.push(preview)
      }

      setFilePreviews(previews)
    },
    [getCurrentImportSettings]
  )

  // Handle file selection from DropZone
  const handleFileDrop = useCallback(
    (files: File[]) => {
      setSelectedFiles(files)
      validateFiles(files)
    },
    [validateFiles]
  )

  // Handle preset selection
  const handlePresetSelect = (preset: ImportPreset) => {
    setSelectedPreset(preset)
    setCustomSettings(null) // Reset custom settings when selecting preset

    // Re-validate files if any are selected
    if (selectedFiles.length > 0) {
      validateFiles(selectedFiles)
    }
  }

  // Handle custom settings change
  const handleCustomSettingsChange = (newSettings: Partial<ImportSettings>) => {
    const currentSettings = getCurrentImportSettings()
    const updatedSettings = { ...currentSettings, ...newSettings }
    setCustomSettings(updatedSettings)

    // Re-validate files with new settings
    if (selectedFiles.length > 0) {
      validateFiles(selectedFiles)
    }
  }

  // Start import process
  const startImport = async () => {
    if (!selectedFiles.length || !selectedPreset) return

    setIsProcessing(true)
    setProcessingProgress(0)
    setProcessingStatus('Preparing import...')

    try {
      // Convert files to file paths (in a real implementation, files would be saved temporarily)
      const filePaths = selectedFiles.map(file => file.name) // Simplified for demo

      const importConfig = {
        preset: selectedPreset.name,
        settings: getCurrentImportSettings(),
        ai_settings: selectedPreset.ai_settings,
        workspace_path: currentWorkspace || '',
      }

      setProcessingStatus('Processing files...')

      // Simulate progress updates (in real implementation, this would come from backend events)
      const progressInterval = setInterval(() => {
        setProcessingProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval)
            return prev
          }
          return prev + 10
        })
      }, 500)

      // Call backend import command (simplified)
      const results = await invoke<BatchProcessingResult>('batch_import_files', {
        files: filePaths,
        config: importConfig,
      })

      clearInterval(progressInterval)
      setProcessingProgress(100)
      setProcessingStatus('Import completed successfully!')
      setImportResults(results)

      // Auto-advance to completion after a brief delay
      setTimeout(() => {
        onImportComplete?.(results)
      }, 2000)
    } catch (error) {
      setProcessingStatus(`Import failed: ${error}`)
      setIsProcessing(false)
    }
  }

  // Navigation functions
  const canGoNext = () => {
    switch (currentStep) {
      case 0:
        return selectedPreset !== null
      case 1:
        return selectedFiles.length > 0 && filePreviews.some(f => f.validation_status === 'valid')
      case 2:
        return true // Settings are optional
      case 3:
        return true // Preview step
      case 4:
        return false // Import step - no next
      default:
        return false
    }
  }

  const canGoPrevious = () => {
    return currentStep > 0 && !isProcessing
  }

  const goNext = () => {
    if (canGoNext()) {
      if (currentStep === 3) {
        // Start import when moving from preview to import step
        setCurrentStep(4)
        startImport()
      } else {
        setCurrentStep(prev => Math.min(prev + 1, WIZARD_STEPS.length - 1))
      }
    }
  }

  const goPrevious = () => {
    if (canGoPrevious()) {
      setCurrentStep(prev => Math.max(prev - 1, 0))
    }
  }

  // Step content renderers
  const renderPresetStep = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-lg font-semibold text-gray-900">Choose Import Preset</h3>
        <p className="text-sm text-gray-600 mt-2">
          Select a preset that matches your import needs. Each preset optimizes settings for
          different use cases.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {IMPORT_PRESETS.map(preset => (
          <div
            key={preset.name}
            className={`relative border-2 rounded-lg p-6 cursor-pointer transition-all ${
              selectedPreset?.name === preset.name
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
            }`}
            onClick={() => handlePresetSelect(preset)}
          >
            <div className="flex items-start space-x-4">
              <div
                className={`p-2 rounded-lg ${
                  selectedPreset?.name === preset.name
                    ? 'bg-blue-200 text-blue-600'
                    : 'bg-gray-200 text-gray-600'
                }`}
              >
                {preset.icon}
              </div>
              <div className="flex-1">
                <h4 className="font-medium text-gray-900">{preset.name}</h4>
                <p className="text-sm text-gray-600 mt-1">{preset.description}</p>
                <div className="mt-3 space-y-1">
                  <div className="text-xs text-gray-500">
                    Extensions: {preset.import_settings.allowed_extensions.slice(0, 4).join(', ')}
                    {preset.import_settings.allowed_extensions.length > 4 && '...'}
                  </div>
                  <div className="text-xs text-gray-500">
                    Max size: {formatFileSize(preset.import_settings.max_file_size)}
                  </div>
                </div>
              </div>
            </div>

            {selectedPreset?.name === preset.name && (
              <div className="absolute top-2 right-2">
                <CheckCircle className="h-5 w-5 text-blue-500" />
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )

  const renderFilesStep = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-lg font-semibold text-gray-900">Select Files to Import</h3>
        <p className="text-sm text-gray-600 mt-2">
          Drop files or click to browse. Files will be validated according to your selected preset.
        </p>
      </div>

      <DropZone
        onFileDrop={handleFileDrop}
        acceptedFileTypes={getCurrentImportSettings().allowed_extensions}
        maxFileSize={getCurrentImportSettings().max_file_size}
        maxFiles={20}
      />

      {filePreviews.length > 0 && (
        <div className="space-y-3">
          <h4 className="font-medium text-gray-900">Selected Files ({filePreviews.length})</h4>
          <div className="max-h-60 overflow-y-auto space-y-2">
            {filePreviews.map((file, index) => (
              <div
                key={index}
                className={`flex items-center justify-between p-3 rounded-lg border ${
                  file.validation_status === 'valid'
                    ? 'border-green-200 bg-green-50'
                    : file.validation_status === 'invalid'
                      ? 'border-red-200 bg-red-50'
                      : 'border-yellow-200 bg-yellow-50'
                }`}
              >
                <div className="flex items-center space-x-3">
                  <div
                    className={`p-1 rounded ${
                      file.validation_status === 'valid'
                        ? 'bg-green-200 text-green-600'
                        : file.validation_status === 'invalid'
                          ? 'bg-red-200 text-red-600'
                          : 'bg-yellow-200 text-yellow-600'
                    }`}
                  >
                    {file.validation_status === 'valid' ? (
                      <CheckCircle className="h-4 w-4" />
                    ) : (
                      <AlertTriangle className="h-4 w-4" />
                    )}
                  </div>
                  <div>
                    <div className="font-medium text-sm">{file.name}</div>
                    <div className="text-xs text-gray-500">
                      {formatFileSize(file.size)} • ~{file.estimated_processing_time}s to process
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div
                    className={`text-xs font-medium ${
                      file.validation_status === 'valid'
                        ? 'text-green-600'
                        : file.validation_status === 'invalid'
                          ? 'text-red-600'
                          : 'text-yellow-600'
                    }`}
                  >
                    {file.validation_status === 'valid'
                      ? 'Ready'
                      : file.validation_status === 'invalid'
                        ? 'Invalid'
                        : 'Checking...'}
                  </div>
                  {file.validation_message && (
                    <div className="text-xs text-gray-500 mt-1 max-w-48 text-right">
                      {file.validation_message}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )

  const renderConfigureStep = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-lg font-semibold text-gray-900">Configure Import Settings</h3>
        <p className="text-sm text-gray-600 mt-2">
          Customize how your files will be imported and processed.
        </p>
      </div>

      <div className="space-y-6">
        {/* Duplicate Handling */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Duplicate File Handling
          </label>
          <div className="grid grid-cols-2 gap-3">
            {[
              { value: 'Prompt', label: 'Ask me', description: 'Prompt for each duplicate' },
              { value: 'Skip', label: 'Skip', description: 'Skip duplicate files' },
              { value: 'Replace', label: 'Replace', description: 'Replace existing files' },
              { value: 'KeepBoth', label: 'Keep both', description: 'Keep both versions' },
            ].map(option => (
              <div
                key={option.value}
                className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                  getCurrentImportSettings().duplicate_handling === option.value
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300'
                }`}
                onClick={() =>
                  handleCustomSettingsChange({
                    duplicate_handling: option.value as 'Prompt' | 'Skip' | 'Replace' | 'KeepBoth',
                  })
                }
              >
                <div className="font-medium text-sm">{option.label}</div>
                <div className="text-xs text-gray-500 mt-1">{option.description}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Auto Processing */}
        <div>
          <label className="flex items-center space-x-3">
            <input
              type="checkbox"
              checked={getCurrentImportSettings().auto_process}
              onChange={e => handleCustomSettingsChange({ auto_process: e.target.checked })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <div>
              <div className="font-medium text-sm">Auto-process imported files</div>
              <div className="text-xs text-gray-500">
                Automatically extract content and generate metadata
              </div>
            </div>
          </label>
        </div>

        {/* File Extensions */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Allowed File Extensions
          </label>
          <div className="flex flex-wrap gap-2">
            {getCurrentImportSettings().allowed_extensions.map(ext => (
              <span key={ext} className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                {ext}
              </span>
            ))}
          </div>
        </div>
      </div>
    </div>
  )

  const renderPreviewStep = () => {
    const validFiles = filePreviews.filter(f => f.validation_status === 'valid')
    const totalSize = validFiles.reduce((sum, file) => sum + file.size, 0)
    const estimatedTime = validFiles.reduce(
      (sum, file) => sum + (file.estimated_processing_time || 0),
      0
    )

    return (
      <div className="space-y-6">
        <div className="text-center">
          <h3 className="text-lg font-semibold text-gray-900">Review Import Configuration</h3>
          <p className="text-sm text-gray-600 mt-2">
            Review your settings before starting the import process.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Import Summary */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="font-medium text-gray-900 mb-3 flex items-center">
              <Database className="h-4 w-4 mr-2" />
              Import Summary
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Preset:</span>
                <span className="font-medium">{selectedPreset?.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Valid files:</span>
                <span className="font-medium">{validFiles.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Total size:</span>
                <span className="font-medium">{formatFileSize(totalSize)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Est. time:</span>
                <span className="font-medium">~{estimatedTime}s</span>
              </div>
            </div>
          </div>

          {/* Settings Summary */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="font-medium text-gray-900 mb-3 flex items-center">
              <Settings className="h-4 w-4 mr-2" />
              Settings
            </h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Duplicates:</span>
                <span className="font-medium">{getCurrentImportSettings().duplicate_handling}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Auto-process:</span>
                <span className="font-medium">
                  {getCurrentImportSettings().auto_process ? 'Yes' : 'No'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">AI Model:</span>
                <span className="font-medium">
                  {selectedPreset?.ai_settings.preferred_local_model || 'Default'}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Warning for invalid files */}
        {filePreviews.some(f => f.validation_status !== 'valid') && (
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
            <div className="flex items-start">
              <AlertTriangle className="h-5 w-5 text-yellow-600 mt-0.5 mr-3" />
              <div>
                <h5 className="font-medium text-yellow-800">Some files will be skipped</h5>
                <p className="text-sm text-yellow-700 mt-1">
                  {filePreviews.filter(f => f.validation_status !== 'valid').length} files have
                  validation issues and will not be imported.
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    )
  }

  const renderImportStep = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-lg font-semibold text-gray-900">Import Progress</h3>
        <p className="text-sm text-gray-600 mt-2">{processingStatus}</p>
      </div>

      <div className="space-y-4">
        {/* Progress Bar */}
        <div className="w-full bg-gray-200 rounded-full h-2">
          <div
            className="bg-blue-600 h-2 rounded-full transition-all duration-300"
            style={{ width: `${processingProgress}%` }}
          />
        </div>

        <div className="text-center text-sm text-gray-600">{processingProgress}% complete</div>

        {/* Processing Status */}
        <div className="bg-gray-50 rounded-lg p-4">
          <div className="flex items-center justify-center space-x-2">
            {isProcessing ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600" />
                <span className="text-sm text-gray-600">Processing files...</span>
              </>
            ) : importResults ? (
              <>
                <CheckCircle className="h-5 w-5 text-green-600" />
                <span className="text-sm text-green-600">Import completed successfully!</span>
              </>
            ) : (
              <>
                <AlertTriangle className="h-5 w-5 text-red-600" />
                <span className="text-sm text-red-600">Import failed</span>
              </>
            )}
          </div>
        </div>

        {importResults && (
          <div className="bg-green-50 border border-green-200 rounded-lg p-4">
            <h4 className="font-medium text-green-800 mb-2">Import Results</h4>
            <div className="text-sm text-green-700">
              <div>Successfully imported {importResults.successful_files || 0} files</div>
              {importResults.failed_files > 0 && (
                <div>Failed to import {importResults.failed_files} files</div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  )

  return (
    <div className={`bg-white rounded-lg shadow-lg ${className}`}>
      {/* Header */}
      <div className="px-6 py-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold text-gray-900">Import Wizard</h2>
          {onCancel && (
            <button
              onClick={onCancel}
              className="text-gray-400 hover:text-gray-600 transition-colors"
              disabled={isProcessing}
            >
              ×
            </button>
          )}
        </div>

        {/* Step Indicator */}
        <div className="mt-4">
          <div className="flex items-center justify-between">
            {WIZARD_STEPS.map((step, index) => (
              <div key={step.id} className="flex items-center">
                <div
                  className={`flex items-center justify-center w-8 h-8 rounded-full text-sm font-medium ${
                    index < currentStep
                      ? 'bg-blue-600 text-white'
                      : index === currentStep
                        ? 'bg-blue-100 text-blue-600 border-2 border-blue-600'
                        : 'bg-gray-200 text-gray-400'
                  }`}
                >
                  {index < currentStep ? <CheckCircle className="h-4 w-4" /> : index + 1}
                </div>
                {index < WIZARD_STEPS.length - 1 && (
                  <div
                    className={`w-16 h-1 mx-2 ${
                      index < currentStep ? 'bg-blue-600' : 'bg-gray-200'
                    }`}
                  />
                )}
              </div>
            ))}
          </div>
          <div className="mt-2 text-center">
            <div className="font-medium text-gray-900">
              {WIZARD_STEPS[currentStep]?.title || 'Step'}
            </div>
            <div className="text-sm text-gray-600">
              {WIZARD_STEPS[currentStep]?.description || 'Processing...'}
            </div>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="px-6 py-6 min-h-96">
        {currentStep === 0 && renderPresetStep()}
        {currentStep === 1 && renderFilesStep()}
        {currentStep === 2 && renderConfigureStep()}
        {currentStep === 3 && renderPreviewStep()}
        {currentStep === 4 && renderImportStep()}
      </div>

      {/* Footer */}
      <div className="px-6 py-4 border-t border-gray-200 flex justify-between">
        <button
          onClick={goPrevious}
          disabled={!canGoPrevious()}
          className="flex items-center space-x-2 px-4 py-2 text-gray-600 hover:text-gray-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          <span>Previous</span>
        </button>

        <div className="flex items-center space-x-3">
          {currentStep < 3 && (
            <button
              onClick={goNext}
              disabled={!canGoNext()}
              className="flex items-center space-x-2 px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <span>Next</span>
              <ArrowRight className="h-4 w-4" />
            </button>
          )}

          {currentStep === 3 && (
            <button
              onClick={goNext}
              disabled={
                !canGoNext() ||
                filePreviews.filter(f => f.validation_status === 'valid').length === 0
              }
              className="flex items-center space-x-2 px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <Upload className="h-4 w-4" />
              <span>Start Import</span>
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export default ImportWizard
