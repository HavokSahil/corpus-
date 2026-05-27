import { useState, useRef, useEffect } from 'react';
import { auth } from '../api';

interface LoginPageProps {
  onLogin: () => void;
}

export function LoginPage({ onLogin }: LoginPageProps) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [shake, setShake] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!password.trim()) return;

    setLoading(true);
    setError('');

    try {
      await auth.login(password);
      onLogin();
    } catch {
      setError('Incorrect password');
      setShake(true);
      setTimeout(() => setShake(false), 600);
      setPassword('');
      inputRef.current?.focus();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      {/* Animated background orbs */}
      <div className="login-orb login-orb-1" />
      <div className="login-orb login-orb-2" />
      <div className="login-orb login-orb-3" />

      <form
        className={`login-card ${shake ? 'login-shake' : ''}`}
        onSubmit={handleSubmit}
      >
        <div className="login-logo">
          <div className="login-logo-icon">C+</div>
        </div>
        <h1 className="login-title">Corpus<span>+</span></h1>
        <p className="login-subtitle">Enter your password to continue</p>

        <div className="login-field">
          <input
            ref={inputRef}
            id="login-password"
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
              setError('');
            }}
            disabled={loading}
            autoComplete="current-password"
          />
          <div className="login-field-glow" />
        </div>

        {error && (
          <div className="login-error">
            <span className="login-error-icon">⚠</span>
            {error}
          </div>
        )}

        <button
          id="login-submit"
          type="submit"
          className="login-btn"
          disabled={loading || !password.trim()}
        >
          {loading ? (
            <span className="login-spinner" />
          ) : (
            <>
              <span>Unlock</span>
              <span className="login-btn-arrow">→</span>
            </>
          )}
        </button>
      </form>
    </div>
  );
}
