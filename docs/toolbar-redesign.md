# Document Toolbar Redesign

## Overview

The DocumentToolbar has been completely redesigned with a Microsoft Word-inspired ribbon interface that provides better organization, scalability, and user familiarity.

## Architecture

### Three-Row Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Row 1: Document Metadata                                                 │
│ [Icon] Document Name [Confidence] [Type] [Active Users] [Offline]  [X]  │
├─────────────────────────────────────────────────────────────────────────┤
│ Row 2: Tab Navigation & Primary Actions                                  │
│ Home │ Review │ AI Tools │ More                    [Save] [Actions ▼]   │
├─────────────────────────────────────────────────────────────────────────┤
│ Row 3: Contextual Tools (Based on Active Tab)                           │
│ [Group 1] | [Group 2] | [Group 3]                                       │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Structure

```
DocumentToolbar/
├── Row 1: Document Info Bar
│   ├── Document Icon + Name
│   ├── Confidence Badge
│   ├── Document Type Badge
│   ├── Active Users Component
│   ├── Offline Indicator
│   └── Close Button
│
├── Row 2: Tab Bar & Primary Actions
│   ├── Tab Navigation (Home, Review, AI Tools, More)
│   ├── Save Button (when in edit mode)
│   ├── Save Status Indicators
│   └── Actions Dropdown
│
└── Row 3: Contextual Toolbar
    ├── Tool Groups (based on active tab)
    └── Individual Tool Buttons
```

## Tab Groups

### Home Tab
**Purpose**: Core document editing and viewing functions

**Groups**:
- **Mode Group**
  - Edit/View Toggle
  - AI Autocomplete Toggle (when in edit mode)

### Review Tab
**Purpose**: Collaboration and version management

**Groups**:
- **Versions Group**
  - Version History
  - Compare Versions

- **Collaboration Group**
  - Comments & Annotations
  - Track Changes (with badge for pending changes)
  - Review Changes

### AI Tools Tab
**Purpose**: AI-powered analysis and generation

**Groups**:
- **AI Analysis Group**
  - Analyze Document
  - Deep Analysis

- **AI Generation Group**
  - Generate Content
  - AI Suggestions (with badge for available suggestions)

### More Tab
**Purpose**: Additional document actions

**Groups**:
- **Document Actions Group**
  - Rename Document
  - Download
  - (Expandable for future actions)

## Design Principles

### 1. **Familiar Pattern**
- Follows Microsoft Word/Google Docs ribbon pattern
- Users instantly understand tab-based organization
- Reduces cognitive load

### 2. **Progressive Disclosure**
- Primary actions always visible (Save, Close)
- Secondary actions grouped by context in tabs
- Tertiary actions in dropdown menu

### 3. **Scalability**
- New tools added to appropriate tab groups
- Groups maintain visual boundaries
- Horizontal scrolling for overflow (rare)

### 4. **Visual Hierarchy**
```
Level 1: Document Info (Always visible)
Level 2: Tab Navigation + Primary Actions (Always visible)
Level 3: Contextual Tools (Changes with tab)
Level 4: Dropdown Actions (On-demand)
```

### 5. **Responsive Behavior**
- Groups never split mid-action
- Overflow scrolls horizontally
- Tab labels show action count
- Compact mode on smaller screens

## Implementation Details

### Component API

```typescript
interface DocumentToolbarProps {
  // Document metadata
  documentName: string
  documentIcon?: IconName
  confidence?: number
  documentType?: string

  // Collaboration
  activeUsers?: CollaborationUser[]
  showPresence?: boolean

  // Sync status
  isSyncing?: boolean
  queuedOperations?: number

  // Save functionality
  onSave?: () => void
  isSaving?: boolean
  isDirty?: boolean
  saveError?: string
  lastSaved?: Date

  // Tab action groups
  homeActions?: ToolbarGroup[]
  reviewActions?: ToolbarGroup[]
  aiToolsActions?: ToolbarGroup[]
  moreActions?: ToolbarGroup[]

  // Additional actions
  dropdownActions?: DropdownOption[]
  onDropdownAction?: (value: string) => void

  // Callbacks
  onClose?: () => void
  onTabChange?: (tab: ToolbarTab) => void
  defaultTab?: ToolbarTab
}

interface ToolbarGroup {
  id: string
  label: string
  actions: ToolbarAction[]
}

interface ToolbarAction {
  id: string
  label: string
  icon: IconName
  onClick: () => void
  disabled?: boolean
  badge?: string | number  // For notification counts
  tooltip?: string
  variant?: 'ghost' | 'primary' | 'secondary'
  showLabel?: boolean
}
```

### Adding New Tools

