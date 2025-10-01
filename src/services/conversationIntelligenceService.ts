// Conversation Intelligence Service
import { apiClient } from '../api'
import {
  ConversationIntelligence,
  ConversationInsight,
  ConversationRecommendation,
  ConversationPerformance,
  ApiResponse,
} from '../types'

export class ConversationIntelligenceService {
  /**
   * Analyze conversation for insights
   */
  async analyzeConversation(
    conversationId: string
  ): Promise<ApiResponse<ConversationIntelligence>> {
    return apiClient.invoke('analyze_conversation_intelligence', {
      conversation_id: conversationId,
    })
  }

  /**
   * Get conversation insights
   */
  async getConversationInsights(
    conversationId: string
  ): Promise<ApiResponse<ConversationInsight[]>> {
    return apiClient.invoke('get_conversation_insights', { conversation_id: conversationId })
  }

  /**
   * Generate conversation recommendations
   */
  async getConversationRecommendations(
    conversationId: string
  ): Promise<ApiResponse<ConversationRecommendation[]>> {
    return apiClient.invoke('get_conversation_recommendations', { conversation_id: conversationId })
  }

  /**
   * Analyze conversation sentiment
   */
  async analyzeConversationSentiment(conversationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_conversation_sentiment', { conversation_id: conversationId })
  }

  /**
   * Extract conversation topics
   */
  async extractConversationTopics(conversationId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_conversation_topics', { conversation_id: conversationId })
  }

  /**
   * Detect conversation patterns
   */
  async detectConversationPatterns(conversationIds: string[]): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('detect_conversation_patterns', { conversation_ids: conversationIds })
  }

  /**
   * Measure conversation performance
   */
  async measureConversationPerformance(
    conversationId: string
  ): Promise<ApiResponse<ConversationPerformance>> {
    return apiClient.invoke('measure_conversation_performance', { conversation_id: conversationId })
  }

  /**
   * Summarize conversation
   */
  async summarizeConversation(
    conversationId: string,
    options?: unknown
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('summarize_conversation', {
      conversation_id: conversationId,
      options: options || {},
    })
  }

  /**
   * Extract action items from conversation
   */
  async extractActionItems(conversationId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_conversation_action_items', {
      conversation_id: conversationId,
    })
  }

  /**
   * Analyze conversation flow
   */
  async analyzeConversationFlow(conversationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_conversation_flow', { conversation_id: conversationId })
  }

  /**
   * Detect conversation anomalies
   */
  async detectConversationAnomalies(conversationId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('detect_conversation_anomalies', { conversation_id: conversationId })
  }

  /**
   * Compare conversations
   */
  async compareConversations(
    conversationAId: string,
    conversationBId: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('compare_conversations', {
      conversation_a_id: conversationAId,
      conversation_b_id: conversationBId,
    })
  }

  /**
   * Classify conversation intent
   */
  async classifyConversationIntent(conversationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('classify_conversation_intent', { conversation_id: conversationId })
  }

  /**
   * Analyze participant engagement
   */
  async analyzeParticipantEngagement(conversationId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('analyze_participant_engagement', { conversation_id: conversationId })
  }

  /**
   * Generate conversation quality score
   */
  async generateQualityScore(conversationId: string): Promise<ApiResponse<number>> {
    return apiClient.invoke('generate_conversation_quality_score', {
      conversation_id: conversationId,
    })
  }

  /**
   * Predict conversation outcome
   */
  async predictConversationOutcome(conversationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('predict_conversation_outcome', { conversation_id: conversationId })
  }

  /**
   * Get conversation analytics
   */
  async getConversationAnalytics(
    workspaceId?: string,
    timeframe?: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('get_conversational_intelligence_status', {
      workspace_id: workspaceId,
      timeframe: timeframe || 'last_30_days',
    })
  }

  /**
   * Track conversation metrics
   */
  async trackConversationMetrics(conversationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('track_conversation_metrics', { conversation_id: conversationId })
  }

  /**
   * Generate conversation report
   */
  async generateConversationReport(
    conversationId: string,
    reportType: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_conversation_report', {
      conversation_id: conversationId,
      report_type: reportType,
    })
  }

  /**
   * Optimize conversation flow
   */
  async optimizeConversationFlow(conversationId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('optimize_conversation_flow', { conversation_id: conversationId })
  }

  /**
   * Analyze conversation effectiveness
   */
  async analyzeConversationEffectiveness(conversationIds: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_conversation_effectiveness', {
      conversation_ids: conversationIds,
    })
  }
}

export const conversationIntelligenceService = new ConversationIntelligenceService()
