import { useEffect, useState } from 'react';
import { Button } from '../../components/button/Button';
import { Card } from '../../components/card/Card';
import { PageSection } from '../../components/layout/PageSection';
import { formatDateLabel } from '../../lib/dateFormat';
import { tauriClient } from '../../lib/tauriClient';
import type { RecentSessionSummary } from './recordingTypes';
import { RecordingControlBar } from './RecordingControlBar';
import { RecordingStatusPanel } from './RecordingStatusPanel';
import styles from './RecordingHomePage.module.css';

const emptyRecentSessions: RecentSessionSummary[] = [
  {
    id: 'empty-session-list',
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
  const [recentSessions, setRecentSessions] = useState<RecentSessionSummary[]>(emptyRecentSessions);
  const [recentSessionsStatus, setRecentSessionsStatus] = useState<'loading' | 'ready' | 'error'>('loading');

  useEffect(() => {
    let isMounted = true;

    tauriClient
      .listSessions({ limit: 5, includeArchived: false })
      .then((sessions) => {
        if (!isMounted) {
          return;
        }

        setRecentSessions(
          sessions.length > 0
            ? sessions.map((session) => ({
                id: session.id,
                title: session.title,
                stepCount: session.stepCount,
                updatedAtLabel: formatSessionDateLabel(session.endedAt ?? session.startedAt),
              }))
            : emptyRecentSessions,
        );
        setRecentSessionsStatus('ready');
      })
      .catch(() => {
        if (!isMounted) {
          return;
        }

        setRecentSessions(emptyRecentSessions);
        setRecentSessionsStatus('error');
      });

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <div className={styles.page}>
      <section className={styles.hero}>
        <div className={styles.heroCopy}>
          <p className={styles.eyebrow}>Windows documentation workflow</p>
          <h1 className={styles.title}>Capture polished process steps without breaking flow.</h1>
          <p className={styles.description}>
            Steps Recorder will turn Windows clicks and screenshots into editable sessions. This shell keeps native
            capture behind a typed Tauri boundary until the safe Rust implementation is added.
          </p>
        </div>

        <Card className={styles.readyPanel} aria-labelledby="ready-to-record-title">
          <div className={styles.readyHeader}>
            <p className={styles.panelEyebrow}>Recorder workspace</p>
            <h2 id="ready-to-record-title" className={styles.panelTitle}>Ready to Record</h2>
          </div>
          <RecordingStatusPanel />
          <RecordingControlBar />
          <p className={styles.privacyNote}>
            Privacy reminder: close passwords, customer data, and personal documents before starting a session.
          </p>
        </Card>
      </section>

      <PageSection
        eyebrow="Step 1 foundation"
        title="Secondary placeholders"
        description="These cards reserve space for future recording, session review, privacy, and settings flows without enabling native capture yet."
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
          {recentSessionsStatus === 'loading' && (
            <p className={styles.sessionStatus}>Loading recent sessions…</p>
          )}
          {recentSessionsStatus === 'error' && (
            <p className={styles.sessionStatus}>Recent sessions load when the app is running in Tauri.</p>
          )}
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

function formatSessionDateLabel(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return formatDateLabel(date);
}
