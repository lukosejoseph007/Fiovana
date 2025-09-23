import React, { useState } from 'react'
import { Outlet, NavLink, useLocation } from 'react-router-dom'
import {
  FileText,
  Eye,
  Copy,
  Settings,
  Menu,
  X,
  Home,
  Upload,
  BarChart3,
  FolderOpen,
  FileUp,
  MessageSquare,
  Search,
  GitCompare,
  Database,
  ChevronLeft,
  ChevronRight,
  Sparkles,
} from 'lucide-react'

const Layout: React.FC = () => {
  const [isSidebarOpen, setIsSidebarOpen] = useState(true)
  const [isCollapsed, setIsCollapsed] = useState(false)
  const location = useLocation()

  // Organize navigation items into categories
  const navigationCategories = [
    {
      label: 'Core',
      items: [
        {
          path: '/',
          label: 'Dashboard',
          icon: Home,
          description: 'Overview and quick actions',
        },
        {
          path: '/chat',
          label: 'AI Assistant',
          icon: MessageSquare,
          description: 'Conversational document processing',
          badge: 'AI',
        },
      ],
    },
    {
      label: 'Intelligence',
      items: [
        {
          path: '/search',
          label: 'Intelligent Search',
          icon: Search,
          description: 'AI-powered semantic document search',
          badge: 'NEW',
        },
        {
          path: '/comparison',
          label: 'Document Comparison',
          icon: GitCompare,
          description: 'Compare documents with AI insights',
          badge: 'NEW',
        },
        {
          path: '/index',
          label: 'Document Index',
          icon: Database,
          description: 'Browse indexed documents and chunks',
          badge: 'NEW',
        },
      ],
    },
    {
      label: 'File Management',
      items: [
        {
          path: '/file-management',
          label: 'File Management',
          icon: Upload,
          description: 'Upload and manage documents',
        },
        {
          path: '/import-wizard',
          label: 'Import Wizard',
          icon: FileUp,
          description: 'Guided file import with presets',
        },
        {
          path: '/file-watcher',
          label: 'File Watcher',
          icon: Eye,
          description: 'Monitor file system changes',
        },
        {
          path: '/deduplication',
          label: 'Deduplication',
          icon: Copy,
          description: 'Find and manage duplicate files',
        },
      ],
    },
    {
      label: 'Project',
      items: [
        {
          path: '/workspace',
          label: 'Workspace',
          icon: FolderOpen,
          description: 'Manage project workspaces',
        },
        {
          path: '/progress',
          label: 'Progress',
          icon: BarChart3,
          description: 'Monitor import operations',
        },
      ],
    },
    {
      label: 'System',
      items: [
        {
          path: '/settings',
          label: 'Settings',
          icon: Settings,
          description: 'Application configuration',
        },
      ],
    },
  ]

  const toggleSidebar = () => {
    setIsSidebarOpen(!isSidebarOpen)
  }

  const toggleCollapse = () => {
    setIsCollapsed(!isCollapsed)
  }

  const closeSidebar = () => {
    setIsSidebarOpen(false)
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Mobile Menu Button */}
      <div className="lg:hidden fixed top-4 left-4 z-50">
        <button
          onClick={toggleSidebar}
          className="p-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700"
        >
          {isSidebarOpen ? (
            <X className="h-6 w-6 text-gray-600 dark:text-gray-300" />
          ) : (
            <Menu className="h-6 w-6 text-gray-600 dark:text-gray-300" />
          )}
        </button>
      </div>

      {/* Sidebar Overlay for Mobile */}
      {isSidebarOpen && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 z-30 lg:hidden"
          onClick={closeSidebar}
        />
      )}

      {/* Sidebar */}
      <aside
        className={`
        fixed top-0 left-0 h-full bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 shadow-lg z-40 transition-all duration-300 ease-in-out flex flex-col
        ${isCollapsed ? 'w-16' : 'w-80'}
        ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
      `}
      >
        {/* Header */}
        <div
          className={`${isCollapsed ? 'p-3' : 'p-6'} border-b border-gray-200 dark:border-gray-700 flex-shrink-0`}
        >
          <div className="flex items-center justify-between">
            <div className={`flex items-center ${isCollapsed ? 'justify-center' : 'space-x-3'}`}>
              <div className="w-10 h-10 bg-gradient-to-r from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                <FileText className="h-6 w-6 text-white" />
              </div>
              {!isCollapsed && (
                <div>
                  <h1 className="text-xl font-bold text-gray-900 dark:text-white">Proxemic</h1>
                  <p className="text-sm text-gray-500 dark:text-gray-400">Document Intelligence</p>
                </div>
              )}
            </div>
            {/* Desktop Collapse Toggle */}
            <button
              onClick={toggleCollapse}
              className="hidden lg:block p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300 transition-colors"
            >
              {isCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
            </button>
          </div>
        </div>

        {/* Navigation - Scrollable */}
        <nav className="flex-1 overflow-y-auto overflow-x-hidden p-3 space-y-1 scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600">
          {navigationCategories.map((category, categoryIndex) => (
            <div key={category.label} className={categoryIndex > 0 ? 'mt-6' : ''}>
              {/* Category Label */}
              {!isCollapsed && (
                <div className="px-3 py-2 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  {category.label}
                </div>
              )}

              {/* Category Items */}
              <div className="space-y-1">
                {category.items.map(item => {
                  const Icon = item.icon
                  const isActive = location.pathname === item.path

                  return (
                    <NavLink
                      key={item.path}
                      to={item.path}
                      onClick={closeSidebar}
                      className={`
                        group flex items-center rounded-lg transition-all duration-200 relative
                        ${isCollapsed ? 'px-3 py-3 justify-center' : 'px-3 py-2.5'}
                        ${
                          isActive
                            ? 'bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 text-blue-600 dark:text-blue-400 shadow-sm border border-blue-200 dark:border-blue-800'
                            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700/50'
                        }
                      `}
                    >
                      <Icon
                        className={`${isCollapsed ? 'h-5 w-5' : 'h-5 w-5 mr-3'} flex-shrink-0 ${
                          isActive
                            ? 'text-blue-600 dark:text-blue-400'
                            : 'text-gray-500 dark:text-gray-400'
                        }`}
                      />

                      {!isCollapsed && (
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center justify-between">
                            <div className="font-medium text-sm truncate">{item.label}</div>
                            {item.badge && (
                              <span
                                className={`ml-2 px-1.5 py-0.5 text-xs font-medium rounded-full flex-shrink-0 ${
                                  item.badge === 'AI'
                                    ? 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
                                    : 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                                }`}
                              >
                                {item.badge === 'AI' ? (
                                  <div className="flex items-center gap-1">
                                    <Sparkles size={10} />
                                    {item.badge}
                                  </div>
                                ) : (
                                  item.badge
                                )}
                              </span>
                            )}
                          </div>
                          <div className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">
                            {item.description}
                          </div>
                        </div>
                      )}

                      {/* Active Indicator */}
                      {isActive && (
                        <div
                          className={`${isCollapsed ? 'absolute right-0 top-1/2 transform -translate-y-1/2 w-1 h-6' : 'w-2 h-2 rounded-full ml-2'} bg-blue-600 dark:bg-blue-400`}
                        />
                      )}

                      {/* Tooltip for Collapsed State */}
                      {isCollapsed && (
                        <div className="absolute left-full top-1/2 transform -translate-y-1/2 ml-2 px-3 py-2 bg-gray-900 dark:bg-gray-700 text-white text-sm rounded-lg opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none whitespace-nowrap z-50">
                          <div className="font-medium">{item.label}</div>
                          <div className="text-xs text-gray-300 dark:text-gray-400">
                            {item.description}
                          </div>
                          {/* Arrow */}
                          <div className="absolute left-0 top-1/2 transform -translate-y-1/2 -translate-x-1 w-2 h-2 bg-gray-900 dark:bg-gray-700 rotate-45"></div>
                        </div>
                      )}
                    </NavLink>
                  )
                })}
              </div>
            </div>
          ))}
        </nav>

        {/* Footer */}
        <div
          className={`border-t border-gray-200 dark:border-gray-700 ${isCollapsed ? 'p-3' : 'p-4'} flex-shrink-0`}
        >
          <div
            className={`text-xs text-gray-500 dark:text-gray-400 ${isCollapsed ? 'text-center' : ''}`}
          >
            {isCollapsed ? 'v0.1.0' : 'Proxemic v0.1.0'}
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main
        className={`min-h-screen transition-all duration-300 ease-in-out ${
          isCollapsed ? 'lg:ml-16' : 'lg:ml-80'
        }`}
      >
        {/* Top Bar */}
        <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4 lg:ml-0 ml-12">
              <div>
                <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                  {navigationCategories
                    .flatMap(cat => cat.items)
                    .find(item => item.path === location.pathname)?.label || 'Proxemic'}
                </h2>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {navigationCategories
                    .flatMap(cat => cat.items)
                    .find(item => item.path === location.pathname)?.description ||
                    'AI-Powered Document Intelligence Platform'}
                </p>
              </div>
            </div>

            {/* User Actions */}
            <div className="flex items-center space-x-4">
              <button className="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 transition-colors">
                <Settings className="h-5 w-5" />
              </button>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <div className="p-6">
          <Outlet />
        </div>
      </main>
    </div>
  )
}

export default Layout
