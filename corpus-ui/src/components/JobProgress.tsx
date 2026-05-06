import { useEffect, useState, useRef } from 'react';
import { api } from '../api';
import type { JobState } from '../api';

interface Props {
  jobId: string;
  onComplete: () => void;
}

export function JobProgress({ jobId, onComplete }: Props) {
  const [job, setJob] = useState<JobState | null>(null);
  const onCompleteRef = useRef(onComplete);

  // Keep the ref updated so we don't need it in the dependency array
  useEffect(() => {
    onCompleteRef.current = onComplete;
  }, [onComplete]);

  useEffect(() => {
    let interval: ReturnType<typeof setInterval>;
    let hasCompleted = false;

    const poll = async () => {
      try {
        const j = await api.getJob(jobId);
        setJob(j);
        if (j.status === 'done' || j.status === 'failed') {
          clearInterval(interval);
          if (!hasCompleted && (j.status === 'done' || j.done > 0)) {
            hasCompleted = true;
            onCompleteRef.current();
          }
          // Hide toast after 3 seconds
          setTimeout(() => setJob(null), 3000);
        }
      } catch (e) {
        console.error('Job poll error', e);
        clearInterval(interval);
      }
    };

    poll(); // Initial check
    interval = setInterval(poll, 1000);

    return () => clearInterval(interval);
  }, [jobId]);

  if (!job) return null;

  const pct = job.total === 0 ? 0 : Math.round((job.done / job.total) * 100);
  const isDone = job.status === 'done' || job.status === 'failed';

  return (
    <div className="job-toast">
      <div className="job-toast-header">
        <div className="job-toast-title">
          {isDone ? 'Processing Complete' : 'Processing Images'}
        </div>
        <div className="job-toast-status">{pct}%</div>
      </div>
      <div className="progress-bar-bg">
        <div
          className={`progress-bar-fill ${isDone ? 'done' : ''}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <div className="job-toast-meta">
        {job.done} / {job.total} images scanned
        {job.errors.length > 0 && ` • ${job.errors.length} failed`}
      </div>
    </div>
  );
}
