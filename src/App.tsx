import React from 'react';
import AppShell from './components/layout/AppShell';
import HeaderBar from './components/layout/HeaderBar';
import { Card, Button, Badge, Progress } from './components/ui';

const App: React.FC = () => {
  return (
    <AppShell>
      {/* Header */}
      <AppShell.Header>
        <HeaderBar />
      </AppShell.Header>

      {/* Main Content Area */}
      <AppShell.Main>
        {/* Navigation Panel */}
        <AppShell.Navigation>
          <NavigationContent />
        </AppShell.Navigation>

        {/* Document Canvas */}
        <AppShell.Canvas>
          <CanvasContent />
        </AppShell.Canvas>

        {/* Intelligence Panel */}
        <AppShell.Intelligence>
          <IntelligenceContent />
        </AppShell.Intelligence>
      </AppShell.Main>
    </AppShell>
  );
};

// Navigation Panel Content
const NavigationContent: React.FC = () => {
  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div>
        <h3 style={{ fontSize: '14px', fontWeight: '600', color: '#ffffff', marginBottom: '12px' }}>
          Workspace Intelligence
        </h3>
        <Progress value={92} variant="health" showLabel label="Health Score" showPercentage />
        <div style={{ fontSize: '12px', color: '#a8a8a8', marginTop: '8px' }}>
          47 documents analyzed
        </div>
        <div style={{ fontSize: '12px', color: '#a8a8a8' }}>
          3 recommendations
        </div>
      </div>

      <div>
        <h3 style={{ fontSize: '14px', fontWeight: '600', color: '#ffffff', marginBottom: '12px' }}>
          Active Documents
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: '#00ff88' }} />
            <span style={{ fontSize: '13px', color: '#ffffff' }}>user-guide.docx</span>
            <Badge variant="ai" size="sm">editing</Badge>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: '#6b6b6b' }} />
            <span style={{ fontSize: '13px', color: '#a8a8a8' }}>technical-spec.pdf</span>
            <Badge variant="default" size="sm">referenced</Badge>
          </div>
        </div>
      </div>

      <div>
        <h3 style={{ fontSize: '14px', fontWeight: '600', color: '#ffffff', marginBottom: '12px' }}>
          Recent Conversations
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ fontSize: '13px', color: '#a8a8a8', cursor: 'pointer' }}>
            "Update training materials"
          </div>
          <div style={{ fontSize: '13px', color: '#a8a8a8', cursor: 'pointer' }}>
            "Compare API documentation"
          </div>
          <div style={{ fontSize: '13px', color: '#00d4ff', cursor: 'pointer' }}>
            View all...
          </div>
        </div>
      </div>

      <div>
        <h3 style={{ fontSize: '14px', fontWeight: '600', color: '#ffffff', marginBottom: '12px' }}>
          Smart Collections
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '13px' }}>📊</span>
            <span style={{ fontSize: '13px', color: '#a8a8a8' }}>Needs Review</span>
            <Badge variant="warning" size="sm">3</Badge>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '13px' }}>🔄</span>
            <span style={{ fontSize: '13px', color: '#a8a8a8' }}>Recently Updated</span>
            <Badge variant="success" size="sm">8</Badge>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '13px' }}>⚠️</span>
            <span style={{ fontSize: '13px', color: '#a8a8a8' }}>Outdated Content</span>
            <Badge variant="error" size="sm">2</Badge>
          </div>
        </div>
      </div>
    </div>
  );
};

