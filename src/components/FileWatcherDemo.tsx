// src/components/FileWatcherDemo.tsx
// Demo component for file watcher functionality

import React, { useState } from 'react'
import { useAppState } from '../context/AppStateContext'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { FileEvent } from '../types/fileWatcher'
import './FileWatcherDemo.css'

const FileWatcherDemo: React.FC = () => {
  const { state, dispatch } = useAppState()
  const { isWatching, watchedPaths, fileEvents, workspacePath } = state.fileWatcher
  const [newWatchPath, setNewWatchPath] = useState('')

  // Listen for file events from the backend
  React.useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupEventListener = async () => {
      try {
        unlisten = await listen<FileEvent>('file-event', event => {
          dispatch({ type: 'FILE_WATCHER_ADD_EVENT', payload: event.payload })
        })
      } catch (error) {
        console.error('Failed to set up file event listener:', error)
      }
    }

    setupEventListener()

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [dispatch])

  const startWatching = async (workspacePath: string) => {
    try {
      await invoke('start_file_watching', { workspacePath })
      dispatch({ type: 'FILE_WATCHER_SET_WATCHING', payload: true })
      dispatch({ type: 'FILE_WATCHER_SET_WORKSPACE_PATH', payload: workspacePath })

      // Get initial watched paths
      const paths = await invoke<string[]>('get_watched_paths')
      dispatch({ type: 'FILE_WATCHER_SET_PATHS', payload: paths })
    } catch (error) {
      console.error('Failed to start file watching:', error)
      throw error
    }
  }

  const stopWatching = async () => {
    try {
      await invoke('stop_file_watching')
      dispatch({ type: 'FILE_WATCHER_SET_WATCHING', payload: false })
      dispatch({ type: 'FILE_WATCHER_SET_PATHS', payload: [] })
      dispatch({ type: 'FILE_WATCHER_SET_WORKSPACE_PATH', payload: '' })
    } catch (error) {
      console.error('Failed to stop file watching:', error)
      throw error
    }
  }

  const pauseWatching = async () => {
    try {
      await invoke('pause_file_watching')
      dispatch({ type: 'FILE_WATCHER_SET_WATCHING', payload: false })
    } catch (error) {
      console.error('Failed to pause file watching:', error)
      throw error
    }
  }

  const resumeWatching = async () => {
    try {
      await invoke('resume_file_watching')
      dispatch({ type: 'FILE_WATCHER_SET_WATCHING', payload: true })
    } catch (error) {
      console.error('Failed to resume file watching:', error)
      throw error
    }
  }

  const addWatchPath = async (path: string) => {
    try {
      await invoke('add_watch_path', { path })

      // Refresh watched paths
      const paths = await invoke<string[]>('get_watched_paths')
      dispatch({ type: 'FILE_WATCHER_SET_PATHS', payload: paths })
    } catch (error) {
      console.error('Failed to add watch path:', error)
      throw error
    }
  }

  const removeWatchPath = async (path: string) => {
    try {
      await invoke('remove_watch_path', { path })

      // Refresh watched paths
      const paths = await invoke<string[]>('get_watched_paths')
      dispatch({ type: 'FILE_WATCHER_SET_PATHS', payload: paths })
    } catch (error) {
      console.error('Failed to remove watch path:', error)
      throw error
    }
  }

  const clearEvents = () => {
    dispatch({ type: 'FILE_WATCHER_CLEAR_EVENTS' })
  }

  const [localWorkspacePath, setLocalWorkspacePath] = useState(workspacePath)

  const handleStartWatching = async () => {
    if (!localWorkspacePath.trim()) {
      alert('Please enter a workspace path')
      return
    }
    try {
      await startWatching(localWorkspacePath)
    } catch (error) {
      console.error('Failed to start watching:', error)
    }
  }

  const handleAddWatchPath = async () => {
    if (!newWatchPath.trim()) {
      alert('Please enter a path to watch')
      return
    }
    try {
      await addWatchPath(newWatchPath)
      setNewWatchPath('')
    } catch (error) {
      console.error('Failed to add watch path:', error)
    }
  }

  const formatEvent = (event: FileEvent) => {
    switch (event.type) {
      case 'file-created':
        return `📄 Created: ${event.path}${event.is_directory ? ' (directory)' : ''}`
      case 'file-modified':
        return `✏️ Modified: ${event.path}${event.size ? ` (${event.size} bytes)` : ''}`
      case 'file-deleted':
        return `🗑️ Deleted: ${event.path}${event.is_directory ? ' (directory)' : ''}`
      case 'file-renamed':
        return `🔄 Renamed: ${event.old_path} → ${event.path}`
      case 'file-moved':
        return `🚚 Moved: ${event.old_path} → ${event.path}`
      default:
        return `Unknown event: ${JSON.stringify(event)}`
    }
  }

  return (
    <div className="file-watcher-demo">
      <h2>File Watcher Demo</h2>

      <div className="controls">
        <div className="input-group">
          <input
            type="text"
            placeholder="Workspace path to watch"
            value={localWorkspacePath}
            onChange={e => setLocalWorkspacePath(e.target.value)}
            disabled={isWatching}
          />
          <button onClick={handleStartWatching} disabled={isWatching}>
            Start Watching
          </button>
        </div>

        {isWatching && (
          <>
            <div className="button-group">
              <button onClick={pauseWatching}>Pause</button>
              <button onClick={resumeWatching}>Resume</button>
              <button onClick={stopWatching}>Stop</button>
            </div>

            <div className="input-group">
              <input
                type="text"
                placeholder="Add path to watch"
                value={newWatchPath}
                onChange={e => setNewWatchPath(e.target.value)}
              />
              <button onClick={handleAddWatchPath}>Add Path</button>
            </div>
          </>
        )}
      </div>

      {watchedPaths.length > 0 && (
        <div className="watched-paths">
          <h3>Watched Paths:</h3>
          <ul>
            {watchedPaths.map((path, index) => (
              <li key={index}>
                {path}
                <button onClick={() => removeWatchPath(path)}>Remove</button>
              </li>
            ))}
          </ul>
        </div>
      )}

      {fileEvents.length > 0 && (
        <div className="events">
          <div className="events-header">
            <h3>File Events ({fileEvents.length})</h3>
            <button onClick={clearEvents}>Clear</button>
          </div>
          <div className="events-list">
            {fileEvents.map((event, index) => (
              <div key={index} className="event-item">
                {formatEvent(event)}
              </div>
            ))}
          </div>
        </div>
      )}

      {!isWatching && fileEvents.length === 0 && (
        <div className="instructions">
          <p>Enter a workspace path and click "Start Watching" to begin monitoring file changes.</p>
          <p>
            Try creating, modifying, or deleting files in the watched directory to see events appear
            here.
          </p>
        </div>
      )}
    </div>
  )
}

export default FileWatcherDemo
