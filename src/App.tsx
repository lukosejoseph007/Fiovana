import React from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { AppStateProvider } from './context/AppStateContext'
import Layout from './components/layout/Layout'
import Dashboard from './pages/Dashboard'
import Chat from './pages/Chat'
import FileManagement from './pages/FileManagement'
import ImportWizard from './pages/ImportWizard'
import FileWatcher from './pages/FileWatcher'
import Deduplication from './pages/Deduplication'
import ProgressDashboard from './pages/ProgressDashboard'
import Workspace from './pages/Workspace'
import Settings from './pages/Settings'
import Search from './pages/Search'
import DocumentComparison from './pages/DocumentComparison'
import DocumentIndex from './pages/DocumentIndex'

const App: React.FC = () => {
  return (
    <AppStateProvider>
      <Router>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Dashboard />} />
            <Route path="/chat" element={<Chat />} />
            <Route path="/file-management" element={<FileManagement />} />
            <Route path="/import-wizard" element={<ImportWizard />} />
            <Route path="/file-watcher" element={<FileWatcher />} />
            <Route path="/deduplication" element={<Deduplication />} />
            <Route path="/progress" element={<ProgressDashboard />} />
            <Route path="/workspace" element={<Workspace />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/search" element={<Search />} />
            <Route path="/comparison" element={<DocumentComparison />} />
            <Route path="/index" element={<DocumentIndex />} />
          </Route>
        </Routes>
      </Router>
    </AppStateProvider>
  )
}

export default App
