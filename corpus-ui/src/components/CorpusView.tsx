import { useState, useRef, useEffect } from 'react';
import { api } from '../api';
import type { Corpus } from '../api';
import { ImageGrid } from './ImageGrid';
import { UploadModal } from './UploadModal';
import { Reader } from './Reader';

interface Props {
  corpus: Corpus;
  onUpdate: (c: Corpus) => void;
  onDelete: (id: string) => void;
  onUploadStarted: (jobId: string) => void;
}

export function CorpusView({ corpus, onUpdate, onDelete, onUploadStarted }: Props) {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(corpus.name);
  const [showUpload, setShowUpload] = useState(false);
  const [readingIndex, setReadingIndex] = useState<number | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setName(corpus.name);
  }, [corpus.id, corpus.name]);

  useEffect(() => {
    if (editing) inputRef.current?.focus();
  }, [editing]);

  const handleRename = async () => {
    setEditing(false);
    if (name.trim() && name !== corpus.name) {
      try {
        const updated = await api.renameCorpus(corpus.id, name.trim());
        onUpdate(updated);
      } catch (e) {
        console.error(e);
        setName(corpus.name);
      }
    } else {
      setName(corpus.name);
    }
  };

  const handleDelete = async () => {
    if (confirm(`Are you sure you want to delete the corpus "${corpus.name}" and all its images?`)) {
      try {
        await api.deleteCorpus(corpus.id);
        onDelete(corpus.id);
      } catch (e) {
        console.error(e);
        alert('Failed to delete corpus');
      }
    }
  };

  const formattedDate = new Date(corpus.created_at).toLocaleDateString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric'
  });

  return (
    <div className="corpus-view">
      <div className="corpus-header">
        <div className="corpus-header-info">
          <div className="corpus-name-row">
            {editing ? (
              <input
                ref={inputRef}
                className="corpus-name-input"
                value={name}
                onChange={e => setName(e.target.value)}
                onBlur={handleRename}
                onKeyDown={e => e.key === 'Enter' && handleRename()}
              />
            ) : (
              <div className="corpus-name-display" onClick={() => setEditing(true)}>
                {corpus.name}
              </div>
            )}
          </div>
          <div className="corpus-meta-row">
            Created {formattedDate} • {corpus.images.length} images
          </div>
        </div>
        <div className="corpus-actions">
          <button 
            className="btn btn-primary" 
            onClick={() => setReadingIndex(0)}
            disabled={corpus.images.length === 0}
          >
            <span>📖</span> Read Mode
          </button>
          <button className="btn btn-ghost" onClick={() => setShowUpload(true)}>
            <span>☁️</span> Upload
          </button>
          
          <div style={{ display: 'flex', gap: 4 }}>
            <a
              href={api.exportUrl(corpus.id)}
              className="btn btn-ghost"
              style={{ textDecoration: 'none' }}
              download
            >
              <span>📦</span> ZIP
            </a>
            <a
              href={api.exportPdfUrl(corpus.id)}
              className="btn btn-ghost"
              style={{ textDecoration: 'none' }}
              target="_blank"
              rel="noopener noreferrer"
            >
              <span>📄</span> PDF
            </a>
          </div>

          <button className="btn btn-danger" onClick={handleDelete} title="Delete Corpus">
            <span>🗑️</span>
          </button>
        </div>
      </div>

      <ImageGrid 
        corpus={corpus} 
        onCorpusUpdate={onUpdate} 
        onImageClick={(index) => setReadingIndex(index)}
      />

      {showUpload && (
        <UploadModal
          corpusId={corpus.id}
          onClose={() => setShowUpload(false)}
          onUploadStarted={onUploadStarted}
        />
      )}

      {readingIndex !== null && (
        <Reader
          corpusId={corpus.id}
          images={corpus.images}
          initialIndex={readingIndex}
          onClose={() => setReadingIndex(null)}
        />
      )}
    </div>
  );
}