#### Step 1: Choose the Appropriate Tab
```typescript
// For editing features → homeActions
// For collaboration features → reviewActions
// For AI features → aiToolsActions
// For document management → moreActions
```

#### Step 2: Define the Action
```typescript
const newAction: ToolbarAction = {
  id: 'my-new-feature',
  label: 'My Feature',
  icon: 'IconName' as const,
  onClick: () => handleMyFeature(),
  tooltip: 'Description of my feature',
  showLabel: true,
  badge: notificationCount > 0 ? notificationCount : undefined,
}
```

#### Step 3: Add to Existing Group or Create New Group
```typescript
const myTabActions = useMemo<ToolbarGroup[]>(() => [
  // Existing groups...
  {
    id: 'my-new-group',
    label: 'My Group',
    actions: [newAction],
  },
], [dependencies])
```

### Visual Tokens

```typescript
// Colors
background: 'linear-gradient(to bottom, rgba(26, 26, 30, 0.95), rgba(22, 22, 26, 0.98))'
borderBottom: '1px solid rgba(255, 255, 255, 0.08)'
divider: '1px solid rgba(255, 255, 255, 0.1)'

// Spacing
row1Height: '44px'
row2Height: '40px'
row3Height: '48px'
groupGap: '12px' (spacing[3])
actionGap: '4px' (spacing[1])

// Typography
documentName: fontSize.base, fontWeight.semibold
tabLabel: fontSize.sm, fontWeight.semibold
actionLabel: fontSize.sm, fontWeight.normal
```

## Accessibility

- **Keyboard Navigation**: Tab through all interactive elements
- **ARIA Labels**: All icon buttons have descriptive tooltips
- **Focus Management**: Clear focus states with accent color
- **Screen Reader Support**: Proper semantic HTML structure

## Migration from Old Toolbar

### Before (Old Design)
```tsx
// Single row with all buttons
<div style={{ display: 'flex', gap: '8px' }}>
  <Button>Edit</Button>
  <Button>Save</Button>
  <Button>Versions</Button>
  <Button>Comments</Button>
  <Button>Track Changes</Button>
  <Button>Review</Button>
  <Button>Compare</Button>
  {/* More buttons causing overflow... */}
</div>
```

### After (New Design)
```tsx
<DocumentToolbar
  documentName={document.name}
  homeActions={[
    {
      id: 'mode',
      label: 'Mode',
      actions: [editToggle, aiToggle]
    }
  ]}
  reviewActions={[
    {
      id: 'versions',
      label: 'Versions',
      actions: [versionHistory, compare]
    },
    {
      id: 'collaboration',
      label: 'Collaboration',
      actions: [comments, trackChanges, review]
    }
  ]}
  // ... other tabs
/>
```

## Benefits

### 1. **Organization**
- Related tools grouped logically
- Clear visual separation
- Reduced clutter

### 2. **Scalability**
- Add new tools without UI disruption
- Groups maintain consistent spacing
- Tabs prevent horizontal overflow

### 3. **Discoverability**
- Users know where to find features
- Tab labels provide context
- Tooltips explain each action

### 4. **Flexibility**
- Easy to add/remove tools
- Supports future features
- Configurable per document type

### 5. **Consistency**
- Follows industry standards (Word, Docs)
- Familiar interaction patterns
- Predictable behavior

## Future Enhancements

### Phase 2: Responsive Design
- Collapse to hamburger menu on mobile
- Touch-friendly button sizes
- Swipeable tabs

### Phase 3: Customization
- User-configurable tabs
- Favorite actions quick access
- Keyboard shortcuts

### Phase 4: Context-Aware
- Show/hide tabs based on document type
- Dynamic groups based on permissions
- Smart action recommendations

## Examples

### Adding a "Print" Action

```typescript
// 1. Add to More Actions
const moreActions = useMemo<ToolbarGroup[]>(() => [
  {
    id: 'document-actions',
    label: 'Document',
    actions: [
      {
        id: 'print',
        label: 'Print',
        icon: 'Printer' as const,  // Add icon if needed
        onClick: () => window.print(),
        tooltip: 'Print Document',
        showLabel: true,
      },
      // ... other actions
    ]
  }
], [])

// 2. Pass to toolbar
<DocumentToolbar
  // ...
  moreActions={moreActions}
/>
```

### Adding Notification Badge

```typescript
const action: ToolbarAction = {
  id: 'comments',
  label: 'Comments',
  icon: 'MessageCircle' as const,
  onClick: () => toggleComments(),
  tooltip: 'View Comments',
  showLabel: true,
  badge: unreadComments > 0 ? unreadComments : undefined,  // Shows count badge
}
```

## Conclusion

The new DocumentToolbar provides a scalable, familiar, and well-organized interface that can grow with the application's needs without compromising usability or aesthetics.
