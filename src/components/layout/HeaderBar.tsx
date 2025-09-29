import React from 'react';
import { designTokens } from '../../styles/tokens';
import { useLayout } from './AppShell';

export interface HeaderBarProps {
  className?: string;
  style?: React.CSSProperties;
}

const HeaderBar: React.FC<HeaderBarProps> = ({ className = '', style }) => {
  const { isMobile } = useLayout();

  const headerContentStyles = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    width: '100%',
    height: '100%',
  };

  const leftZoneStyles = {
    display: 'flex',
    alignItems: 'center',
    width: isMobile ? 'auto' : designTokens.layout.navigation.width,
    flexShrink: 0,
  };

  const centerZoneStyles = {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    maxWidth: '600px',
    margin: `0 ${designTokens.spacing[4]}`,
  };

  const rightZoneStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    flexShrink: 0,
  };

  const logoStyles = {
    fontSize: designTokens.typography.fontSize.xl,
    fontWeight: designTokens.typography.fontWeight.thin,
    color: designTokens.colors.text.primary,
    textDecoration: 'none',
    letterSpacing: designTokens.typography.letterSpacing.wide,
  };

  const searchBarStyles = {
    width: '100%',
    maxWidth: '400px',
    height: '36px',
    backgroundColor: designTokens.colors.surface.tertiary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.lg,
    padding: `0 ${designTokens.spacing[4]}`,
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.primary,
    outline: 'none',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    fontFamily: designTokens.typography.fonts.sans.join(', '),
  };

  return (
    <>
      <style>
        {`
          .proxemic-search-bar:focus {
            border-color: ${designTokens.colors.accent.ai};
            box-shadow: 0 0 0 3px ${designTokens.colors.accent.ai}40;
            background-color: ${designTokens.colors.surface.secondary};
          }

          .proxemic-search-bar::placeholder {
            color: ${designTokens.colors.text.tertiary};
          }

          .proxemic-header-icon:hover {
            color: ${designTokens.colors.text.primary};
            background-color: ${designTokens.colors.state.hover};
          }

          .proxemic-avatar-status {
            position: relative;
          }

          .proxemic-avatar-status::after {
            content: '';
            position: absolute;
            bottom: 0;
            right: 0;
            width: 10px;
            height: 10px;
            background-color: ${designTokens.colors.confidence.high};
            border: 2px solid ${designTokens.colors.surface.primary};
            border-radius: 50%;
          }
        `}
      </style>

      <div className={`proxemic-header-bar ${className}`} style={{ ...headerContentStyles, ...style }}>
        {/* Left Zone - Logo and Workspace Selector */}
        <div style={leftZoneStyles}>
          <a href="/" style={logoStyles}>
            Proxemic
          </a>

          {!isMobile && (
            <div style={{ marginLeft: designTokens.spacing[4] }}>
              <WorkspaceSelector />
            </div>
          )}
        </div>

        {/* Center Zone - Search/Command Bar */}
        <div style={centerZoneStyles}>
          <input
            className="proxemic-search-bar"
            style={searchBarStyles}
            placeholder="Search, ask, or type / for commands..."
            onFocus={() => {
              // TODO: Open command palette
              console.log('Open command palette');
            }}
          />
        </div>

        {/* Right Zone - Collaborators, AI Status, Settings */}
        <div style={rightZoneStyles}>
          <CollaboratorAvatars />
          <AIStatusIndicator />
          <SettingsButton />
        </div>
      </div>
    </>
  );
};

// Workspace Selector Component
const WorkspaceSelector: React.FC = () => {
  const selectorStyles = {
    background: 'none',
    border: 'none',
    color: designTokens.colors.text.secondary,
    fontSize: designTokens.typography.fontSize.sm,
    cursor: 'pointer',
    padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
    borderRadius: designTokens.borderRadius.md,
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[1],
    transition: `color ${designTokens.animation.duration.fast}`,
  };

  return (
    <button style={selectorStyles} className="proxemic-header-icon">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
        <polyline points="9,22 9,12 15,12 15,22" />
      </svg>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <polyline points="6,9 12,15 18,9" />
      </svg>
    </button>
  );
};

// Collaborator Avatars Component
const CollaboratorAvatars: React.FC = () => {
  const avatarGroupStyles = {
    display: 'flex',
    alignItems: 'center',
  };

  const avatarStyles = {
    width: '28px',
    height: '28px',
    borderRadius: '50%',
    backgroundColor: designTokens.colors.accent.ai,
    border: `2px solid ${designTokens.colors.surface.primary}`,
    marginLeft: '-8px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: designTokens.typography.fontSize.xs,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.surface.primary,
  };

  return (
    <div style={avatarGroupStyles}>
      <div style={avatarStyles} className="proxemic-avatar-status">
        JD
      </div>
    </div>
  );
};

// AI Status Indicator Component
const AIStatusIndicator: React.FC = () => {
  const [isThinking, setIsThinking] = React.useState(false);

  const indicatorStyles = {
    width: '12px',
    height: '12px',
    borderRadius: '50%',
    backgroundColor: isThinking ? designTokens.colors.accent.ai : designTokens.colors.confidence.high,
    animation: isThinking ? 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite' : 'none',
    cursor: 'pointer',
  };

  React.useEffect(() => {
    // Simulate AI thinking state
    const interval = setInterval(() => {
      setIsThinking(prev => !prev);
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  return (
    <>
      <style>
        {`
          @keyframes pulse {
            0%, 100% {
              opacity: 1;
            }
            50% {
              opacity: 0.5;
            }
          }
        `}
      </style>

      <div
        style={indicatorStyles}
        title={isThinking ? 'AI is thinking...' : 'AI ready'}
        aria-label={isThinking ? 'AI is thinking' : 'AI ready'}
      />
    </>
  );
};

// Settings Button Component
const SettingsButton: React.FC = () => {
  const buttonStyles = {
    background: 'none',
    border: 'none',
    color: designTokens.colors.text.secondary,
    cursor: 'pointer',
    padding: designTokens.spacing[2],
    borderRadius: designTokens.borderRadius.md,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: `all ${designTokens.animation.duration.fast}`,
  };

  return (
    <button
      style={buttonStyles}
      className="proxemic-header-icon"
      aria-label="Settings"
      onClick={() => {
        // TODO: Open settings modal
        console.log('Open settings');
      }}
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1" />
      </svg>
    </button>
  );
};

export default HeaderBar;