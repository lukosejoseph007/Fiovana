import React, { useState, useCallback } from 'react'
import AppShell from './components/layout/AppShell'
import HeaderBar from './components/layout/HeaderBar'
import IntelligencePanel from './components/intelligence/IntelligencePanel'
import NavigationPanel from './components/navigation/NavigationPanel'
import DocumentCanvas from './components/canvas/DocumentCanvas'
import WorkspaceIntelligence from './components/workspace/WorkspaceIntelligence'
import { AnalyticsDashboard } from './components/analytics'
import { useLayout } from './components/layout/useLayoutContext'

type ViewMode = 'document' | 'dashboard' | 'analytics'

// Component that uses layout context
const AppContent: React.FC = () => {
  const { navigationCollapsed } = useLayout()
  const [viewMode, setViewMode] = useState<ViewMode>('document')

  // Handle navigation item selection
  const handleNavigationSelect = useCallback(
    (item: { id: string; label: string; icon: string }) => {
      if (item.id === 'workspace-dashboard') {
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

        {/* Center Content - Document Canvas or Dashboard */}
        <AppShell.Canvas>
          {viewMode === 'document' ? (
            <DocumentCanvas workspaceId="default" />
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
          ) : (
            <AnalyticsDashboard
              workspaceId="default"
              style={{
                height: '100%',
              }}
            />
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
