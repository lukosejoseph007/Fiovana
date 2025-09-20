import React, { useState } from 'react'
import ImportWizard from '../components/ImportWizard'
import { FileUp, CheckCircle, AlertCircle } from 'lucide-react'

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

const ImportWizardPage: React.FC = () => {
  const [showWizard, setShowWizard] = useState(false)
  const [importResults, setImportResults] = useState<BatchProcessingResult | null>(null)

  const handleImportComplete = (results: BatchProcessingResult) => {
    setImportResults(results)
    setShowWizard(false)
  }

  const handleStartImport = () => {
    setImportResults(null)
    setShowWizard(true)
  }

  const handleCancelImport = () => {
    setShowWizard(false)
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="text-center">
        <div className="flex items-center justify-center mb-4">
          <div className="p-3 bg-blue-100 rounded-full">
            <FileUp className="h-8 w-8 text-blue-600" />
          </div>
        </div>
        <h1 className="text-3xl font-bold text-gray-900">Import Wizard</h1>
        <p className="text-lg text-gray-600 mt-2">
          Easily import files with guided setup and intelligent presets
        </p>
      </div>

      {!showWizard && !importResults && (
        <div className="max-w-4xl mx-auto">
          {/* Features Overview */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-2 bg-green-100 rounded-lg">
                  <CheckCircle className="h-6 w-6 text-green-600" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900">Smart Presets</h3>
              </div>
              <p className="text-gray-600">
                Choose from pre-configured import settings optimized for different workflows:
                Research, Documentation, Collaboration, and Basic imports.
              </p>
            </div>

            <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-2 bg-blue-100 rounded-lg">
                  <FileUp className="h-6 w-6 text-blue-600" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900">Batch Processing</h3>
              </div>
              <p className="text-gray-600">
                Import multiple files at once with parallel processing, progress tracking, and
                intelligent error handling for maximum efficiency.
              </p>
            </div>

            <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-2 bg-purple-100 rounded-lg">
                  <AlertCircle className="h-6 w-6 text-purple-600" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900">Validation & Preview</h3>
              </div>
              <p className="text-gray-600">
                Files are validated before import with detailed preview and confirmation. Duplicate
                detection and format validation ensure data integrity.
              </p>
            </div>
          </div>

          {/* Quick Start Guide */}
          <div className="bg-gray-50 rounded-lg p-6 mb-8">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">How It Works</h3>
            <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
              {[
                { step: 1, title: 'Choose Preset', desc: 'Select import configuration' },
                { step: 2, title: 'Select Files', desc: 'Drop or browse files' },
                { step: 3, title: 'Configure', desc: 'Customize settings' },
                { step: 4, title: 'Preview', desc: 'Review before import' },
                { step: 5, title: 'Import', desc: 'Process files' },
              ].map(item => (
                <div key={item.step} className="text-center">
                  <div className="w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-semibold mx-auto mb-2">
                    {item.step}
                  </div>
                  <h4 className="font-medium text-gray-900 text-sm">{item.title}</h4>
                  <p className="text-xs text-gray-600 mt-1">{item.desc}</p>
                </div>
              ))}
            </div>
          </div>

          {/* Start Import Button */}
          <div className="text-center">
            <button
              onClick={handleStartImport}
              className="inline-flex items-center space-x-3 px-8 py-4 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-lg font-medium shadow-lg"
            >
              <FileUp className="h-6 w-6" />
              <span>Start Import Wizard</span>
            </button>
            <p className="text-sm text-gray-500 mt-3">
              Import files with guided setup and intelligent processing
            </p>
          </div>
        </div>
      )}

      {/* Import Wizard Modal/Component */}
      {showWizard && (
        <div className="max-w-5xl mx-auto">
          <ImportWizard
            onImportComplete={handleImportComplete}
            onCancel={handleCancelImport}
            className="w-full"
          />
        </div>
      )}

      {/* Import Results */}
      {importResults && (
        <div className="max-w-4xl mx-auto">
          <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <div className="flex items-center space-x-3 mb-4">
              <CheckCircle className="h-6 w-6 text-green-600" />
              <h3 className="text-lg font-semibold text-gray-900">Import Completed</h3>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
              <div className="text-center p-4 bg-green-50 rounded-lg">
                <div className="text-2xl font-bold text-green-600">
                  {importResults.successful_files || 0}
                </div>
                <div className="text-sm text-green-700">Files Imported</div>
              </div>

              <div className="text-center p-4 bg-gray-50 rounded-lg">
                <div className="text-2xl font-bold text-gray-600">
                  {importResults.total_files || 0}
                </div>
                <div className="text-sm text-gray-700">Total Files</div>
              </div>

              <div className="text-center p-4 bg-red-50 rounded-lg">
                <div className="text-2xl font-bold text-red-600">
                  {importResults.failed_files || 0}
                </div>
                <div className="text-sm text-red-700">Failed</div>
              </div>
            </div>

            {importResults.processing_time && (
              <div className="text-sm text-gray-600 mb-4">
                Processing completed in {Math.round(importResults.processing_time.secs)}s
              </div>
            )}

            <div className="flex space-x-3">
              <button
                onClick={handleStartImport}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Import More Files
              </button>
              <button
                onClick={() => setImportResults(null)}
                className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default ImportWizardPage
