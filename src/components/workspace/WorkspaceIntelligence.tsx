import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  AlertTriangle,
  BarChart3,
  Brain,
  CheckCircle,
  Clock,
  Lightbulb,
  RefreshCw,
  TrendingUp,
  Users,
  Zap,
  AlertCircle,
  Target,
  Gauge,
} from 'lucide-react'

// Data structures matching the Rust backend
interface WorkspaceInsights {
  health_score: number
  organization_quality: number
  content_freshness: number
  knowledge_gaps_count: number
  recommendations_count: number
  key_insights: string[]
  action_suggestions: string[]
}

interface WorkspaceInsightsResponse {
  success: boolean
  insights?: WorkspaceInsights
  error?: string
}

interface WorkspaceRecommendation {
  title: string
  description: string
  priority: 'Urgent' | 'High' | 'Medium' | 'Low'
  estimated_effort: 'Low' | 'Medium' | 'High'
  expected_impact: 'Low' | 'Medium' | 'High'
}

interface WorkspaceRecommendationsResponse {
  success: boolean
  recommendations: WorkspaceRecommendation[]
  error?: string
}

interface WorkspaceHealthResponse {
  success: boolean
  health_score?: number
  error?: string
}

interface WorkspaceIntelligenceProps {
  workspacePath?: string
  sessionId?: string
  className?: string
}

