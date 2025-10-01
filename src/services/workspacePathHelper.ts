/**
 * Helper to convert workspace ID to filesystem path
 * Many backend commands expect workspace_path instead of workspace_id
 */
export function getWorkspacePath(workspaceId?: string): string {
  // If a path-like workspaceId is provided, use it
  if (workspaceId && (workspaceId.startsWith('/') || workspaceId.startsWith('.'))) {
    return workspaceId
  }
  // Default to current directory as the workspace
  return '.'
}
