import { useState, useRef } from 'react';
import { api } from '../api';

interface Props {
  corpusId: string;
  onClose: () => void;
  onUploadStarted: (jobId: string) => void;
}

export function UploadModal({ corpusId, onClose, onUploadStarted }: Props) {
  const [dragOver, setDragOver] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [config, setConfig] = useState({
    enhance_mode: 'binary',
    use_morphology: false,
    max_width: 1200,
    canny_low: 40,
    canny_high: 120,
    adaptive_block_radius: 20,
    adaptive_c: 8,
  });

  const fileInput = useRef<HTMLInputElement>(null);

  const handleFiles = async (files: FileList | null) => {
    if (!files || files.length === 0) return;
    setUploading(true);
    try {
      const res = await api.uploadImages(corpusId, files, config);
      onUploadStarted(res.job_id);
      onClose();
    } catch (e) {
      console.error(e);
      alert('Upload failed: ' + String(e));
      setUploading(false);
    }
  };

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    handleFiles(e.dataTransfer.files);
  };

  return (
    <div className="modal-overlay" onClick={!uploading ? onClose : undefined}>
      <div className="modal upload-modal" onClick={e => e.stopPropagation()}>
        <h2>Upload Images</h2>
        
        <div
          className={`upload-zone ${dragOver ? 'drag-over' : ''}`}
          onDragOver={e => { e.preventDefault(); setDragOver(true); }}
          onDragLeave={() => setDragOver(false)}
          onDrop={onDrop}
          onClick={() => fileInput.current?.click()}
        >
          <div className="upload-icon">📁</div>
          <p>Drag and drop images or a .zip archive here</p>
          <p style={{ marginTop: 8 }}><span>Browse files</span></p>
          <input
            ref={fileInput}
            type="file"
            multiple
            accept="image/*,.zip"
            onChange={e => handleFiles(e.target.files)}
          />
        </div>

        <div className="advanced-toggle" onClick={() => setShowAdvanced(!showAdvanced)}>
          {showAdvanced ? '− Hide' : '+ Show'} Advanced Settings
        </div>

        {showAdvanced && (
          <div className="advanced-options">
            <div className="config-item" style={{ marginBottom: 20 }}>
              <label>Enhance Mode</label>
              <div className="mode-selector">
                {['binary', 'grayscale', 'color'].map(m => (
                  <button
                    key={m}
                    className={`mode-btn ${config.enhance_mode === m ? 'active' : ''}`}
                    onClick={() => setConfig({ ...config, enhance_mode: m })}
                  >
                    {m}
                  </button>
                ))}
              </div>
            </div>

            <div className="config-item" style={{ marginBottom: 24, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between' }}>
              <label style={{ margin: 0 }}>Noise Reduction (Morphology)</label>
              <input 
                type="checkbox" 
                className="toggle-input"
                checked={config.use_morphology} 
                onChange={e => setConfig({...config, use_morphology: e.target.checked})} 
              />
            </div>
            
            <div className="config-grid">
              <div className="config-item">
                <label>Max Width (px)</label>
                <input 
                  type="number" 
                  value={config.max_width} 
                  onChange={e => setConfig({...config, max_width: parseInt(e.target.value)})} 
                />
              </div>
              <div className="config-item">
                <label>Canny High</label>
                <input 
                  type="range" min="0" max="255"
                  value={config.canny_high} 
                  onChange={e => setConfig({...config, canny_high: parseInt(e.target.value)})} 
                />
                <span>{config.canny_high}</span>
              </div>
              <div className="config-item">
                <label>Adaptive Radius</label>
                <input 
                  type="number" 
                  value={config.adaptive_block_radius} 
                  onChange={e => setConfig({...config, adaptive_block_radius: parseInt(e.target.value)})} 
                />
              </div>
              <div className="config-item">
                <label>Adaptive C</label>
                <input 
                  type="number" 
                  value={config.adaptive_c} 
                  onChange={e => setConfig({...config, adaptive_c: parseInt(e.target.value)})} 
                />
              </div>
            </div>
          </div>
        )}

        <div className="modal-actions">
          <button className="btn btn-ghost" onClick={onClose} disabled={uploading}>Cancel</button>
          <button className="btn btn-primary" onClick={() => fileInput.current?.click()} disabled={uploading}>
            {uploading ? 'Processing...' : 'Upload'}
          </button>
        </div>
      </div>
    </div>
  );
}
