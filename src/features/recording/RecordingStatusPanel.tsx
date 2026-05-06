import type { RecordingStatus } from '../../lib/tauriClient';
import styles from './RecordingStatusPanel.module.css';

type RecordingStatusPanelProps = {
  status: RecordingStatus;
  loadState: 'loading' | 'ready' | 'error';
  errorMessage?: string;
};

export function RecordingStatusPanel({ status, loadState, errorMessage }: RecordingStatusPanelProps) {
  return (
    <div className={styles.panel} aria-label="Recording status">
      <span
        className={[styles.statusDot, status.isRecording ? styles.statusDotRecording : ''].filter(Boolean).join(' ')}
        aria-hidden="true"
      />
      <div className={styles.content}>
        <p className={styles.label}>Recorder status</p>
        <p className={styles.value}>{getStatusLabel(status, loadState)}</p>
        {status.activeSessionId && (
          <p className={styles.meta}>Active session: {status.activeSessionId}</p>
        )}
        <dl className={styles.stats}>
          <div>
            <dt>Elapsed</dt>
            <dd>{formatElapsedTime(status.elapsedSeconds ?? 0)}</dd>
          </div>
          <div>
            <dt>Steps</dt>
            <dd>{status.stepCount}</dd>
          </div>
        </dl>
        {loadState === 'error' && (
          <p className={styles.error}>{errorMessage ?? 'Recording status is available when running in Tauri.'}</p>
        )}
      </div>
    </div>
  );
}

function getStatusLabel(status: RecordingStatus, loadState: RecordingStatusPanelProps['loadState']): string {
  if (loadState === 'loading') {
    return 'Loading recorder status…';
  }

  return status.isRecording ? 'Recording' : 'Not recording';
}

function formatElapsedTime(totalSeconds: number): string {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds));
  const minutes = Math.floor(safeSeconds / 60);
  const seconds = safeSeconds % 60;

  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}
