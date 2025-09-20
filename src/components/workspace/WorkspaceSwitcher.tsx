import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  FolderOpen,
  Star,
  Clock,
  Search,
  Trash2,
  StarOff,
  FileText,
  Users,
  BookOpen,
  Layers,
} from 'lucide-react'

interface RecentWorkspace {
  path: string
  name: string
  last_accessed: string
  access_count: number
  is_favorite: boolean
  template: 'Basic' | 'Research' | 'Documentation' | 'Collaboration' | { Custom: string }
}

interface WorkspaceStats {
  total_files: number
  total_size: number
  import_count: number
  reference_count: number
  output_count: number
  last_import?: string
  last_output?: string
}

interface WorkspaceSwitcherProps {
  onWorkspaceSelected?: (workspace: RecentWorkspace) => void
  currentWorkspace?: string
}

const WorkspaceSwitcher: React.FC<WorkspaceSwitcherProps> = ({
  onWorkspaceSelected,
  currentWorkspace,
}) => {
  const [recentWorkspaces, setRecentWorkspaces] = useState<RecentWorkspace[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [workspaceStats, setWorkspaceStats] = useState<Record<string, WorkspaceStats>>({})

  useEffect(() => {
    loadRecentWorkspaces()
  }, [])

  const loadRecentWorkspaces = async () => {
    try {
      setLoading(true)
      setError(null)
      const workspaces = await invoke<RecentWorkspace[]>('get_recent_workspaces')
      setRecentWorkspaces(workspaces)

      // Load stats for each workspace
      const stats: Record<string, WorkspaceStats> = {}
      for (const workspace of workspaces) {
        try {
          const workspaceStats = await invoke<WorkspaceStats>('get_workspace_stats', {
            path: workspace.path,
          })
          stats[workspace.path] = workspaceStats
        } catch (err) {
          // Skip stats if there's an error loading them
          console.warn(`Failed to load stats for workspace ${workspace.path}:`, err)
        }
      }
      setWorkspaceStats(stats)
    } catch (err) {
      setError(err as string)
    } finally {
      setLoading(false)
    }
  }

  const toggleFavorite = async (workspace: RecentWorkspace) => {
    try {
      const newFavoriteStatus = await invoke<boolean>('toggle_workspace_favorite', {
        path: workspace.path,
      })

      setRecentWorkspaces(prev =>
        prev.map(w => (w.path === workspace.path ? { ...w, is_favorite: newFavoriteStatus } : w))
      )
    } catch (err) {
      setError(err as string)
    }
  }

  const removeFromRecent = async (workspace: RecentWorkspace) => {
    try {
      await invoke('remove_workspace_from_recent', {
        path: workspace.path,
      })

      setRecentWorkspaces(prev => prev.filter(w => w.path !== workspace.path))
    } catch (err) {
      setError(err as string)
    }
  }

  const handleWorkspaceClick = async (workspace: RecentWorkspace) => {
    try {
      // Update recent workspace access
      await invoke('update_recent_workspace', {
        path: workspace.path,
        name: workspace.name,
        template: getTemplateString(workspace.template),
      })

      onWorkspaceSelected?.(workspace)
    } catch (err) {
      setError(err as string)
    }
  }

  const getTemplateString = (template: RecentWorkspace['template']): string => {
    if (typeof template === 'string') {
      return template.toLowerCase()
    } else if (typeof template === 'object' && 'Custom' in template) {
      return template.Custom
    }
    return 'basic'
  }

  const getTemplateIcon = (template: RecentWorkspace['template']) => {
    const templateStr = getTemplateString(template)
    switch (templateStr) {
      case 'research':
        return <BookOpen className="h-4 w-4" />
      case 'documentation':
        return <FileText className="h-4 w-4" />
      case 'collaboration':
        return <Users className="h-4 w-4" />
      default:
        return <Layers className="h-4 w-4" />
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  const formatLastAccessed = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffTime = Math.abs(now.getTime() - date.getTime())
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))

    if (diffDays === 1) return 'Today'
    if (diffDays === 2) return 'Yesterday'
    if (diffDays <= 7) return `${diffDays - 1} days ago`
    return date.toLocaleDateString()
  }

  const filteredWorkspaces = recentWorkspaces.filter(
    workspace =>
      workspace.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      workspace.path.toLowerCase().includes(searchTerm.toLowerCase())
  )

  // Sort: favorites first, then by last accessed
  const sortedWorkspaces = [...filteredWorkspaces].sort((a, b) => {
    if (a.is_favorite && !b.is_favorite) return -1
    if (!a.is_favorite && b.is_favorite) return 1
    return new Date(b.last_accessed).getTime() - new Date(a.last_accessed).getTime()
  })

  if (loading) {
    return (
      <div className="p-6 space-y-4">
        <div className="animate-pulse space-y-4">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-20 bg-gray-200 rounded-lg"></div>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-red-600">Error loading workspaces: {error}</p>
          <button
            onClick={loadRecentWorkspaces}
            className="mt-2 text-red-700 hover:text-red-900 underline"
          >
            Try again
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">Recent Workspaces</h2>
        <button
          onClick={loadRecentWorkspaces}
          className="text-blue-600 hover:text-blue-700 text-sm"
        >
          Refresh
        </button>
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
        <input
          type="text"
          placeholder="Search workspaces..."
          value={searchTerm}
          onChange={e => setSearchTerm(e.target.value)}
          className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {/* Workspace List */}
      <div className="space-y-3">
        {sortedWorkspaces.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            {searchTerm ? 'No workspaces match your search' : 'No recent workspaces found'}
          </div>
        ) : (
          sortedWorkspaces.map(workspace => {
            const stats = workspaceStats[workspace.path]
            const isCurrentWorkspace = currentWorkspace === workspace.path

            return (
              <div
                key={workspace.path}
                className={`border rounded-lg p-4 transition-colors ${
                  isCurrentWorkspace
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div
                    className="flex-1 cursor-pointer"
                    onClick={() => handleWorkspaceClick(workspace)}
                  >
                    <div className="flex items-center space-x-3">
                      <div className="flex items-center space-x-2">
                        {getTemplateIcon(workspace.template)}
                        <div className="flex items-center space-x-1">
                          {workspace.is_favorite && (
                            <Star className="h-4 w-4 text-yellow-500 fill-current" />
                          )}
                          <h3 className="font-medium text-gray-900">{workspace.name}</h3>
                        </div>
                      </div>
                    </div>

                    <p className="text-sm text-gray-500 mt-1">{workspace.path}</p>

                    {stats && (
                      <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                        <span>{stats.total_files} files</span>
                        <span>{formatFileSize(stats.total_size)}</span>
                        <span>{stats.import_count} imports</span>
                        <span>{stats.output_count} outputs</span>
                      </div>
                    )}

                    <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                      <div className="flex items-center space-x-1">
                        <Clock className="h-3 w-3" />
                        <span>{formatLastAccessed(workspace.last_accessed)}</span>
                      </div>
                      <span>Opened {workspace.access_count} times</span>
                    </div>
                  </div>

                  <div className="flex items-center space-x-1">
                    <button
                      onClick={e => {
                        e.stopPropagation()
                        toggleFavorite(workspace)
                      }}
                      className="p-1 text-gray-400 hover:text-yellow-500 transition-colors"
                      title={workspace.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
                    >
                      {workspace.is_favorite ? (
                        <Star className="h-4 w-4 fill-current text-yellow-500" />
                      ) : (
                        <StarOff className="h-4 w-4" />
                      )}
                    </button>

                    <button
                      onClick={e => {
                        e.stopPropagation()
                        removeFromRecent(workspace)
                      }}
                      className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                      title="Remove from recent"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </div>
            )
          })
        )}
      </div>

      {/* Quick Actions */}
      <div className="border-t pt-4">
        <button
          onClick={() => {
            // TODO: Implement new workspace creation
            console.log('Create new workspace clicked')
          }}
          className="flex items-center space-x-2 text-blue-600 hover:text-blue-700 transition-colors"
        >
          <FolderOpen className="h-4 w-4" />
          <span>Create New Workspace</span>
        </button>
      </div>
    </div>
  )
}

export default WorkspaceSwitcher
