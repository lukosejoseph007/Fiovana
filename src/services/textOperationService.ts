// src/services/textOperationService.ts
import { invoke } from '@tauri-apps/api/core'

/**
 * Text operation types for AI-powered text manipulation
 */
export type TextOperation =
  | { type: 'Define' }
  | { type: 'Explain' }
  | { type: 'Expand' }
  | { type: 'Simplify' }
  | { type: 'Rewrite'; params: { style?: string } }
  | { type: 'Improve' }
  | { type: 'Summarize'; params: { length?: 'short' | 'medium' | 'long' } }
  | { type: 'Translate'; params: { target_language: string } }
  | { type: 'FindRelated' }
  | { type: 'Custom'; params: string }

/**
 * Document context for text operations
 */
export interface DocumentContext {
  document_id?: string
  document_title?: string
  document_type?: string
  surrounding_text?: string
  metadata?: Record<string, unknown>
}

/**
 * Result of a text operation
 */
export interface TextOperationResult {
  original: string
  result: string
  operation: string
  confidence: number
  reasoning?: string
  suggestions: string[]
  alternative_results: string[]
}

/**
 * Request structure for text operations
 */
export interface TextOperationRequest {
  text: string
  operation: TextOperation
  context?: DocumentContext
}

/**
 * Operation information
 */
export interface OperationInfo {
  name: string
  description: string
  requires_params: boolean
}

/**
 * Execute a text operation with AI
 */
export async function executeTextOperation(
  request: TextOperationRequest
): Promise<TextOperationResult> {
  try {
    return await invoke<TextOperationResult>('execute_text_operation', { request })
  } catch (error) {
    console.error('Error executing text operation:', error)
    throw new Error(`Failed to execute text operation: ${error}`)
  }
}

/**
 * Get list of available text operations
 */
export async function getAvailableTextOperations(): Promise<string[]> {
  try {
    return await invoke<string[]>('get_available_text_operations')
  } catch (error) {
    console.error('Error getting available text operations:', error)
    throw new Error(`Failed to get available text operations: ${error}`)
  }
}

/**
 * Get description for a specific operation
 */
export async function getTextOperationDescription(operationName: string): Promise<string | null> {
  try {
    return await invoke<string | null>('get_text_operation_description', {
      operationName,
    })
  } catch (error) {
    console.error('Error getting text operation description:', error)
    throw new Error(`Failed to get text operation description: ${error}`)
  }
}

/**
 * Get all operations with their descriptions
 */
export async function getTextOperationsInfo(): Promise<OperationInfo[]> {
  try {
    return await invoke<OperationInfo[]>('get_text_operations_info')
  } catch (error) {
    console.error('Error getting text operations info:', error)
    throw new Error(`Failed to get text operations info: ${error}`)
  }
}

/**
 * Test text operations with sample text
 */
export async function testTextOperations(sampleText?: string): Promise<TextOperationResult[]> {
  try {
    return await invoke<TextOperationResult[]>('test_text_operations', {
      sampleText,
    })
  } catch (error) {
    console.error('Error testing text operations:', error)
    throw new Error(`Failed to test text operations: ${error}`)
  }
}

/**
 * Helper function to create operation objects
 */
export const TextOperations = {
  define: (): TextOperation => ({ type: 'Define' }),
  explain: (): TextOperation => ({ type: 'Explain' }),
  expand: (): TextOperation => ({ type: 'Expand' }),
  simplify: (): TextOperation => ({ type: 'Simplify' }),
  rewrite: (style?: string): TextOperation => ({
    type: 'Rewrite',
    params: { style },
  }),
  improve: (): TextOperation => ({ type: 'Improve' }),
  summarize: (length?: 'short' | 'medium' | 'long'): TextOperation => ({
    type: 'Summarize',
    params: { length },
  }),
  translate: (targetLanguage: string): TextOperation => ({
    type: 'Translate',
    params: { target_language: targetLanguage },
  }),
  findRelated: (): TextOperation => ({ type: 'FindRelated' }),
  custom: (prompt: string): TextOperation => ({ type: 'Custom', params: prompt }),
}
