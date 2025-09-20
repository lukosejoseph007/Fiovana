import React from 'react'
import { Link } from 'react-router-dom'
import {
  Upload,
  Eye,
  Copy,
  FileText,
  Activity,
  Clock,
  CheckCircle,
  AlertCircle,
} from 'lucide-react'

const Dashboard: React.FC = () => {
  const quickActions = [
    {
      title: 'Upload Documents',
      description: 'Drag and drop files to analyze and process',
      icon: Upload,
      path: '/file-management',
      color: 'blue',
    },
    {
      title: 'Watch Files',
      description: 'Monitor file system changes in real-time',
      icon: Eye,
      path: '/file-watcher',
      color: 'green',
    },
    {
      title: 'Find Duplicates',
      description: 'Detect and manage duplicate files',
      icon: Copy,
      path: '/deduplication',
      color: 'purple',
    },
  ]

  const recentActivity = [
    {
      id: 1,
      action: 'Document uploaded',
      file: 'user-guide.pdf',
      timestamp: '2 minutes ago',
      status: 'completed',
      icon: FileText,
    },
    {
      id: 2,
      action: 'Duplicate detection',
      file: 'training-manual.docx',
      timestamp: '15 minutes ago',
      status: 'completed',
      icon: Copy,
    },
    {
      id: 3,
      action: 'File watcher started',
      file: '/workspace/documents',
      timestamp: '1 hour ago',
      status: 'active',
      icon: Eye,
    },
    {
      id: 4,
      action: 'Processing document',
      file: 'policy-update.pdf',
      timestamp: '2 hours ago',
      status: 'processing',
      icon: Activity,
    },
  ]

  const stats = [
    {
      label: 'Documents Processed',
      value: '1,247',
      change: '+12%',
      changeType: 'positive' as const,
      icon: FileText,
    },
    {
      label: 'Duplicates Found',
      value: '89',
      change: '-5%',
      changeType: 'positive' as const,
      icon: Copy,
    },
    {
      label: 'Active Watchers',
      value: '3',
      change: '+1',
      changeType: 'positive' as const,
      icon: Eye,
    },
    {
      label: 'Storage Saved',
      value: '2.4 GB',
      change: '+0.3 GB',
      changeType: 'positive' as const,
      icon: Activity,
    },
  ]

  const getColorClasses = (color: string) => {
    switch (color) {
      case 'blue':
        return 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800 hover:bg-blue-100 dark:hover:bg-blue-900/30'
      case 'green':
        return 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 border-green-200 dark:border-green-800 hover:bg-green-100 dark:hover:bg-green-900/30'
      case 'purple':
        return 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400 border-purple-200 dark:border-purple-800 hover:bg-purple-100 dark:hover:bg-purple-900/30'
      default:
        return 'bg-gray-50 dark:bg-gray-800 text-gray-600 dark:text-gray-400 border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-700'
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'processing':
        return <Clock className="h-4 w-4 text-yellow-500" />
      case 'active':
        return <Activity className="h-4 w-4 text-blue-500" />
      default:
        return <AlertCircle className="h-4 w-4 text-gray-500" />
    }
  }

  return (
    <div className="space-y-6">
      {/* Welcome Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
              Welcome to Proxemic
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              AI-powered document intelligence platform for secure content processing
            </p>
          </div>
          <div className="hidden sm:block">
            <div className="w-16 h-16 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
              <FileText className="h-8 w-8 text-white" />
            </div>
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat, index) => {
          const Icon = stat.icon
          return (
            <div
              key={index}
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6"
            >
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                    {stat.label}
                  </p>
                  <p className="text-2xl font-bold text-gray-900 dark:text-white mt-2">
                    {stat.value}
                  </p>
                  <p
                    className={`text-sm mt-2 ${
                      stat.changeType === 'positive'
                        ? 'text-green-600 dark:text-green-400'
                        : 'text-red-600 dark:text-red-400'
                    }`}
                  >
                    {stat.change}
                  </p>
                </div>
                <div className="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                  <Icon className="h-6 w-6 text-gray-600 dark:text-gray-400" />
                </div>
              </div>
            </div>
          )
        })}
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {quickActions.map((action, index) => {
          const Icon = action.icon
          return (
            <Link
              key={index}
              to={action.path}
              className={`block p-6 rounded-lg border transition-colors ${getColorClasses(action.color)}`}
            >
              <div className="flex items-center space-x-4">
                <div className="p-3 bg-white dark:bg-gray-800 rounded-lg shadow-sm">
                  <Icon className="h-6 w-6" />
                </div>
                <div className="flex-1">
                  <h3 className="font-semibold">{action.title}</h3>
                  <p className="text-sm opacity-80 mt-1">{action.description}</p>
                </div>
              </div>
            </Link>
          )
        })}
      </div>

      {/* Recent Activity */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Recent Activity</h2>
        </div>
        <div className="p-6 space-y-4">
          {recentActivity.map(activity => {
            const Icon = activity.icon
            return (
              <div key={activity.id} className="flex items-center space-x-4">
                <div className="p-2 bg-gray-50 dark:bg-gray-700 rounded-lg">
                  <Icon className="h-4 w-4 text-gray-600 dark:text-gray-400" />
                </div>
                <div className="flex-1">
                  <div className="flex items-center space-x-2">
                    <span className="font-medium text-gray-900 dark:text-white">
                      {activity.action}
                    </span>
                    {getStatusIcon(activity.status)}
                  </div>
                  <div className="flex items-center space-x-2 mt-1">
                    <span className="text-sm text-gray-600 dark:text-gray-400">
                      {activity.file}
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-500">
                      • {activity.timestamp}
                    </span>
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

export default Dashboard
