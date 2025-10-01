import React, { useState, useCallback, Component, ErrorInfo } from 'react'
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
  const [activeOperation, setActiveOperation] = useState<{
    type: string
    label: string
    progress: number
    status: 'running' | 'completed' | 'error'
  } | null>(null)
  const [isGenerationModalOpen, setIsGenerationModalOpen] = useState(false)
  const [isStyleTransferModalOpen, setIsStyleTransferModalOpen] = useState(false)
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false)

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
        console.log('Switching to document view (default)')
        setViewMode('document')
      }
    },
    []
  )

  const handleLogoClick = useCallback(() => {
    setViewMode('document')
  }, [])

  const handleSettingsClick = useCallback(() => {
    console.log('Settings button clicked')
    setIsSettingsModalOpen(true)
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
          onSettingsClick={handleSettingsClick}
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
              return <DocumentCanvas workspaceId="default" />
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
      {isSettingsModalOpen && (
        <AIProvidersModal
          isOpen={isSettingsModalOpen}
          onClose={() => setIsSettingsModalOpen(false)}
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
