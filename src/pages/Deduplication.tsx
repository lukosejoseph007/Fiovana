import React from 'react'
import DeduplicationDemo from '../components/DeduplicationDemo'

const Deduplication: React.FC = () => {
  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          File Deduplication
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Detect and manage duplicate files to optimize storage and organization
        </p>
      </div>

      {/* Deduplication Component */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <DeduplicationDemo />
      </div>
    </div>
  )
}

export default Deduplication
