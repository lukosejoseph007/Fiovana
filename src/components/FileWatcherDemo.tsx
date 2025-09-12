// src/components/FileWatcherDemo.tsx
// Demo component for file watcher functionality

import React, { useState } from 'react'
import useFileWatcher from '../hooks/useFileWatcher'
import { FileEvent } from '../types/fileWatcher'
import './FileWatcherDemo.css'

const FileWatcherDemo: React.FC = () => {
  const {
    isWatching,
    watchedPaths,
    fileEvents,
    startWatching,
    stopWatching,
    pauseWatching,
    resumeWatching,
    addWatchPath,
    removeWatchPath,
    clearEvents,
  } = useFileWatcher()

  const [workspacePath, setWorkspacePath] = useState('')
  const [newWatchPath, setNewWatchPath] = useState('')

  const handleStartWatching = async () => {
    if (!workspacePath.trim()) {
      alert('Please enter a workspace path')
      return
    }
    try {
      await startWatching(workspacePath)
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
    switch (event.event_type) {
      case 'created':
        return `📄 Created: ${event.path}`
      case 'modified':
        return `✏️ Modified: ${event.path}`
      case 'deleted':
        return `🗑️ Deleted: ${event.path}`
      case 'renamed':
        return `🔄 Renamed: ${event.from} → ${event.to}`
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
            value={workspacePath}
            onChange={e => setWorkspacePath(e.target.value)}
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
