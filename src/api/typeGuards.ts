// Runtime type validation guards
import {
  WorkspaceConfig,
  WorkspaceAnalysis,
  Document,
  DocumentComparison,
  StyleProfile,
  KnowledgeGap,
  SearchResult,
  EmbeddingResponse,
  Conversation
} from '../types'

/**
 * Type guard utilities for runtime validation
 */

// Base validation helpers
export function isString(value: unknown): value is string {
  return typeof value === 'string'
}

export function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !isNaN(value)
}

export function isBoolean(value: unknown): value is boolean {
  return typeof value === 'boolean'
}

export function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value)
}

export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function isDate(value: unknown): value is Date {
  return value instanceof Date && !isNaN(value.getTime())
}

export function isStringOrDate(value: unknown): value is string | Date {
  return isString(value) || isDate(value)
}

export function hasProperty<T extends string>(
  obj: unknown,
  prop: T
): obj is Record<T, unknown> {
  return isObject(obj) && prop in obj
}

export function hasProperties<T extends string>(
  obj: unknown,
  ...props: T[]
): obj is Record<T, unknown> {
  return isObject(obj) && props.every(prop => prop in obj)
}

// Workspace type guards
export function isWorkspaceConfig(value: unknown): value is WorkspaceConfig {
  return (
    isObject(value) &&
    hasProperties(value, 'id', 'name', 'path', 'createdAt', 'updatedAt') &&
    isString(value.id) &&
    isString(value.name) &&
    isString(value.path) &&
    isStringOrDate(value.createdAt) &&
    isStringOrDate(value.updatedAt) &&
    (!('description' in value) || isString(value.description))
  )
}

export function isWorkspaceAnalysis(value: unknown): value is WorkspaceAnalysis {
  return (
    isObject(value) &&
    hasProperties(value, 'overview', 'health', 'insights', 'performanceMetrics', 'organizationSuggestions') &&
    isObject(value.overview) &&
    isObject(value.health) &&
    isArray(value.insights) &&
    isObject(value.performanceMetrics) &&
    isArray(value.organizationSuggestions)
  )
}

// Document type guards
export function isDocument(value: unknown): value is Document {
  return (
    isObject(value) &&
    hasProperties(value, 'id', 'name', 'path', 'type', 'size', 'createdAt', 'updatedAt', 'metadata') &&
    isString(value.id) &&
    isString(value.name) &&
    isString(value.path) &&
    isString(value.type) &&
    isNumber(value.size) &&
    isStringOrDate(value.createdAt) &&
    isStringOrDate(value.updatedAt) &&
    isObject(value.metadata)
  )
}

export function isDocumentComparison(value: unknown): value is DocumentComparison {
  return (
    isObject(value) &&
    hasProperties(value, 'documentA', 'documentB', 'similarity', 'differences', 'commonElements', 'analysis') &&
    isString(value.documentA) &&
    isString(value.documentB) &&
    isNumber(value.similarity) &&
    isArray(value.differences) &&
    isArray(value.commonElements) &&
    isObject(value.analysis)
  )
}

// Style type guards
export function isStyleProfile(value: unknown): value is StyleProfile {
  return (
    isObject(value) &&
    hasProperties(value, 'id', 'name', 'source', 'features', 'patterns', 'confidence', 'createdAt') &&
    isString(value.id) &&
    isString(value.name) &&
    isString(value.source) &&
    isArray(value.features) &&
    isArray(value.patterns) &&
    isNumber(value.confidence) &&
    isStringOrDate(value.createdAt)
  )
}

// Knowledge type guards
export function isKnowledgeGap(value: unknown): value is KnowledgeGap {
  return (
    isObject(value) &&
    hasProperties(value, 'id', 'type', 'severity', 'description', 'impact', 'suggestedSources', 'priority', 'identifiedAt') &&
    isString(value.id) &&
    isString(value.type) &&
    isString(value.severity) &&
    isString(value.description) &&
    isString(value.impact) &&
    isArray(value.suggestedSources) &&
    isNumber(value.priority) &&
    isStringOrDate(value.identifiedAt)
  )
}

// Search type guards
export function isSearchResult(value: unknown): value is SearchResult {
  return (
    isObject(value) &&
    hasProperties(value, 'query', 'results', 'totalCount', 'executionTime', 'metadata') &&
    isObject(value.query) &&
    isArray(value.results) &&
    isNumber(value.totalCount) &&
    isNumber(value.executionTime) &&
    isObject(value.metadata)
  )
}

