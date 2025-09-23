import React from 'react'
import DocumentIndex from '../components/indexing/DocumentIndex'
import DocumentIndexErrorBoundary from '../components/indexing/DocumentIndexErrorBoundary'

const DocumentIndexPage: React.FC = () => {
  return (
    <DocumentIndexErrorBoundary>
      <DocumentIndex />
    </DocumentIndexErrorBoundary>
  )
}

export default DocumentIndexPage
