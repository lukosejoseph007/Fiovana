// AI Integration Service
import { apiClient } from '../api'
import { ApiResponse } from '../types'

export interface AIModel {
  id: string
  name: string
  provider: string
  capabilities: string[]
  maxTokens: number
  costPerToken?: number
  status: 'active' | 'inactive' | 'deprecated'
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
  metadata?: Record<string, any>
}

export interface ChatRequest {
  messages: ChatMessage[]
  model?: string
  maxTokens?: number
  temperature?: number
  options?: Record<string, any>
}

export interface ChatResponse {
  message: ChatMessage
  usage: TokenUsage
  model: string
  finishReason: string
  metadata?: Record<string, any>
}

export interface TokenUsage {
  promptTokens: number
  completionTokens: number
  totalTokens: number
  cost?: number
}

export interface CompletionRequest {
  prompt: string
  model?: string
  maxTokens?: number
  temperature?: number
  stopSequences?: string[]
  options?: Record<string, any>
}

export interface CompletionResponse {
  text: string
  usage: TokenUsage
  model: string
  finishReason: string
  metadata?: Record<string, any>
}

export class AIService {
  /**
   * Get available AI models
   */
  async getModels(): Promise<ApiResponse<AIModel[]>> {
    return apiClient.invoke('get_ai_models')
  }

  /**
   * Get specific AI model info
   */
  async getModel(modelId: string): Promise<ApiResponse<AIModel>> {
    return apiClient.invoke('get_ai_model', { model_id: modelId })
  }

  /**
   * Set the default AI model
   */
  async setDefaultModel(modelId: string): Promise<ApiResponse<void>> {
    return apiClient.invoke('set_default_ai_model', { model_id: modelId })
  }

  /**
   * Send chat completion request
   */
  async chat(request: ChatRequest): Promise<ApiResponse<ChatResponse>> {
    return apiClient.invoke('ai_chat', {
      messages: request.messages,
      model: request.model,
      max_tokens: request.maxTokens,
      temperature: request.temperature,
      options: request.options || {}
    })
  }

  /**
   * Send completion request
   */
  async complete(request: CompletionRequest): Promise<ApiResponse<CompletionResponse>> {
    return apiClient.invoke('ai_complete', {
      prompt: request.prompt,
      model: request.model,
      max_tokens: request.maxTokens,
      temperature: request.temperature,
      stop_sequences: request.stopSequences,
      options: request.options || {}
    })
  }

  /**
   * Stream chat completion
   */
  async streamChat(request: ChatRequest, onChunk: (chunk: any) => void): Promise<ApiResponse<ChatResponse>> {
    // Implementation would use the streaming API
    return apiClient.invoke('ai_chat_stream', {
      messages: request.messages,
      model: request.model,
      max_tokens: request.maxTokens,
      temperature: request.temperature,
      options: request.options || {}
    })
  }

  /**
   * Analyze text sentiment
   */
  async analyzeSentiment(text: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('analyze_sentiment', { text })
  }

  /**
   * Extract entities from text
   */
  async extractEntities(text: string): Promise<ApiResponse<any[]>> {
    return apiClient.invoke('extract_entities', { text })
  }

  /**
   * Classify text content
   */
  async classifyText(text: string, categories?: string[]): Promise<ApiResponse<any>> {
    return apiClient.invoke('classify_text', {
      text,
      categories: categories || []
    })
  }

  /**
   * Summarize text
   */
  async summarize(text: string, options?: any): Promise<ApiResponse<string>> {
    return apiClient.invoke('summarize_text', {
      text,
      options: options || {}
    })
  }

  /**
   * Translate text
   */
  async translate(text: string, targetLanguage: string, sourceLanguage?: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('translate_text', {
      text,
      target_language: targetLanguage,
      source_language: sourceLanguage
    })
  }

  /**
   * Generate text embeddings
   */
  async generateEmbeddings(texts: string[], model?: string): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('generate_embeddings', {
      texts,
      model: model
    })
  }

  /**
   * Ask AI about document content
   */
  async queryDocument(documentId: string, question: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('query_document', {
      document_id: documentId,
      question
    })
  }

  /**
   * Get AI conversation suggestions
   */
  async getConversationSuggestions(conversationId: string): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('get_conversation_suggestions', {
      conversation_id: conversationId
    })
  }

  /**
   * Analyze conversation for insights
   */
  async analyzeConversation(conversationId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('analyze_conversation', {
      conversation_id: conversationId
    })
  }

  /**
   * Generate content based on prompt
   */
  async generateContent(prompt: string, contentType: string, options?: any): Promise<ApiResponse<string>> {
    return apiClient.invoke('generate_content', {
      prompt,
      content_type: contentType,
      options: options || {}
    })
  }

  /**
   * Improve text quality
   */
  async improveText(text: string, improvements: string[]): Promise<ApiResponse<string>> {
    return apiClient.invoke('improve_text', {
      text,
      improvements
    })
  }

  /**
   * Check text for grammar and style
   */
  async checkGrammar(text: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('check_grammar', { text })
  }

  /**
   * Rewrite text in different style
   */
  async rewriteText(text: string, style: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('rewrite_text', {
      text,
      style
    })
  }

  /**
   * Get AI model usage statistics
   */
  async getUsageStats(timeframe?: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_ai_usage_stats', {
      timeframe: timeframe || 'last_30_days'
    })
  }

  /**
   * Configure AI model settings
   */
  async configureModel(modelId: string, settings: any): Promise<ApiResponse<void>> {
    return apiClient.invoke('configure_ai_model', {
      model_id: modelId,
      settings
    })
  }

  /**
   * Test AI model connection
   */
  async testModelConnection(modelId: string): Promise<ApiResponse<boolean>> {
    return apiClient.invoke('test_ai_model', { model_id: modelId })
  }

  /**
   * Get AI conversation context
   */
  async getConversationContext(conversationId: string): Promise<ApiResponse<any>> {
    return apiClient.invoke('get_conversation_context', {
      conversation_id: conversationId
    })
  }

  /**
   * Update conversation context
   */
  async updateConversationContext(conversationId: string, context: any): Promise<ApiResponse<void>> {
    return apiClient.invoke('update_conversation_context', {
      conversation_id: conversationId,
      context
    })
  }
}

export const aiService = new AIService()