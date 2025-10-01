import { useState, useEffect } from 'react'

/**
 * Hook to use error boundary programmatically
 *
 * Example usage:
 * ```tsx
 * const throwError = useErrorHandler();
 *
 * try {
 *   await someDangerousOperation();
 * } catch (error) {
 *   throwError(error);
 * }
 * ```
 */
export const useErrorHandler = () => {
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    if (error) {
      throw error
    }
  }, [error])

  return setError
}
