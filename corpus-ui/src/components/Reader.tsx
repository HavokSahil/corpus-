import { useState, useEffect, useCallback, useRef } from 'react';
import type { ImageMeta } from '../api';
import { api } from '../api';

interface ReaderProps {
  corpusId: string;
  images: ImageMeta[];
  initialIndex: number;
  onClose: () => void;
}

export function Reader({ corpusId, images, initialIndex, onClose }: ReaderProps) {
  const [currentIndex, setCurrentIndex] = useState(initialIndex);
  const contentRef = useRef<HTMLDivElement>(null);

  // Ensure images are sorted by their index property
  const sortedImages = [...images].sort((a, b) => a.index - b.index);

  const handlePrev = useCallback(() => {
    setCurrentIndex(prev => Math.max(0, prev - 1));
  }, []);

  const handleNext = useCallback(() => {
    setCurrentIndex(prev => Math.min(sortedImages.length - 1, prev + 1));
  }, [sortedImages.length]);

  // Reset scroll to top when page changes
  useEffect(() => {
    if (contentRef.current) {
      contentRef.current.scrollTop = 0;
    }
  }, [currentIndex]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === 'ArrowLeft') handlePrev();
      if (e.key === 'ArrowRight') handleNext();
      
      if (contentRef.current) {
        const scrollAmount = 100;
        if (e.key === 'ArrowUp') {
          contentRef.current.scrollBy({ top: -scrollAmount, behavior: 'smooth' });
        }
        if (e.key === 'ArrowDown') {
          contentRef.current.scrollBy({ top: scrollAmount, behavior: 'smooth' });
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose, handlePrev, handleNext]);

  const currentImage = sortedImages[currentIndex];
  if (!currentImage) return null;

  const imageUrl = api.imageUrl(corpusId, currentImage.id);

  return (
    <div className="reader-overlay">
      <div className="reader-top-bar">
        <div className="reader-page-info">
          Page {currentIndex + 1} of {sortedImages.length} — {currentImage.original_name}
        </div>
        <button className="reader-close" onClick={onClose} title="Close (Esc)">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      {currentIndex > 0 && (
        <button className="reader-nav prev" onClick={handlePrev} title="Previous (Left Arrow)">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
      )}

      {currentIndex < sortedImages.length - 1 && (
        <button className="reader-nav next" onClick={handleNext} title="Next (Right Arrow)">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      )}

      <div 
        ref={contentRef}
        className="reader-content" 
        onClick={(e) => {
          // Only close if clicking the actual backdrop, not the image
          if (e.target === e.currentTarget) onClose();
        }}
      >
        <img 
          key={currentImage.id}
          src={imageUrl} 
          alt={`Page ${currentIndex + 1}`}
          className="reader-image"
          onClick={e => e.stopPropagation()}
        />
      </div>
    </div>
  );
}
