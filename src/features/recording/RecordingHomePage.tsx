import { useEffect, useState } from 'react';
import { Card } from '../../components/card/Card';
import { PageSection } from '../../components/layout/PageSection';
import { formatDateLabel } from '../../lib/dateFormat';
import { tauriClient, type RecordingStatus } from '../../lib/tauriClient';
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

const idleRecordingStatus: RecordingStatus = {
  isRecording: false,
  stepCount: 0,
};

const workflowCards = [
  {
    title: 'Start Recording',
    description: 'Native mouse and screenshot capture will be connected in a later step.',
  },
  {
    title: 'Recent Sessions',
    description: 'Open recent SQLite-backed sessions for review once capture creates steps.',
  },
  {
    title: 'Privacy Reminder',
    description: 'Only record workflows that are safe to capture. Close sensitive windows first.',
  },
  {
    title: 'Settings shortcut',
    description: 'Configure capture preferences saved through the typed Tauri settings commands.',
  },
];

export function RecordingHomePage() {
  const [recentSessions, setRecentSessions] = useState<RecentSessionSummary[]>(emptyRecentSessions);
  const [recentSessionsStatus, setRecentSessionsStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [recordingStatus, setRecordingStatus] = useState<RecordingStatus>(idleRecordingStatus);
  const [recordingStatusState, setRecordingStatusState] = useState<'loading' | 'ready' | 'error'>('loading');
  const [recordingActionState, setRecordingActionState] = useState<'idle' | 'starting' | 'stopping'>('idle');
  const [recordingError, setRecordingError] = useState<string | undefined>();

  useEffect(() => {
    refreshRecentSessions();
    refreshRecordingStatus();
  }, []);

  useEffect(() => {
    if (!recordingStatus.isRecording) {
      return undefined;
    }

    const intervalId = window.setInterval(() => {
      refreshRecordingStatus();
    }, 1000);

    return () => window.clearInterval(intervalId);
  }, [recordingStatus.isRecording]);

  function refreshRecentSessions() {
    setRecentSessionsStatus('loading');

    tauriClient
      .listSessions({ limit: 5, includeArchived: false })
      .then((sessions) => {
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
        setRecentSessions(emptyRecentSessions);
        setRecentSessionsStatus('error');
      });
  }

  function refreshRecordingStatus() {
    tauriClient
      .getRecordingStatus()
      .then((status) => {
        setRecordingStatus(status);
        setRecordingStatusState('ready');
      })
      .catch(() => {
        setRecordingStatus(idleRecordingStatus);
        setRecordingStatusState('error');
      });
  }

  function handleStartRecording() {
    setRecordingActionState('starting');
    setRecordingError(undefined);

    tauriClient
      .startRecordingSession({})
      .then((session) => {
        setRecordingStatus({
          isRecording: session.status === 'recording',
          activeSessionId: session.id,
          elapsedSeconds: 0,
          stepCount: session.stepCount,
        });
        setRecordingStatusState('ready');
        refreshRecentSessions();
      })
      .catch((error: unknown) => {
        setRecordingError(getErrorMessage(error));
        refreshRecordingStatus();
      })
      .finally(() => setRecordingActionState('idle'));
  }

  function handleStopRecording() {
    if (!recordingStatus.activeSessionId) {
      return;
    }

    setRecordingActionState('stopping');
    setRecordingError(undefined);

    tauriClient
      .stopRecordingSession({ sessionId: recordingStatus.activeSessionId })
      .then(() => {
        setRecordingStatus(idleRecordingStatus);
        setRecordingStatusState('ready');
        refreshRecentSessions();
      })
      .catch((error: unknown) => {
        setRecordingError(getErrorMessage(error));
        refreshRecordingStatus();
      })
      .finally(() => setRecordingActionState('idle'));
  }

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
          <RecordingStatusPanel
            errorMessage={recordingError}
            loadState={recordingStatusState}
            status={recordingStatus}
          />
          <RecordingControlBar
            activeSessionId={recordingStatus.activeSessionId}
            isBusy={recordingActionState !== 'idle'}
            isRecording={recordingStatus.isRecording}
            onStartRecording={handleStartRecording}
            onStopRecording={handleStopRecording}
          />
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
            <a className={styles.panelLink} href="#/">View all</a>
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
                {session.id === 'empty-session-list' ? (
                  <span>
                    <strong>{session.title}</strong>
                    <small>{session.updatedAtLabel}</small>
                  </span>
                ) : (
                  <a className={styles.sessionLink} href={`#/sessions/${encodeURIComponent(session.id)}`}>
                    <strong>{session.title}</strong>
                    <small>{session.updatedAtLabel}</small>
                  </a>
                )}
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
          <a className={styles.settingsLink} id="settings" href="#/settings">Open Settings</a>
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

function getErrorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = (error as { message?: unknown }).message;

    if (typeof message === 'string' && message.length > 0) {
      return message;
    }
  }

  return 'The recording command could not be completed.';
}