// Document Canvas Content
const CanvasContent: React.FC = () => {
  return (
    <div style={{
      height: '100%',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      gap: '32px',
      textAlign: 'center'
    }}>
      <div>
        <h1 style={{
          fontSize: '32px',
          fontWeight: '300',
          color: '#0a0a0b',
          marginBottom: '16px',
          letterSpacing: '-0.025em'
        }}>
          Welcome back. What would you like to work on today?
        </h1>

        <Card variant="glass" padding="lg" style={{ maxWidth: '500px', margin: '0 auto' }}>
          <h3 style={{
            fontSize: '16px',
            fontWeight: '600',
            color: '#0a0a0b',
            marginBottom: '16px'
          }}>
            Continue from yesterday:
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', alignItems: 'flex-start' }}>
            <Button variant="ghost" size="sm">• Update user guide</Button>
            <Button variant="ghost" size="sm">• Review API changes</Button>
          </div>
        </Card>
      </div>

      <div style={{
        width: '100%',
        maxWidth: '400px',
        padding: '24px',
        border: '2px dashed #e0e0e0',
        borderRadius: '12px',
        color: '#6b6b6b',
        fontSize: '16px'
      }}>
        Type or drag documents here...
      </div>

      <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', justifyContent: 'center' }}>
        <Button variant="primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14,2 14,8 20,8" />
          </svg>
          Import Document
        </Button>
        <Button variant="secondary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <path d="M21 21l-4.35-4.35" />
          </svg>
          Search Content
        </Button>
        <Button variant="ghost">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
          </svg>
          AI Assistant
        </Button>
      </div>
    </div>
  );
};

// Intelligence Panel Content
const IntelligenceContent: React.FC = () => {
  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div>
        <h3 style={{ fontSize: '14px', fontWeight: '600', color: '#ffffff', marginBottom: '12px' }}>
          AI Assistant
        </h3>

        <Card variant="glass" padding="md">
          <div style={{ marginBottom: '16px' }}>
            <div style={{ fontSize: '14px', color: '#ffffff', marginBottom: '8px' }}>
              How can I help with your documents today?
            </div>
            <Button variant="ghost" size="sm" fullWidth>
              Suggested: Compare recent changes...
            </Button>
          </div>

          <div style={{
            display: 'flex',
            gap: '8px',
            padding: '8px',
            backgroundColor: 'rgba(255, 255, 255, 0.05)',
            borderRadius: '8px',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          }}>
            <input
              style={{
                flex: 1,
                background: 'none',
                border: 'none',
                color: '#ffffff',
                fontSize: '14px',
                outline: 'none'
              }}
              placeholder="Type a message..."
            />
            <Button variant="primary" size="sm">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="22" y1="2" x2="11" y2="13" />
                <polygon points="22,2 15,22 11,13 2,9 22,2" />
              </svg>
            </Button>
          </div>
        </Card>
      </div>

      <div>
        <h3 style={{ fontSize: '14px', fontWeight: '600', color: '#ffffff', marginBottom: '12px' }}>
          Document Analysis
        </h3>

        <Card variant="default" padding="md">
          <div style={{ marginBottom: '16px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
              <span style={{ fontSize: '12px', color: '#a8a8a8' }}>Structure</span>
              <span style={{ fontSize: '12px', color: '#ffffff' }}>100%</span>
            </div>
            <Progress value={100} variant="confidence" size="sm" />
          </div>

          <div style={{ marginBottom: '16px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
              <span style={{ fontSize: '12px', color: '#a8a8a8' }}>Clarity</span>
              <span style={{ fontSize: '12px', color: '#ffffff' }}>82%</span>
            </div>
            <Progress value={82} variant="confidence" size="sm" />
          </div>

          <div style={{ marginBottom: '16px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
              <span style={{ fontSize: '12px', color: '#a8a8a8' }}>Completeness</span>
              <span style={{ fontSize: '12px', color: '#ffffff' }}>85%</span>
            </div>
            <Progress value={85} variant="confidence" size="sm" />
          </div>

          <div style={{ fontSize: '12px', color: '#a8a8a8', marginBottom: '8px' }}>
            24 concepts identified<br />
            12 procedures mapped<br />
            3 gaps detected
          </div>

          <div style={{ display: 'flex', gap: '8px' }}>
            <Button variant="ghost" size="sm">View Details</Button>
            <Button variant="secondary" size="sm">Get Suggestions</Button>
          </div>
        </Card>
      </div>
    </div>
  );
};

export default App;