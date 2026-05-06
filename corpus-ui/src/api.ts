// Typed API client for corpus-server

export interface ImageMeta {
  id: string;
  index: number;
  original_name: string;
  filename: string;
}

export interface Corpus {
  id: string;
  name: string;
  created_at: string;
  images: ImageMeta[];
}

export interface JobState {
  id: string;
  corpus_id: string;
  status: 'pending' | 'running' | 'done' | 'failed';
  total: number;
  done: number;
  errors: string[];
}

export interface UploadResponse {
  job_id: string;
  total: number;
}

const BASE = '/api';

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status}: ${text}`);
  }
  return res.json();
}

export const api = {
  listCorpora: (): Promise<Corpus[]> =>
    fetch(`${BASE}/corpora`).then(r => json<Corpus[]>(r)),

  createCorpus: (name: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    }).then(r => json<Corpus>(r)),

  getCorpus: (id: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${id}`).then(r => json<Corpus>(r)),

  renameCorpus: (id: string, name: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    }).then(r => json<Corpus>(r)),

  deleteCorpus: (id: string): Promise<void> =>
    fetch(`${BASE}/corpora/${id}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error(`${r.status}`);
    }),

  uploadImages: (corpusId: string, files: FileList | File[], config?: any): Promise<UploadResponse> => {
    const form = new FormData();
    Array.from(files).forEach(f => form.append('file', f, f.name));
    if (config) {
      form.append('config', JSON.stringify(config));
    }
    return fetch(`${BASE}/corpora/${corpusId}/images`, {
      method: 'POST',
      body: form,
    }).then(r => json<UploadResponse>(r));
  },

  imageUrl: (corpusId: string, imgId: string): string =>
    `${BASE}/corpora/${corpusId}/images/${imgId}`,

  reorderImage: (corpusId: string, imgId: string, index: number): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${corpusId}/images/${imgId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ index }),
    }).then(r => json<Corpus>(r)),

  deleteImage: (corpusId: string, imgId: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${corpusId}/images/${imgId}`, {
      method: 'DELETE',
    }).then(r => json<Corpus>(r)),

  exportUrl: (corpusId: string): string =>
    `${BASE}/corpora/${corpusId}/export`,

  exportPdfUrl: (corpusId: string): string =>
    `${BASE}/corpora/${corpusId}/export/pdf`,

  getJob: (jobId: string): Promise<JobState> =>
    fetch(`${BASE}/jobs/${jobId}`).then(r => json<JobState>(r)),
};
