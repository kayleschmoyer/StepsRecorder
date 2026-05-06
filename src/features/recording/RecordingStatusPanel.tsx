import styles from './RecordingStatusPanel.module.css';

export function RecordingStatusPanel() {
  return (
    <div className={styles.panel} aria-label="Recording status">
      <span className={styles.statusDot} aria-hidden="true" />
      <div>
        <p className={styles.label}>Recorder status</p>
        <p className={styles.value}>Ready for a future native capture step</p>
      </div>
    </div>
  );
}
