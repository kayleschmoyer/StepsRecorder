import { type FormEvent, useEffect, useMemo, useState } from 'react';
import { Button } from '../../components/button/Button';
import { Card } from '../../components/card/Card';
import { formatDateLabel } from '../../lib/dateFormat';
import { tauriClient, type RecordingStep, type SessionDetail } from '../../lib/tauriClient';
import styles from './SessionDetailPage.module.css';

type StepDraft = {
  title: string;
  description: string;
};

type StepDrafts = Record<string, StepDraft>;

type SessionDetailPageProps = {
  sessionId: string;
};

export function SessionDetailPage({ sessionId }: SessionDetailPageProps) {
  const [session, setSession] = useState<SessionDetail | null>(null);
  const [stepDrafts, setStepDrafts] = useState<StepDrafts>({});
  const [status, setStatus] = useState<'loading' | 'ready' | 'saving' | 'error'>('loading');
  const [message, setMessage] = useState('Loading session…');

  useEffect(() => {
    let isMounted = true;

    setStatus('loading');
    setMessage('Loading session…');
    tauriClient
      .getSession({ sessionId })
      .then((loadedSession) => {
        if (!isMounted) {
          return;
        }

        setSession(loadedSession);
        setStepDrafts(createStepDrafts(loadedSession.steps));
        setStatus('ready');
        setMessage('Session loaded from SQLite.');
      })
      .catch(() => {
        if (!isMounted) {
          return;
        }

        setStatus('error');
        setMessage('Session detail loads when the app is running in Tauri and the session exists.');
      });

    return () => {
      isMounted = false;
    };
  }, [sessionId]);

  const capturedAtLabel = useMemo(() => {
    if (!session) {
      return '';
    }

    return formatValueDate(session.endedAt ?? session.startedAt);
  }, [session]);

  async function handleStepSubmit(stepId: string, event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const draft = stepDrafts[stepId];

    if (!draft || draft.title.trim().length === 0) {
      setStatus('error');
      setMessage('Step title cannot be empty.');
      return;
    }

    setStatus('saving');
    setMessage('Saving step…');

    try {
      const updatedStep = await tauriClient.updateStep({
        stepId,
        title: draft.title.trim(),
        description: draft.description.trim(),
      });

      setSession((current) => {
        if (!current) {
          return current;
        }

        return {
          ...current,
          steps: current.steps.map((step) => (step.id === updatedStep.id ? updatedStep : step)),
        };
      });
      setStepDrafts((current) => ({ ...current, [updatedStep.id]: stepToDraft(updatedStep) }));
      setStatus('ready');
      setMessage('Step saved.');
    } catch {
      setStatus('error');
      setMessage('Step could not be saved.');
    }
  }

  async function handleDeleteStep(stepId: string) {
    if (!session) {
      return;
    }

    setStatus('saving');
    setMessage('Deleting step…');

    try {
      await tauriClient.deleteStep({ stepId });
      setSession({ ...session, steps: session.steps.filter((step) => step.id !== stepId) });
      setStepDrafts((current) => {
        const nextDrafts = { ...current };
        delete nextDrafts[stepId];
        return nextDrafts;
      });
      setStatus('ready');
      setMessage('Step deleted.');
    } catch {
      setStatus('error');
      setMessage('Step could not be deleted.');
    }
  }

  async function handleMoveStep(stepId: string, direction: 'up' | 'down') {
    if (!session) {
      return;
    }

    const currentIndex = session.steps.findIndex((step) => step.id === stepId);
    const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;

    if (currentIndex < 0 || targetIndex < 0 || targetIndex >= session.steps.length) {
      return;
    }

    const reorderedSteps = [...session.steps];
    const [movedStep] = reorderedSteps.splice(currentIndex, 1);
    reorderedSteps.splice(targetIndex, 0, movedStep);

    setStatus('saving');
    setMessage('Reordering steps…');

    try {
      const result = await tauriClient.reorderSteps({
        sessionId: session.id,
        orderedStepIds: reorderedSteps.map((step) => step.id),
      });
      setSession({ ...session, steps: result.steps });
      setStepDrafts(createStepDrafts(result.steps));
      setStatus('ready');
      setMessage('Step order saved.');
    } catch {
      setStatus('error');
      setMessage('Step order could not be saved.');
    }
  }

  return (
    <div className={styles.page}>
      <section className={styles.hero}>
        <a className={styles.backLink} href="#/">← Recent Sessions</a>
        <p className={styles.eyebrow}>Session Review</p>
        <h1 className={styles.title}>{session?.title ?? 'Loading session'}</h1>
        <p className={styles.description}>
          {session?.description || 'Review captured steps, edit safe text metadata, delete mistakes, or reorder steps before export is implemented.'}
        </p>
      </section>

      <Card className={styles.summaryPanel} aria-labelledby="session-summary-title">
        <div>
          <p className={styles.eyebrow}>Session status</p>
          <h2 id="session-summary-title" className={styles.panelTitle}>{session?.status ?? status}</h2>
        </div>
        <dl className={styles.metaList}>
          <div>
            <dt>Captured</dt>
            <dd>{capturedAtLabel || '—'}</dd>
          </div>
          <div>
            <dt>Active steps</dt>
            <dd>{session?.steps.length ?? 0}</dd>
          </div>
          <div>
            <dt>Command state</dt>
            <dd className={status === 'error' ? styles.errorText : undefined}>{message}</dd>
          </div>
        </dl>
      </Card>

      <section className={styles.stepsSection} aria-labelledby="session-steps-title">
        <div className={styles.sectionHeader}>
          <p className={styles.eyebrow}>Review Steps</p>
          <h2 id="session-steps-title" className={styles.panelTitle}>Captured workflow</h2>
        </div>

        {status === 'loading' && <p className={styles.emptyState}>Loading steps…</p>}

        {session && session.steps.length === 0 && (
          <Card className={styles.emptyCard}>
            <h3>No steps captured yet</h3>
            <p>
              This session exists, but it has no active steps. Native mouse capture and screenshot capture remain out of
              scope for Step 3.
            </p>
          </Card>
        )}

        <ol className={styles.stepList}>
          {session?.steps.map((step, index) => {
            const draft = stepDrafts[step.id] ?? stepToDraft(step);

            return (
              <li className={styles.stepCard} key={step.id}>
                <div className={styles.stepHeader}>
                  <div>
                    <p className={styles.stepNumber}>Step {step.stepNumber}</p>
                    <h3>{step.title}</h3>
                    <p>{formatValueDate(step.capturedAt)}</p>
                  </div>
                  <div className={styles.stepActions}>
                    <Button disabled={index === 0 || status === 'saving'} variant="ghost" onClick={() => handleMoveStep(step.id, 'up')}>
                      Move up
                    </Button>
                    <Button
                      disabled={index === session.steps.length - 1 || status === 'saving'}
                      variant="ghost"
                      onClick={() => handleMoveStep(step.id, 'down')}
                    >
                      Move down
                    </Button>
                  </div>
                </div>

                <div className={styles.screenshotPlaceholder}>
                  <span>No screenshot preview in Step 3</span>
                  <small>Original path: {step.originalScreenshotPath}</small>
                  {step.editedScreenshotPath ? <small>Edited path: {step.editedScreenshotPath}</small> : null}
                </div>

                <form className={styles.editForm} onSubmit={(event) => handleStepSubmit(step.id, event)}>
                  <label className={styles.field}>
                    <span>Title</span>
                    <input
                      value={draft.title}
                      onChange={(event) => updateStepDraft(step.id, { title: event.target.value })}
                    />
                  </label>
                  <label className={styles.field}>
                    <span>Description</span>
                    <textarea
                      rows={3}
                      value={draft.description}
                      onChange={(event) => updateStepDraft(step.id, { description: event.target.value })}
                    />
                  </label>
                  <div className={styles.formActions}>
                    <Button disabled={status === 'saving'} type="submit">Save step</Button>
                    <Button disabled={status === 'saving'} variant="text" onClick={() => handleDeleteStep(step.id)}>
                      Delete step
                    </Button>
                  </div>
                </form>
              </li>
            );
          })}
        </ol>
      </section>
    </div>
  );

  function updateStepDraft(stepId: string, patch: Partial<StepDraft>) {
    setStepDrafts((current) => ({
      ...current,
      [stepId]: {
        ...(current[stepId] ?? { title: '', description: '' }),
        ...patch,
      },
    }));
  }
}

function createStepDrafts(steps: RecordingStep[]): StepDrafts {
  return Object.fromEntries(steps.map((step) => [step.id, stepToDraft(step)]));
}

function stepToDraft(step: RecordingStep): StepDraft {
  return {
    title: step.title,
    description: step.description ?? '',
  };
}

function formatValueDate(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return formatDateLabel(date);
}