const WorkspaceIntelligence: React.FC<WorkspaceIntelligenceProps> = ({
  workspacePath,
  sessionId = 'default-session',
  className = '',
}) => {
  // State management
  const [insights, setInsights] = useState<WorkspaceInsights | null>(null)
  const [recommendations, setRecommendations] = useState<WorkspaceRecommendation[]>([])
  const [healthScore, setHealthScore] = useState<number | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date())
  const [refreshing, setRefreshing] = useState(false)

  // Load workspace intelligence data
  useEffect(() => {
    if (workspacePath) {
      loadWorkspaceIntelligence()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [workspacePath])

  const loadWorkspaceIntelligence = async () => {
    if (!workspacePath) return

    try {
      setLoading(true)
      setError(null)

      // Load insights from workspace-AI integration (Task 8.5)
      const insightsResponse = await invoke<WorkspaceInsightsResponse>(
        'get_workspace_insights_for_ai',
        { sessionId }
      )

      if (insightsResponse.success && insightsResponse.insights) {
        setInsights(insightsResponse.insights)
        setHealthScore(insightsResponse.insights.health_score)
      } else {
        // Fallback to traditional workspace intelligence commands
        await loadFallbackIntelligence()
      }

      // Load recommendations
      await loadRecommendations()
      setLastRefresh(new Date())
    } catch (err) {
      console.error('Failed to load workspace intelligence:', err)
      setError(err as string)
      // Try fallback approach
      await loadFallbackIntelligence()
    } finally {
      setLoading(false)
    }
  }

  const loadFallbackIntelligence = async () => {
    if (!workspacePath) return

    try {
      // Try traditional workspace health score
      const healthResponse = await invoke<WorkspaceHealthResponse>('get_workspace_health_score', {
        workspacePath,
      })

      if (healthResponse.success && healthResponse.health_score !== undefined) {
        setHealthScore(healthResponse.health_score)

        // Create synthetic insights for display
        setInsights({
          health_score: healthResponse.health_score,
          organization_quality: healthResponse.health_score * 0.8,
          content_freshness: healthResponse.health_score * 0.9,
          knowledge_gaps_count: Math.max(0, Math.floor((100 - healthResponse.health_score) / 20)),
          recommendations_count: Math.max(1, Math.floor((100 - healthResponse.health_score) / 15)),
          key_insights: [
            `Workspace health score: ${healthResponse.health_score.toFixed(1)}/100`,
            'Analysis based on current workspace structure and content',
          ],
          action_suggestions: [
            'Enable workspace-aware conversations for detailed insights',
            'Analyze documents for comprehensive recommendations',
          ],
        })
      }
    } catch (err) {
      console.warn('Fallback intelligence loading failed:', err)
    }
  }

  const loadRecommendations = async () => {
    if (!workspacePath) return

    try {
      const recResponse = await invoke<WorkspaceRecommendationsResponse>(
        'get_workspace_recommendations',
        { workspacePath }
      )

      if (recResponse.success) {
        setRecommendations(recResponse.recommendations)
      }
    } catch (err) {
      console.warn('Failed to load recommendations:', err)
    }
  }

  const handleRefresh = async () => {
    setRefreshing(true)
    await loadWorkspaceIntelligence()
    setRefreshing(false)
  }

  const enableWorkspaceAI = async () => {
    if (!workspacePath) return

    try {
      setRefreshing(true)
      await invoke('enable_workspace_ai_integration', {
        request: {
          sessionId,
          workspacePath,
        },
      })

      // Reload after enabling AI integration
      await loadWorkspaceIntelligence()
    } catch (err) {
      setError(err as string)
    } finally {
      setRefreshing(false)
    }
  }

  const getHealthColor = (score: number) => {
    if (score >= 80) return 'text-green-600'
    if (score >= 60) return 'text-yellow-600'
    return 'text-red-600'
  }

  const getHealthBgColor = (score: number) => {
    if (score >= 80) return 'bg-green-100'
    if (score >= 60) return 'bg-yellow-100'
    return 'bg-red-100'
  }

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'Urgent':
        return 'bg-red-100 text-red-800'
      case 'High':
        return 'bg-orange-100 text-orange-800'
      case 'Medium':
        return 'bg-yellow-100 text-yellow-800'
      case 'Low':
        return 'bg-green-100 text-green-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  const getPriorityIcon = (priority: string) => {
    switch (priority) {
      case 'Urgent':
        return <AlertTriangle className="h-4 w-4" />
      case 'High':
        return <AlertCircle className="h-4 w-4" />
      case 'Medium':
        return <Target className="h-4 w-4" />
      case 'Low':
        return <CheckCircle className="h-4 w-4" />
      default:
        return <Target className="h-4 w-4" />
    }
  }

  if (!workspacePath) {
    return (
      <div className={`bg-white rounded-lg shadow p-6 ${className}`}>
        <div className="text-center py-8 text-gray-500">
          <Brain className="h-12 w-12 mx-auto mb-4 text-gray-300" />
          <p>Select a workspace to view intelligence insights</p>
        </div>
      </div>
    )
  }

  if (loading) {
    return (
      <div className={`bg-white rounded-lg shadow p-6 ${className}`}>
        <div className="animate-pulse space-y-4">
          <div className="h-4 bg-gray-200 rounded w-3/4"></div>
          <div className="space-y-2">
            <div className="h-3 bg-gray-200 rounded"></div>
            <div className="h-3 bg-gray-200 rounded w-5/6"></div>
          </div>
        </div>
      </div>
    )
  }

  if (error && !insights) {
    return (
      <div className={`bg-white rounded-lg shadow p-6 ${className}`}>
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <AlertTriangle className="h-5 w-5 text-red-600" />
            <p className="text-red-600">Failed to load workspace intelligence</p>
          </div>
          <p className="text-red-500 text-sm mt-1">{error}</p>
          <button
            onClick={handleRefresh}
            className="mt-2 text-red-700 hover:text-red-900 underline text-sm"
          >
            Try again
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className={`bg-white rounded-lg shadow ${className}`}>
      {/* Header */}
      <div className="p-6 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <Brain className="h-6 w-6 text-blue-600" />
            <div>
              <h2 className="text-xl font-semibold text-gray-900">Workspace Intelligence</h2>
              <p className="text-sm text-gray-500">
                Last updated: {lastRefresh.toLocaleTimeString()}
              </p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={enableWorkspaceAI}
              className="inline-flex items-center px-3 py-1.5 border border-blue-300 text-sm font-medium rounded-md text-blue-700 bg-blue-50 hover:bg-blue-100 transition-colors"
              disabled={refreshing}
            >
              <Zap className="h-4 w-4 mr-1" />
              Enable AI
            </button>
            <button
              onClick={handleRefresh}
              className="inline-flex items-center px-3 py-1.5 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 transition-colors"
              disabled={refreshing}
            >
              <RefreshCw className={`h-4 w-4 mr-1 ${refreshing ? 'animate-spin' : ''}`} />
              Refresh
            </button>
          </div>
        </div>
      </div>

      <div className="p-6 space-y-6">
        {/* Health Score Section */}
        {healthScore !== null && (
          <div className="bg-gray-50 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-lg font-medium text-gray-900 flex items-center">
                <Gauge className="h-5 w-5 mr-2 text-blue-600" />
                Workspace Health
              </h3>
              <div
                className={`px-3 py-1 rounded-full text-lg font-bold ${getHealthBgColor(healthScore)} ${getHealthColor(healthScore)}`}
              >
                {healthScore.toFixed(1)}/100
              </div>
            </div>

            <div className="w-full bg-gray-200 rounded-full h-3">
              <div
                className={`h-3 rounded-full transition-all duration-500 ${
                  healthScore >= 80
                    ? 'bg-green-500'
                    : healthScore >= 60
                      ? 'bg-yellow-500'
                      : 'bg-red-500'
                }`}
                style={{ width: `${healthScore}%` }}
              ></div>
            </div>
          </div>
        )}

        {/* Metrics Grid */}
        {insights && (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-blue-50 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-blue-600 text-sm font-medium">Organization Quality</p>
                  <p className="text-2xl font-bold text-blue-700">
                    {insights.organization_quality.toFixed(1)}
                  </p>
                </div>
                <BarChart3 className="h-8 w-8 text-blue-600" />
              </div>
            </div>

            <div className="bg-green-50 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-600 text-sm font-medium">Content Freshness</p>
                  <p className="text-2xl font-bold text-green-700">
                    {insights.content_freshness.toFixed(1)}
                  </p>
                </div>
                <Clock className="h-8 w-8 text-green-600" />
              </div>
            </div>

            <div className="bg-purple-50 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-purple-600 text-sm font-medium">Knowledge Gaps</p>
                  <p className="text-2xl font-bold text-purple-700">
                    {insights.knowledge_gaps_count}
                  </p>
                </div>
                <AlertCircle className="h-8 w-8 text-purple-600" />
              </div>
            </div>
          </div>
        )}

        {/* Key Insights */}
        {insights && insights.key_insights.length > 0 && (
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-3 flex items-center">
              <Lightbulb className="h-5 w-5 mr-2 text-yellow-600" />
              Key Insights
            </h3>
            <div className="space-y-2">
              {insights.key_insights.map((insight, index) => (
                <div key={index} className="bg-yellow-50 border-l-4 border-yellow-400 p-3">
                  <p className="text-yellow-800">{insight}</p>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Action Suggestions */}
        {insights && insights.action_suggestions.length > 0 && (
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-3 flex items-center">
              <TrendingUp className="h-5 w-5 mr-2 text-green-600" />
              Action Suggestions
            </h3>
            <div className="space-y-2">
              {insights.action_suggestions.map((suggestion, index) => (
                <div key={index} className="bg-green-50 border-l-4 border-green-400 p-3">
                  <p className="text-green-800">{suggestion}</p>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Recommendations */}
        {recommendations.length > 0 && (
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-3 flex items-center">
              <Users className="h-5 w-5 mr-2 text-indigo-600" />
              Recommendations ({recommendations.length})
            </h3>
            <div className="space-y-3">
              {recommendations.slice(0, 5).map((rec, index) => (
                <div key={index} className="border border-gray-200 rounded-lg p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-2">
                        {getPriorityIcon(rec.priority)}
                        <h4 className="font-medium text-gray-900">{rec.title}</h4>
                        <span
                          className={`px-2 py-1 text-xs font-medium rounded-full ${getPriorityColor(rec.priority)}`}
                        >
                          {rec.priority}
                        </span>
                      </div>
                      <p className="text-gray-600 text-sm mb-2">{rec.description}</p>
                      <div className="flex items-center space-x-4 text-xs text-gray-500">
                        <span>Effort: {rec.estimated_effort}</span>
                        <span>Impact: {rec.expected_impact}</span>
                      </div>
                    </div>
                  </div>
                </div>
              ))}

              {recommendations.length > 5 && (
                <div className="text-center py-2">
                  <button className="text-blue-600 hover:text-blue-700 text-sm">
                    View all {recommendations.length} recommendations
                  </button>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Empty State */}
        {!insights && recommendations.length === 0 && (
          <div className="text-center py-8 text-gray-500">
            <Brain className="h-12 w-12 mx-auto mb-4 text-gray-300" />
            <p className="mb-2">No workspace intelligence data available</p>
            <p className="text-sm">
              Enable AI integration to get detailed insights and recommendations
            </p>
            <button
              onClick={enableWorkspaceAI}
              className="mt-3 inline-flex items-center px-4 py-2 border border-blue-300 text-sm font-medium rounded-md text-blue-700 bg-blue-50 hover:bg-blue-100 transition-colors"
            >
              <Zap className="h-4 w-4 mr-2" />
              Enable Workspace AI
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export default WorkspaceIntelligence
