# Collaboration Components

This directory contains real-time collaboration components for Fiovana's document editing features.

## Components

### UserPresence
Displays a list of active users with connection status, avatars, and typing indicators.

**Features:**
- Connection status indicator (Wifi/WifiOff)
- User count display
- Avatar with initials and color coding
- "You" badge for current user
- Active status indicators
- Click handler for user interaction

**Usage:**
```tsx
<UserPresence
  users={collaborationUsers}
  currentUserId={currentUserId}
  isConnected={isConnected}
  onUserClick={(userId) => console.log('User clicked:', userId)}
/>
```

### LiveCursors
Shows floating cursor positions for all active collaborators in real-time.

**Features:**
- Real-time cursor position tracking
- User name labels
- Color-coded cursors
- Auto-fade after inactivity (configurable timeout)
- Smooth position transitions
- Stale cursor cleanup

**Usage:**
```tsx
<LiveCursors
  cursors={cursorPositions}
  containerRef={contentRef}
  showLabels={true}
  fadeTimeout={3000}
/>
```

### ActiveUsers
Compact badge showing active user count with avatars and typing indicators.

**Features:**
- User count badge
- Stacked avatar display (max 5 visible)
- Typing indicators with animation
- Extra user count (+N more)
- Clickable to open full presence panel
- Configurable size (small/medium/large)

**Usage:**
```tsx
<ActiveUsers
  users={collaborationUsers}
  currentUserId={currentUserId}
  size="small"
  onClick={() => setShowPresencePanel(true)}
/>
```

## Hooks

### useTypingIndicator
Tracks typing state with debouncing and auto-timeout.

**Features:**
- Debounced typing detection
- Configurable timeout
- Callback on typing state change
- Automatic cleanup

**Usage:**
```tsx
const { isTyping, handleTyping, startTyping, stopTyping } = useTypingIndicator({
  debounceMs: 300,
  typingTimeoutMs: 3000,
  onTypingChange: (typing) => {
    collaboration.updateUserTyping(userId, typing);
  },
});
```

### useRemoteTyping
Tracks typing states for remote users with stale state cleanup.

**Usage:**
```tsx
const { typingUsers, updateUserTyping, isUserTyping, getTypingUsers } = useRemoteTyping();
```

## Integration

To enable presence and awareness features:

1. Wrap your app with `CollaborationProvider`:
```tsx
<CollaborationProvider>
  <App />
</CollaborationProvider>
```

2. Use the collaboration context:
```tsx
const collaboration = useCollaboration();
```

3. Enable collaboration for a document:
```tsx
collaboration.enableCollaboration(documentId, lexicalEditor);
```

4. The presence components will automatically show when:
   - `collaboration.settings.enabled` is true
   - `collaboration.settings.showPresence` is true
   - `collaboration.settings.showCursors` is true

## UI Location

In DocumentViewer:
- **ActiveUsers badge**: Shown in document header, right side after classification badge
- **UserPresence panel**: Opens as fixed overlay (top-right) when ActiveUsers is clicked
- **LiveCursors**: Overlaid on document content area, follows mouse positions

## Future Enhancements

- Video/audio presence indicators
- Focus on user cursor when clicking user in panel
- User status messages ("editing section X")
- Collaborative selection highlighting
- User activity timeline
