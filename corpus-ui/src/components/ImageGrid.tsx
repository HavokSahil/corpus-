import { useSortable } from '@dnd-kit/sortable';
import Fuse from 'fuse.js';
import { useState, useMemo } from 'react';
import { CSS } from '@dnd-kit/utilities';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import type { DragEndEvent } from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  rectSortingStrategy,
} from '@dnd-kit/sortable';

import { api } from '../api';
import type { Corpus, ImageMeta } from '../api';

interface Props {
  corpus: Corpus;
  onCorpusUpdate: (c: Corpus) => void;
  onImageClick: (index: number) => void;
}

function SortableItem({ 
  img, 
  corpusId, 
  onDelete, 
  onClick 
}: { 
  img: ImageMeta, 
  corpusId: string, 
  onDelete: (id: string) => void,
  onClick: () => void 
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: img.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: isDragging ? 10 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`image-card ${isDragging ? 'dragging' : ''}`}
      onClick={() => {
        // Prevent click if we were dragging
        if (!isDragging) onClick();
      }}
    >
      <div className="image-card-drag-handle" {...attributes} {...listeners}>
        <img
          className="image-card-thumb"
          src={api.imageUrl(corpusId, img.id)}
          alt={img.original_name}
          loading="lazy"
          draggable={false}
        />
      </div>
      <div className="image-card-body">
        <div className="image-card-index">{img.index}</div>
        <div className="image-card-name" title={img.original_name}>{img.original_name}</div>
        <button
          className="image-card-del"
          onPointerDown={e => e.stopPropagation()}
          onClick={(e) => {
            e.stopPropagation();
            if (confirm(`Delete ${img.original_name}?`)) onDelete(img.id);
          }}
          title="Delete image"
        >
          🗑️
        </button>
      </div>
    </div>
  );
}

export function ImageGrid({ corpus, onCorpusUpdate, onImageClick }: Props) {
  const [search, setSearch] = useState('');
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  const handleDelete = async (imgId: string) => {
    try {
      const updated = await api.deleteImage(corpus.id, imgId);
      onCorpusUpdate(updated);
    } catch (e) {
      console.error(e);
      alert('Delete failed');
    }
  };

  const fuse = useMemo(() => new Fuse(corpus.images, {
    keys: ['original_name'],
    threshold: 0.4,
  }), [corpus.images]);

  const filteredImages = useMemo(() => {
    if (!search.trim()) {
      return [...corpus.images].sort((a, b) => a.index - b.index);
    }
    return fuse.search(search).map(r => r.item);
  }, [fuse, search, corpus.images]);

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      const oldIndex = corpus.images.findIndex(i => i.id === active.id);
      const newIndex = corpus.images.findIndex(i => i.id === over.id);

      // Optimistic update
      const newImages = arrayMove(corpus.images, oldIndex, newIndex);
      // Re-index optimistically
      const optimisticCorpus = {
        ...corpus,
        images: newImages.map((img, i) => ({ ...img, index: i + 1 })),
      };
      onCorpusUpdate(optimisticCorpus);

      // Send to server: API expects 1-based index
      try {
        const updated = await api.reorderImage(corpus.id, active.id as string, newIndex + 1);
        onCorpusUpdate(updated);
      } catch (e) {
        console.error(e);
        // Revert on failure by re-fetching
        api.getCorpus(corpus.id).then(onCorpusUpdate);
      }
    }
  };

  if (corpus.images.length === 0) {
    return (
      <div className="empty-state" style={{ marginTop: 40 }}>
        <div className="empty-icon">🖼️</div>
        <h2>No images yet</h2>
        <p>Upload images or a zip archive to start scanning.</p>
      </div>
    );
  }


  return (
    <>
      <div className="image-grid-header">
        <div className="image-grid-title">
          Scanned Pages
          <span className="count-badge">{corpus.images.length}</span>
        </div>

        <div className="search-container image-grid-search">
          <span className="search-icon">🔍</span>
          <input
            type="text"
            className="search-input"
            placeholder="Search images by name…"
            value={search}
            onChange={e => setSearch(e.target.value)}
          />
        </div>
      </div>
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
        <SortableContext 
          items={filteredImages.map(i => i.id)} 
          strategy={rectSortingStrategy}
          disabled={!!search.trim()}
        >
          <div className="image-grid">
            {filteredImages.length === 0 && (
              <div className="empty-state" style={{ gridColumn: '1 / -1', marginTop: 20 }}>
                <p>No images match your search.</p>
              </div>
            )}
            {filteredImages.map((img) => (
              <SortableItem 
                key={img.id} 
                img={img} 
                corpusId={corpus.id} 
                onDelete={handleDelete}
                onClick={() => {
                  const idx = corpus.images.findIndex(i => i.id === img.id);
                  onImageClick(idx);
                }}
              />
            ))}
          </div>
        </SortableContext>
      </DndContext>
    </>
  );
}
