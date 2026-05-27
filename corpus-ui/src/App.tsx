import { useState, useEffect, useCallback } from 'react';
import { Sidebar } from './components/Sidebar';
import { CorpusView } from './components/CorpusView';
import { JobProgress } from './components/JobProgress';
import { LoginPage } from './components/LoginPage';
import { api, auth } from './api';
import type { Corpus } from './api';

function App() {
  const [authenticated, setAuthenticated] = useState<boolean | null>(null); // null = checking
  const [selectedCorpus, setSelectedCorpus] = useState<Corpus | null>(null);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [refreshSidebar, setRefreshSidebar] = useState(0);

  // Check auth status on mount.
  useEffect(() => {
    if (!auth.isLoggedIn()) {
      setAuthenticated(false);
      return;
    }
    auth.check()
      .then((res) => setAuthenticated(res.authenticated))
      .catch(() => setAuthenticated(false));
  }, []);

  // Listen for 401 events from the API layer.
  useEffect(() => {
    const handler = () => setAuthenticated(false);
    window.addEventListener('corpus-auth-expired', handler);
    return () => window.removeEventListener('corpus-auth-expired', handler);
  }, []);

  const handleLogin = useCallback(() => {
    setAuthenticated(true);
  }, []);

  const handleLogout = useCallback(async () => {
    await auth.logout();
    setAuthenticated(false);
    setSelectedCorpus(null);
  }, []);

  useEffect(() => {
    if (selectedCorpus) {
      api.getCorpus(selectedCorpus.id)
        .then(setSelectedCorpus)
        .catch(() => setSelectedCorpus(null));
    }
  }, [activeJobId]); // Refresh current corpus when a job runs/finishes

  const handleCorpusUpdate = (c: Corpus) => {
    setSelectedCorpus(c);
    setRefreshSidebar(n => n + 1);
  };

  const handleCorpusDelete = (id: string) => {
    if (selectedCorpus?.id === id) {
      setSelectedCorpus(null);
    }
    setRefreshSidebar(n => n + 1);
  };

  const handleJobComplete = () => {
    if (selectedCorpus) {
      api.getCorpus(selectedCorpus.id).then(setSelectedCorpus);
    }
    setRefreshSidebar(n => n + 1);
  };

  // Still checking auth status — show nothing (prevents flash).
  if (authenticated === null) {
    return (
      <div className="login-page">
        <div className="login-orb login-orb-1" />
        <div className="login-orb login-orb-2" />
        <div className="login-orb login-orb-3" />
        <div className="login-card" style={{ border: 'none', background: 'transparent' }}>
          <div className="login-logo">
            <div className="login-logo-icon">C+</div>
          </div>
          <div className="login-spinner" style={{ margin: '32px auto' }} />
        </div>
      </div>
    );
  }

  // Not authenticated — show login.
  if (!authenticated) {
    return <LoginPage onLogin={handleLogin} />;
  }

  // Authenticated — show the app.
  return (
    <div className="layout">
      <Sidebar
        selected={selectedCorpus?.id || null}
        onSelect={setSelectedCorpus}
        refresh={refreshSidebar}
        onLogout={handleLogout}
      />
      <main className="main">
        {selectedCorpus ? (
          <CorpusView
            corpus={selectedCorpus}
            onUpdate={handleCorpusUpdate}
            onDelete={handleCorpusDelete}
            onUploadStarted={setActiveJobId}
          />
        ) : (
          <div className="empty-state">
            <div className="empty-icon">📁</div>
            <h2>Select or Create a Corpus</h2>
            <p>A corpus is a collection of scanned pages. Group related documents together.</p>
          </div>
        )}
      </main>
      
      {activeJobId && (
        <JobProgress
          jobId={activeJobId}
          onComplete={handleJobComplete}
        />
      )}
    </div>
  );
}

export default App;
