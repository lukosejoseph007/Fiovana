// Document Structure Service
import { apiClient } from '../api'
import {
  Document,
  DocumentStructure,
  ApiResponse
} from '../types'

export class StructureService {
  /**
   * Analyze document structure
   */
  async analyzeDocumentStructure(documentId: string): Promise<ApiResponse<DocumentStructure>> {
    return apiClient.invoke('analyze_document_structure', {
      document_id: documentId
    })
  }

  /**
   * Extract document hierarchy
   */
  async extractHierarchy(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('extract_document_hierarchy', {
      document_id: documentId
    })
  }

  /**
   * Identify document sections
   */
  async identifySections(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('identify_document_sections', {
      document_id: documentId
    })
  }

  /**
   * Extract table of contents
   */
  async extractTableOfContents(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('extract_table_of_contents', {
      document_id: documentId
    })
  }

  /**
   * Analyze document flow and organization
   */
  async analyzeDocumentFlow(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_document_flow', {
      document_id: documentId
    })
  }

  /**
   * Identify structural patterns
   */
  async identifyStructuralPatterns(
    documentIds: string[]
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('identify_structural_patterns', {
      document_ids: documentIds
    })
  }

  /**
   * Extract document outline
   */
  async extractOutline(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('extract_document_outline', {
      document_id: documentId
    })
  }

  /**
   * Analyze paragraph structure
   */
  async analyzeParagraphStructure(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('analyze_paragraph_structure', {
      document_id: documentId
    })
  }

  /**
   * Identify document components
   */
  async identifyComponents(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('identify_document_components', {
      document_id: documentId
    })
  }

  /**
   * Analyze structural consistency
   */
  async analyzeStructuralConsistency(
    documentIds: string[]
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_structural_consistency', {
      document_ids: documentIds
    })
  }

  /**
   * Generate structure-based navigation
   */
  async generateStructureNavigation(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_structure_navigation', {
      document_id: documentId
    })
  }

  /**
   * Compare document structures
   */
  async compareStructures(
    documentId1: string,
    documentId2: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('compare_document_structures', {
      document_id_1: documentId1,
      document_id_2: documentId2
    })
  }

  /**
   * Suggest structure improvements
   */
  async suggestStructureImprovements(documentId: string): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('suggest_structure_improvements', {
      document_id: documentId
    })
  }

  /**
   * Validate document structure
   */
  async validateStructure(
    documentId: string,
    structureRules?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('validate_document_structure', {
      document_id: documentId,
      structure_rules: structureRules || {}
    })
  }

  /**
   * Extract structural metadata
   */
  async extractStructuralMetadata(documentId: string): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('extract_structural_metadata', {
      document_id: documentId
    })
  }

  /**
   * Reorganize document structure
   */
  async reorganizeStructure(
    documentId: string,
    reorganizationPlan: unknown
  ): Promise<ApiResponse<Document>> {
    return apiClient.invoke('reorganize_document_structure', {
      document_id: documentId,
      reorganization_plan: reorganizationPlan
    })
  }

  /**
   * Generate structure templates
   */
  async generateStructureTemplate(
    documentIds: string[],
    templateName: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('generate_structure_template', {
      document_ids: documentIds,
      template_name: templateName
    })
  }

  /**
   * Apply structure template to document
   */
  async applyStructureTemplate(
    documentId: string,
    templateId: string
  ): Promise<ApiResponse<Document>> {
    return apiClient.invoke('apply_structure_template', {
      document_id: documentId,
      template_id: templateId
    })
  }
}

export const structureService = new StructureService()