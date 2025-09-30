import React, { useState, useCallback, Component, ErrorInfo } from 'react'
import AppShell from './components/layout/AppShell'
import HeaderBar from './components/layout/HeaderBar'
import IntelligencePanel from './components/intelligence/IntelligencePanel'
import NavigationPanel from './components/navigation/NavigationPanel'
import DocumentCanvas from './components/canvas/DocumentCanvas'
import WorkspaceIntelligence from './components/workspace/WorkspaceIntelligence'
import { AnalyticsDashboard } from './components/analytics'
import SearchInterface from './components/search/SearchInterface'
import { useLayout } from './components/layout/useLayoutContext'

type ViewMode = 'document' | 'dashboard' | 'analytics' | 'search'

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
          <button onClick={() => this.setState({ hasError: false, error: null })}>
            Try again
          </button>
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
      console.log('Navigation item selected:', item.id)
      if (item.id === 'search') {
        console.log('Switching to search view')
        setViewMode('search')
      } else if (item.id === 'workspace-dashboard') {
        setViewMode('dashboard')
      } else if (item.id === 'analytics-dashboard') {
        setViewMode('analytics')
      } else {
        setViewMode('document')
      }
    },
    []
  )

  return (
    <>
      {/* Header */}
      <AppShell.Header>
        <HeaderBar />
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

        {/* Center Content - Document Canvas, Dashboard, Analytics, or Search */}
        <AppShell.Canvas>
          {viewMode === 'search' ? (
            <ErrorBoundary>
              <SearchInterface />
            </ErrorBoundary>
          ) : viewMode === 'dashboard' ? (
            <WorkspaceIntelligence
              workspaceId="default"
              style={{
                height: '100%',
                padding: '24px',
                overflowY: 'auto',
              }}
              onActionClick={(action, data) => {
                console.log('Dashboard action:', action, data)
                // Handle dashboard actions (e.g., navigate to specific document)
                if (action === 'close' || action === 'view-document') {
                  setViewMode('document')
                }
              }}
            />
          ) : viewMode === 'analytics' ? (
            <AnalyticsDashboard
              workspaceId="default"
              style={{
                height: '100%',
              }}
            />
          ) : (
            <DocumentCanvas workspaceId="default" />
          )}
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
