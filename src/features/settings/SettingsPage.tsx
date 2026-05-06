import { type FormEvent, useEffect, useState } from 'react';
import { Button } from '../../components/button/Button';
import { Card } from '../../components/card/Card';
import { tauriClient, type AppSettings, type ScreenshotMode } from '../../lib/tauriClient';
import styles from './SettingsPage.module.css';

type SettingsFormState = {
  screenshotMode: ScreenshotMode;
  clickDebounceMs: string;
  includeTimestampsInExport: boolean;
  includeClickMarkers: boolean;
  privacyReminderBeforeExport: boolean;
  defaultExportDirectory: string;
};

const emptyFormState: SettingsFormState = {
  screenshotMode: 'clicked_monitor',
  clickDebounceMs: '500',
  includeTimestampsInExport: true,
  includeClickMarkers: true,
  privacyReminderBeforeExport: true,
  defaultExportDirectory: '',
};

export function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [formState, setFormState] = useState<SettingsFormState>(emptyFormState);
  const [status, setStatus] = useState<'loading' | 'ready' | 'saving' | 'saved' | 'error'>('loading');
  const [message, setMessage] = useState('Loading settings from SQLite…');
  const [devFixtureStatus, setDevFixtureStatus] = useState<'idle' | 'saving' | 'error'>('idle');
  const [devFixtureMessage, setDevFixtureMessage] = useState('Developer-only fixture actions are available in local dev builds.');

  useEffect(() => {
    let isMounted = true;

    tauriClient
      .getSettings()
      .then((loadedSettings) => {
        if (!isMounted) {
          return;
        }

        setSettings(loadedSettings);
        setFormState(settingsToFormState(loadedSettings));
        setStatus('ready');
        setMessage('Settings loaded from the local app database.');
      })
      .catch(() => {
        if (!isMounted) {
          return;
        }

        setStatus('error');
        setMessage('Settings load when this page is running inside the Tauri app.');
      });

    return () => {
      isMounted = false;
    };
  }, []);


  async function handleSeedSampleData() {
    setDevFixtureStatus('saving');
    setDevFixtureMessage('Seeding development sample data…');

    try {
      const seededSession = await tauriClient.devSeedSampleData();
      setDevFixtureStatus('idle');
      setDevFixtureMessage(`Seeded ${seededSession.steps.length} placeholder steps. Open it from Recent Sessions or jump directly to Session Review.`);
    } catch {
      setDevFixtureStatus('error');
      setDevFixtureMessage('Dev seed command is only available in the Tauri debug app.');
    }
  }

  async function handleClearSeededData() {
    setDevFixtureStatus('saving');
    setDevFixtureMessage('Clearing development sample data…');

    try {
      const result = await tauriClient.devClearSeededData();
      setDevFixtureStatus('idle');
      setDevFixtureMessage(`Cleared ${result.deletedSessions} seeded session and ${result.deletedSteps} seeded steps.`);
    } catch {
      setDevFixtureStatus('error');
      setDevFixtureMessage('Dev clear command is only available in the Tauri debug app.');
    }
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const clickDebounceMs = Number.parseInt(formState.clickDebounceMs, 10);
    if (!Number.isFinite(clickDebounceMs) || clickDebounceMs < 0) {
      setStatus('error');
      setMessage('Click debounce must be a non-negative number of milliseconds.');
      return;
    }

    setStatus('saving');
    setMessage('Saving settings…');

    try {
      const updatedSettings = await tauriClient.updateSettings({
        screenshotMode: formState.screenshotMode,
        clickDebounceMs,
        includeTimestampsInExport: formState.includeTimestampsInExport,
        includeClickMarkers: formState.includeClickMarkers,
        privacyReminderBeforeExport: formState.privacyReminderBeforeExport,
        defaultExportDirectory: formState.defaultExportDirectory.trim(),
      });

      setSettings(updatedSettings);
      setFormState(settingsToFormState(updatedSettings));
      setStatus('saved');
      setMessage('Settings saved. Reload this page to verify they persist.');
    } catch {
      setStatus('error');
      setMessage('Settings could not be saved. Check that the Tauri backend is running.');
    }
  }

  return (
    <div className={styles.page}>
      <section className={styles.hero}>
        <p className={styles.eyebrow}>Application Settings</p>
        <h1 className={styles.title}>Tune safe capture defaults before native recording is enabled.</h1>
        <p className={styles.description}>
          These settings are loaded through the typed Tauri command layer and persisted to the existing SQLite
          app_settings table. Choose whether clicks capture the full clicked monitor or only the visible bounds of the clicked window.
        </p>
      </section>

      <Card className={styles.panel} aria-labelledby="settings-form-title">
        <div className={styles.panelHeader}>
          <div>
            <p className={styles.eyebrow}>SQLite-backed preferences</p>
            <h2 id="settings-form-title" className={styles.panelTitle}>Settings</h2>
          </div>
          <p className={status === 'error' ? styles.errorText : styles.statusText}>{message}</p>
        </div>

        <form className={styles.form} onSubmit={handleSubmit}>
          <label className={styles.field}>
            <span>Screenshot mode</span>
            <select
              value={formState.screenshotMode}
              onChange={(event) =>
                setFormState((current) => ({ ...current, screenshotMode: event.target.value as ScreenshotMode }))
              }
            >
              <option value="clicked_monitor">Clicked monitor (fallback)</option>
              <option value="clicked_window">Clicked window (Windows only)</option>
            </select>
            <small>
              Clicked window captures the visible screen rectangle for the top-level window under the click, including
              its title bar and borders. If that fails, capture falls back to clicked monitor.
            </small>
          </label>

          <label className={styles.field}>
            <span>Click debounce (ms)</span>
            <input
              min="0"
              type="number"
              value={formState.clickDebounceMs}
              onChange={(event) => setFormState((current) => ({ ...current, clickDebounceMs: event.target.value }))}
            />
          </label>

          <label className={styles.checkboxField}>
            <input
              checked={formState.includeTimestampsInExport}
              type="checkbox"
              onChange={(event) =>
                setFormState((current) => ({ ...current, includeTimestampsInExport: event.target.checked }))
              }
            />
            <span>Include timestamps in exports by default</span>
          </label>

          <label className={styles.checkboxField}>
            <input
              checked={formState.includeClickMarkers}
              type="checkbox"
              onChange={(event) => setFormState((current) => ({ ...current, includeClickMarkers: event.target.checked }))}
            />
            <span>Include click markers in exports by default</span>
          </label>

          <label className={styles.checkboxField}>
            <input
              checked={formState.privacyReminderBeforeExport}
              type="checkbox"
              onChange={(event) =>
                setFormState((current) => ({ ...current, privacyReminderBeforeExport: event.target.checked }))
              }
            />
            <span>Show privacy reminder before export</span>
          </label>

          <label className={styles.field}>
            <span>Default export directory</span>
            <input
              placeholder="Optional local folder path"
              value={formState.defaultExportDirectory}
              onChange={(event) => setFormState((current) => ({ ...current, defaultExportDirectory: event.target.value }))}
            />
          </label>

          <div className={styles.actions}>
            <Button disabled={status === 'loading' || status === 'saving'} type="submit">
              {status === 'saving' ? 'Saving…' : 'Save settings'}
            </Button>
            <a className={styles.backLink} href="#/">Back home</a>
          </div>
        </form>
      </Card>

      {import.meta.env.DEV && (
        <Card className={styles.devPanel} aria-labelledby="dev-fixtures-title">
          <div className={styles.panelHeader}>
            <div>
              <p className={styles.eyebrow}>Development only</p>
              <h2 id="dev-fixtures-title" className={styles.panelTitle}>Test fixtures</h2>
            </div>
            <p className={devFixtureStatus === 'error' ? styles.errorText : styles.statusText}>{devFixtureMessage}</p>
          </div>
          <p className={styles.devWarning}>
            Local debug tooling only: creates one deterministic sample session with three editable steps and placeholder
            screenshot path strings. It does not create image files or enable native capture.
          </p>
          <div className={styles.actions}>
            <Button disabled={devFixtureStatus === 'saving'} onClick={handleSeedSampleData}>
              Seed dev sample data
            </Button>
            <Button disabled={devFixtureStatus === 'saving'} variant="ghost" onClick={handleClearSeededData}>
              Clear dev sample data
            </Button>
            <a className={styles.backLink} href="#/sessions/dev-seed-session-settings-review">Open seeded Session Review</a>
          </div>
        </Card>
      )}
    </div>
  );
}

function settingsToFormState(settings: AppSettings): SettingsFormState {
  return {
    screenshotMode: settings.screenshotMode === 'clicked_window' ? 'clicked_window' : 'clicked_monitor',
    clickDebounceMs: String(settings.clickDebounceMs),
    includeTimestampsInExport: settings.includeTimestampsInExport,
    includeClickMarkers: settings.includeClickMarkers,
    privacyReminderBeforeExport: settings.privacyReminderBeforeExport,
    defaultExportDirectory: settings.defaultExportDirectory ?? '',
  };
}
