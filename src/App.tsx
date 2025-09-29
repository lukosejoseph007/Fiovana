import React from 'react'
import AppShell from './components/layout/AppShell'
import HeaderBar from './components/layout/HeaderBar'
import IntelligencePanel from './components/intelligence/IntelligencePanel'
import NavigationPanel from './components/navigation/NavigationPanel'
import DocumentCanvas from './components/canvas/DocumentCanvas'
import { useLayout } from './components/layout/useLayoutContext'

// Component that uses layout context
const AppContent: React.FC = () => {
  const { navigationCollapsed } = useLayout()

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
          <NavigationPanel workspaceId="default" collapsed={navigationCollapsed} />
        </AppShell.Navigation>

        {/* Document Canvas */}
        <AppShell.Canvas>
          <DocumentCanvas workspaceId="default" />
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
