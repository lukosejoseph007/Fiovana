import React from 'react'
import { AlertTriangle } from 'lucide-react'

interface State {
  hasError: boolean
  error?: Error
}

class SearchErrorBoundary extends React.Component<React.PropsWithChildren<object>, State> {
  constructor(props: React.PropsWithChildren<object>) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Search component error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="max-w-6xl mx-auto p-6">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
            <AlertTriangle className="mx-auto h-12 w-12 text-red-500 mb-4" />
            <h2 className="text-xl font-semibold text-red-900 mb-2">Search Unavailable</h2>
            <p className="text-red-700 mb-4">
              The search functionality is currently experiencing issues. This may be because:
            </p>
            <ul className="text-left text-red-700 mb-4 max-w-md mx-auto">
              <li>• No documents have been indexed yet</li>
              <li>• The vector system is not initialized</li>
              <li>• Backend services are not running</li>
            </ul>
            <p className="text-red-600 text-sm">
              Please try importing some documents first, or check the application logs for more
              details.
            </p>
            <button
              onClick={() => this.setState({ hasError: false })}
              className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
            >
              Try Again
            </button>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

export default SearchErrorBoundary