// Embedding type guards
export function isEmbeddingResponse(value: unknown): value is EmbeddingResponse {
  return (
    isObject(value) &&
    hasProperties(value, 'embeddings', 'model', 'dimensions', 'tokenUsage', 'processingTime', 'metadata') &&
    isArray(value.embeddings) &&
    isString(value.model) &&
    isNumber(value.dimensions) &&
    isObject(value.tokenUsage) &&
    isNumber(value.processingTime) &&
    isObject(value.metadata)
  )
}

// Conversation type guards
export function isConversation(value: unknown): value is Conversation {
  return (
    isObject(value) &&
    hasProperties(value, 'id', 'title', 'participants', 'messages', 'context', 'metadata', 'createdAt', 'updatedAt', 'status') &&
    isString(value.id) &&
    isString(value.title) &&
    isArray(value.participants) &&
    isArray(value.messages) &&
    isObject(value.context) &&
    isObject(value.metadata) &&
    isStringOrDate(value.createdAt) &&
    isStringOrDate(value.updatedAt) &&
    isString(value.status)
  )
}

// Generic API response validators
export function validateApiResponse<T>(
  value: unknown,
  dataValidator?: (data: unknown) => data is T
): value is { success: boolean; data?: T; error?: string; metadata?: Record<string, unknown> } {
  if (!isObject(value) || !hasProperty(value, 'success') || !isBoolean(value.success)) {
    return false
  }

  if ('data' in value && dataValidator && !dataValidator(value.data)) {
    return false
  }

  if ('error' in value && value.error !== undefined && !isString(value.error)) {
    return false
  }

  if ('metadata' in value && value.metadata !== undefined && !isObject(value.metadata)) {
    return false
  }

  return true
}

// Array validators
export function isArrayOf<T>(
  value: unknown,
  itemValidator: (item: unknown) => item is T
): value is T[] {
  return isArray(value) && value.every(itemValidator)
}

export function isStringArray(value: unknown): value is string[] {
  return isArrayOf(value, isString)
}

export function isNumberArray(value: unknown): value is number[] {
  return isArrayOf(value, isNumber)
}

// Validation result type
export interface ValidationResult {
  valid: boolean
  errors: string[]
  path: string
}

// Comprehensive object validator
export class ObjectValidator {
  private path: string[] = []

  validate<T>(value: unknown, validator: (v: unknown) => v is T): ValidationResult {
    this.path = []
    const errors: string[] = []

    if (!validator(value)) {
      errors.push(`Invalid type at root`)
    }

    return {
      valid: errors.length === 0,
      errors,
      path: this.path.join('.')
    }
  }

  validateProperty<T>(
    obj: Record<string, unknown>,
    prop: string,
    validator: (v: unknown) => v is T,
    required = true
  ): boolean {
    this.path.push(prop)

    if (!(prop in obj)) {
      if (required) {
        return false
      }
      this.path.pop()
      return true
    }

    const isValid = validator(obj[prop])
    this.path.pop()
    return isValid
  }

  validateArrayProperty<T>(
    obj: Record<string, unknown>,
    prop: string,
    itemValidator: (v: unknown) => v is T,
    required = true
  ): boolean {
    this.path.push(prop)

    if (!(prop in obj)) {
      if (required) {
        this.path.pop()
        return false
      }
      this.path.pop()
      return true
    }

    const isValid = isArrayOf(obj[prop], itemValidator)
    this.path.pop()
    return isValid
  }
}

// Export convenience validator instance
export const validator = new ObjectValidator()

// Utility functions for common validation patterns
export function validateRequired(value: unknown, name: string): ValidationResult {
  if (value === undefined || value === null) {
    return {
      valid: false,
      errors: [`${name} is required`],
      path: name
    }
  }

  return {
    valid: true,
    errors: [],
    path: name
  }
}

export function validateStringEnum<T extends string>(
  value: unknown,
  allowedValues: readonly T[],
  name: string
): value is T {
  return isString(value) && (allowedValues as readonly string[]).includes(value)
}

export function validateNumberRange(
  value: unknown,
  min: number,
  max: number,
  name: string
): ValidationResult {
  if (!isNumber(value)) {
    return {
      valid: false,
      errors: [`${name} must be a number`],
      path: name
    }
  }

  if (value < min || value > max) {
    return {
      valid: false,
      errors: [`${name} must be between ${min} and ${max}`],
      path: name
    }
  }

  return {
    valid: true,
    errors: [],
    path: name
  }
}