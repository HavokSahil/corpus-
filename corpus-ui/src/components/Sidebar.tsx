import { useState, useEffect, useRef, useMemo } from 'react';
import Fuse from 'fuse.js';
import { api } from '../api';
import type { Corpus } from '../api';

interface Props {
  onSelect: (corpus: Corpus | null) => void;
  selected: string | null;
  refresh: number;
}

export function Sidebar({ onSelect, selected, refresh }: Props) {
  const [corpora, setCorpora] = useState<Corpus[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [newName, setNewName] = useState('');
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    api.listCorpora().then(setCorpora).catch(console.error);
  }, [refresh]);

  useEffect(() => {
    if (showModal) setTimeout(() => inputRef.current?.focus(), 50);
  }, [showModal]);

  const fuse = useMemo(() => new Fuse(corpora, {
    keys: ['name'],
    threshold: 0.4,
  }), [corpora]);

  const filteredCorpora = useMemo(() => {
    if (!search.trim()) return corpora;
    return fuse.search(search).map(r => r.item);
  }, [fuse, search, corpora]);

  const handleCreate = async () => {
    if (!newName.trim()) return;
    setLoading(true);
    try {
      const corpus = await api.createCorpus(newName.trim());
      setCorpora(prev => [...prev, corpus]);
      setShowModal(false);
      setNewName('');
      onSelect(corpus);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="sidebar-brand">
            <div className="logo">C+</div>
            <h1>corpus<span>+</span></h1>
          </div>
          <button className="btn-new-corpus" onClick={() => setShowModal(true)}>
            <span>＋</span> New Corpus
          </button>
        </div>

        <div className="sidebar-search">
          <div className="search-container">
            <span className="search-icon">🔍</span>
            <input
              type="text"
              className="search-input"
              placeholder="Search corpora…"
              value={search}
              onChange={e => setSearch(e.target.value)}
            />
          </div>
        </div>

        <div className="sidebar-list">
          {filteredCorpora.length === 0 && (
            <div style={{ padding: '16px', color: 'var(--text-3)', fontSize: 12, textAlign: 'center' }}>
              {search ? 'No matches found' : 'No corpora yet'}
            </div>
          )}
          {filteredCorpora.map(c => (
            <div
              key={c.id}
              className={`corpus-item ${selected === c.id ? 'active' : ''}`}
              onClick={() => onSelect(c)}
            >
              <span className="corpus-icon">📂</span>
              <div className="corpus-item-info">
                <div className="corpus-item-name">{c.name}</div>
                <div className="corpus-item-count">{c.images.length} image{c.images.length !== 1 ? 's' : ''}</div>
              </div>
            </div>
          ))}
        </div>
      </aside>

      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <h2>Create New Corpus</h2>
            <input
              ref={inputRef}
              type="text"
              placeholder="Corpus name…"
              value={newName}
              onChange={e => setNewName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleCreate()}
            />
            <div className="modal-actions">
              <button className="btn btn-ghost" onClick={() => setShowModal(false)}>Cancel</button>
              <button className="btn btn-primary" onClick={handleCreate} disabled={loading || !newName.trim()}>
                {loading ? 'Creating…' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
