import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

export interface AppErrorResponse {
  code: string;
  message: string;
  details?: string;
}

export interface ListSessionsInput {
  limit?: number;
  includeArchived?: boolean;
}

export interface SessionSummary {
  id: string;
  title: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  stepCount: number;
}

export interface GetSessionInput {
  sessionId: string;
}

export interface ListScreenshotEditsInput {
  stepId: string;
}

export interface ScreenshotEdit {
  id: string;
  stepId: string;
  editType: 'crop' | 'redact' | 'highlight' | 'arrow' | 'text';
  editDataJson: string;
  createdAt: string;
}

export interface RecordingStep {
  id: string;
  sessionId: string;
  stepNumber: number;
  title: string;
  description?: string;
  actionType: 'click';
  capturedAt: string;
  clickX?: number;
  clickY?: number;
  monitorId?: string;
  appWindowTitle?: string;
  processName?: string;
  originalScreenshotPath: string;
  editedScreenshotPath?: string;
  thumbnailPath?: string;
  isDeleted: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface SessionDetail {
  id: string;
  title: string;
  description?: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  steps: RecordingStep[];
}

export interface UpdateSessionInput {
  sessionId: string;
  title?: string;
  description?: string;
  includeTimestampsDefault?: boolean;
  includeClickMarkersDefault?: boolean;
}

export interface RecordingSession {
  id: string;
  title: string;
  description?: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  createdAt: string;
  updatedAt: string;
  defaultExportDirectory?: string;
  stepCount: number;
  includeTimestampsDefault: boolean;
  includeClickMarkersDefault: boolean;
}

export interface AppSettings {
  screenshotMode: 'clicked_monitor';
  clickDebounceMs: number;
  includeTimestampsInExport: boolean;
  includeClickMarkers: boolean;
  privacyReminderBeforeExport: boolean;
  defaultExportDirectory?: string;
}

export interface UpdateSettingsInput {
  clickDebounceMs?: number;
  includeTimestampsInExport?: boolean;
  includeClickMarkers?: boolean;
  privacyReminderBeforeExport?: boolean;
  defaultExportDirectory?: string;
}

export interface ListExportHistoryInput {
  sessionId: string;
}

export interface ExportHistoryRecord {
  id: string;
  sessionId: string;
  exportType: 'docx' | 'pdf';
  outputPath: string;
  exportedAt: string;
  includeTimestamps: boolean;
  includeClickMarkers: boolean;
  status: 'success' | 'failed';
  errorMessage?: string;
}

type TauriCommandDefinition = {
  request: InvokeArgs | undefined;
  response: unknown;
};

type StepsRecorderCommands = {
  get_app_version: {
    request: undefined;
    response: string;
  };
  get_settings: {
    request: undefined;
    response: AppSettings;
  };
  update_settings: {
    request: { input: UpdateSettingsInput };
    response: AppSettings;
  };
  list_sessions: {
    request: { input?: ListSessionsInput } | undefined;
    response: SessionSummary[];
  };
  get_session: {
    request: { input: GetSessionInput };
    response: SessionDetail;
  };
  update_session: {
    request: { input: UpdateSessionInput };
    response: RecordingSession;
  };
  list_screenshot_edits: {
    request: { input: ListScreenshotEditsInput };
    response: ScreenshotEdit[];
  };
  list_export_history: {
    request: { input: ListExportHistoryInput };
    response: ExportHistoryRecord[];
  };
};

type CommandMap = Record<string, TauriCommandDefinition>;

type AppCommandMap = StepsRecorderCommands & CommandMap;

export type TauriCommandName = keyof StepsRecorderCommands;

export async function invokeTauriCommand<Name extends TauriCommandName>(
  commandName: Name,
  request: AppCommandMap[Name]['request'],
): Promise<AppCommandMap[Name]['response']> {
  return invoke<AppCommandMap[Name]['response']>(commandName, request);
}

export const tauriClient = {
  getAppVersion: () => invokeTauriCommand('get_app_version', undefined),
  getSettings: () => invokeTauriCommand('get_settings', undefined),
  updateSettings: (input: UpdateSettingsInput) => invokeTauriCommand('update_settings', { input }),
  listSessions: (input?: ListSessionsInput) => invokeTauriCommand('list_sessions', input ? { input } : undefined),
  getSession: (input: GetSessionInput) => invokeTauriCommand('get_session', { input }),
  updateSession: (input: UpdateSessionInput) => invokeTauriCommand('update_session', { input }),
  listScreenshotEdits: (input: ListScreenshotEditsInput) => invokeTauriCommand('list_screenshot_edits', { input }),
  listExportHistory: (input: ListExportHistoryInput) => invokeTauriCommand('list_export_history', { input }),
};
