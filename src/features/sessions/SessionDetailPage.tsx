import { type FormEvent, type PointerEvent, useEffect, useMemo, useRef, useState } from 'react';
import { Button } from '../../components/button/Button';
import { Card } from '../../components/card/Card';
import { formatDateLabel } from '../../lib/dateFormat';
import { tauriClient, type RecordingStep, type SessionDetail, type StepScreenshotPreview } from '../../lib/tauriClient';
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
  const [editingStep, setEditingStep] = useState<RecordingStep | null>(null);

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


  function handleEditedScreenshotSaved(updatedStep: RecordingStep) {
    setSession((current) => {
      if (!current) {
        return current;
      }

      return {
        ...current,
        steps: current.steps.map((step) => (step.id === updatedStep.id ? updatedStep : step)),
      };
    });
    setEditingStep(null);
    setStatus('ready');
    setMessage('Edited screenshot saved. Original screenshot preserved.');
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
              This session exists, but it has no active steps. Start a recording and click in another application. Accepted native clicks are persisted with visible-monitor screenshots when capture succeeds.
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
                    <Button disabled={status === 'saving' || step.originalScreenshotPath.trim().length === 0} variant="ghost" onClick={() => setEditingStep(step)}>
                      Edit screenshot
                    </Button>
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

                <StepScreenshotPreviewPanel step={step} />

                <dl className={styles.stepMetadata}>
                  <div>
                    <dt>Click position</dt>
                    <dd>{formatClickPosition(step)}</dd>
                  </div>
                  <div>
                    <dt>Monitor</dt>
                    <dd>{step.monitorId ?? 'Unavailable'}</dd>
                  </div>
                  <div>
                    <dt>Application</dt>
                    <dd title={step.processName ?? undefined}>{step.processName ?? 'Unknown application'}</dd>
                  </div>
                  <div>
                    <dt>Window title</dt>
                    <dd title={step.appWindowTitle ?? undefined}>{step.appWindowTitle ?? 'Unavailable'}</dd>
                  </div>
                </dl>

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

      {editingStep ? (
        <ScreenshotEditorModal
          step={editingStep}
          onCancel={() => {
            setEditingStep(null);
            setMessage('Screenshot editing canceled. No changes were saved.');
          }}
          onSaved={handleEditedScreenshotSaved}
        />
      ) : null}
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


type ScreenshotPreviewState =
  | { status: 'missing'; dataUrl?: undefined }
  | { status: 'loading'; dataUrl?: undefined }
  | { status: 'ready'; dataUrl: string; previewKind: 'original' | 'click_marker' | 'edited'; displayedScreenshotPath: string; editedScreenshotPath?: string }
  | { status: 'error'; dataUrl?: undefined };

function StepScreenshotPreviewPanel({ step }: { step: RecordingStep }) {
  const [preview, setPreview] = useState<ScreenshotPreviewState>({
    status: step.originalScreenshotPath.trim().length > 0 ? 'loading' : 'missing',
  });

  useEffect(() => {
    let isMounted = true;

    if (step.originalScreenshotPath.trim().length === 0) {
      setPreview({ status: 'missing' });
      return () => {
        isMounted = false;
      };
    }

    setPreview({ status: 'loading' });
    tauriClient
      .getStepScreenshotPreview({ stepId: step.id })
      .then((result) => {
        if (!isMounted) {
          return;
        }

        if (result.exists && result.dataUrl) {
          setPreview({
            status: 'ready',
            dataUrl: result.dataUrl,
            previewKind: normalizePreviewKind(result.previewKind),
            displayedScreenshotPath: result.displayedScreenshotPath ?? result.originalScreenshotPath,
            editedScreenshotPath: result.editedScreenshotPath,
          });
        } else {
          setPreview({ status: 'missing' });
        }
      })
      .catch(() => {
        if (isMounted) {
          setPreview({ status: 'error' });
        }
      });

    return () => {
      isMounted = false;
    };
  }, [step.id, step.originalScreenshotPath, step.editedScreenshotPath]);

  if (preview.status === 'ready') {
    const isMarkedPreview = preview.previewKind === 'click_marker';
    const isEditedPreview = preview.previewKind === 'edited';

    return (
      <figure className={styles.screenshotPreview}>
        <img src={preview.dataUrl} alt={`Visible monitor screenshot for step ${step.stepNumber}`} />
        <figcaption>
          <span className={styles.previewStatusList}>
            <span>Original screenshot preserved</span>
            <span>{isEditedPreview ? 'Edited screenshot shown' : isMarkedPreview ? 'Click marker screenshot shown' : 'Original screenshot shown'}</span>
          </span>
          <span className={styles.previewPath}>Original: {step.originalScreenshotPath}</span>
          {isMarkedPreview ? <span className={styles.previewPath}>Marked preview: {preview.displayedScreenshotPath}</span> : null}
          {isEditedPreview ? <span className={styles.previewPath}>Edited: {preview.displayedScreenshotPath}</span> : null}
        </figcaption>
      </figure>
    );
  }

  const message = preview.status === 'loading'
    ? 'Loading screenshot preview…'
    : preview.status === 'error'
      ? 'Screenshot preview could not be loaded.'
      : 'Screenshot missing or capture failed.';

  return (
    <div className={styles.screenshotPlaceholder}>
      <span>{message}</span>
      <small>
        {step.originalScreenshotPath.trim().length > 0
          ? `Expected original path: ${step.originalScreenshotPath}`
          : 'No original screenshot path has been stored for this step yet.'}
      </small>
      {step.editedScreenshotPath ? <small>Edited path: {step.editedScreenshotPath}</small> : null}
    </div>
  );
}


type EditorTool = 'redact' | 'crop';

type EditorRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type EditorStatus = 'loading' | 'ready' | 'saving' | 'error';

function ScreenshotEditorModal({
  step,
  onCancel,
  onSaved,
}: {
  step: RecordingStep;
  onCancel: () => void;
  onSaved: (step: RecordingStep) => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const [tool, setTool] = useState<EditorTool>('redact');
  const [redactions, setRedactions] = useState<EditorRect[]>([]);
  const [cropRect, setCropRect] = useState<EditorRect | null>(null);
  const [activeRect, setActiveRect] = useState<EditorRect | null>(null);
  const [source, setSource] = useState<StepScreenshotPreview | null>(null);
  const [status, setStatus] = useState<EditorStatus>('loading');
  const [message, setMessage] = useState('Loading screenshot for editing…');

  useEffect(() => {
    let isMounted = true;

    setStatus('loading');
    setMessage('Loading screenshot for editing…');
    tauriClient
      .getStepScreenshotPreview({ stepId: step.id })
      .then((preview) => {
        if (!isMounted) {
          return;
        }

        if (!preview.exists || !preview.dataUrl) {
          setStatus('error');
          setMessage('Screenshot cannot be edited because no image file is available.');
          return;
        }

        const image = document.createElement('img');
        image.onload = () => {
          if (!isMounted) {
            return;
          }

          imageRef.current = image;
          setSource(preview);
          setStatus('ready');
          setMessage('Draw a redaction rectangle or crop area, then save the edited copy.');
        };
        image.onerror = () => {
          if (isMounted) {
            setStatus('error');
            setMessage('Screenshot image could not be loaded into the editor.');
          }
        };
        image.src = preview.dataUrl;
      })
      .catch(() => {
        if (isMounted) {
          setStatus('error');
          setMessage('Screenshot editor could not load the step screenshot.');
        }
      });

    return () => {
      isMounted = false;
    };
  }, [step.id]);

  useEffect(() => {
    redrawEditorCanvas(canvasRef.current, imageRef.current, redactions, cropRect, activeRect, tool);
  }, [source, redactions, cropRect, activeRect, tool]);

  function handlePointerDown(event: PointerEvent<HTMLCanvasElement>) {
    if (status !== 'ready') {
      return;
    }

    const point = pointerToCanvasPoint(event);
    dragStartRef.current = point;
    setActiveRect({ ...point, width: 0, height: 0 });
  }

  function handlePointerMove(event: PointerEvent<HTMLCanvasElement>) {
    const start = dragStartRef.current;
    if (!start) {
      return;
    }

    setActiveRect(normalizeRect(start, pointerToCanvasPoint(event)));
  }

  function handlePointerUp(event: PointerEvent<HTMLCanvasElement>) {
    const start = dragStartRef.current;
    if (!start) {
      return;
    }

    dragStartRef.current = null;
    const rect = normalizeRect(start, pointerToCanvasPoint(event));
    setActiveRect(null);

    if (!isMeaningfulRect(rect)) {
      return;
    }

    if (tool === 'redact') {
      setRedactions((current) => [...current, rect]);
    } else {
      setCropRect(rect);
    }
  }

  async function handleSave() {
    const image = imageRef.current;
    if (!image) {
      setStatus('error');
      setMessage('Screenshot image is not ready yet.');
      return;
    }

    setStatus('saving');
    setMessage('Saving edited screenshot copy…');

    try {
      const dataUrl = renderEditedScreenshot(image, redactions, cropRect);
      const updatedStep = await tauriClient.saveEditedScreenshot({
        stepId: step.id,
        screenshotDataUrl: dataUrl,
      });
      onSaved(updatedStep);
    } catch {
      setStatus('error');
      setMessage('Edited screenshot could not be saved. Existing screenshot paths were left unchanged.');
    }
  }

  function pointerToCanvasPoint(event: PointerEvent<HTMLCanvasElement>) {
    const canvas = event.currentTarget;
    const bounds = canvas.getBoundingClientRect();
    return {
      x: clamp(((event.clientX - bounds.left) / bounds.width) * canvas.width, 0, canvas.width),
      y: clamp(((event.clientY - bounds.top) / bounds.height) * canvas.height, 0, canvas.height),
    };
  }

  const isSaving = status === 'saving';
  const canSave = status === 'ready' && (redactions.length > 0 || cropRect !== null);
  const sourceKind = source?.previewKind === 'edited'
    ? 'existing edited screenshot'
    : source?.previewKind === 'click_marker'
      ? 'click-marker screenshot'
      : 'original screenshot';

  return (
    <div className={styles.modalBackdrop} role="presentation">
      <section className={styles.editorModal} role="dialog" aria-modal="true" aria-labelledby={`screenshot-editor-${step.id}`}>
        <div className={styles.editorHeader}>
          <div>
            <p className={styles.eyebrow}>Screenshot Editor</p>
            <h2 id={`screenshot-editor-${step.id}`}>Edit screenshot for Step {step.stepNumber}</h2>
            <p>Original screenshot preserved. Edited screenshot shown after save.</p>
          </div>
          <Button disabled={status === 'saving'} variant="text" onClick={onCancel}>Cancel</Button>
        </div>

        <div className={styles.editorToolbar} aria-label="Screenshot editing tools">
          <Button variant={tool === 'redact' ? 'primary' : 'ghost'} disabled={status === 'saving'} onClick={() => setTool('redact')}>Redaction rectangle</Button>
          <Button variant={tool === 'crop' ? 'primary' : 'ghost'} disabled={status === 'saving'} onClick={() => setTool('crop')}>Crop region</Button>
          <Button variant="text" disabled={status === 'saving' || redactions.length === 0} onClick={() => setRedactions((current) => current.slice(0, -1))}>Undo redaction</Button>
          <Button variant="text" disabled={status === 'saving' || cropRect === null} onClick={() => setCropRect(null)}>Clear crop</Button>
        </div>

        <div className={styles.editorInfo}>
          <span>Editing source: {sourceKind}</span>
          <span>Original: {step.originalScreenshotPath}</span>
          {source?.displayedScreenshotPath ? <span>Current editor image: {source.displayedScreenshotPath}</span> : null}
        </div>

        <div className={styles.editorCanvasShell}>
          {status === 'loading' ? <p>{message}</p> : null}
          {status !== 'loading' ? (
            <canvas
              ref={canvasRef}
              className={styles.editorCanvas}
              onPointerDown={handlePointerDown}
              onPointerMove={handlePointerMove}
              onPointerUp={handlePointerUp}
              onPointerCancel={() => {
                dragStartRef.current = null;
                setActiveRect(null);
              }}
            />
          ) : null}
        </div>

        <p className={status === 'error' ? styles.errorText : styles.editorMessage}>{message}</p>

        <div className={styles.editorActions}>
          <Button disabled={!canSave || isSaving} onClick={handleSave}>Save edited screenshot</Button>
          <Button disabled={isSaving} variant="ghost" onClick={onCancel}>Cancel without saving</Button>
        </div>
      </section>
    </div>
  );
}

function redrawEditorCanvas(
  canvas: HTMLCanvasElement | null,
  image: HTMLImageElement | null,
  redactions: EditorRect[],
  cropRect: EditorRect | null,
  activeRect: EditorRect | null,
  tool: EditorTool,
) {
  if (!canvas || !image) {
    return;
  }

  canvas.width = image.naturalWidth;
  canvas.height = image.naturalHeight;
  const context = canvas.getContext('2d');
  if (!context) {
    return;
  }

  context.clearRect(0, 0, canvas.width, canvas.height);
  context.drawImage(image, 0, 0);
  context.fillStyle = '#080808';
  for (const rect of redactions) {
    context.fillRect(rect.x, rect.y, rect.width, rect.height);
  }

  const visibleSelection = activeRect ?? cropRect;
  if (visibleSelection) {
    context.save();
    context.strokeStyle = tool === 'crop' || cropRect === visibleSelection ? '#f8f5ef' : '#080808';
    context.lineWidth = Math.max(2, canvas.width / 600);
    context.setLineDash(tool === 'crop' || cropRect === visibleSelection ? [12, 8] : []);
    context.strokeRect(visibleSelection.x, visibleSelection.y, visibleSelection.width, visibleSelection.height);
    if (tool === 'crop' || cropRect === visibleSelection) {
      context.fillStyle = 'rgba(248, 245, 239, 0.14)';
      context.fillRect(visibleSelection.x, visibleSelection.y, visibleSelection.width, visibleSelection.height);
    }
    context.restore();
  }
}

function renderEditedScreenshot(image: HTMLImageElement, redactions: EditorRect[], cropRect: EditorRect | null): string {
  const workingCanvas = document.createElement('canvas');
  workingCanvas.width = image.naturalWidth;
  workingCanvas.height = image.naturalHeight;
  const workingContext = workingCanvas.getContext('2d');
  if (!workingContext) {
    throw new Error('Canvas context unavailable.');
  }

  workingContext.drawImage(image, 0, 0);
  workingContext.fillStyle = '#080808';
  for (const rect of redactions) {
    workingContext.fillRect(rect.x, rect.y, rect.width, rect.height);
  }

  if (!cropRect) {
    return workingCanvas.toDataURL('image/png');
  }

  const crop = clampRectToImage(cropRect, workingCanvas.width, workingCanvas.height);
  const outputCanvas = document.createElement('canvas');
  outputCanvas.width = Math.max(1, Math.round(crop.width));
  outputCanvas.height = Math.max(1, Math.round(crop.height));
  const outputContext = outputCanvas.getContext('2d');
  if (!outputContext) {
    throw new Error('Canvas context unavailable.');
  }

  outputContext.drawImage(
    workingCanvas,
    crop.x,
    crop.y,
    crop.width,
    crop.height,
    0,
    0,
    outputCanvas.width,
    outputCanvas.height,
  );

  return outputCanvas.toDataURL('image/png');
}

function normalizeRect(start: { x: number; y: number }, end: { x: number; y: number }): EditorRect {
  return {
    x: Math.min(start.x, end.x),
    y: Math.min(start.y, end.y),
    width: Math.abs(end.x - start.x),
    height: Math.abs(end.y - start.y),
  };
}

function clampRectToImage(rect: EditorRect, width: number, height: number): EditorRect {
  const x = clamp(rect.x, 0, width);
  const y = clamp(rect.y, 0, height);
  return {
    x,
    y,
    width: clamp(rect.width, 1, width - x),
    height: clamp(rect.height, 1, height - y),
  };
}

function isMeaningfulRect(rect: EditorRect): boolean {
  return rect.width >= 4 && rect.height >= 4;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function normalizePreviewKind(kind: StepScreenshotPreview['previewKind']): 'original' | 'click_marker' | 'edited' {
  return kind === 'edited' || kind === 'click_marker' ? kind : 'original';
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

function formatClickPosition(step: RecordingStep): string {
  if (typeof step.clickX !== 'number' || typeof step.clickY !== 'number') {
    return 'Unavailable';
  }

  return `(${step.clickX}, ${step.clickY})`;
}

function formatValueDate(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return formatDateLabel(date);
}
