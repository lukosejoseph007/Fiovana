// AI-related type definitions

export interface AISettings {
  provider: string
  openrouterApiKey: string
  anthropicApiKey: string
  selectedModel: string
  preferLocalModels: boolean
  recentModels: string[]
}

export interface AIStatus {
  available: boolean
  models: string[]
  current_model: string
  error?: string
}

export interface ChatResponse {
  success: boolean
  response?: {
    content: string
    intent?: string
    confidence?: number
  }
  error?: string
}

// Tauri command argument types
export interface SaveSettingsArgs {
  settings: AISettings
}

export interface RestartAIArgs {
  config: Partial<AISettings>
}

// Mock window interface
export interface MockTauriArgs {
  settings?: AISettings
  config?: Partial<AISettings>
}
