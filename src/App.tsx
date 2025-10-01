import React, { useState, useCallback, useEffect, Component, ErrorInfo } from 'react'
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
import { documentService } from './services'
import { apiClient } from './api/client'

type ViewMode = 'document' | 'dashboard' | 'analytics' | 'search' | 'discovery' | 'collections'

// Simple Error Boundary
class ErrorBoundary extends Component<
  { children: React.ReactNode },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: '24px', color: '#ff5555' }}>
          <h2>Something went wrong</h2>
          <p>{this.state.error?.message}</p>
          <button onClick={() => this.setState({ hasError: false, error: null })}>Try again</button>
        </div>
      )
    }

    return this.props.children
  }
}

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

  // Initialize document indexer and AI system on app startup
  useEffect(() => {
    const initializeDocumentSystem = async () => {
      try {
        console.log('Initializing document indexer...')
        const result = await documentService.initializeIndexer()
        if (result.success) {
          console.log('Document indexer initialized successfully')
          // Get initial stats
          const stats = await documentService.getIndexStats()
          if (stats.success && stats.data) {
            console.log(
              `Document index ready: ${stats.data.total_documents} documents, ${stats.data.total_keywords} keywords`
            )
          }
        } else {
          console.warn('Document indexer initialization returned false:', result.error)
        }
      } catch (error) {
        console.error('Failed to initialize document indexer:', error)
      }
    }

    const initializeAISystem = async () => {
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
            await apiClient.invoke('init_ai_system', { config: settings })
            console.log('AI system initialized successfully')
          } else {
            console.log('No valid AI configuration found, skipping AI initialization')
          }
        }
      } catch (error) {
        console.error('Failed to initialize AI system:', error)
      }
    }

    initializeDocumentSystem()
    initializeAISystem()
  }, [])

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
          <NavigationPanel
            workspaceId="default"
            collapsed={navigationCollapsed}
            onItemSelect={handleNavigationSelect}
          />
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
              )
            } else if (viewMode === 'analytics') {
              return (
                <AnalyticsDashboard
                  workspaceId="default"
                  style={{
                    height: '100%',
                  }}
                />
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
            } else {
              return (
                <DocumentCanvas
                  workspaceId="default"
                  documentId={currentDocumentId}
                  onModeChange={mode => {
                    if (mode === 'chat') {
                      setCurrentDocumentId(null)
                    }
                  }}
                />
              )
            }
          })()}
        </AppShell.Canvas>

        {/* Intelligence Panel */}
        <AppShell.Intelligence>
          <IntelligencePanel />
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
    <AppShell>
      <AppContent />
    </AppShell>
  )
}

export default App
