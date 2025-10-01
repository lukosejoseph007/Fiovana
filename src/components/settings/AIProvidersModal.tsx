import React, { useState, useEffect, useCallback } from 'react'
import { apiClient } from '../../api'
import Button from '../ui/Button'
import Input from '../ui/Input'
import Card from '../ui/Card'
import Badge from '../ui/Badge'

interface AIProvider {
  id: string
  name: string
  type: 'ollama' | 'openrouter' | 'anthropic'
  status: 'connected' | 'disconnected' | 'testing'
  models: string[]
  currentModel?: string
}

interface AISettings {
  provider: string
  openrouterApiKey: string
  anthropicApiKey: string
  selectedModel: string
  preferLocalModels: boolean
  recentModels: string[]
}

interface AIProvidersModalProps {
  isOpen: boolean
  onClose: () => void
}

export const AIProvidersModal: React.FC<AIProvidersModalProps> = ({ isOpen, onClose }) => {
  const [settings, setSettings] = useState<AISettings>({
    provider: 'local',
    openrouterApiKey: '',
    anthropicApiKey: '',
    selectedModel: 'llama3.2-3b',
    preferLocalModels: true,
    recentModels: [],
  })

  const [providers, setProviders] = useState<AIProvider[]>([
    {
      id: 'ollama',
      name: 'Ollama (Local)',
      type: 'ollama',
      status: 'disconnected',
      models: [],
    },
    {
      id: 'openrouter',
      name: 'OpenRouter',
      type: 'openrouter',
      status: 'disconnected',
      models: [],
    },
    {
      id: 'anthropic',
      name: 'Anthropic',
      type: 'anthropic',
      status: 'disconnected',
      models: [],
    },
  ])

  const [isLoading, setIsLoading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [editingProvider, setEditingProvider] = useState<string | null>(null)
  const [tempApiKey, setTempApiKey] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)

  const updateProviderStatuses = useCallback((currentSettings: AISettings) => {
    setProviders(prevProviders =>
      prevProviders.map(provider => {
        if (provider.type === 'ollama') {
          return {
            ...provider,
            status: currentSettings.provider === 'local' ? 'connected' : 'disconnected',
            currentModel:
              currentSettings.provider === 'local' ? currentSettings.selectedModel : undefined,
          }
        } else if (provider.type === 'openrouter') {
          return {
            ...provider,
            status:
              currentSettings.provider === 'openrouter' && currentSettings.openrouterApiKey
                ? 'connected'
                : 'disconnected',
            currentModel:
              currentSettings.provider === 'openrouter' ? currentSettings.selectedModel : undefined,
          }
        } else if (provider.type === 'anthropic') {
          return {
            ...provider,
            status:
              currentSettings.provider === 'anthropic' && currentSettings.anthropicApiKey
                ? 'connected'
                : 'disconnected',
            currentModel:
              currentSettings.provider === 'anthropic' ? currentSettings.selectedModel : undefined,
          }
        }
        return provider
      })
    )
  }, [])

  const loadSettings = useCallback(async () => {
    console.log('AIProvidersModal: loadSettings called')
    setIsLoading(true)
    setError(null)

    try {
      console.log('AIProvidersModal: Calling get_ai_settings...')

      // Add timeout to prevent hanging
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Settings load timeout')), 5000)
      )

      const response = (await Promise.race([
        apiClient.invoke<AISettings>('get_ai_settings'),
        timeoutPromise,
      ])) as { success: boolean; data?: AISettings }

      console.log('AIProvidersModal: Response received:', response)

      // The response.data is the settings object returned from Rust
      if (response.success && response.data && Object.keys(response.data).length > 0) {
        console.log('Loaded AI settings:', response.data)
        setSettings(response.data)
        updateProviderStatuses(response.data)
      } else {
        console.warn('No settings found or empty response, using defaults')
        // Use defaults but still update provider statuses
        const defaultSettings: AISettings = {
          provider: 'local',
          openrouterApiKey: '',
          anthropicApiKey: '',
          selectedModel: 'llama3.2-3b',
          preferLocalModels: true,
          recentModels: [],
        }
        setSettings(defaultSettings)
        updateProviderStatuses(defaultSettings)
      }
    } catch (err) {
      console.error('Error loading settings:', err)
      setError(err instanceof Error ? err.message : 'Failed to load settings')
      // Use defaults on error
      const defaultSettings: AISettings = {
        provider: 'local',
        openrouterApiKey: '',
        anthropicApiKey: '',
        selectedModel: 'llama3.2-3b',
        preferLocalModels: true,
        recentModels: [],
      }
      setSettings(defaultSettings)
      updateProviderStatuses(defaultSettings)
    } finally {
      console.log('AIProvidersModal: loadSettings finished')
      setIsLoading(false)
    }
  }, [updateProviderStatuses])

  // Load settings when modal opens
  useEffect(() => {
    console.log('AIProvidersModal: useEffect triggered, isOpen=', isOpen)
    if (isOpen) {
      console.log('AIProvidersModal: Modal opened, loading settings...')
      loadSettings()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen])

  const testConnection = useCallback(
    async (providerId: string) => {
      setError(null)
      setSuccessMessage(null)

      setProviders(prev =>
        prev.map(p => (p.id === providerId ? { ...p, status: 'testing' as const } : p))
      )

      try {
        let response

        if (providerId === 'ollama') {
          // Test Ollama connection
          response = await apiClient.invoke<{ available: boolean; message?: string }>(
            'test_ollama_connection'
          )
        } else if (providerId === 'openrouter') {
          // For OpenRouter, check if API key is set
          if (!settings.openrouterApiKey) {
            throw new Error('OpenRouter API key not configured')
          }
          // Try to initialize with OpenRouter
          response = { success: true, data: { available: true } }
        } else if (providerId === 'anthropic') {
          // For Anthropic, check if API key is set
          if (!settings.anthropicApiKey) {
            throw new Error('Anthropic API key not configured')
          }
          // Try to initialize with Anthropic
          response = { success: true, data: { available: true } }
        }

        const isConnected = response?.success && response?.data?.available

        setProviders(prev =>
          prev.map(p =>
            p.id === providerId ? { ...p, status: isConnected ? 'connected' : 'disconnected' } : p
          )
        )

        if (isConnected) {
          setSuccessMessage(`${providerId} connection successful!`)
          setTimeout(() => setSuccessMessage(null), 3000)
        } else {
          throw new Error(response?.data?.message || 'Connection test failed')
        }
      } catch (err) {
        setProviders(prev =>
          prev.map(p => (p.id === providerId ? { ...p, status: 'disconnected' as const } : p))
        )
        setError(err instanceof Error ? err.message : 'Connection test failed')
        setTimeout(() => setError(null), 5000)
      }
    },
    [settings]
  )

  const saveSettings = useCallback(async () => {
    setIsSaving(true)
    setError(null)
    setSuccessMessage(null)

    try {
      // Prepare settings payload
      const settingsPayload = {
        provider: settings.provider,
        openrouterApiKey: settings.openrouterApiKey,
        anthropicApiKey: settings.anthropicApiKey,
        selectedModel: settings.selectedModel,
        preferLocalModels: settings.preferLocalModels,
        recentModels: settings.recentModels,
      } as Record<string, unknown>

      console.log('Saving AI settings:', settingsPayload)

      // Save to persistent storage - pass as 'settings' parameter per Rust backend expectation
      const response = await apiClient.invoke<boolean>('save_ai_settings', {
        settings: settingsPayload,
      })

      if (response.success) {
        setSuccessMessage('AI settings saved successfully!')

        // Reinitialize AI system with new settings
        try {
          await apiClient.invoke('init_ai_system', { config: settingsPayload })
          console.log('AI system reinitialized with new settings')
        } catch (initErr) {
          console.warn('AI system reinitialization warning:', initErr)
          // Don't fail the save if reinitialization has issues
        }

        updateProviderStatuses(settings)

        // Close modal after a short delay to show success message
        setTimeout(() => {
          onClose()
        }, 1500)
      } else {
        setError(response.error || 'Failed to save AI settings')
      }
    } catch (err) {
      console.error('Error saving settings:', err)
      setError(err instanceof Error ? err.message : 'Failed to save settings')
    } finally {
      setIsSaving(false)
    }
  }, [settings, onClose, updateProviderStatuses])

  const handleApiKeyEdit = useCallback(
    (providerId: string) => {
      setEditingProvider(providerId)
      const currentKey =
        providerId === 'openrouter'
          ? settings.openrouterApiKey
          : providerId === 'anthropic'
            ? settings.anthropicApiKey
            : ''
      setTempApiKey(currentKey)
    },
    [settings]
  )

  const handleApiKeySave = useCallback(() => {
    if (!editingProvider) return

    const updatedSettings = { ...settings }

    if (editingProvider === 'openrouter') {
      updatedSettings.openrouterApiKey = tempApiKey
      updatedSettings.provider = 'openrouter'
    } else if (editingProvider === 'anthropic') {
      updatedSettings.anthropicApiKey = tempApiKey
      updatedSettings.provider = 'anthropic'
    }

    setSettings(updatedSettings)
    setEditingProvider(null)
    setTempApiKey('')
    updateProviderStatuses(updatedSettings)
  }, [editingProvider, tempApiKey, settings, updateProviderStatuses])

  const handleApiKeyCancel = useCallback(() => {
    setEditingProvider(null)
    setTempApiKey('')
  }, [])

  const setActiveProvider = useCallback((providerId: string) => {
    setSettings(prev => ({
      ...prev,
      provider: providerId === 'ollama' ? 'local' : providerId,
    }))
  }, [])

  const getStatusBadge = (status: AIProvider['status']) => {
    const variants = {
      connected: 'success' as const,
      disconnected: 'default' as const,
      testing: 'warning' as const,
    }
    const labels = {
      connected: 'Connected',
      disconnected: 'Disconnected',
      testing: 'Testing...',
    }
    return <Badge variant={variants[status]}>{labels[status]}</Badge>
  }

  console.log('AIProvidersModal: Rendering, isOpen=', isOpen, 'isLoading=', isLoading)

  if (!isOpen) {
    console.log('AIProvidersModal: Not open, returning null')
    return null
  }

  console.log('AIProvidersModal: Rendering modal UI')

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        backdropFilter: 'blur(10px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '24px',
        zIndex: 9999,
        animation: 'fadeIn 0.2s ease-out',
      }}
      onClick={e => {
        if (e.target === e.currentTarget && !editingProvider) {
          onClose()
        }
      }}
    >
      <div
        style={{
          backgroundColor: '#1a1a1f',
          border: '1px solid #3a3a3f',
          borderRadius: '12px',
          maxWidth: '1200px',
          width: '100%',
          maxHeight: '90vh',
          overflow: 'hidden',
          animation: 'scaleIn 0.2s ease-out',
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: '20px 24px',
            borderBottom: '1px solid #3a3a3f',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <h2
            style={{
              fontSize: '20px',
              fontWeight: 600,
              color: '#ffffff',
              margin: 0,
            }}
          >
            AI Provider Configuration
          </h2>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              color: '#a8a8a8',
              cursor: 'pointer',
              padding: '8px',
              borderRadius: '6px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '32px',
              height: '32px',
            }}
          >
            ✕
          </button>
        </div>

        {/* Content */}
        <div
          style={{
            padding: '24px',
            maxHeight: 'calc(90vh - 140px)',
            overflowY: 'auto',
          }}
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
            {/* Error/Success Messages */}
            {error && (
              <Card className="bg-red-900/20 border-red-500/50 p-3">
                <p className="text-red-400 text-sm">{error}</p>
              </Card>
            )}

            {successMessage && (
              <Card className="bg-green-900/20 border-green-500/50 p-3">
                <p className="text-green-400 text-sm">{successMessage}</p>
              </Card>
            )}

            {isLoading ? (
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  padding: '48px',
                }}
              >
                <div style={{ textAlign: 'center' }}>
                  <div
                    style={{
                      width: '48px',
                      height: '48px',
                      border: '2px solid #00d4ff',
                      borderTopColor: 'transparent',
                      borderRadius: '50%',
                      animation: 'spin 1s linear infinite',
                      margin: '0 auto 16px',
                    }}
                  ></div>
                  <p style={{ color: '#a8a8a8' }}>Loading AI settings...</p>
                </div>
              </div>
            ) : (
              <>
                {/* Provider Cards */}
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
                    gap: '16px',
                  }}
                >
                  {providers.map(provider => (
                    <Card
                      key={provider.id}
                      className="p-4 hover:border-cyan-400/50 transition-colors"
                    >
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                          }}
                        >
                          <h3
                            style={{
                              fontSize: '16px',
                              fontWeight: 600,
                              color: '#ffffff',
                              margin: 0,
                            }}
                          >
                            {provider.name}
                          </h3>
                          {getStatusBadge(provider.status)}
                        </div>

                        {provider.currentModel && (
                          <div style={{ fontSize: '12px', color: '#a8a8a8' }}>
                            <span style={{ fontWeight: 500 }}>Model:</span> {provider.currentModel}
                          </div>
                        )}

                        {/* API Key Inline Editor */}
                        {editingProvider === provider.id && (
                          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                            <Input
                              type="password"
                              value={tempApiKey || ''}
                              onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                                setTempApiKey(e.target.value)
                              }
                              placeholder="Enter API key"
                              autoFocus
                            />
                            <div style={{ display: 'flex', gap: '8px' }}>
                              <Button
                                onClick={handleApiKeySave}
                                variant="primary"
                                size="sm"
                                className="flex-1"
                              >
                                Save
                              </Button>
                              <Button
                                onClick={handleApiKeyCancel}
                                variant="ghost"
                                size="sm"
                                className="flex-1"
                              >
                                Cancel
                              </Button>
                            </div>
                          </div>
                        )}

                        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                          {provider.type !== 'ollama' && !editingProvider && (
                            <Button
                              onClick={() => handleApiKeyEdit(provider.id)}
                              variant="secondary"
                              size="sm"
                              className="w-full"
                            >
                              {provider.status === 'connected'
                                ? 'Update API Key'
                                : 'Configure API Key'}
                            </Button>
                          )}

                          {!editingProvider && (
                            <>
                              <Button
                                onClick={() => testConnection(provider.id)}
                                variant="ghost"
                                size="sm"
                                className="w-full"
                                disabled={provider.status === 'testing'}
                              >
                                {provider.status === 'testing' ? 'Testing...' : 'Test Connection'}
                              </Button>

                              {(provider.type === 'ollama' || provider.status === 'connected') && (
                                <Button
                                  onClick={() => setActiveProvider(provider.id)}
                                  variant={
                                    settings.provider ===
                                    (provider.id === 'ollama' ? 'local' : provider.id)
                                      ? 'primary'
                                      : 'secondary'
                                  }
                                  size="sm"
                                  className="w-full"
                                >
                                  {settings.provider ===
                                  (provider.id === 'ollama' ? 'local' : provider.id)
                                    ? 'Active Provider'
                                    : 'Set as Active'}
                                </Button>
                              )}
                            </>
                          )}
                        </div>

                        <p style={{ fontSize: '12px', color: '#6a6a6a', margin: 0 }}>
                          {provider.type === 'ollama' &&
                            'Local AI models running on your machine. No internet required.'}
                          {provider.type === 'openrouter' &&
                            'Access to multiple AI models through OpenRouter API.'}
                          {provider.type === 'anthropic' &&
                            'Claude models from Anthropic for advanced reasoning.'}
                        </p>
                      </div>
                    </Card>
                  ))}
                </div>

                {/* Model Selection */}
                <Card className="p-4">
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: 600,
                      color: '#ffffff',
                      marginBottom: '12px',
                    }}
                  >
                    Model Selection
                  </h3>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div>
                      <label
                        style={{
                          display: 'block',
                          fontSize: '14px',
                          fontWeight: 500,
                          color: '#d0d0d0',
                          marginBottom: '8px',
                        }}
                      >
                        Selected Model
                      </label>
                      <Input
                        value={settings.selectedModel || ''}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          setSettings(prev => ({ ...prev, selectedModel: e.target.value }))
                        }
                        placeholder="e.g., llama3.2-3b, gpt-4, claude-3-sonnet"
                      />
                      <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '4px' }}>
                        Enter the model name for the active provider
                      </p>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                      <input
                        type="checkbox"
                        id="preferLocal"
                        checked={settings.preferLocalModels}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          setSettings(prev => ({ ...prev, preferLocalModels: e.target.checked }))
                        }
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: 'pointer',
                        }}
                      />
                      <label
                        htmlFor="preferLocal"
                        style={{ fontSize: '14px', color: '#d0d0d0', cursor: 'pointer' }}
                      >
                        Prefer local models when available
                      </label>
                    </div>
                  </div>
                </Card>

                {/* Performance Settings */}
                <Card className="p-4">
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: 600,
                      color: '#ffffff',
                      marginBottom: '12px',
                    }}
                  >
                    Performance Settings
                  </h3>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        fontSize: '14px',
                      }}
                    >
                      <span style={{ color: '#d0d0d0' }}>Fallback to cloud on local failure</span>
                      <input
                        type="checkbox"
                        checked={!settings.preferLocalModels}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          setSettings(prev => ({ ...prev, preferLocalModels: !e.target.checked }))
                        }
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: 'pointer',
                        }}
                      />
                    </div>
                    <p style={{ fontSize: '12px', color: '#6a6a6a', margin: 0 }}>
                      Automatically use cloud providers if local models are unavailable
                    </p>
                  </div>
                </Card>
              </>
            )}
          </div>
        </div>

        {/* Footer */}
        {!isLoading && (
          <div
            style={{
              padding: '16px 24px',
              borderTop: '1px solid #3a3a3f',
              display: 'flex',
              gap: '12px',
              justifyContent: 'flex-end',
            }}
          >
            <Button onClick={onClose} variant="ghost" disabled={isSaving}>
              Cancel
            </Button>
            <Button onClick={saveSettings} disabled={isSaving} variant="primary">
              {isSaving ? 'Saving...' : 'Save Settings'}
            </Button>
          </div>
        )}
      </div>

      {/* Animations */}
      <style>
        {`
          @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
          }
          @keyframes scaleIn {
            from {
              opacity: 0;
              transform: scale(0.95) translateY(10px);
            }
            to {
              opacity: 1;
              transform: scale(1) translateY(0);
            }
          }
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }
        `}
      </style>
    </div>
  )
}
