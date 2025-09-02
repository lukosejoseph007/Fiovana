// Utility functions
export const formatDate = (date: Date): string => {
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(date)
}

export const generateId = (): string => {
  return Math.random().toString(36).substring(2, 15)
}
