import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { AISettings } from '../types/ai'
import {
  Save,
  RefreshCw,
  Shield,
  Database,
  Cpu,
  Globe,
  Bell,
  Palette,
  Monitor,
  Moon,
  Sun,
  CheckCircle,
  XCircle,
  AlertCircle,
  Loader,
  Zap,
} from 'lucide-react'

const Settings: React.FC = () => {
  const [activeTab, setActiveTab] = useState('general')
  // const [isDarkMode, setIsDarkMode] = useState(false)
  const [notifications, setNotifications] = useState(true)
  const [autoSave, setAutoSave] = useState(true)

  // AI Configuration State
  const [aiProvider, setAiProvider] = useState<'local' | 'openrouter' | 'anthropic'>('local')
  const [openrouterApiKey, setOpenrouterApiKey] = useState('')
  const [anthropicApiKey, setAnthropicApiKey] = useState('')
  const [selectedModel, setSelectedModel] = useState('')
  const [customModel, setCustomModel] = useState('')
  const [preferLocalModels, setPreferLocalModels] = useState(true)
  const [ollamaStatus, setOllamaStatus] = useState<'checking' | 'connected' | 'disconnected'>(
    'checking'
  )
  const [availableModels, setAvailableModels] = useState<string[]>([])
  const [recentModels, setRecentModels] = useState<string[]>([])
  const [isLoadingModels, setIsLoadingModels] = useState(false)
  const [isCustomModelMode, setIsCustomModelMode] = useState(false)
  const [isLoadingSettings, setIsLoadingSettings] = useState(true)
  const [isSavingSettings, setIsSavingSettings] = useState(false)

  // Embedding Configuration State
  const [embeddingProvider, setEmbeddingProvider] = useState<'openai' | 'openrouter'>('openai')
  const [openaiApiKey, setOpenaiApiKey] = useState('')
  const [embeddingModel, setEmbeddingModel] = useState('text-embedding-3-small')
  const [customDimensions, setCustomDimensions] = useState<number | null>(null)
  const [batchSize, setBatchSize] = useState(25)
  const [timeoutSeconds, setTimeoutSeconds] = useState(90)
  const [isTestingConnection, setIsTestingConnection] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState<'unknown' | 'connected' | 'failed'>(
    'unknown'
  )
  const [connectionError, setConnectionError] = useState('')

  const tabs = [
    { id: 'general', label: 'General', icon: Monitor },
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'ai', label: 'AI Models', icon: Cpu },
    { id: 'embeddings', label: 'Embeddings', icon: Zap },
    { id: 'security', label: 'Security', icon: Shield },
    { id: 'storage', label: 'Storage', icon: Database },
    { id: 'network', label: 'Network', icon: Globe },
    { id: 'notifications', label: 'Notifications', icon: Bell },
  ]

  // Load settings on mount
  useEffect(() => {
    loadSettingsFromStorage()
    loadEmbeddingSettings()
  }, [])

  // Load embedding settings when tab changes to embeddings
  useEffect(() => {
    if (activeTab === 'embeddings') {
      loadEmbeddingSettings()
    }
  }, [activeTab])

  // Check Ollama connection on mount and when tab changes to AI
  useEffect(() => {
    if (activeTab === 'ai') {
      checkOllamaConnection()
      loadAvailableModels()
    }
  }, [activeTab]) // eslint-disable-line react-hooks/exhaustive-deps

  // Load models when provider or API keys change
  useEffect(() => {
    if (activeTab === 'ai') {
      loadAvailableModels()
    }
  }, [aiProvider, openrouterApiKey, anthropicApiKey]) // eslint-disable-line react-hooks/exhaustive-deps

  // Load settings from localStorage/backend
  const loadSettingsFromStorage = async () => {
    setIsLoadingSettings(true)
    try {
      // Try to load from Tauri backend first
      try {
        const settings = (await invoke('get_ai_settings')) as AISettings
        setAiProvider((settings.provider as 'local' | 'openrouter' | 'anthropic') || 'local')
        setOpenrouterApiKey(settings.openrouterApiKey || '')
        setAnthropicApiKey(settings.anthropicApiKey || '')
        setSelectedModel(settings.selectedModel || '')
        setPreferLocalModels(settings.preferLocalModels ?? true)
        setRecentModels(settings.recentModels || [])
      } catch {
        // Fallback to localStorage if Tauri command fails
        const stored = localStorage.getItem('ai_settings')
        if (stored) {
          const settings = JSON.parse(stored)
          setAiProvider((settings.provider as 'local' | 'openrouter' | 'anthropic') || 'local')
          setOpenrouterApiKey(settings.openrouterApiKey || '')
          setAnthropicApiKey(settings.anthropicApiKey || '')
          setSelectedModel(settings.selectedModel || '')
          setPreferLocalModels(settings.preferLocalModels ?? true)
          setRecentModels(settings.recentModels || [])
        }
      }
    } catch (error) {
      console.error('Failed to load AI settings:', error)
    } finally {
      setIsLoadingSettings(false)
    }
  }

  // Save settings to localStorage/backend
  const saveSettingsToStorage = async () => {
    const settings = {
      provider: aiProvider,
      openrouterApiKey,
      anthropicApiKey,
      selectedModel: isCustomModelMode ? customModel : selectedModel,
      preferLocalModels,
      recentModels,
    }

    try {
      // Try to save to Tauri backend first
      try {
        await invoke('save_ai_settings', { settings })
      } catch {
        // Fallback to localStorage if Tauri command fails
        localStorage.setItem('ai_settings', JSON.stringify(settings))
      }

      // Update recent models
      const modelToAdd = isCustomModelMode ? customModel : selectedModel
      if (modelToAdd && !recentModels.includes(modelToAdd)) {
        const newRecentModels = [modelToAdd, ...recentModels.slice(0, 2)]
        setRecentModels(newRecentModels)

        // Save updated recent models
        const updatedSettings = { ...settings, recentModels: newRecentModels }
        try {
          await invoke('save_ai_settings', { settings: updatedSettings })
        } catch {
          localStorage.setItem('ai_settings', JSON.stringify(updatedSettings))
        }
      }

      return true
    } catch (error) {
      console.error('Failed to save AI settings:', error)
      throw error
    }
  }

  const checkOllamaConnection = async () => {
    setOllamaStatus('checking')
    try {
      const isConnected = await invoke('check_ollama_connection')
      setOllamaStatus(isConnected ? 'connected' : 'disconnected')
    } catch {
      setOllamaStatus('disconnected')
    }
  }

  const loadAvailableModels = async () => {
    setIsLoadingModels(true)
    try {
      let models: string[] = []

      if (aiProvider === 'local') {
        try {
          models = (await invoke('get_available_models')) as string[]
        } catch {
          // Fallback models if Ollama is not available
          models = []
        }
      } else if (aiProvider === 'openrouter' && openrouterApiKey) {
        // Popular OpenRouter models
        models = [
          'deepseek/deepseek-chat-v3-0324:free',
          'openai/gpt-4o-mini',
          'anthropic/claude-3-haiku',
          'meta-llama/llama-3.1-8b-instruct:free',
          'microsoft/wizardlm-2-8x22b',
          'google/gemini-flash-1.5',
          'anthropic/claude-3-sonnet',
          'openai/gpt-4o',
        ]
      } else if (aiProvider === 'anthropic' && anthropicApiKey) {
        models = ['claude-3-haiku-20240307', 'claude-3-sonnet-20240229', 'claude-3-opus-20240229']
      }

      setAvailableModels(models)

      // Set default model if none selected
      if (models.length > 0 && !selectedModel && !isCustomModelMode) {
        setSelectedModel(models[0] || '')
      }
    } catch (error) {
      console.error('Failed to load models:', error)
      setAvailableModels([])
    } finally {
      setIsLoadingModels(false)
    }
  }

  const addCustomModel = () => {
    if (customModel && !recentModels.includes(customModel)) {
      const newRecentModels = [customModel, ...recentModels.slice(0, 2)]
      setRecentModels(newRecentModels)
    }
    setSelectedModel(customModel)
    setIsCustomModelMode(false)
    setCustomModel('')
  }

  const getStatusIcon = () => {
    switch (ollamaStatus) {
      case 'checking':
        return <Loader className="h-4 w-4 animate-spin text-blue-500" />
      case 'connected':
        return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'disconnected':
        return <XCircle className="h-4 w-4 text-red-500" />
    }
  }

  const getProviderDescription = () => {
    switch (aiProvider) {
      case 'local':
        return 'Use locally installed AI models via Ollama for maximum privacy'
      case 'openrouter':
        return 'Access multiple AI models through OpenRouter API with competitive pricing'
      case 'anthropic':
        return 'Use Claude models directly from Anthropic for high-quality responses'
    }
  }

  const handleSaveSettings = async () => {
    setIsSavingSettings(true)
    try {
      await saveSettingsToStorage()

      // Save embedding settings
      try {
        await saveEmbeddingSettings()
        console.log('Embedding settings saved successfully')
      } catch (error) {
        console.warn('Failed to save embedding settings:', error)
      }

      // Initialize AI system with new settings
      try {
        const config = {
          provider: aiProvider,
          openrouterApiKey: aiProvider === 'openrouter' ? openrouterApiKey : '',
          anthropicApiKey: aiProvider === 'anthropic' ? anthropicApiKey : '',
          selectedModel: isCustomModelMode ? customModel : selectedModel,
          preferLocalModels,
        }
        console.log('Restarting AI system with config:', config)

        await invoke('restart_ai_system', { config })
        console.log('AI system restarted successfully')
      } catch (error) {
        console.warn('Failed to restart AI system:', error)
      }

      // Show success feedback
      alert('Settings saved successfully!')
    } catch (error) {
      console.error('Failed to save settings:', error)
      alert('Failed to save settings. Please try again.')
    } finally {
      setIsSavingSettings(false)
    }
  }

  const handleResetSettings = () => {
    // Implementation for resetting settings
    console.log('Settings reset to defaults')
  }

  // Embedding settings functions
  const loadEmbeddingSettings = async () => {
    try {
      const settings = await invoke('load_embedding_settings')
      if (settings && typeof settings === 'object') {
        const embeddingSettings = settings as {
          provider: string
          api_key: string
          model: string
          custom_dimensions?: number
          batch_size: number
          timeout_seconds: number
        }

        setEmbeddingProvider(embeddingSettings.provider === 'openrouter' ? 'openrouter' : 'openai')
        setOpenaiApiKey(embeddingSettings.api_key || '')
        setEmbeddingModel(embeddingSettings.model || 'text-embedding-3-small')
        setCustomDimensions(embeddingSettings.custom_dimensions || null)
        setBatchSize(embeddingSettings.batch_size || 25)
        setTimeoutSeconds(embeddingSettings.timeout_seconds || 90)
      }
    } catch (error) {
      console.error('Failed to load embedding settings:', error)
    }
  }

  const saveEmbeddingSettings = async () => {
    try {
      const settings = {
        provider: embeddingProvider,
        api_key: openaiApiKey,
        model: embeddingModel,
        custom_dimensions: customDimensions,
        batch_size: batchSize,
        timeout_seconds: timeoutSeconds,
      }

      await invoke('save_embedding_settings', { settings })

      // Apply settings to vector system if API key is provided
      if (openaiApiKey) {
        await invoke('apply_embedding_settings', { settings })
      }

      return true
    } catch (error) {
      console.error('Failed to save embedding settings:', error)
      throw error
    }
  }

  const testEmbeddingConnection = async () => {
    setIsTestingConnection(true)
    setConnectionStatus('unknown')
    setConnectionError('')

    try {
      const result = await invoke('test_embedding_settings_connection', {
        provider: embeddingProvider,
        apiKey: openaiApiKey,
        model: embeddingModel,
      })

      if (result) {
        setConnectionStatus('connected')
      } else {
        setConnectionStatus('failed')
        setConnectionError('Connection test failed')
      }
    } catch (error: unknown) {
      setConnectionStatus('failed')
      setConnectionError(error instanceof Error ? error.message : 'Unknown error')
    } finally {
      setIsTestingConnection(false)
    }
  }

  const renderGeneralSettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          General Preferences
        </h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Auto-save documents
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Automatically save changes as you work
              </p>
            </div>
            <button
              onClick={() => setAutoSave(!autoSave)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                autoSave ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-700'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  autoSave ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Enable notifications
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Receive notifications for important events
              </p>
            </div>
            <button
              onClick={() => setNotifications(!notifications)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                notifications ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-700'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  notifications ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Default workspace path
            </label>
            <input
              type="text"
              defaultValue="/Users/username/Documents/Proxemic"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </div>
    </div>
  )

  const renderAppearanceSettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          Appearance Settings
        </h3>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Theme
            </label>
            <div className="grid grid-cols-3 gap-3">
              <button className="flex items-center justify-center p-3 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700">
                <Sun className="h-5 w-5 mr-2" />
                Light
              </button>
              <button className="flex items-center justify-center p-3 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700">
                <Moon className="h-5 w-5 mr-2" />
                Dark
              </button>
              <button className="flex items-center justify-center p-3 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700">
                <Monitor className="h-5 w-5 mr-2" />
                System
              </button>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Font size
            </label>
            <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option>Small</option>
              <option>Medium</option>
              <option>Large</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  )

  const renderAISettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          AI Provider Configuration
        </h3>

        {/* Provider Selection */}
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              AI Provider
            </label>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              {/* Local Ollama Option */}
              <div
                className={`relative p-4 border-2 rounded-lg cursor-pointer transition-all ${
                  aiProvider === 'local'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                }`}
                onClick={() => setAiProvider('local')}
              >
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-gray-900 dark:text-white">Local (Ollama)</h4>
                  {aiProvider === 'local' && getStatusIcon()}
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Private, local processing
                </p>
                {aiProvider === 'local' && (
                  <div className="mt-2 text-xs">
                    {ollamaStatus === 'connected' && (
                      <span className="text-green-600 dark:text-green-400">✓ Connected</span>
                    )}
                    {ollamaStatus === 'disconnected' && (
                      <span className="text-red-600 dark:text-red-400">✗ Not connected</span>
                    )}
                    {ollamaStatus === 'checking' && (
                      <span className="text-blue-600 dark:text-blue-400">⟳ Checking...</span>
                    )}
                  </div>
                )}
              </div>

              {/* OpenRouter Option */}
              <div
                className={`relative p-4 border-2 rounded-lg cursor-pointer transition-all ${
                  aiProvider === 'openrouter'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                }`}
                onClick={() => setAiProvider('openrouter')}
              >
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-gray-900 dark:text-white">OpenRouter</h4>
                  {aiProvider === 'openrouter' && openrouterApiKey && (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  )}
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Multiple models, competitive pricing
                </p>
              </div>

              {/* Anthropic Option */}
              <div
                className={`relative p-4 border-2 rounded-lg cursor-pointer transition-all ${
                  aiProvider === 'anthropic'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                }`}
                onClick={() => setAiProvider('anthropic')}
              >
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-gray-900 dark:text-white">Anthropic</h4>
                  {aiProvider === 'anthropic' && anthropicApiKey && (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  )}
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Claude models direct access
                </p>
              </div>
            </div>
            <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
              {getProviderDescription()}
            </p>
          </div>

          {/* API Key Configuration for Cloud Providers */}
          {aiProvider === 'openrouter' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                OpenRouter API Key
              </label>
              <input
                type="password"
                value={openrouterApiKey}
                onChange={e => setOpenrouterApiKey(e.target.value)}
                placeholder="Enter your OpenRouter API key"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              />
              <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Get your API key from{' '}
                <a
                  href="https://openrouter.ai/keys"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-500 hover:underline"
                >
                  openrouter.ai/keys
                </a>
              </p>
            </div>
          )}

          {aiProvider === 'anthropic' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Anthropic API Key
              </label>
              <input
                type="password"
                value={anthropicApiKey}
                onChange={e => setAnthropicApiKey(e.target.value)}
                placeholder="Enter your Anthropic API key"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              />
              <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Get your API key from{' '}
                <a
                  href="https://console.anthropic.com/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-500 hover:underline"
                >
                  console.anthropic.com
                </a>
              </p>
            </div>
          )}

          {/* Model Selection */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Select Model
              </label>
              {(aiProvider === 'openrouter' || aiProvider === 'anthropic') && (
                <button
                  onClick={() => setIsCustomModelMode(!isCustomModelMode)}
                  className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                >
                  {isCustomModelMode ? 'Use dropdown' : 'Enter custom model'}
                </button>
              )}
            </div>

            {isCustomModelMode ? (
              <div className="space-y-2">
                <div className="flex space-x-2">
                  <input
                    type="text"
                    value={customModel}
                    onChange={e => setCustomModel(e.target.value)}
                    placeholder={
                      aiProvider === 'openrouter'
                        ? 'e.g., deepseek/deepseek-chat-v3-0324:free'
                        : 'e.g., claude-3-haiku-20240307'
                    }
                    className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                  />
                  <button
                    onClick={addCustomModel}
                    disabled={!customModel.trim()}
                    className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Add
                  </button>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {aiProvider === 'openrouter'
                    ? 'Enter the exact model name from OpenRouter (e.g., "deepseek/deepseek-chat-v3-0324:free")'
                    : 'Enter the exact model name from Anthropic'}
                </p>
              </div>
            ) : (
              <>
                {isLoadingModels ? (
                  <div className="flex items-center space-x-2 py-2">
                    <Loader className="h-4 w-4 animate-spin text-blue-500" />
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                      Loading available models...
                    </span>
                  </div>
                ) : (
                  <select
                    value={selectedModel}
                    onChange={e => setSelectedModel(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                    disabled={availableModels.length === 0}
                  >
                    {/* Recent Models Section */}
                    {recentModels.length > 0 && (
                      <optgroup label="📝 Recently Used">
                        {recentModels.map(model => (
                          <option key={`recent-${model}`} value={model}>
                            {model}
                          </option>
                        ))}
                      </optgroup>
                    )}

                    {/* Available Models Section */}
                    {availableModels.length > 0 && (
                      <optgroup
                        label={
                          aiProvider === 'local' ? '🖥️ Local Models' : `🌐 ${aiProvider} Models`
                        }
                      >
                        {availableModels.map(model => (
                          <option key={model} value={model}>
                            {model}
                          </option>
                        ))}
                      </optgroup>
                    )}

                    {/* No models fallback */}
                    {availableModels.length === 0 && recentModels.length === 0 && (
                      <option value="" disabled>
                        {aiProvider === 'local' && ollamaStatus === 'disconnected'
                          ? 'Ollama not connected'
                          : aiProvider !== 'local' &&
                              (aiProvider === 'openrouter' ? !openrouterApiKey : !anthropicApiKey)
                            ? 'Please enter API key'
                            : 'No models available'}
                      </option>
                    )}
                  </select>
                )}

                {availableModels.length === 0 && !isLoadingModels && (
                  <div className="flex items-center space-x-2 py-2">
                    <AlertCircle className="h-4 w-4 text-yellow-500" />
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                      {aiProvider === 'local' && ollamaStatus === 'disconnected'
                        ? 'Ollama not connected. Please ensure Ollama is running.'
                        : aiProvider !== 'local' &&
                            (aiProvider === 'openrouter' ? !openrouterApiKey : !anthropicApiKey)
                          ? 'Please enter your API key to load available models.'
                          : 'No models available.'}
                    </span>
                  </div>
                )}
              </>
            )}
          </div>

          {/* Privacy Preference */}
          <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-600">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Prefer local models when available
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Automatically use local Ollama models over cloud APIs when possible
              </p>
            </div>
            <button
              onClick={() => setPreferLocalModels(!preferLocalModels)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                preferLocalModels ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-700'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  preferLocalModels ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
        </div>
      </div>
    </div>
  )

  const renderEmbeddingSettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          Embedding Configuration
        </h3>
        <p className="text-sm text-gray-600 dark:text-gray-400 mb-6">
          Configure API-based embeddings for intelligent document search. Local embeddings are
          disabled for performance and stability.
        </p>

        {/* Provider Selection */}
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              Embedding Provider
            </label>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {/* OpenAI Option */}
              <div
                className={`relative p-4 border-2 rounded-lg cursor-pointer transition-all ${
                  embeddingProvider === 'openai'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                }`}
                onClick={() => setEmbeddingProvider('openai')}
              >
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-gray-900 dark:text-white">OpenAI</h4>
                  {embeddingProvider === 'openai' && connectionStatus === 'connected' && (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  )}
                  {embeddingProvider === 'openai' && connectionStatus === 'failed' && (
                    <XCircle className="h-4 w-4 text-red-500" />
                  )}
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Most reliable, newest models available
                </p>
                <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Cost: ~$0.00002 per 1K tokens (5x cheaper than legacy models)
                </div>
              </div>

              {/* OpenRouter Option */}
              <div
                className={`relative p-4 border-2 rounded-lg cursor-pointer transition-all ${
                  embeddingProvider === 'openrouter'
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                }`}
                onClick={() => setEmbeddingProvider('openrouter')}
              >
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-gray-900 dark:text-white">OpenRouter</h4>
                  {embeddingProvider === 'openrouter' && connectionStatus === 'connected' && (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  )}
                  {embeddingProvider === 'openrouter' && connectionStatus === 'failed' && (
                    <XCircle className="h-4 w-4 text-red-500" />
                  )}
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Access OpenAI models through OpenRouter
                </p>
                <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Cost: Competitive pricing, same models
                </div>
              </div>
            </div>
          </div>

          {/* API Key Configuration */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {embeddingProvider === 'openai' ? 'OpenAI API Key' : 'OpenRouter API Key'}
            </label>
            <div className="flex space-x-2">
              <input
                type="password"
                value={openaiApiKey}
                onChange={e => setOpenaiApiKey(e.target.value)}
                placeholder={`Enter your ${embeddingProvider === 'openai' ? 'OpenAI' : 'OpenRouter'} API key`}
                className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              />
              <button
                onClick={testEmbeddingConnection}
                disabled={!openaiApiKey || isTestingConnection}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                {isTestingConnection ? (
                  <>
                    <Loader className="h-4 w-4 animate-spin" />
                    Testing...
                  </>
                ) : (
                  'Test Connection'
                )}
              </button>
            </div>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Get your API key from{' '}
              <a
                href={
                  embeddingProvider === 'openai'
                    ? 'https://platform.openai.com/api-keys'
                    : 'https://openrouter.ai/keys'
                }
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline"
              >
                {embeddingProvider === 'openai' ? 'platform.openai.com' : 'openrouter.ai/keys'}
              </a>
            </p>
            {connectionStatus === 'connected' && (
              <div className="mt-2 flex items-center text-green-600 dark:text-green-400 text-sm">
                <CheckCircle className="h-4 w-4 mr-1" />
                API connection successful
              </div>
            )}
            {connectionStatus === 'failed' && connectionError && (
              <div className="mt-2 flex items-start text-red-600 dark:text-red-400 text-sm">
                <XCircle className="h-4 w-4 mr-1 mt-0.5" />
                <div>
                  <div>Connection failed</div>
                  <div className="text-xs mt-1 opacity-80">{connectionError}</div>
                </div>
              </div>
            )}
          </div>

          {/* Model Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Embedding Model
            </label>
            <select
              value={embeddingModel}
              onChange={e => setEmbeddingModel(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <optgroup label="Recommended (Cost-Effective)">
                <option value="text-embedding-3-small">
                  text-embedding-3-small (1536 dim) - 5x cheaper
                </option>
              </optgroup>
              <optgroup label="High Performance">
                <option value="text-embedding-3-large">
                  text-embedding-3-large (3072 dim) - Best quality
                </option>
              </optgroup>
              <optgroup label="Legacy">
                <option value="text-embedding-ada-002">
                  text-embedding-ada-002 (1536 dim) - Legacy
                </option>
              </optgroup>
            </select>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              text-embedding-3-small is recommended for most use cases - 80% cost savings with same
              quality
            </p>
          </div>

          {/* Advanced Options */}
          <div className="border-t border-gray-200 dark:border-gray-600 pt-4">
            <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              Advanced Options
            </h4>

            {/* Custom Dimensions */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-700 dark:text-gray-300 mb-1">
                  Custom Dimensions (Optional)
                </label>
                <input
                  type="number"
                  value={customDimensions || ''}
                  onChange={e =>
                    setCustomDimensions(e.target.value ? parseInt(e.target.value) : null)
                  }
                  placeholder="Default: 1536 or 3072"
                  min="1"
                  max="3072"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Reduce dimensions for even lower costs
                </p>
              </div>

              <div>
                <label className="block text-sm text-gray-700 dark:text-gray-300 mb-1">
                  Batch Size
                </label>
                <input
                  type="number"
                  value={batchSize}
                  onChange={e => setBatchSize(parseInt(e.target.value) || 25)}
                  min="1"
                  max="100"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Lower for slower systems
                </p>
              </div>
            </div>

            <div className="mt-4">
              <label className="block text-sm text-gray-700 dark:text-gray-300 mb-1">
                Timeout (seconds)
              </label>
              <input
                type="number"
                value={timeoutSeconds}
                onChange={e => setTimeoutSeconds(parseInt(e.target.value) || 90)}
                min="10"
                max="300"
                className="w-full md:w-48 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              />
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Increase for slow internet connections
              </p>
            </div>
          </div>

          {/* Safety Notice */}
          <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md p-4">
            <div className="flex items-start">
              <AlertCircle className="h-5 w-5 text-yellow-600 dark:text-yellow-400 mt-0.5 mr-2" />
              <div>
                <h5 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                  Performance & Safety Notice
                </h5>
                <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                  Local embeddings have been disabled to prevent CPU overload and system crashes.
                  API-based embeddings are much faster, more reliable, and cost-effective.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )

  const renderSecuritySettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          Security & Privacy
        </h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Enable audit logging
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Track all file operations and system access
              </p>
            </div>
            <button className="relative inline-flex h-6 w-11 items-center rounded-full bg-blue-600">
              <span className="inline-block h-4 w-4 transform rounded-full bg-white translate-x-6" />
            </button>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Workspace boundary enforcement
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Restrict file operations to designated workspace
              </p>
            </div>
            <button className="relative inline-flex h-6 w-11 items-center rounded-full bg-blue-600">
              <span className="inline-block h-4 w-4 transform rounded-full bg-white translate-x-6" />
            </button>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Security level
            </label>
            <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option>Development</option>
              <option>Production</option>
              <option>High Security</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Maximum file size (MB)
            </label>
            <input
              type="number"
              defaultValue="100"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </div>
    </div>
  )

  const renderCurrentTab = () => {
    switch (activeTab) {
      case 'general':
        return renderGeneralSettings()
      case 'appearance':
        return renderAppearanceSettings()
      case 'ai':
        return renderAISettings()
      case 'embeddings':
        return renderEmbeddingSettings()
      case 'security':
        return renderSecuritySettings()
      default:
        return (
          <div className="text-center py-12">
            <p className="text-gray-500 dark:text-gray-400">
              Settings for {tabs.find(tab => tab.id === activeTab)?.label} coming soon...
            </p>
          </div>
        )
    }
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Settings</h1>
        <p className="text-gray-600 dark:text-gray-400">
          Configure your Proxemic application preferences and security settings
        </p>
      </div>

      {/* Settings Interface */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div className="flex flex-col lg:flex-row">
          {/* Sidebar */}
          <div className="lg:w-64 bg-gray-50 dark:bg-gray-700 border-r border-gray-200 dark:border-gray-600">
            <nav className="p-4 space-y-1">
              {tabs.map(tab => {
                const Icon = tab.icon
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors ${
                      activeTab === tab.id
                        ? 'bg-blue-100 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300'
                        : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-600'
                    }`}
                  >
                    <Icon className="h-5 w-5 mr-3" />
                    {tab.label}
                  </button>
                )
              })}
            </nav>
          </div>

          {/* Content */}
          <div className="flex-1 p-6">
            {renderCurrentTab()}

            {/* Action Buttons */}
            <div className="flex justify-end space-x-3 mt-8 pt-6 border-t border-gray-200 dark:border-gray-600">
              <button
                onClick={handleResetSettings}
                className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <RefreshCw className="h-4 w-4 mr-2 inline" />
                Reset to Defaults
              </button>
              <button
                onClick={handleSaveSettings}
                disabled={isSavingSettings || isLoadingSettings}
                className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSavingSettings ? (
                  <>
                    <Loader className="h-4 w-4 mr-2 inline animate-spin" />
                    Saving...
                  </>
                ) : (
                  <>
                    <Save className="h-4 w-4 mr-2 inline" />
                    Save Settings
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Settings
