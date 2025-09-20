import React, { useState } from 'react'
import {
  DuplicateDetectionDialog,
  DuplicateFileComparison,
  DeduplicationStats,
  DuplicateResolutionPolicyConfig,
} from '../components'
import {
  DuplicateFile,
  DuplicateAction,
  DuplicateResolutionPolicy,
  GarbageCollectionResult,
} from '../types/deduplication'

const DeduplicationDemo: React.FC = () => {
  const [showDuplicateDialog, setShowDuplicateDialog] = useState(false)
  const [showComparison, setShowComparison] = useState(false)
  const [showStats, setShowStats] = useState(false)
  const [showPolicyConfig, setShowPolicyConfig] = useState(false)
  const [currentPolicy, setCurrentPolicy] = useState<DuplicateResolutionPolicy>({
    auto_deduplicate: false,
    always_prompt: true,
    prefer_newest: false,
    prefer_largest: false,
  })

  // Mock data for demonstration
  const mockOriginalFile: DuplicateFile = {
    path: '/workspace/documents/report.pdf',
    name: 'report.pdf',
    size: 2048576, // 2MB
    hash: 'a1b2c3d4e5f6789012345678901234567890abcdef',
    lastModified: new Date('2024-01-15T10:30:00Z'),
  }

  const mockDuplicateFiles: DuplicateFile[] = [
    {
      path: '/workspace/imports/report.pdf',
      name: 'report.pdf',
      size: 2048576,
      hash: 'a1b2c3d4e5f6789012345678901234567890abcdef',
      lastModified: new Date('2024-01-15T10:30:00Z'),
    },
    {
      path: '/workspace/archives/old_report.pdf',
      name: 'old_report.pdf',
      size: 2048576,
      hash: 'a1b2c3d4e5f6789012345678901234567890abcdef',
      lastModified: new Date('2024-01-14T15:45:00Z'),
    },
    {
      path: '/workspace/temp/report_copy.pdf',
      name: 'report_copy.pdf',
      size: 2048576,
      hash: 'a1b2c3d4e5f6789012345678901234567890abcdef',
      lastModified: new Date('2024-01-16T09:15:00Z'),
    },
  ]

  const handleDuplicateResolution = async (action: DuplicateAction) => {
    console.log('Resolving duplicates with action:', action)

    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000))

    setShowDuplicateDialog(false)

    // Show notification or update state
    alert(`Duplicates resolved with action: ${action.type}`)
  }

  const handleComparisonResolve = (keepOriginal: boolean) => {
    console.log('Comparison resolved, keep original:', keepOriginal)
    alert(`Resolved: ${keepOriginal ? 'Keeping original' : 'Keeping duplicate'} file`)
  }

  const handlePolicyChange = (policy: DuplicateResolutionPolicy) => {
    setCurrentPolicy(policy)
    console.log('Policy updated:', policy)
    alert('Deduplication policy saved successfully!')
  }

  const handleGarbageCollectionComplete = (result: GarbageCollectionResult) => {
    console.log('Garbage collection completed:', result)
    alert(
      `Garbage collection completed! Deleted ${result.deleted_files} files, freed ${(result.space_freed / 1024 / 1024).toFixed(1)}MB`
    )
  }

  return (
    <div className="p-6 space-y-6 bg-gray-50 min-h-screen">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">File Deduplication System Demo</h1>

        {/* Control Panel */}
        <div className="bg-white rounded-lg shadow p-6 mb-8">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">Demo Controls</h2>
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <button
              onClick={() => setShowDuplicateDialog(true)}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              Show Duplicate Dialog
            </button>
            <button
              onClick={() => setShowComparison(!showComparison)}
              className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
            >
              {showComparison ? 'Hide' : 'Show'} File Comparison
            </button>
            <button
              onClick={() => setShowStats(!showStats)}
              className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 transition-colors"
            >
              {showStats ? 'Hide' : 'Show'} Stats
            </button>
            <button
              onClick={() => setShowPolicyConfig(!showPolicyConfig)}
              className="px-4 py-2 bg-amber-600 text-white rounded hover:bg-amber-700 transition-colors"
            >
              {showPolicyConfig ? 'Hide' : 'Show'} Policy Config
            </button>
          </div>
        </div>

        {/* File Comparison Component */}
        {showComparison && (
          <div className="mb-8">
            <h2 className="text-xl font-semibold text-gray-900 mb-4">File Comparison</h2>
            <DuplicateFileComparison
              originalFile={mockOriginalFile}
              duplicateFile={mockDuplicateFiles[0]!}
              onResolve={handleComparisonResolve}
              showResolveButtons={true}
            />
          </div>
        )}

        {/* Deduplication Statistics */}
        {showStats && (
          <div className="mb-8">
            <h2 className="text-xl font-semibold text-gray-900 mb-4">Deduplication Statistics</h2>
            <DeduplicationStats
              workspacePath="/workspace/demo"
              showGarbageCollection={true}
              onGarbageCollectionComplete={handleGarbageCollectionComplete}
            />
          </div>
        )}

        {/* Policy Configuration */}
        {showPolicyConfig && (
          <div className="mb-8">
            <h2 className="text-xl font-semibold text-gray-900 mb-4">
              Resolution Policy Configuration
            </h2>
            <DuplicateResolutionPolicyConfig
              initialPolicy={currentPolicy}
              onPolicyChange={handlePolicyChange}
            />
          </div>
        )}

        {/* Feature Overview */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">Feature Overview</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-medium text-gray-900 mb-2">✅ Implemented Features</h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Duplicate file detection dialog with multiple resolution options</li>
                <li>• Side-by-side file comparison with metadata details</li>
                <li>• Storage statistics with space savings calculation</li>
                <li>• Garbage collection with automatic cleanup</li>
                <li>• Configurable resolution policies</li>
                <li>• Hard link creation for space efficiency</li>
                <li>• Batch duplicate processing</li>
                <li>• Progress tracking and cancellation</li>
              </ul>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 mb-2">🔗 Integration Points</h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Tauri backend commands for file operations</li>
                <li>• React components with TypeScript types</li>
                <li>• Service layer for API abstraction</li>
                <li>• Configuration management integration</li>
                <li>• Workspace-aware deduplication</li>
                <li>• Security and validation framework</li>
                <li>• Notification system integration</li>
                <li>• Progress persistence system</li>
              </ul>
            </div>
          </div>

          <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <div className="flex items-start space-x-3">
              <svg
                className="w-5 h-5 text-blue-600 mt-0.5"
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
                <h4 className="font-medium text-blue-900">Demo Note</h4>
                <p className="text-sm text-blue-800 mt-1">
                  This demo uses mock data and simulated API calls. In the actual implementation,
                  these components integrate with the Rust backend for real file operations,
                  deduplication processing, and workspace management.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Duplicate Detection Dialog */}
      <DuplicateDetectionDialog
        isOpen={showDuplicateDialog}
        duplicateFiles={mockDuplicateFiles}
        originalFile={mockOriginalFile}
        workspacePath="/workspace/demo"
        onResolve={handleDuplicateResolution}
        onCancel={() => setShowDuplicateDialog(false)}
      />
    </div>
  )
}

export default DeduplicationDemo
