import { appRoutes } from './routes';
import styles from './App.module.css';

export function App() {
  const homeRoute = appRoutes[0];

  return (
    <div className={styles.appShell}>
      <header className={styles.topBar}>
        <a className={styles.logo} href={homeRoute.path} aria-label="Steps Recorder home">
          <span className={styles.logoMark} aria-hidden="true" />
          Steps Recorder
        </a>
        <nav className={styles.nav} aria-label="Primary navigation">
          <a className={styles.navLink} href={homeRoute.path}>{homeRoute.label}</a>
          <a className={styles.navLink} href="#settings">Settings</a>
        </nav>
      </header>
      <main className={styles.main}>{homeRoute.element}</main>
    </div>
  );
}
