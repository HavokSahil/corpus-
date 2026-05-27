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

export interface LoginResponse {
  token: string;
  expires_in: number;
}

export interface CheckResponse {
  authenticated: boolean;
  remaining_secs: number;
}

const BASE = '/api';

// ── Token Management ─────────────────────────────────────────────

const TOKEN_KEY = 'corpus_auth_token';

function getToken(): string | null {
  return sessionStorage.getItem(TOKEN_KEY);
}

function setToken(token: string): void {
  sessionStorage.setItem(TOKEN_KEY, token);
}

function clearToken(): void {
  sessionStorage.removeItem(TOKEN_KEY);
}

function authHeaders(): Record<string, string> {
  const token = getToken();
  if (token) {
    return { Authorization: `Bearer ${token}` };
  }
  return {};
}

// ── Response Helpers ─────────────────────────────────────────────

async function json<T>(res: Response): Promise<T> {
  if (res.status === 401) {
    clearToken();
    // Signal to the app that we're no longer authenticated.
    window.dispatchEvent(new CustomEvent('corpus-auth-expired'));
    throw new Error('Unauthorized');
  }
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status}: ${text}`);
  }
  return res.json();
}

// ── Auth API ─────────────────────────────────────────────────────

export const auth = {
  login: (password: string): Promise<LoginResponse> =>
    fetch(`${BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password }),
    }).then(async (res) => {
      if (res.status === 401) {
        throw new Error('Invalid password');
      }
      const data = await json<LoginResponse>(res);
      setToken(data.token);
      return data;
    }),

  logout: (): Promise<void> =>
    fetch(`${BASE}/auth/logout`, {
      method: 'POST',
      headers: { ...authHeaders() },
    }).then(() => {
      clearToken();
    }),

  check: (): Promise<CheckResponse> =>
    fetch(`${BASE}/auth/check`, {
      headers: { ...authHeaders() },
    }).then((res) => {
      if (res.status === 401) {
        clearToken();
        return { authenticated: false, remaining_secs: 0 };
      }
      return res.json();
    }),

  isLoggedIn: (): boolean => getToken() !== null,
};

// ── Corpus / Image API ───────────────────────────────────────────

export const api = {
  listCorpora: (): Promise<Corpus[]> =>
    fetch(`${BASE}/corpora`, { headers: authHeaders() }).then(r => json<Corpus[]>(r)),

  createCorpus: (name: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ name }),
    }).then(r => json<Corpus>(r)),

  getCorpus: (id: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${id}`, { headers: authHeaders() }).then(r => json<Corpus>(r)),

  renameCorpus: (id: string, name: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ name }),
    }).then(r => json<Corpus>(r)),

  deleteCorpus: (id: string): Promise<void> =>
    fetch(`${BASE}/corpora/${id}`, { method: 'DELETE', headers: authHeaders() }).then(r => {
      if (r.status === 401) {
        clearToken();
        window.dispatchEvent(new CustomEvent('corpus-auth-expired'));
        throw new Error('Unauthorized');
      }
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
      headers: authHeaders(),
      body: form,
    }).then(r => json<UploadResponse>(r));
  },

  imageUrl: (corpusId: string, imgId: string): string => {
    const token = getToken();
    return `${BASE}/corpora/${corpusId}/images/${imgId}${token ? `?token=${token}` : ''}`;
  },

  reorderImage: (corpusId: string, imgId: string, index: number): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${corpusId}/images/${imgId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ index }),
    }).then(r => json<Corpus>(r)),

  deleteImage: (corpusId: string, imgId: string): Promise<Corpus> =>
    fetch(`${BASE}/corpora/${corpusId}/images/${imgId}`, {
      method: 'DELETE',
      headers: authHeaders(),
    }).then(r => json<Corpus>(r)),

  exportUrl: (corpusId: string): string => {
    const token = getToken();
    return `${BASE}/corpora/${corpusId}/export${token ? `?token=${token}` : ''}`;
  },

  exportPdfUrl: (corpusId: string): string => {
    const token = getToken();
    return `${BASE}/corpora/${corpusId}/export/pdf${token ? `?token=${token}` : ''}`;
  },

  getJob: (jobId: string): Promise<JobState> =>
    fetch(`${BASE}/jobs/${jobId}`, { headers: authHeaders() }).then(r => json<JobState>(r)),
};
