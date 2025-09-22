// src/hooks/useAISettings.ts
// This file shows how to integrate the UI with actual Tauri commands

import { useState, useEffect } from 'react'
// Using the direct window.__TAURI__ approach like the rest of the app

export interface AISettings {
  provider: 'local' | 'openrouter' | 'anthropic'
  openrouterApiKey: string
  anthropicApiKey: string
  selectedModel: string
  preferLocalModels: boolean
}

export function useAISettings() {
  const [settings, setSettings] = useState<AISettings>({
    provider: 'local',
    openrouterApiKey: '',
    anthropicApiKey: '',
    selectedModel: '',
    preferLocalModels: true,
  })

  // Load settings from backend
  const loadSettings = async () => {
    try {
      // @ts-expect-error - Tauri command
      const savedSettings = await window.__TAURI__.invoke('get_ai_settings')
      setSettings(savedSettings)
    } catch (error) {
      console.error('Failed to load AI settings:', error)
    }
  }

  // Save settings to backend
  const saveSettings = async (newSettings: AISettings) => {
    try {
      // @ts-expect-error - Tauri command
      await window.__TAURI__.invoke('save_ai_settings', { settings: newSettings })
      setSettings(newSettings)
    } catch (error) {
      console.error('Failed to save AI settings:', error)
      throw error
    }
  }

  // Check Ollama connection
  const checkOllamaConnection = async (): Promise<boolean> => {
    try {
      // @ts-expect-error - Tauri command
      return await window.__TAURI__.invoke('check_ollama_connection')
    } catch (error) {
      console.error('Failed to check Ollama connection:', error)
      return false
    }
  }

  // Get available models for current provider
  const getAvailableModels = async (provider: string, apiKey?: string): Promise<string[]> => {
    try {
      if (provider === 'local') {
        // @ts-expect-error - Tauri command
        return await window.__TAURI__.invoke('get_available_models')
      } else if (provider === 'openrouter' && apiKey) {
        // @ts-expect-error - Tauri command
        return await window.__TAURI__.invoke('get_openrouter_models', { apiKey })
      } else if (provider === 'anthropic' && apiKey) {
        // @ts-expect-error - Tauri command
        return await window.__TAURI__.invoke('get_anthropic_models', { apiKey })
      }
      return []
    } catch (error) {
      console.error('Failed to get available models:', error)
      return []
    }
  }

  useEffect(() => {
    loadSettings()
  }, [])

  return {
    settings,
    loadSettings,
    saveSettings,
    checkOllamaConnection,
    getAvailableModels,
  }
}
