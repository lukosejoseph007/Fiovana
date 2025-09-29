// Icon utilities and mappings

import React from 'react'
import type { IconProps } from './types'
import {
  DocumentIcon,
  PDFIcon,
  WordIcon,
  PowerPointIcon,
  AIStatusIcon,
  HealthIcon,
  ConfidenceIcon,
  CompareIcon,
  GenerateIcon,
  AnalyzeIcon,
  SearchIcon,
  SettingsIcon,
  WorkspaceIcon,
  SpinnerIcon,
  PulseIcon,
  UserIcon,
  CollaborationIcon,
  ChevronDownIcon,
} from './index'

// Utility function to get document type icon component
export const getDocumentTypeIcon = (mimeType: string): React.FC<IconProps> => {
  if (mimeType.includes('pdf')) return PDFIcon
  if (mimeType.includes('word') || mimeType.includes('document')) return WordIcon
  if (mimeType.includes('presentation') || mimeType.includes('powerpoint')) return PowerPointIcon
  return DocumentIcon
}

// Export all icons for easy access
export const Icons = {
  // Document types
  Document: DocumentIcon,
  PDF: PDFIcon,
  Word: WordIcon,
  PowerPoint: PowerPointIcon,

  // Status indicators
  AIStatus: AIStatusIcon,
  Health: HealthIcon,
  Confidence: ConfidenceIcon,

  // Actions
  Compare: CompareIcon,
  Generate: GenerateIcon,
  Analyze: AnalyzeIcon,

  // Navigation
  Search: SearchIcon,
  Settings: SettingsIcon,
  Workspace: WorkspaceIcon,

  // Loading
  Spinner: SpinnerIcon,
  Pulse: PulseIcon,

  // Collaboration
  User: UserIcon,
  Collaboration: CollaborationIcon,

  // Navigation elements
  ChevronDown: ChevronDownIcon,

  // Utility
  getDocumentTypeIcon,
}

export default Icons
