import React from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import Layout from './components/layout/Layout'
import Dashboard from './pages/Dashboard'
import FileManagement from './pages/FileManagement'
import FileWatcher from './pages/FileWatcher'
import Deduplication from './pages/Deduplication'
import Settings from './pages/Settings'

const App: React.FC = () => {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="/file-management" element={<FileManagement />} />
          <Route path="/file-watcher" element={<FileWatcher />} />
          <Route path="/deduplication" element={<Deduplication />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </Router>
  )
}

export default App
