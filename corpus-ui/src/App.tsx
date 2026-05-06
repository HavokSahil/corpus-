import { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { CorpusView } from './components/CorpusView';
import { JobProgress } from './components/JobProgress';
import { api } from './api';
import type { Corpus } from './api';

function App() {
  const [selectedCorpus, setSelectedCorpus] = useState<Corpus | null>(null);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [refreshSidebar, setRefreshSidebar] = useState(0);

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

  return (
    <div className="layout">
      <Sidebar
        selected={selectedCorpus?.id || null}
        onSelect={setSelectedCorpus}
        refresh={refreshSidebar}
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
