import React, { useState, useEffect, useCallback } from 'react'
import { apiClient } from '../../api'
import Button from '../ui/Button'
import Input from '../ui/Input'
import Card from '../ui/Card'
import Badge from '../ui/Badge'

interface WorkspaceConfig {
  base_config: Record<string, unknown>
  workspace: {
    version: string
    template: string
    created: string
    last_modified: string
    import_settings: {
      allowed_extensions: string[]
      max_file_size: number
      auto_process: boolean
      duplicate_handling: 'Prompt' | 'Skip' | 'Replace' | 'KeepBoth'
    }
    ai_settings: {
      preferred_local_model: string | null
      cloud_fallback: boolean
      privacy_mode: boolean
    }
    custom_settings: Record<string, unknown>
  }
}

interface BackupConfig {
  enabled: boolean
  auto_backup_enabled: boolean
  backup_interval_hours: number
  max_backups: number
}

interface CacheStats {
  total_cached_items: number
  total_cache_size: number
  last_cleared: string | null
}

interface WorkspaceSettingsModalProps {
  isOpen: boolean
  onClose: () => void
  workspacePath: string
}

export const WorkspaceSettingsModal: React.FC<WorkspaceSettingsModalProps> = ({
  isOpen,
  onClose,
  workspacePath,
}) => {
  const [config, setConfig] = useState<WorkspaceConfig | null>(null)
  const [backupConfig, setBackupConfig] = useState<BackupConfig>({
    enabled: false,
    auto_backup_enabled: false,
    backup_interval_hours: 24,
    max_backups: 5,
  })
  const [cacheStats, setCacheStats] = useState<CacheStats | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [isClearing, setIsClearing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<
    'general' | 'processing' | 'ai' | 'backup' | 'performance'
  >('general')

  // Load workspace configuration
  const loadConfiguration = useCallback(async () => {
    console.log('WorkspaceSettingsModal: loadConfiguration called')
    setIsLoading(true)
    setError(null)

    try {
      // Load workspace config
      const configResponse = await apiClient.invoke<WorkspaceConfig>('get_workspace_config', {
        workspace_path: workspacePath,
      })

      if (
        configResponse.success &&
        configResponse.data &&
        Object.keys(configResponse.data).length > 0
      ) {
        console.log('Loaded workspace config:', configResponse.data)
        setConfig(configResponse.data)
      } else {
        // Use default config if none exists
        const defaultConfig: WorkspaceConfig = {
          base_config: {},
          workspace: {
            version: '1.1.2',
            template: 'Basic',
            created: new Date().toISOString(),
            last_modified: new Date().toISOString(),
            import_settings: {
              allowed_extensions: ['.pdf', '.docx', '.txt', '.md'],
              max_file_size: 100 * 1024 * 1024, // 100MB
              auto_process: true,
              duplicate_handling: 'Prompt',
            },
            ai_settings: {
              preferred_local_model: null,
              cloud_fallback: true,
              privacy_mode: false,
            },
            custom_settings: {},
          },
        }
        setConfig(defaultConfig)
      }

      // Load backup config
      try {
        const backupResponse = await apiClient.invoke<BackupConfig>('get_backup_config')
        if (backupResponse.success && backupResponse.data) {
          setBackupConfig(backupResponse.data)
        }
      } catch (err) {
        console.warn('Backup config not available:', err)
      }

      // Load cache stats
      try {
        const cacheResponse = await apiClient.invoke<CacheStats>('get_cache_stats')
        if (cacheResponse.success && cacheResponse.data) {
          setCacheStats(cacheResponse.data)
        }
      } catch (err) {
        console.warn('Cache stats not available:', err)
      }
    } catch (err) {
      console.error('Error loading workspace configuration:', err)
      setError(err instanceof Error ? err.message : 'Failed to load configuration')
    } finally {
      setIsLoading(false)
    }
  }, [workspacePath])

  // Load configuration when modal opens
  useEffect(() => {
    if (isOpen && workspacePath) {
      loadConfiguration()
    }
  }, [isOpen, workspacePath, loadConfiguration])

  // Save configuration
  const saveConfiguration = useCallback(async () => {
    if (!config) return

    setIsSaving(true)
    setError(null)
    setSuccessMessage(null)

    try {
      // Save workspace config
      const response = await apiClient.invoke<boolean>('update_workspace_config', {
        workspace_path: workspacePath,
        config: config,
      })

      if (response.success) {
        // Save backup config
        try {
          await apiClient.invoke('update_backup_config', {
            config: backupConfig,
          })
        } catch (err) {
          console.warn('Failed to save backup config:', err)
        }

        setSuccessMessage('Workspace settings saved successfully!')

        // Close modal after a short delay
        setTimeout(() => {
          onClose()
        }, 1500)
      } else {
        setError(response.error || 'Failed to save workspace settings')
      }
    } catch (err) {
      console.error('Error saving settings:', err)
      setError(err instanceof Error ? err.message : 'Failed to save settings')
    } finally {
      setIsSaving(false)
    }
  }, [config, backupConfig, workspacePath, onClose])

  // Clear workspace cache
  const clearCache = useCallback(async () => {
    setIsClearing(true)
    setError(null)

    try {
      const response = await apiClient.invoke('clear_performance_caches')

      if (response.success) {
        setSuccessMessage('Cache cleared successfully!')
        // Reload cache stats
        const cacheResponse = await apiClient.invoke<CacheStats>('get_cache_stats')
        if (cacheResponse.success && cacheResponse.data) {
          setCacheStats(cacheResponse.data)
        }
        setTimeout(() => setSuccessMessage(null), 3000)
      } else {
        setError('Failed to clear cache')
        setTimeout(() => setError(null), 3000)
      }
    } catch (err) {
      console.error('Error clearing cache:', err)
      setError(err instanceof Error ? err.message : 'Failed to clear cache')
      setTimeout(() => setError(null), 3000)
    } finally {
      setIsClearing(false)
    }
  }, [])

  // Update config helper
  const updateConfig = useCallback((updater: (prev: WorkspaceConfig) => WorkspaceConfig) => {
    setConfig(prev => (prev ? updater(prev) : null))
  }, [])

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  if (!isOpen) return null

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
        if (e.target === e.currentTarget) {
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
            Workspace Settings
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

        {/* Tabs */}
        <div
          style={{
            padding: '16px 24px',
            borderBottom: '1px solid #3a3a3f',
            display: 'flex',
            gap: '8px',
            overflowX: 'auto',
          }}
        >
          {(['general', 'processing', 'ai', 'backup', 'performance'] as const).map(tab => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                padding: '8px 16px',
                backgroundColor: activeTab === tab ? '#00d4ff' : 'transparent',
                color: activeTab === tab ? '#0a0a0f' : '#a8a8a8',
                border: activeTab === tab ? 'none' : '1px solid #3a3a3f',
                borderRadius: '6px',
                cursor: 'pointer',
                fontSize: '14px',
                fontWeight: activeTab === tab ? 600 : 400,
                transition: 'all 0.2s ease-out',
                whiteSpace: 'nowrap',
              }}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        {/* Content */}
        <div
          style={{
            padding: '24px',
            maxHeight: 'calc(90vh - 200px)',
            overflowY: 'auto',
          }}
        >
          {/* Error/Success Messages */}
          {error && (
            <Card className="bg-red-900/20 border-red-500/50 p-3 mb-4">
              <p className="text-red-400 text-sm">{error}</p>
            </Card>
          )}

          {successMessage && (
            <Card className="bg-green-900/20 border-green-500/50 p-3 mb-4">
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
                <p style={{ color: '#a8a8a8' }}>Loading workspace settings...</p>
              </div>
            </div>
          ) : config ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              {/* General Tab */}
              {activeTab === 'general' && (
                <>
                  <Card className="p-4">
                    <h3
                      style={{
                        fontSize: '16px',
                        fontWeight: 600,
                        color: '#ffffff',
                        marginBottom: '16px',
                      }}
                    >
                      Workspace Information
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
                          Template Type
                        </label>
                        <Badge variant="default">{config.workspace.template}</Badge>
                      </div>
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
                          Version
                        </label>
                        <span style={{ fontSize: '14px', color: '#a8a8a8' }}>
                          {config.workspace.version}
                        </span>
                      </div>
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
                          Created
                        </label>
                        <span style={{ fontSize: '14px', color: '#a8a8a8' }}>
                          {new Date(config.workspace.created).toLocaleString()}
                        </span>
                      </div>
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
                          Last Modified
                        </label>
                        <span style={{ fontSize: '14px', color: '#a8a8a8' }}>
                          {new Date(config.workspace.last_modified).toLocaleString()}
                        </span>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-4">
                    <h3
                      style={{
                        fontSize: '16px',
                        fontWeight: 600,
                        color: '#ffffff',
                        marginBottom: '16px',
                      }}
                    >
                      Auto-Organization Preferences
                    </h3>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                      <input
                        type="checkbox"
                        id="autoProcess"
                        checked={config.workspace.import_settings.auto_process}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updateConfig(prev => ({
                            ...prev,
                            workspace: {
                              ...prev.workspace,
                              import_settings: {
                                ...prev.workspace.import_settings,
                                auto_process: e.target.checked,
                              },
                            },
                          }))
                        }
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: 'pointer',
                        }}
                      />
                      <label
                        htmlFor="autoProcess"
                        style={{ fontSize: '14px', color: '#d0d0d0', cursor: 'pointer' }}
                      >
                        Automatically process imported documents
                      </label>
                    </div>
                    <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '8px' }}>
                      When enabled, documents will be automatically analyzed and organized upon
                      import
                    </p>
                  </Card>
                </>
              )}

              {/* Document Processing Tab */}
              {activeTab === 'processing' && (
                <>
                  <Card className="p-4">
                    <h3
                      style={{
                        fontSize: '16px',
                        fontWeight: 600,
                        color: '#ffffff',
                        marginBottom: '16px',
                      }}
                    >
                      Document Processing Settings
                    </h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
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
                          Maximum File Size
                        </label>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                          <Input
                            type="number"
                            value={Math.round(
                              config.workspace.import_settings.max_file_size / (1024 * 1024)
                            )}
                            onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                              updateConfig(prev => ({
                                ...prev,
                                workspace: {
                                  ...prev.workspace,
                                  import_settings: {
                                    ...prev.workspace.import_settings,
                                    max_file_size: parseInt(e.target.value) * 1024 * 1024,
                                  },
                                },
                              }))
                            }
                            placeholder="100"
                            style={{ maxWidth: '150px' }}
                          />
                          <span style={{ fontSize: '14px', color: '#a8a8a8' }}>MB</span>
                        </div>
                        <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '4px' }}>
                          Current: {formatFileSize(config.workspace.import_settings.max_file_size)}
                        </p>
                      </div>

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
                          Duplicate File Handling
                        </label>
                        <select
                          value={config.workspace.import_settings.duplicate_handling}
                          onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
                            updateConfig(prev => ({
                              ...prev,
                              workspace: {
                                ...prev.workspace,
                                import_settings: {
                                  ...prev.workspace.import_settings,
                                  duplicate_handling: e.target.value as
                                    | 'Prompt'
                                    | 'Skip'
                                    | 'Replace'
                                    | 'KeepBoth',
                                },
                              },
                            }))
                          }
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            backgroundColor: '#2a2a2f',
                            border: '1px solid #3a3a3f',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px',
                          }}
                        >
                          <option value="Prompt">Ask me each time</option>
                          <option value="Skip">Skip duplicates</option>
                          <option value="Replace">Replace existing files</option>
                          <option value="KeepBoth">Keep both versions</option>
                        </select>
                      </div>

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
                          Allowed File Extensions
                        </label>
                        <div
                          style={{
                            display: 'flex',
                            flexWrap: 'wrap',
                            gap: '8px',
                            marginBottom: '12px',
                          }}
                        >
                          {config.workspace.import_settings.allowed_extensions.map(ext => (
                            <Badge key={ext} variant="default">
                              {ext}
                            </Badge>
                          ))}
                        </div>
                        <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                          {config.workspace.import_settings.allowed_extensions.length} file type(s)
                          allowed
                        </p>
                      </div>
                    </div>
                  </Card>
                </>
              )}

              {/* AI Settings Tab */}
              {activeTab === 'ai' && (
                <>
                  <Card className="p-4">
                    <h3
                      style={{
                        fontSize: '16px',
                        fontWeight: 600,
                        color: '#ffffff',
                        marginBottom: '16px',
                      }}
                    >
                      Workspace AI Configuration
                    </h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
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
                          Preferred Local Model
                        </label>
                        <Input
                          value={config.workspace.ai_settings.preferred_local_model || ''}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            updateConfig(prev => ({
                              ...prev,
                              workspace: {
                                ...prev.workspace,
                                ai_settings: {
                                  ...prev.workspace.ai_settings,
                                  preferred_local_model: e.target.value || null,
                                },
                              },
                            }))
                          }
                          placeholder="e.g., llama3.2-3b"
                        />
                        <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '4px' }}>
                          Specify a local model for this workspace (optional)
                        </p>
                      </div>

                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                        }}
                      >
                        <div>
                          <label
                            style={{
                              display: 'block',
                              fontSize: '14px',
                              fontWeight: 500,
                              color: '#d0d0d0',
                              marginBottom: '4px',
                            }}
                          >
                            Cloud Fallback
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Use cloud AI when local models are unavailable
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={config.workspace.ai_settings.cloud_fallback}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            updateConfig(prev => ({
                              ...prev,
                              workspace: {
                                ...prev.workspace,
                                ai_settings: {
                                  ...prev.workspace.ai_settings,
                                  cloud_fallback: e.target.checked,
                                },
                              },
                            }))
                          }
                          style={{
                            width: '16px',
                            height: '16px',
                            cursor: 'pointer',
                          }}
                        />
                      </div>

                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                        }}
                      >
                        <div>
                          <label
                            style={{
                              display: 'block',
                              fontSize: '14px',
                              fontWeight: 500,
                              color: '#d0d0d0',
                              marginBottom: '4px',
                            }}
                          >
                            Privacy Mode
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Restrict AI processing to local models only
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={config.workspace.ai_settings.privacy_mode}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            updateConfig(prev => ({
                              ...prev,
                              workspace: {
                                ...prev.workspace,
                                ai_settings: {
                                  ...prev.workspace.ai_settings,
                                  privacy_mode: e.target.checked,
                                },
                              },
                            }))
                          }
                          style={{
                            width: '16px',
                            height: '16px',
                            cursor: 'pointer',
                          }}
                        />
                      </div>
                    </div>
                  </Card>
                </>
              )}

              {/* Backup & Sync Tab */}
              {activeTab === 'backup' && (
                <>
                  <Card className="p-4">
                    <h3
                      style={{
                        fontSize: '16px',
                        fontWeight: 600,
                        color: '#ffffff',
                        marginBottom: '16px',
                      }}
                    >
                      Backup Configuration
                    </h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                        }}
                      >
                        <div>
                          <label
                            style={{
                              display: 'block',
                              fontSize: '14px',
                              fontWeight: 500,
                              color: '#d0d0d0',
                              marginBottom: '4px',
                            }}
                          >
                            Enable Backups
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Create regular backups of workspace data
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={backupConfig.enabled}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            setBackupConfig(prev => ({ ...prev, enabled: e.target.checked }))
                          }
                          style={{
                            width: '16px',
                            height: '16px',
                            cursor: 'pointer',
                          }}
                        />
                      </div>

                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                        }}
                      >
                        <div>
                          <label
                            style={{
                              display: 'block',
                              fontSize: '14px',
                              fontWeight: 500,
                              color: '#d0d0d0',
                              marginBottom: '4px',
                            }}
                          >
                            Automatic Backups
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Automatically create backups at scheduled intervals
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={backupConfig.auto_backup_enabled}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            setBackupConfig(prev => ({
                              ...prev,
                              auto_backup_enabled: e.target.checked,
                            }))
                          }
                          disabled={!backupConfig.enabled}
                          style={{
                            width: '16px',
                            height: '16px',
                            cursor: backupConfig.enabled ? 'pointer' : 'not-allowed',
                            opacity: backupConfig.enabled ? 1 : 0.5,
                          }}
                        />
                      </div>

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
                          Backup Interval (hours)
                        </label>
                        <Input
                          type="number"
                          value={backupConfig.backup_interval_hours}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            setBackupConfig(prev => ({
                              ...prev,
                              backup_interval_hours: parseInt(e.target.value) || 24,
                            }))
                          }
                          disabled={!backupConfig.enabled || !backupConfig.auto_backup_enabled}
                          min="1"
                          max="168"
                          placeholder="24"
                        />
                      </div>

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
                          Maximum Backups to Keep
                        </label>
                        <Input
                          type="number"
                          value={backupConfig.max_backups}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            setBackupConfig(prev => ({
                              ...prev,
                              max_backups: parseInt(e.target.value) || 5,
                            }))
                          }
                          disabled={!backupConfig.enabled}
                          min="1"
                          max="50"
                          placeholder="5"
                        />
                        <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '4px' }}>
                          Older backups will be automatically deleted
                        </p>
                      </div>
                    </div>
                  </Card>
                </>
              )}

              {/* Performance & Cache Tab */}
              {activeTab === 'performance' && (
                <>
                  <Card className="p-4">
                    <h3
                      style={{
                        fontSize: '16px',
                        fontWeight: 600,
                        color: '#ffffff',
                        marginBottom: '16px',
                      }}
                    >
                      Performance & Cache Management
                    </h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                      {cacheStats && (
                        <>
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
                              Cache Statistics
                            </label>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                              <div
                                style={{
                                  fontSize: '14px',
                                  color: '#a8a8a8',
                                  display: 'flex',
                                  justifyContent: 'space-between',
                                }}
                              >
                                <span>Cached Items:</span>
                                <span>{cacheStats.total_cached_items}</span>
                              </div>
                              <div
                                style={{
                                  fontSize: '14px',
                                  color: '#a8a8a8',
                                  display: 'flex',
                                  justifyContent: 'space-between',
                                }}
                              >
                                <span>Cache Size:</span>
                                <span>{formatFileSize(cacheStats.total_cache_size)}</span>
                              </div>
                              {cacheStats.last_cleared && (
                                <div
                                  style={{
                                    fontSize: '14px',
                                    color: '#a8a8a8',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                  }}
                                >
                                  <span>Last Cleared:</span>
                                  <span>{new Date(cacheStats.last_cleared).toLocaleString()}</span>
                                </div>
                              )}
                            </div>
                          </div>
                          <div>
                            <Button
                              variant="secondary"
                              onClick={clearCache}
                              disabled={isClearing}
                              style={{ width: '100%' }}
                            >
                              {isClearing ? 'Clearing Cache...' : 'Clear Cache'}
                            </Button>
                            <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '8px' }}>
                              Clearing the cache may temporarily reduce performance until data is
                              re-cached
                            </p>
                          </div>
                        </>
                      )}

                      {!cacheStats && (
                        <div
                          style={{
                            textAlign: 'center',
                            padding: '24px',
                            color: '#a8a8a8',
                          }}
                        >
                          Cache statistics not available
                        </div>
                      )}
                    </div>
                  </Card>
                </>
              )}
            </div>
          ) : (
            <div
              style={{
                textAlign: 'center',
                padding: '48px',
                color: '#a8a8a8',
              }}
            >
              No workspace configuration found
            </div>
          )}
        </div>

        {/* Footer */}
        {!isLoading && config && (
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
            <Button onClick={saveConfiguration} disabled={isSaving} variant="primary">
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
