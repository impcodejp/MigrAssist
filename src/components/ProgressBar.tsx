import type { ProgressPayload } from '../types';

interface Props {
  progress: ProgressPayload;
}

export function ProgressBar({ progress }: Props) {
  const pct =
    progress.total > 0
      ? Math.round((progress.step / progress.total) * 100)
      : 0;

  return (
    <div className="progress-container">
      <div className="progress-message">{progress.message}</div>
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      <div className="progress-pct">
        {progress.step} / {progress.total}
      </div>
    </div>
  );
}
