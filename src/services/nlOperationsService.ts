// Natural Language Operations Service
import { apiClient } from '../api'
import { ApiResponse } from '../types'

export class NLOperationsService {
  /**
   * Extract named entities from document
   */
  async extractNamedEntities(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_named_entities', {
      document_id: documentId,
    })
  }

  /**
   * Perform sentiment analysis
   */
  async analyzeSentiment(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_sentiment', {
      document_id: documentId,
    })
  }

  /**
   * Extract key phrases from document
   */
  async extractKeyPhrases(documentId: string): Promise<ApiResponse<string[]>> {
    return apiClient.invoke('extract_key_phrases', {
      document_id: documentId,
    })
  }

  /**
   * Perform text summarization
   */
  async summarizeText(
    documentId: string,
    summaryLength?: 'short' | 'medium' | 'long'
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('summarize_text', {
      document_id: documentId,
      summary_length: summaryLength || 'medium',
    })
  }

  /**
   * Detect language of document
   */
  async detectLanguage(documentId: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('detect_language', {
      document_id: documentId,
    })
  }

  /**
   * Translate document content
   */
  async translateDocument(
    documentId: string,
    targetLanguage: string,
    preserveFormatting?: boolean
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('translate_document', {
      document_id: documentId,
      target_language: targetLanguage,
      preserve_formatting: preserveFormatting || true,
    })
  }

  /**
   * Extract document topics
   */
  async extractTopics(
    documentId: string,
    numberOfTopics?: number
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_topics', {
      document_id: documentId,
      number_of_topics: numberOfTopics || 5,
    })
  }

  /**
   * Analyze readability metrics
   */
  async analyzeReadability(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_readability', {
      document_id: documentId,
    })
  }

  /**
   * Extract document structure
   */
  async extractStructure(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('extract_document_structure', {
      document_id: documentId,
    })
  }

  /**
   * Perform intent classification
   */
  async classifyIntent(text: string, possibleIntents?: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('classify_intent', {
      text,
      possible_intents: possibleIntents || [],
    })
  }

  /**
   * Extract relationships between entities
   */
  async extractEntityRelationships(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_entity_relationships', {
      document_id: documentId,
    })
  }

  /**
   * Perform document clustering based on content
   */
  async clusterDocuments(
    documentIds: string[],
    numberOfClusters?: number
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('cluster_documents', {
      document_ids: documentIds,
      number_of_clusters: numberOfClusters || 3,
    })
  }

  /**
   * Generate document embeddings
   */
  async generateDocumentEmbeddings(documentId: string): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('generate_document_embeddings', {
      document_id: documentId,
    })
  }

  /**
   * Perform text classification
   */
  async classifyText(text: string, categories: string[]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('classify_text', {
      text,
      categories,
    })
  }

  /**
   * Extract document metadata through NL processing
   */
  async extractNLMetadata(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('extract_nl_metadata', {
      document_id: documentId,
    })
  }

  /**
   * Perform question answering on document
   */
  async answerQuestion(documentId: string, question: string): Promise<ApiResponse<string>> {
    return apiClient.invoke('answer_question_about_document', {
      document_id: documentId,
      question,
    })
  }

  /**
   * Generate document insights
   */
  async generateInsights(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('generate_document_insights', {
      document_id: documentId,
    })
  }

  /**
   * Batch process multiple NL operations
   */
  async batchProcessNL(
    documentIds: string[],
    operations: string[]
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('batch_process_nl_operations', {
      document_ids: documentIds,
      operations,
    })
  }
}

export const nlOperationsService = new NLOperationsService()
