import React, { useState, useCallback, useEffect } from 'react'
import AppShell from './components/layout/AppShell'
import HeaderBar from './components/layout/HeaderBar'
import IntelligencePanel from './components/intelligence/IntelligencePanel'
import NavigationPanel from './components/navigation/NavigationPanel'
import DocumentCanvas from './components/canvas/DocumentCanvas'
import WorkspaceIntelligence from './components/workspace/WorkspaceIntelligence'
import { AnalyticsDashboard } from './components/analytics'
import SearchInterface from './components/search/SearchInterface'
import { ContentDiscovery } from './components/discovery/ContentDiscovery'
import SmartCollections from './components/collections/SmartCollections'
import { useLayout } from './components/layout/useLayoutContext'
import GenerationModal from './components/generation/GenerationModal'
import StyleTransfer from './components/generation/StyleTransfer'
import { AIProvidersModal } from './components/settings/AIProvidersModal'
import { WorkspaceSettingsModal } from './components/settings/WorkspaceSettingsModal'
import { UserPreferencesModal } from './components/settings/UserPreferencesModal'
import { ErrorBoundary } from './components/error/ErrorBoundary'
import LoadingState from './components/ui/LoadingState'
import { LongOperationProgress, type OperationProgress } from './components/ui/LoadingStates'
import { documentService } from './services'
import { apiClient } from './api/client'
import LoadingStatesDemo from './components/demo/LoadingStatesDemo'
import { designTokens } from './styles/tokens'
import { CollaborationProvider } from './context/CollaborationContext'

type ViewMode =
  | 'document'
  | 'dashboard'
  | 'analytics'
  | 'search'
  | 'discovery'
  | 'collections'
  | 'loading-demo'

