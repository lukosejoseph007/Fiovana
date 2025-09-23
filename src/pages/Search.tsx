import React from 'react'
import SearchInterface from '../components/search/SearchInterface'
import SearchErrorBoundary from '../components/search/SearchErrorBoundary'

const Search: React.FC = () => {
  return (
    <SearchErrorBoundary>
      <SearchInterface />
    </SearchErrorBoundary>
  )
}

export default Search
