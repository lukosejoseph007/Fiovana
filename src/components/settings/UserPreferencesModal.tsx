import React, { useState, useEffect, useCallback } from 'react'
import Button from '../ui/Button'
import Input from '../ui/Input'
import Card from '../ui/Card'

export interface UserPreferences {
  theme: 'light' | 'dark' | 'system'
  layout: {
    navigationWidth: number
    intelligenceWidth: number
    defaultView: 'document' | 'dashboard' | 'search'
  }
  keyboardShortcuts: {
    enabled: boolean
    commandPalette: string
    search: string
    newDocument: string
  }
  notifications: {
    enabled: boolean
    documentProcessing: boolean
    aiCompletion: boolean
    systemUpdates: boolean
  }
  accessibility: {
    reducedMotion: boolean
    highContrast: boolean
    fontSize: 'small' | 'medium' | 'large'
    screenReaderOptimized: boolean
  }
}

interface UserPreferencesModalProps {
  isOpen: boolean
  onClose: () => void
}

const defaultPreferences: UserPreferences = {
  theme: 'dark',
  layout: {
    navigationWidth: 240,
    intelligenceWidth: 320,
    defaultView: 'document',
  },
  keyboardShortcuts: {
    enabled: true,
    commandPalette: 'Cmd+K',
    search: '/',
    newDocument: 'Cmd+N',
  },
  notifications: {
    enabled: true,
    documentProcessing: true,
    aiCompletion: true,
    systemUpdates: false,
  },
  accessibility: {
    reducedMotion: false,
    highContrast: false,
    fontSize: 'medium',
    screenReaderOptimized: false,
  },
}

