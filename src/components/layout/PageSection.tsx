import type { ReactNode } from 'react';
import styles from './PageSection.module.css';

type PageSectionProps = {
  children: ReactNode;
  eyebrow?: string;
  title: string;
  description?: string;
};

export function PageSection({ children, description, eyebrow, title }: PageSectionProps) {
  return (
    <section className={styles.section}>
      <div className={styles.header}>
        {eyebrow ? <p className={styles.eyebrow}>{eyebrow}</p> : null}
        <h2 className={styles.title}>{title}</h2>
        {description ? <p className={styles.description}>{description}</p> : null}
      </div>
      {children}
    </section>
  );
}
