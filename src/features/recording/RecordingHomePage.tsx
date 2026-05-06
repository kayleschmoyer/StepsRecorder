import { Button } from '../../components/button/Button';
import { Card } from '../../components/card/Card';
import { PageSection } from '../../components/layout/PageSection';
import type { RecentSessionSummary } from './recordingTypes';
import { RecordingControlBar } from './RecordingControlBar';
import { RecordingStatusPanel } from './RecordingStatusPanel';
import styles from './RecordingHomePage.module.css';

const recentSessions: RecentSessionSummary[] = [
  {
    id: 'placeholder-session-1',
    title: 'No saved sessions yet',
    stepCount: 0,
    updatedAtLabel: 'Start a recording to populate this list',
  },
];

const workflowCards = [
  {
    title: 'Start Recording',
    description: 'Native mouse and screenshot capture will be connected in a later step.',
  },
  {
    title: 'Recent Sessions',
    description: 'Review captured walkthroughs after SQLite-backed sessions are added.',
  },
  {
    title: 'Privacy Reminder',
    description: 'Only record workflows that are safe to capture. Close sensitive windows first.',
  },
  {
    title: 'Settings shortcut',
    description: 'Configure capture preferences once settings storage is implemented.',
  },
];

export function RecordingHomePage() {
  return (
    <div className={styles.page}>
      <section className={styles.hero}>
        <p className={styles.eyebrow}>Windows documentation workflow</p>
        <h1 className={styles.title}>Capture polished process steps without breaking flow.</h1>
        <p className={styles.description}>
          Steps Recorder will turn Windows clicks and screenshots into editable sessions. This foundation keeps
          native capture behind a typed Tauri boundary until the safe Rust implementation is added.
        </p>
        <div className={styles.heroActions}>
          <RecordingControlBar />
          <RecordingStatusPanel />
        </div>
      </section>

      <PageSection
        eyebrow="Step 1 foundation"
        title="Home screen placeholders"
        description="The app shell is ready for recording, session review, privacy, and settings flows without enabling native capture yet."
      >
        <div className={styles.cardGrid}>
          {workflowCards.map((card) => (
            <Card className={styles.workflowCard} key={card.title}>
              <div className={styles.cardAccent} aria-hidden="true" />
              <h3 className={styles.cardTitle}>{card.title}</h3>
              <p className={styles.cardDescription}>{card.description}</p>
            </Card>
          ))}
        </div>
      </PageSection>

      <section className={styles.dashboardGrid}>
        <Card className={styles.recentPanel} aria-labelledby="recent-sessions-title">
          <div className={styles.panelHeader}>
            <div>
              <p className={styles.panelEyebrow}>Recent Sessions</p>
              <h2 id="recent-sessions-title" className={styles.panelTitle}>Latest recordings</h2>
            </div>
            <Button variant="text">View all</Button>
          </div>
          <ul className={styles.sessionList}>
            {recentSessions.map((session) => (
              <li className={styles.sessionItem} key={session.id}>
                <span>
                  <strong>{session.title}</strong>
                  <small>{session.updatedAtLabel}</small>
                </span>
                <span className={styles.stepCount}>{session.stepCount} steps</span>
              </li>
            ))}
          </ul>
        </Card>

        <Card className={styles.privacyPanel} aria-labelledby="privacy-title">
          <p className={styles.panelEyebrow}>Privacy Reminder</p>
          <h2 id="privacy-title" className={styles.panelTitle}>Prepare before recording</h2>
          <p className={styles.cardDescription}>
            Screenshots can include private data. Before starting a session, move passwords, tokens, customer data,
            and personal documents away from the visible workspace.
          </p>
          <a className={styles.settingsLink} id="settings" href="#settings">Open Settings shortcut</a>
        </Card>
      </section>
    </div>
  );
}
