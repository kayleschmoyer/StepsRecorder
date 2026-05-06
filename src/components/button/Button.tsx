import type { ButtonHTMLAttributes, ReactNode } from 'react';
import styles from './Button.module.css';

type ButtonVariant = 'primary' | 'ghost' | 'text';

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
  variant?: ButtonVariant;
};

export function Button({ children, className, variant = 'primary', ...props }: ButtonProps) {
  const classNames = [styles.button, styles[variant], className].filter(Boolean).join(' ');

  return (
    <button className={classNames} type="button" {...props}>
      {children}
    </button>
  );
}
