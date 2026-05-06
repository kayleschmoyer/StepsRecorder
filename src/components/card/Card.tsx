import type { HTMLAttributes, ReactNode } from 'react';
import styles from './Card.module.css';

type CardProps = HTMLAttributes<HTMLElement> & {
  children: ReactNode;
};

export function Card({ children, className, ...props }: CardProps) {
  const classNames = [styles.card, className].filter(Boolean).join(' ');

  return (
    <section className={classNames} {...props}>
      {children}
    </section>
  );
}