export const UserPreferencesModal: React.FC<UserPreferencesModalProps> = ({ isOpen, onClose }) => {
  const [preferences, setPreferences] = useState<UserPreferences>(defaultPreferences)
  const [isLoading, setIsLoading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<
    'theme' | 'layout' | 'keyboard' | 'notifications' | 'accessibility'
  >('theme')

  // Load preferences from localStorage
  const loadPreferences = useCallback(() => {
    setIsLoading(true)
    setError(null)

    try {
      const stored = localStorage.getItem('userPreferences')
      if (stored) {
        const parsed = JSON.parse(stored)
        setPreferences({ ...defaultPreferences, ...parsed })
      } else {
        setPreferences(defaultPreferences)
      }
    } catch (err) {
      console.error('Error loading user preferences:', err)
      setError(err instanceof Error ? err.message : 'Failed to load preferences')
      setPreferences(defaultPreferences)
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Load preferences when modal opens
  useEffect(() => {
    if (isOpen) {
      loadPreferences()
    }
  }, [isOpen, loadPreferences])

  // Save preferences to localStorage
  const savePreferences = useCallback(async () => {
    setIsSaving(true)
    setError(null)
    setSuccessMessage(null)

    try {
      localStorage.setItem('userPreferences', JSON.stringify(preferences))
      setSuccessMessage('User preferences saved successfully!')

      // Apply theme immediately
      document.documentElement.setAttribute('data-theme', preferences.theme)

      // Apply accessibility settings immediately
      if (preferences.accessibility.reducedMotion) {
        document.documentElement.style.setProperty('--animation-duration', '0.01ms')
      } else {
        document.documentElement.style.removeProperty('--animation-duration')
      }

      // Close modal after a short delay
      setTimeout(() => {
        onClose()
      }, 1500)
    } catch (err) {
      console.error('Error saving preferences:', err)
      setError(err instanceof Error ? err.message : 'Failed to save preferences')
    } finally {
      setIsSaving(false)
    }
  }, [preferences, onClose])

  // Update preference helper
  const updatePreferences = useCallback(
    <K extends keyof UserPreferences>(key: K, value: UserPreferences[K]) => {
      setPreferences(prev => ({
        ...prev,
        [key]: value,
      }))
    },
    []
  )

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
            User Preferences
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
          {(
            [
              'theme',
              'layout',
              'keyboard',
              'notifications',
              'accessibility',
            ] as (typeof activeTab)[]
          ).map(tab => (
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
                <p style={{ color: '#a8a8a8' }}>Loading user preferences...</p>
              </div>
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              {/* Theme Tab */}
              {activeTab === 'theme' && (
                <Card className="p-4">
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: 600,
                      color: '#ffffff',
                      marginBottom: '16px',
                    }}
                  >
                    Theme Settings
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
                        Theme Mode
                      </label>
                      <div style={{ display: 'flex', gap: '12px' }}>
                        {(['light', 'dark', 'system'] as const).map(theme => (
                          <button
                            key={theme}
                            onClick={() => updatePreferences('theme', theme)}
                            style={{
                              flex: 1,
                              padding: '12px',
                              backgroundColor: preferences.theme === theme ? '#00d4ff' : '#2a2a2f',
                              color: preferences.theme === theme ? '#0a0a0f' : '#d0d0d0',
                              border: preferences.theme === theme ? 'none' : '1px solid #3a3a3f',
                              borderRadius: '6px',
                              cursor: 'pointer',
                              fontSize: '14px',
                              fontWeight: preferences.theme === theme ? 600 : 400,
                              transition: 'all 0.2s ease-out',
                            }}
                          >
                            {theme.charAt(0).toUpperCase() + theme.slice(1)}
                          </button>
                        ))}
                      </div>
                      <p style={{ fontSize: '12px', color: '#6a6a6a', marginTop: '8px' }}>
                        {preferences.theme === 'system'
                          ? 'Automatically match your system theme'
                          : `Use ${preferences.theme} theme`}
                      </p>
                    </div>
                  </div>
                </Card>
              )}

              {/* Layout Tab */}
              {activeTab === 'layout' && (
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
                      Panel Widths
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
                          Navigation Panel Width
                        </label>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                          <Input
                            type="number"
                            value={preferences.layout.navigationWidth}
                            onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                              updatePreferences('layout', {
                                ...preferences.layout,
                                navigationWidth: parseInt(e.target.value) || 240,
                              })
                            }
                            min="200"
                            max="400"
                            style={{ maxWidth: '150px' }}
                          />
                          <span style={{ fontSize: '14px', color: '#a8a8a8' }}>px</span>
                        </div>
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
                          Intelligence Panel Width
                        </label>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                          <Input
                            type="number"
                            value={preferences.layout.intelligenceWidth}
                            onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                              updatePreferences('layout', {
                                ...preferences.layout,
                                intelligenceWidth: parseInt(e.target.value) || 320,
                              })
                            }
                            min="280"
                            max="500"
                            style={{ maxWidth: '150px' }}
                          />
                          <span style={{ fontSize: '14px', color: '#a8a8a8' }}>px</span>
                        </div>
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
                      Default View
                    </h3>
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
                        Start Page
                      </label>
                      <select
                        value={preferences.layout.defaultView}
                        onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
                          updatePreferences('layout', {
                            ...preferences.layout,
                            defaultView: e.target.value as 'document' | 'dashboard' | 'search',
                          })
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
                        <option value="document">Document Canvas</option>
                        <option value="dashboard">Workspace Dashboard</option>
                        <option value="search">Search Interface</option>
                      </select>
                    </div>
                  </Card>
                </>
              )}

              {/* Keyboard Shortcuts Tab */}
              {activeTab === 'keyboard' && (
                <Card className="p-4">
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: 600,
                      color: '#ffffff',
                      marginBottom: '16px',
                    }}
                  >
                    Keyboard Shortcuts
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
                          Enable Keyboard Shortcuts
                        </label>
                        <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                          Enable or disable all keyboard shortcuts
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={preferences.keyboardShortcuts.enabled}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('keyboardShortcuts', {
                            ...preferences.keyboardShortcuts,
                            enabled: e.target.checked,
                          })
                        }
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: 'pointer',
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
                        Command Palette
                      </label>
                      <Input
                        value={preferences.keyboardShortcuts.commandPalette}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('keyboardShortcuts', {
                            ...preferences.keyboardShortcuts,
                            commandPalette: e.target.value,
                          })
                        }
                        disabled={!preferences.keyboardShortcuts.enabled}
                        placeholder="Cmd+K"
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
                        Search
                      </label>
                      <Input
                        value={preferences.keyboardShortcuts.search}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('keyboardShortcuts', {
                            ...preferences.keyboardShortcuts,
                            search: e.target.value,
                          })
                        }
                        disabled={!preferences.keyboardShortcuts.enabled}
                        placeholder="/"
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
                        New Document
                      </label>
                      <Input
                        value={preferences.keyboardShortcuts.newDocument}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('keyboardShortcuts', {
                            ...preferences.keyboardShortcuts,
                            newDocument: e.target.value,
                          })
                        }
                        disabled={!preferences.keyboardShortcuts.enabled}
                        placeholder="Cmd+N"
                      />
                    </div>

                    <p style={{ fontSize: '12px', color: '#6a6a6a', margin: 0 }}>
                      Use Cmd (Mac) or Ctrl (Windows/Linux) for modifier keys
                    </p>
                  </div>
                </Card>
              )}

              {/* Notifications Tab */}
              {activeTab === 'notifications' && (
                <Card className="p-4">
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: 600,
                      color: '#ffffff',
                      marginBottom: '16px',
                    }}
                  >
                    Notification Settings
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
                          Enable Notifications
                        </label>
                        <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                          Enable or disable all notifications
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={preferences.notifications.enabled}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('notifications', {
                            ...preferences.notifications,
                            enabled: e.target.checked,
                          })
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
                          Document Processing
                        </label>
                        <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                          Notify when document processing completes
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={preferences.notifications.documentProcessing}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('notifications', {
                            ...preferences.notifications,
                            documentProcessing: e.target.checked,
                          })
                        }
                        disabled={!preferences.notifications.enabled}
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: preferences.notifications.enabled ? 'pointer' : 'not-allowed',
                          opacity: preferences.notifications.enabled ? 1 : 0.5,
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
                          AI Completion
                        </label>
                        <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                          Notify when AI operations complete
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={preferences.notifications.aiCompletion}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('notifications', {
                            ...preferences.notifications,
                            aiCompletion: e.target.checked,
                          })
                        }
                        disabled={!preferences.notifications.enabled}
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: preferences.notifications.enabled ? 'pointer' : 'not-allowed',
                          opacity: preferences.notifications.enabled ? 1 : 0.5,
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
                          System Updates
                        </label>
                        <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                          Notify about system updates and maintenance
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={preferences.notifications.systemUpdates}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          updatePreferences('notifications', {
                            ...preferences.notifications,
                            systemUpdates: e.target.checked,
                          })
                        }
                        disabled={!preferences.notifications.enabled}
                        style={{
                          width: '16px',
                          height: '16px',
                          cursor: preferences.notifications.enabled ? 'pointer' : 'not-allowed',
                          opacity: preferences.notifications.enabled ? 1 : 0.5,
                        }}
                      />
                    </div>
                  </div>
                </Card>
              )}

              {/* Accessibility Tab */}
              {activeTab === 'accessibility' && (
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
                      Accessibility Settings
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
                            Reduced Motion
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Minimize animations and transitions
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={preferences.accessibility.reducedMotion}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            updatePreferences('accessibility', {
                              ...preferences.accessibility,
                              reducedMotion: e.target.checked,
                            })
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
                            High Contrast
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Increase contrast for better visibility
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={preferences.accessibility.highContrast}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            updatePreferences('accessibility', {
                              ...preferences.accessibility,
                              highContrast: e.target.checked,
                            })
                          }
                          style={{
                            width: '16px',
                            height: '16px',
                            cursor: 'pointer',
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
                          Font Size
                        </label>
                        <div style={{ display: 'flex', gap: '12px' }}>
                          {(['small', 'medium', 'large'] as const).map(size => (
                            <button
                              key={size}
                              onClick={() =>
                                updatePreferences('accessibility', {
                                  ...preferences.accessibility,
                                  fontSize: size,
                                })
                              }
                              style={{
                                flex: 1,
                                padding: '12px',
                                backgroundColor:
                                  preferences.accessibility.fontSize === size
                                    ? '#00d4ff'
                                    : '#2a2a2f',
                                color:
                                  preferences.accessibility.fontSize === size
                                    ? '#0a0a0f'
                                    : '#d0d0d0',
                                border:
                                  preferences.accessibility.fontSize === size
                                    ? 'none'
                                    : '1px solid #3a3a3f',
                                borderRadius: '6px',
                                cursor: 'pointer',
                                fontSize: '14px',
                                fontWeight: preferences.accessibility.fontSize === size ? 600 : 400,
                                transition: 'all 0.2s ease-out',
                              }}
                            >
                              {size.charAt(0).toUpperCase() + size.slice(1)}
                            </button>
                          ))}
                        </div>
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
                            Screen Reader Optimized
                          </label>
                          <p style={{ fontSize: '12px', color: '#6a6a6a' }}>
                            Enhanced support for screen readers
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={preferences.accessibility.screenReaderOptimized}
                          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                            updatePreferences('accessibility', {
                              ...preferences.accessibility,
                              screenReaderOptimized: e.target.checked,
                            })
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
            </div>
          )}
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
            <Button onClick={savePreferences} disabled={isSaving} variant="primary">
              {isSaving ? 'Saving...' : 'Save Preferences'}
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
