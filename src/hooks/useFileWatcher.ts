// src/hooks/useFileWatcher.ts
// React hook for file watching functionality

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'
import { FileEvent } from '../types/fileWatcher'

export interface UseFileWatcherReturn {
  isWatching: boolean
  watchedPaths: string[]
  startWatching: (workspacePath: string) => Promise<void>
  stopWatching: () => Promise<void>
  pauseWatching: () => Promise<void>
  resumeWatching: () => Promise<void>
  addWatchPath: (path: string) => Promise<void>
  removeWatchPath: (path: string) => Promise<void>
  fileEvents: FileEvent[]
  clearEvents: () => void
}

export const useFileWatcher = (): UseFileWatcherReturn => {
  const [isWatching, setIsWatching] = useState(false)
  const [watchedPaths, setWatchedPaths] = useState<string[]>([])
  const [fileEvents, setFileEvents] = useState<FileEvent[]>([])

  // Listen for file events from the backend
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupEventListener = async () => {
      try {
        unlisten = await listen<FileEvent>('file-event', event => {
          setFileEvents(prev => [...prev, event.payload])
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
  }, [])

  const startWatching = useCallback(async (workspacePath: string) => {
    try {
      await invoke('start_file_watching', { workspacePath })
      setIsWatching(true)

      // Get initial watched paths
      const paths = await invoke<string[]>('get_watched_paths')
      setWatchedPaths(paths)
    } catch (error) {
      console.error('Failed to start file watching:', error)
      throw error
    }
  }, [])

  const stopWatching = useCallback(async () => {
    try {
      await invoke('stop_file_watching')
      setIsWatching(false)
      setWatchedPaths([])
    } catch (error) {
      console.error('Failed to stop file watching:', error)
      throw error
    }
  }, [])

  const pauseWatching = useCallback(async () => {
    try {
      await invoke('pause_file_watching')
      setIsWatching(false)
    } catch (error) {
      console.error('Failed to pause file watching:', error)
      throw error
    }
  }, [])

  const resumeWatching = useCallback(async () => {
    try {
      await invoke('resume_file_watching')
      setIsWatching(true)
    } catch (error) {
      console.error('Failed to resume file watching:', error)
      throw error
    }
  }, [])

  const addWatchPath = useCallback(async (path: string) => {
    try {
      await invoke('add_watch_path', { path })

      // Refresh watched paths
      const paths = await invoke<string[]>('get_watched_paths')
      setWatchedPaths(paths)
    } catch (error) {
      console.error('Failed to add watch path:', error)
      throw error
    }
  }, [])

  const removeWatchPath = useCallback(async (path: string) => {
    try {
      await invoke('remove_watch_path', { path })

      // Refresh watched paths
      const paths = await invoke<string[]>('get_watched_paths')
      setWatchedPaths(paths)
    } catch (error) {
      console.error('Failed to remove watch path:', error)
      throw error
    }
  }, [])

  const clearEvents = useCallback(() => {
    setFileEvents([])
  }, [])

  return {
    isWatching,
    watchedPaths,
    startWatching,
    stopWatching,
    pauseWatching,
    resumeWatching,
    addWatchPath,
    removeWatchPath,
    fileEvents,
    clearEvents,
  }
}

export default useFileWatcher
