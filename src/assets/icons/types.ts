// Icon type definitions and utilities

export interface IconProps {
  size?: number | string
  color?: string
  className?: string
  'aria-label'?: string
}

// Utility function to get document type icon
export const getDocumentTypeIcon = (mimeType: string): string => {
  if (mimeType.includes('pdf')) return 'PDF'
  if (mimeType.includes('word') || mimeType.includes('document')) return 'Word'
  if (mimeType.includes('presentation') || mimeType.includes('powerpoint')) return 'PowerPoint'
  return 'Document'
}
