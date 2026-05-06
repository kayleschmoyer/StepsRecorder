import { Button } from '../../components/button/Button';
import styles from './RecordingControlBar.module.css';

export function RecordingControlBar() {
  return (
    <div className={styles.controlBar} aria-label="Recording controls">
      <Button>Start Recording</Button>
      <Button variant="ghost">Review recent sessions</Button>
    </div>
  );
}
