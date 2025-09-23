import React from 'react'
import DocumentComparison from '../components/comparison/DocumentComparison'
import DocumentComparisonErrorBoundary from '../components/comparison/DocumentComparisonErrorBoundary'

const DocumentComparisonPage: React.FC = () => {
  return (
    <DocumentComparisonErrorBoundary>
      <DocumentComparison />
    </DocumentComparisonErrorBoundary>
  )
}

export default DocumentComparisonPage
