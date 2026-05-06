import { Button } from '../../components/button/Button';
import styles from './RecordingControlBar.module.css';

type RecordingControlBarProps = {
  isRecording: boolean;
  isBusy: boolean;
  activeSessionId?: string;
  onStartRecording: () => void;
  onStopRecording: () => void;
};

export function RecordingControlBar({
  isRecording,
  isBusy,
  activeSessionId,
  onStartRecording,
  onStopRecording,
}: RecordingControlBarProps) {
  return (
    <div className={styles.controlBar} aria-label="Recording controls">
      {isRecording ? (
        <Button disabled={isBusy || !activeSessionId} onClick={onStopRecording} variant="ghost">
          {isBusy ? 'Stopping…' : 'Stop Recording'}
        </Button>
      ) : (
        <Button disabled={isBusy} onClick={onStartRecording}>
          {isBusy ? 'Starting…' : 'Start Recording'}
        </Button>
      )}
      <Button variant="ghost">Review recent sessions</Button>
    </div>
  );
}
