import type { ReactNode } from 'react';
import { RecordingHomePage } from '../features/recording/RecordingHomePage';

export type AppRoute = {
  path: '/';
  label: string;
  element: ReactNode;
};

export const appRoutes: AppRoute[] = [
  {
    path: '/',
    label: 'Home',
    element: <RecordingHomePage />,
  },
];
