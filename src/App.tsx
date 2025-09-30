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
import { useLayout } from './components/layout/useLayoutContext'

type ViewMode = 'document' | 'dashboard' | 'analytics' | 'search' | 'discovery'

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

  return (
    <>
      {/* Header */}
      <AppShell.Header>
        <HeaderBar onLogoClick={handleLogoClick} />
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
