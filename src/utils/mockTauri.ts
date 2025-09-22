// src/utils/mockTauri.ts
// Mock Tauri APIs for browser development

import type { MockTauriArgs, AISettings } from '../types/ai'

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown
    __TAURI__?: {
      invoke: (command: string, args?: unknown) => Promise<unknown>
    }
  }
}

interface MockTauriWindow extends Window {
  __TAURI__?: {
    invoke: (command: string, args?: unknown) => Promise<unknown>
  }
}

// Mock AI settings storage
let mockAISettings: AISettings = {
  provider: 'local',
  openrouterApiKey: '',
  anthropicApiKey: '',
  selectedModel: 'llama3.2-3b',
  preferLocalModels: true,
  recentModels: [],
}

// Mock AI system state
let mockAIInitialized = false

export function initializeMockTauri() {
  // Check if we're running in actual Tauri environment
  // In Tauri, the window object has __TAURI__ or __TAURI_INTERNALS__ properties
  if (typeof window !== 'undefined' && (window.__TAURI_INTERNALS__ || window.__TAURI__)) {
    console.log('🔧 Running in actual Tauri environment - skipping mock initialization')
    return
  }

  if (typeof window !== 'undefined' && !(window as MockTauriWindow).__TAURI__) {
    console.log('🔧 Initializing Mock Tauri for browser development')
    ;(window as MockTauriWindow).__TAURI__ = {
      invoke: async (command: string, args?: unknown) => {
        console.log(`🔧 Mock Tauri Command: ${command}`, args)

        switch (command) {
          case 'get_ai_settings':
            return { ...mockAISettings }

          case 'save_ai_settings':
            mockAISettings = { ...mockAISettings, ...(args as MockTauriArgs).settings }
            console.log('🔧 Mock: Saved AI settings', mockAISettings)
            return true

          case 'init_ai_system':
            mockAIInitialized = true
            console.log('🔧 Mock: AI system initialized')
            return true

          case 'restart_ai_system':
            if ((args as MockTauriArgs).config) {
              mockAISettings = { ...mockAISettings, ...(args as MockTauriArgs).config }
            }
            mockAIInitialized = true
            console.log('🔧 Mock: AI system restarted with config', (args as MockTauriArgs).config)
            return true

          case 'get_ai_status': {
            const isOpenRouter =
              mockAISettings.provider === 'openrouter' && mockAISettings.openrouterApiKey
            const isAnthropic =
              mockAISettings.provider === 'anthropic' && mockAISettings.anthropicApiKey
            const isLocal = mockAISettings.provider === 'local'

            const available = mockAIInitialized && (isOpenRouter || isAnthropic || isLocal)

            return {
              available,
              models: available ? [mockAISettings.selectedModel] : [],
              current_model: mockAISettings.selectedModel,
              error: !available ? 'AI system not properly configured' : null,
            }
          }

          case 'chat_with_ai':
            if (mockAISettings.provider === 'openrouter' && mockAISettings.openrouterApiKey) {
              return {
                success: true,
                response: {
                  content: `Hello! This is a mock response from ${mockAISettings.selectedModel}. In a real app, this would come from OpenRouter API.`,
                  intent: 'General',
                  confidence: 0.95,
                },
                error: null,
              }
            } else {
              return {
                success: false,
                response: null,
                error: 'OpenRouter not configured or API key missing',
              }
            }

          case 'check_ollama_connection':
            return mockAISettings.provider === 'local'

          case 'get_available_models':
            if (mockAISettings.provider === 'local') {
              return ['llama3.2-3b', 'llama3.1-8b', 'mistral-7b']
            } else if (mockAISettings.provider === 'openrouter') {
              return [
                'deepseek/deepseek-chat-v3-0324:free',
                'openai/gpt-4o-mini',
                'anthropic/claude-3-haiku',
                'meta-llama/llama-3.1-8b-instruct:free',
              ]
            } else {
              return ['claude-3-haiku-20240307', 'claude-3-sonnet-20240229']
            }

          default:
            console.warn(`🔧 Mock Tauri: Unknown command ${command}`)
            return null
        }
      },
    }
  }
}

// Auto-initialize when imported, but wait for Tauri to load first
if (typeof window !== 'undefined') {
  // Wait a bit for Tauri APIs to be injected in development mode
  setTimeout(() => {
    initializeMockTauri()
  }, 100)
}
