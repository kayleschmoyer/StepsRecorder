import { useEffect, useMemo, useState } from 'react';
import { RecordingHomePage } from '../features/recording/RecordingHomePage';
import { SessionDetailPage } from '../features/sessions/SessionDetailPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { appRoutes } from './routes';
import styles from './App.module.css';

type AppView =
  | { name: 'home' }
  | { name: 'settings' }
  | { name: 'session'; sessionId: string };

export function App() {
  const [hash, setHash] = useState(window.location.hash || '#/');
  const homeRoute = appRoutes[0];
  const view = useMemo(() => parseHash(hash), [hash]);

  useEffect(() => {
    function handleHashChange() {
      setHash(window.location.hash || '#/');
    }

    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  return (
    <div className={styles.appShell}>
      <header className={styles.topBar}>
        <a className={styles.logo} href="#/" aria-label="Steps Recorder home">
          <span className={styles.logoMark} aria-hidden="true" />
          Steps Recorder
        </a>
        <nav className={styles.nav} aria-label="Primary navigation">
          <a className={styles.navLink} href="#/">{homeRoute.label}</a>
          <a className={styles.navLink} href="#/settings">Settings</a>
        </nav>
      </header>
      <main className={styles.main}>{renderView(view)}</main>
    </div>
  );
}

function renderView(view: AppView) {
  if (view.name === 'settings') {
    return <SettingsPage />;
  }

  if (view.name === 'session') {
    return <SessionDetailPage sessionId={view.sessionId} />;
  }

  return <RecordingHomePage />;
}

function parseHash(hash: string): AppView {
  const normalizedHash = hash.replace(/^#/, '') || '/';
  const sessionMatch = normalizedHash.match(/^\/sessions\/([^/]+)$/);

  if (normalizedHash === '/settings') {
    return { name: 'settings' };
  }

  if (sessionMatch) {
    return { name: 'session', sessionId: decodeURIComponent(sessionMatch[1]) };
  }

  return { name: 'home' };
}
