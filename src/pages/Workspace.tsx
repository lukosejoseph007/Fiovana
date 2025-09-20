import React, { useState } from 'react'
import { WorkspaceSwitcher } from '../components'

interface RecentWorkspace {
  path: string
  name: string
  last_accessed: string
  access_count: number
  is_favorite: boolean
  template: 'Basic' | 'Research' | 'Documentation' | 'Collaboration' | { Custom: string }
}

const Workspace: React.FC = () => {
  const [selectedWorkspace, setSelectedWorkspace] = useState<RecentWorkspace | null>(null)

  const handleWorkspaceSelected = (workspace: RecentWorkspace) => {
    setSelectedWorkspace(workspace)
    console.log('Selected workspace:', workspace)
    // TODO: Implement workspace loading logic
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Workspace Management</h1>
          <p className="mt-2 text-gray-600">
            Manage your project workspaces and switch between recent projects.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Workspace Switcher */}
          <div className="lg:col-span-2">
            <div className="bg-white rounded-lg shadow">
              <WorkspaceSwitcher
                onWorkspaceSelected={handleWorkspaceSelected}
                currentWorkspace={selectedWorkspace?.path}
              />
            </div>
          </div>

          {/* Workspace Details */}
          <div className="lg:col-span-1">
            <div className="bg-white rounded-lg shadow p-6">
              {selectedWorkspace ? (
                <div className="space-y-4">
                  <h3 className="text-lg font-medium text-gray-900">Current Workspace</h3>

                  <div className="space-y-2">
                    <div>
                      <label className="text-sm font-medium text-gray-500">Name</label>
                      <p className="text-gray-900">{selectedWorkspace.name}</p>
                    </div>

                    <div>
                      <label className="text-sm font-medium text-gray-500">Path</label>
                      <p className="text-gray-900 text-sm font-mono break-all">
                        {selectedWorkspace.path}
                      </p>
                    </div>

                    <div>
                      <label className="text-sm font-medium text-gray-500">Template</label>
                      <p className="text-gray-900">
                        {typeof selectedWorkspace.template === 'string'
                          ? selectedWorkspace.template
                          : selectedWorkspace.template.Custom}
                      </p>
                    </div>

                    <div>
                      <label className="text-sm font-medium text-gray-500">Last Accessed</label>
                      <p className="text-gray-900">
                        {new Date(selectedWorkspace.last_accessed).toLocaleString()}
                      </p>
                    </div>

                    <div>
                      <label className="text-sm font-medium text-gray-500">Access Count</label>
                      <p className="text-gray-900">{selectedWorkspace.access_count} times</p>
                    </div>
                  </div>

                  <div className="pt-4 border-t">
                    <button
                      className="w-full bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 transition-colors"
                      onClick={() => {
                        // TODO: Implement workspace opening logic
                        console.log('Opening workspace:', selectedWorkspace.path)
                      }}
                    >
                      Open Workspace
                    </button>
                  </div>
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500">
                  <p>Select a workspace to see details</p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Workspace