// Component that uses layout context
const AppContent: React.FC = () => {
  const { navigationCollapsed } = useLayout()
  const [viewMode, setViewMode] = useState<ViewMode>('document')
  const [currentDocumentId, setCurrentDocumentId] = useState<string | null>(null)
  const [activeOperation, setActiveOperation] = useState<{
    type: string
    label: string
    progress: number
    status: 'running' | 'completed' | 'error'
  } | null>(null)
  const [isGenerationModalOpen, setIsGenerationModalOpen] = useState(false)
  const [isStyleTransferModalOpen, setIsStyleTransferModalOpen] = useState(false)
  const [isAISettingsModalOpen, setIsAISettingsModalOpen] = useState(false)
  const [isWorkspaceSettingsModalOpen, setIsWorkspaceSettingsModalOpen] = useState(false)
  const [isUserPreferencesModalOpen, setIsUserPreferencesModalOpen] = useState(false)

  // App initialization state
  const [isAppLoading, setIsAppLoading] = useState(true)
  const [initOperations, setInitOperations] = useState<OperationProgress[]>([])
  const [aiStatus, setAiStatus] = useState<{
    isConnected: boolean
    isProcessing: boolean
    provider?: string
  }>({ isConnected: false, isProcessing: false })

  // Helper to update operation status
  const updateInitOperation = useCallback((id: string, updates: Partial<OperationProgress>) => {
    setInitOperations(prev => prev.map(op => (op.id === id ? { ...op, ...updates } : op)))
  }, [])

  // Initialize document indexer and AI system on app startup
  useEffect(() => {
    const initializeApp = async () => {
      // Initialize operations tracking
      const operations: OperationProgress[] = [
        {
          id: 'init-ui',
          operation: 'Initializing user interface',
          status: 'in-progress',
          progress: 10,
        },
        {
          id: 'init-document-system',
          operation: 'Loading document indexer',
          status: 'pending',
          progress: 0,
        },
        {
          id: 'init-ai-system',
          operation: 'Connecting AI services',
          status: 'pending',
          progress: 0,
        },
        {
          id: 'init-workspace',
          operation: 'Preparing workspace',
          status: 'pending',
          progress: 0,
        },
      ]
      setInitOperations(operations)

      // Small delay to show the loading screen
      await new Promise(resolve => setTimeout(resolve, 300))
      updateInitOperation('init-ui', { status: 'completed', progress: 100 })

      // Initialize document system
      updateInitOperation('init-document-system', { status: 'in-progress', progress: 20 })
      try {
        console.log('Initializing document indexer...')
        const result = await documentService.initializeIndexer()
        if (result.success) {
          console.log('Document indexer initialized successfully')
          updateInitOperation('init-document-system', {
            status: 'in-progress',
            progress: 60,
            details: 'Loading document index...',
          })

          // Get initial stats
          const stats = await documentService.getIndexStats()
          if (stats.success && stats.data) {
            console.log(
              `Document index ready: ${stats.data.total_documents} documents, ${stats.data.total_keywords} keywords`
            )
            updateInitOperation('init-document-system', {
              status: 'completed',
              progress: 100,
              details: `${stats.data.total_documents} documents indexed`,
            })
          } else {
            updateInitOperation('init-document-system', { status: 'completed', progress: 100 })
          }
        } else {
          console.warn('Document indexer initialization returned false:', result.error)
          updateInitOperation('init-document-system', {
            status: 'failed',
            details: result.error || 'Initialization failed',
          })
        }
      } catch (error) {
        console.error('Failed to initialize document indexer:', error)
        updateInitOperation('init-document-system', {
          status: 'failed',
          details: 'Failed to initialize',
        })
      }

      // Initialize AI system (non-blocking with timeout)
      updateInitOperation('init-ai-system', { status: 'in-progress', progress: 20 })

      // Create a timeout promise
      const aiTimeout = new Promise<void>(resolve => {
        setTimeout(() => {
          console.warn('AI system initialization timeout (10s)')
          resolve()
        }, 10000) // 10 second timeout
      })

      // Create the AI initialization promise
      const aiInit = (async () => {
        try {
          console.log('Initializing AI system...')
          // Load existing AI settings
          const settingsResponse = await apiClient.invoke('get_ai_settings')
          if (settingsResponse.success && settingsResponse.data) {
            const settings = settingsResponse.data as {
              provider?: string
              anthropicApiKey?: string
              openrouterApiKey?: string
              selectedModel?: string
            }

            // Only initialize if we have a provider and API key configured
            const hasValidConfig =
              (settings.provider === 'anthropic' && settings.anthropicApiKey) ||
              (settings.provider === 'openrouter' && settings.openrouterApiKey) ||
              settings.provider === 'local'

            if (hasValidConfig) {
              console.log('Valid AI configuration found, initializing AI system...')
              updateInitOperation('init-ai-system', {
                status: 'in-progress',
                progress: 60,
                details: `Connecting to ${settings.provider}...`,
              })
              await apiClient.invoke('init_ai_system', { config: settings })
              console.log('AI system initialized successfully')

              // Update AI status
              setAiStatus({
                isConnected: true,
                isProcessing: false,
                provider: settings.provider,
              })

              updateInitOperation('init-ai-system', {
                status: 'completed',
                progress: 100,
                details: `Connected to ${settings.provider}`,
              })
            } else {
              console.log('No valid AI configuration found, skipping AI initialization')

              // Set AI as disconnected
              setAiStatus({
                isConnected: false,
                isProcessing: false,
              })

              updateInitOperation('init-ai-system', {
                status: 'completed',
                progress: 100,
                details: 'No AI provider configured',
              })
            }
          } else {
            setAiStatus({
              isConnected: false,
              isProcessing: false,
            })
            updateInitOperation('init-ai-system', {
              status: 'completed',
              progress: 100,
              details: 'Using default settings',
            })
          }
        } catch (error) {
          console.error('Failed to initialize AI system:', error)
          setAiStatus({
            isConnected: false,
            isProcessing: false,
          })
          updateInitOperation('init-ai-system', {
            status: 'failed',
            details: 'Connection failed',
          })
        }
      })()

      // Race between timeout and AI init - don't block app startup
      await Promise.race([aiInit, aiTimeout]).catch(() => {
        // Even if there's an error, continue with app initialization
        console.log('AI initialization completed or timed out, continuing...')
        setAiStatus({
          isConnected: false,
          isProcessing: false,
        })
        updateInitOperation('init-ai-system', {
          status: 'failed',
          details: 'Connection timeout',
        })
      })

      // Initialize workspace
      updateInitOperation('init-workspace', { status: 'in-progress', progress: 50 })
      await new Promise(resolve => setTimeout(resolve, 200))
      updateInitOperation('init-workspace', { status: 'completed', progress: 100 })

      // Wait a bit to show completion
      await new Promise(resolve => setTimeout(resolve, 500))
      setIsAppLoading(false)
    }

    initializeApp()
  }, [updateInitOperation])

  // Handle navigation item selection
  const handleNavigationSelect = useCallback(
    (item: { id: string; label: string; icon: string }) => {
      console.log('Navigation item selected:', item.id, 'Full item:', item)
      if (item.id === 'search') {
        console.log('Switching to search view')
        setViewMode('search')
      } else if (item.id === 'workspace-dashboard') {
        console.log('Switching to dashboard view')
        setViewMode('dashboard')
      } else if (item.id === 'analytics-dashboard') {
        console.log('Switching to analytics view')
        setViewMode('analytics')
      } else if (item.id === 'content-discovery') {
        console.log('Switching to discovery view')
        setViewMode('discovery')
      } else if (item.id === 'smart-collections') {
        console.log('Switching to collections view')
        setViewMode('collections')
      } else if (item.id === 'loading-demo') {
        console.log('Switching to loading demo view')
        setViewMode('loading-demo')
      } else {
        // For document items from "Active Documents" section, open the document
        console.log('Opening document with ID:', item.id)
        setCurrentDocumentId(item.id)
        setViewMode('document')
      }
    },
    []
  )

  const handleLogoClick = useCallback(() => {
    setViewMode('document')
  }, [])

  const handleAISettingsClick = useCallback(() => {
    console.log('AI Settings clicked')
    setIsAISettingsModalOpen(true)
  }, [])

  const handleWorkspaceSettingsClick = useCallback(() => {
    console.log('Workspace Settings clicked')
    setIsWorkspaceSettingsModalOpen(true)
  }, [])

  const handleUserPreferencesClick = useCallback(() => {
    console.log('User Preferences clicked')
    setIsUserPreferencesModalOpen(true)
  }, [])

  const handleOperationTrigger = useCallback(async (operationType: string) => {
    console.log('Operation triggered from HeaderBar:', operationType)

    // Handle different operation types
    switch (operationType) {
      case 'generate':
        setIsGenerationModalOpen(true)
        break

      case 'styleTransfer':
        setIsStyleTransferModalOpen(true)
        break

      case 'search':
        // Trigger search - switch to search view
        setViewMode('search')
        break

      case 'analyze':
      case 'compare':
      case 'update':
      case 'organize':
      case 'batch': {
        // Show progress notification
        const labels: Record<string, string> = {
          analyze: 'Analyzing Document',
          compare: 'Comparing Documents',
          update: 'Updating Document',
          organize: 'Organizing Content',
          batch: 'Processing Batch Operations',
        }

        setActiveOperation({
          type: operationType,
          label: labels[operationType] || 'Processing',
          progress: 0,
          status: 'running',
        })

        // Simulate progress
        const progressInterval = setInterval(() => {
          setActiveOperation(prev =>
            prev ? { ...prev, progress: Math.min(prev.progress + 10, 90) } : null
          )
        }, 300)

        try {
          // Execute operation
          if (operationType === 'analyze') {
            // Example: analyze current document
            await new Promise(resolve => setTimeout(resolve, 2000))
          }

          clearInterval(progressInterval)

          // Show completion
          setActiveOperation(prev =>
            prev ? { ...prev, progress: 100, status: 'completed' } : null
          )

          setTimeout(() => setActiveOperation(null), 2000)
        } catch {
          clearInterval(progressInterval)
          setActiveOperation(prev => (prev ? { ...prev, status: 'error' } : null))
          setTimeout(() => setActiveOperation(null), 3000)
        }
        break
      }
    }
  }, [])

  // Show loading screen during app initialization
  if (isAppLoading) {
    return (
      <div
        style={{
          width: '100vw',
          height: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: designTokens.colors.background.canvas,
          flexDirection: 'column',
          gap: designTokens.spacing[8],
        }}
      >
        {/* Logo/Branding */}
        <div
          style={{
            fontSize: designTokens.typography.fontSize['3xl'],
            fontWeight: designTokens.typography.fontWeight.bold,
            color: designTokens.colors.accent.ai,
            marginBottom: designTokens.spacing[4],
            textAlign: 'center',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[3],
              justifyContent: 'center',
            }}
          >
            <div
              style={{
                width: '48px',
                height: '48px',
                borderRadius: designTokens.borderRadius.lg,
                background: `linear-gradient(135deg, ${designTokens.colors.accent.ai}, ${designTokens.colors.accent.semantic})`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '24px',
              }}
            >
              P
            </div>
            <span>Proxemic</span>
          </div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              fontWeight: designTokens.typography.fontWeight.normal,
              color: designTokens.colors.text.secondary,
              marginTop: designTokens.spacing[2],
            }}
          >
            AI-Powered Document Intelligence
          </div>
        </div>

        {/* Loading Progress */}
        <div style={{ width: '100%', maxWidth: '600px', padding: `0 ${designTokens.spacing[6]}` }}>
          <LongOperationProgress
            operation="Initializing Proxemic"
            details={
              initOperations.find(op => op.status === 'in-progress')?.operation || 'Starting up...'
            }
            variant="ai"
          />

          <div style={{ marginTop: designTokens.spacing[6] }}>
            {initOperations.length > 0 && (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: designTokens.spacing[2],
                }}
              >
                {initOperations.map(op => (
                  <div
                    key={op.id}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: designTokens.spacing[3],
                      padding: designTokens.spacing[2],
                      borderRadius: designTokens.borderRadius.md,
                      backgroundColor:
                        op.status === 'in-progress'
                          ? designTokens.colors.surface.secondary
                          : 'transparent',
                      transition: `all ${designTokens.animation.duration.normal}`,
                    }}
                  >
                    {op.status === 'completed' && (
                      <div
                        style={{
                          width: '16px',
                          height: '16px',
                          borderRadius: '50%',
                          backgroundColor: designTokens.colors.confidence.high,
                        }}
                      />
                    )}
                    {op.status === 'in-progress' && (
                      <LoadingState variant="spinner" size="sm" style={{ padding: 0 }} />
                    )}
                    {op.status === 'pending' && (
                      <div
                        style={{
                          width: '16px',
                          height: '16px',
                          borderRadius: '50%',
                          border: `2px solid ${designTokens.colors.border.subtle}`,
                        }}
                      />
                    )}
                    {op.status === 'failed' && (
                      <div
                        style={{
                          width: '16px',
                          height: '16px',
                          borderRadius: '50%',
                          backgroundColor: designTokens.colors.confidence.critical,
                        }}
                      />
                    )}
                    <div style={{ flex: 1 }}>
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.sm,
                          color: designTokens.colors.text.primary,
                          fontWeight:
                            op.status === 'in-progress'
                              ? designTokens.typography.fontWeight.medium
                              : designTokens.typography.fontWeight.normal,
                        }}
                      >
                        {op.operation}
                      </div>
                      {op.details && (
                        <div
                          style={{
                            fontSize: designTokens.typography.fontSize.xs,
                            color: designTokens.colors.text.tertiary,
                            marginTop: designTokens.spacing[0.5],
                          }}
                        >
                          {op.details}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    )
  }

  return (
    <>
      {/* Header */}
      <AppShell.Header>
        <HeaderBar
          onLogoClick={handleLogoClick}
          onOperationTrigger={handleOperationTrigger}
          onAISettingsClick={handleAISettingsClick}
          onWorkspaceSettingsClick={handleWorkspaceSettingsClick}
          onUserPreferencesClick={handleUserPreferencesClick}
          aiStatus={aiStatus}
          activeOperations={
            activeOperation
              ? [
                  {
                    id: activeOperation.type,
                    type: activeOperation.type,
                    label: activeOperation.label,
                    progress: activeOperation.progress,
                    status: activeOperation.status,
                  },
                ]
              : []
          }
        />
      </AppShell.Header>

      {/* Main Content Area */}
      <AppShell.Main>
        {/* Navigation Panel */}
        <AppShell.Navigation>
          <ErrorBoundary>
            <NavigationPanel
              workspaceId="default"
              collapsed={navigationCollapsed}
              onItemSelect={handleNavigationSelect}
            />
          </ErrorBoundary>
        </AppShell.Navigation>

        {/* Center Content - Document Canvas, Dashboard, Analytics, Search, or Discovery */}
        <AppShell.Canvas>
          {(() => {
            console.log('Current viewMode:', viewMode)
            if (viewMode === 'search') {
              return (
                <ErrorBoundary>
                  <SearchInterface />
                </ErrorBoundary>
              )
            } else if (viewMode === 'dashboard') {
              return (
                <ErrorBoundary>
                  <WorkspaceIntelligence
                    workspaceId="default"
                    style={{
                      height: '100%',
                      padding: '24px',
                      overflowY: 'auto',
                    }}
                    onActionClick={(action, data) => {
                      console.log('Dashboard action:', action, data)
                      if (action === 'close' || action === 'view-document') {
                        setViewMode('document')
                      }
                    }}
                  />
                </ErrorBoundary>
              )
            } else if (viewMode === 'analytics') {
              return (
                <ErrorBoundary>
                  <AnalyticsDashboard
                    workspaceId="default"
                    style={{
                      height: '100%',
                    }}
                  />
                </ErrorBoundary>
              )
            } else if (viewMode === 'discovery') {
              console.log('Rendering ContentDiscovery component')
              return (
                <ErrorBoundary>
                  <ContentDiscovery workspaceId="default" />
                </ErrorBoundary>
              )
            } else if (viewMode === 'collections') {
              console.log('Rendering SmartCollections component')
              return (
                <ErrorBoundary>
                  <SmartCollections workspaceId="default" />
                </ErrorBoundary>
              )
            } else if (viewMode === 'loading-demo') {
              console.log('Rendering LoadingStatesDemo component')
              return (
                <ErrorBoundary>
                  <LoadingStatesDemo />
                </ErrorBoundary>
              )
            } else {
              return (
                <ErrorBoundary>
                  <DocumentCanvas
                    workspaceId="default"
                    documentId={currentDocumentId}
                    onModeChange={mode => {
                      if (mode === 'chat') {
                        setCurrentDocumentId(null)
                      }
                    }}
                  />
                </ErrorBoundary>
              )
            }
          })()}
        </AppShell.Canvas>

        {/* Intelligence Panel */}
        <AppShell.Intelligence>
          <ErrorBoundary>
            <IntelligencePanel />
          </ErrorBoundary>
        </AppShell.Intelligence>
      </AppShell.Main>

      {/* Floating Operation Progress Indicator */}
      {activeOperation && (
        <div
          style={{
            position: 'fixed',
            bottom: '24px',
            right: '24px',
            width: '320px',
            backgroundColor: '#16161a',
            border: '1px solid #3a3a3f',
            borderRadius: '8px',
            padding: '16px',
            boxShadow: '0 10px 40px rgba(0, 0, 0, 0.3)',
            zIndex: 10000,
            animation: 'slideInUp 0.3s ease-out',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '12px' }}>
            {activeOperation.status === 'running' && (
              <div
                style={{
                  width: '20px',
                  height: '20px',
                  border: '2px solid #00d4ff',
                  borderTopColor: 'transparent',
                  borderRadius: '50%',
                  animation: 'spin 1s linear infinite',
                }}
              />
            )}
            {activeOperation.status === 'completed' && (
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="#00ff88"
                strokeWidth="2"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            )}
            {activeOperation.status === 'error' && (
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="#ff5555"
                strokeWidth="2"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
            )}
            <div style={{ flex: 1 }}>
              <div style={{ color: '#ffffff', fontWeight: 600, fontSize: '14px' }}>
                {activeOperation.label}
              </div>
              <div style={{ color: '#a8a8a8', fontSize: '12px', marginTop: '4px' }}>
                {activeOperation.status === 'running' && `${activeOperation.progress}% complete`}
                {activeOperation.status === 'completed' && 'Completed successfully'}
                {activeOperation.status === 'error' && 'Operation failed'}
              </div>
            </div>
          </div>
          {activeOperation.status === 'running' && (
            <div
              style={{
                width: '100%',
                height: '4px',
                backgroundColor: '#2a2a2f',
                borderRadius: '2px',
                overflow: 'hidden',
              }}
            >
              <div
                style={{
                  width: `${activeOperation.progress}%`,
                  height: '100%',
                  backgroundColor: '#00d4ff',
                  transition: 'width 0.3s ease',
                }}
              />
            </div>
          )}
        </div>
      )}

      {/* Generation Modal */}
      {isGenerationModalOpen && (
        <GenerationModal
          isOpen={isGenerationModalOpen}
          onClose={() => setIsGenerationModalOpen(false)}
          onGenerationComplete={generation => {
            console.log('Generate with params:', generation)
            setIsGenerationModalOpen(false)
          }}
        />
      )}

      {/* Style Transfer Modal */}
      {isStyleTransferModalOpen && (
        <StyleTransfer
          isOpen={isStyleTransferModalOpen}
          onClose={() => setIsStyleTransferModalOpen(false)}
          documentId="default-doc"
          onTransferComplete={result => {
            console.log('Apply style:', result)
            setIsStyleTransferModalOpen(false)
          }}
        />
      )}

      {/* AI Settings Modal */}
      {isAISettingsModalOpen && (
        <AIProvidersModal
          isOpen={isAISettingsModalOpen}
          onClose={() => setIsAISettingsModalOpen(false)}
        />
      )}

      {/* Workspace Settings Modal */}
      {isWorkspaceSettingsModalOpen && (
        <WorkspaceSettingsModal
          isOpen={isWorkspaceSettingsModalOpen}
          onClose={() => setIsWorkspaceSettingsModalOpen(false)}
          workspacePath="default"
        />
      )}

      {/* User Preferences Modal */}
      {isUserPreferencesModalOpen && (
        <UserPreferencesModal
          isOpen={isUserPreferencesModalOpen}
          onClose={() => setIsUserPreferencesModalOpen(false)}
        />
      )}

      {/* Animations */}
      <style>
        {`
          @keyframes slideInUp {
            from {
              transform: translateY(100%);
              opacity: 0;
            }
            to {
              transform: translateY(0);
              opacity: 1;
            }
          }
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }
        `}
      </style>
    </>
  )
}

const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <CollaborationProvider>
        <AppShell>
          <AppContent />
        </AppShell>
      </CollaborationProvider>
    </ErrorBoundary>
  )
}

export default App
