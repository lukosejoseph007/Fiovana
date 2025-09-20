import React from 'react'
import FileWatcherDemo from '../components/FileWatcherDemo'

const FileWatcher: React.FC = () => {
  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">File Watcher</h1>
        <p className="text-gray-600 dark:text-gray-400">
          Monitor file system changes in real-time and track document modifications
        </p>
      </div>

      {/* File Watcher Component */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <FileWatcherDemo />
      </div>
    </div>
  )
}

export default FileWatcher
